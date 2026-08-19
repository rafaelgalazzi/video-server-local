use std::{
    collections::HashMap,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    sync::{Arc, Mutex},
};

use serde::{Serialize, Serializer};
use thiserror::Error;
use tokio::sync::{mpsc, Mutex as AsyncMutex};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

type JobFuture = Pin<Box<dyn Future<Output = Result<(), JobFailure>> + Send>>;
type JobWork = Box<dyn FnOnce(MediaJobContext) -> JobFuture + Send>;
const OWNED_WORK_DIRECTORY: &str = "localstream-media-jobs-v1";

#[derive(Debug, Clone)]
pub struct MediaJobConfig {
    pub work_root: PathBuf,
    pub max_concurrent: usize,
    pub max_queued: usize,
    pub temporary_byte_quota: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct MediaJobKey(String);

impl MediaJobKey {
    pub fn new(value: impl Into<String>) -> Result<Self, MediaJobSubmitError> {
        let value = value.into();
        if value.is_empty() || value.len() > 512 {
            return Err(MediaJobSubmitError::InvalidKey);
        }
        Ok(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MediaJobId(Uuid);

impl Serialize for MediaJobId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaJobState {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum MediaJobFailureKind {
    Transform,
    TemporaryQuotaExceeded,
    TemporaryStorageUnavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaJobSnapshot {
    pub id: MediaJobId,
    pub state: MediaJobState,
    pub progress_permille: u16,
    pub failure: Option<MediaJobFailureKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitDisposition {
    Admitted,
    Deduplicated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MediaJobSubmission {
    pub id: MediaJobId,
    pub disposition: SubmitDisposition,
}

#[derive(Debug)]
pub struct MediaJobOutput {
    pub file: tokio::fs::File,
    pub size_bytes: u64,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum MediaJobOutputError {
    #[error("the media job does not exist")]
    UnknownJob,
    #[error("the media job output is not ready")]
    NotReady,
    #[error("the media job output name is invalid")]
    InvalidName,
    #[error("the media job output is unavailable")]
    Unavailable,
}

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq)]
pub enum MediaJobSubmitError {
    #[error("the media job key is invalid")]
    InvalidKey,
    #[error("the media job byte reservation is invalid")]
    InvalidReservation,
    #[error("the media job queue is full")]
    QueueFull,
    #[error("the temporary media quota is exhausted")]
    TemporaryQuotaExceeded,
    #[error("the media job manager is unavailable")]
    Unavailable,
}

#[derive(Debug, Error)]
pub enum MediaJobStartError {
    #[error("the media job configuration is invalid")]
    InvalidConfiguration,
    #[error("the temporary media directory is unavailable")]
    TemporaryStorage(#[source] std::io::Error),
}

#[derive(Debug, Error)]
#[error("the media transform failed")]
pub struct JobFailure;

#[derive(Clone)]
pub struct MediaJobContext {
    directory: PathBuf,
    cancellation: CancellationToken,
    progress: MediaJobProgress,
}

impl MediaJobContext {
    pub fn directory(&self) -> &Path {
        &self.directory
    }

    pub fn cancellation(&self) -> CancellationToken {
        self.cancellation.clone()
    }

    pub fn progress(&self) -> MediaJobProgress {
        self.progress.clone()
    }
}

#[derive(Clone)]
pub struct MediaJobProgress {
    record: Arc<Mutex<JobRecord>>,
}

impl MediaJobProgress {
    pub fn set_permille(&self, progress: u16) {
        let mut record = lock(&self.record);
        if record.state == MediaJobState::Running {
            record.progress_permille = progress.min(999);
        }
    }
}

struct JobRecord {
    id: MediaJobId,
    key: MediaJobKey,
    state: MediaJobState,
    progress_permille: u16,
    failure: Option<MediaJobFailureKind>,
    reserved_bytes: u64,
    cancellation: CancellationToken,
}

struct QueuedJob {
    record: Arc<Mutex<JobRecord>>,
    work: JobWork,
}

struct ManagerState {
    jobs: HashMap<MediaJobId, Arc<Mutex<JobRecord>>>,
    active_keys: HashMap<MediaJobKey, MediaJobId>,
    reserved_bytes: u64,
}

struct Inner {
    root: PathBuf,
    quota: u64,
    state: Mutex<ManagerState>,
}

#[derive(Clone)]
pub struct MediaJobManager {
    inner: Arc<Inner>,
    sender: mpsc::Sender<QueuedJob>,
    _lifetime: Arc<ManagerLifetime>,
}

struct ManagerLifetime {
    inner: std::sync::Weak<Inner>,
}

impl Drop for ManagerLifetime {
    fn drop(&mut self) {
        if let Some(inner) = self.inner.upgrade() {
            for record in lock(&inner.state).jobs.values() {
                lock(record).cancellation.cancel();
            }
        }
    }
}

impl MediaJobManager {
    pub async fn start(config: MediaJobConfig) -> Result<Self, MediaJobStartError> {
        if config.max_concurrent == 0 || config.max_queued == 0 || config.temporary_byte_quota == 0
        {
            return Err(MediaJobStartError::InvalidConfiguration);
        }
        let work_root = config.work_root.join(OWNED_WORK_DIRECTORY);
        tokio::fs::create_dir_all(&work_root)
            .await
            .map_err(MediaJobStartError::TemporaryStorage)?;
        cleanup_children(&work_root)
            .await
            .map_err(MediaJobStartError::TemporaryStorage)?;

        let (sender, receiver) = mpsc::channel(config.max_queued);
        let inner = Arc::new(Inner {
            root: work_root,
            quota: config.temporary_byte_quota,
            state: Mutex::new(ManagerState {
                jobs: HashMap::new(),
                active_keys: HashMap::new(),
                reserved_bytes: 0,
            }),
        });
        let lifetime = Arc::new(ManagerLifetime {
            inner: Arc::downgrade(&inner),
        });
        let receiver = Arc::new(AsyncMutex::new(receiver));
        for _ in 0..config.max_concurrent {
            let inner = Arc::clone(&inner);
            let receiver = Arc::clone(&receiver);
            tokio::spawn(async move { worker(inner, receiver).await });
        }
        Ok(Self {
            inner,
            sender,
            _lifetime: lifetime,
        })
    }

    pub fn submit<F, Fut>(
        &self,
        key: MediaJobKey,
        reserved_bytes: u64,
        work: F,
    ) -> Result<MediaJobSubmission, MediaJobSubmitError>
    where
        F: FnOnce(MediaJobContext) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), JobFailure>> + Send + 'static,
    {
        if reserved_bytes == 0 || reserved_bytes > self.inner.quota {
            return Err(MediaJobSubmitError::InvalidReservation);
        }
        let mut state = lock(&self.inner.state);
        if let Some(id) = state.active_keys.get(&key) {
            return Ok(MediaJobSubmission {
                id: *id,
                disposition: SubmitDisposition::Deduplicated,
            });
        }
        if state.reserved_bytes.saturating_add(reserved_bytes) > self.inner.quota {
            return Err(MediaJobSubmitError::TemporaryQuotaExceeded);
        }

        let id = MediaJobId(Uuid::new_v4());
        let record = Arc::new(Mutex::new(JobRecord {
            id,
            key: key.clone(),
            state: MediaJobState::Queued,
            progress_permille: 0,
            failure: None,
            reserved_bytes,
            cancellation: CancellationToken::new(),
        }));
        state.reserved_bytes += reserved_bytes;
        state.active_keys.insert(key, id);
        state.jobs.insert(id, Arc::clone(&record));
        let queued = QueuedJob {
            record,
            work: Box::new(move |context| Box::pin(work(context))),
        };
        if self.sender.try_send(queued).is_err() {
            remove_record(&mut state, id);
            return Err(MediaJobSubmitError::QueueFull);
        }
        Ok(MediaJobSubmission {
            id,
            disposition: SubmitDisposition::Admitted,
        })
    }

    pub fn snapshot(&self, id: MediaJobId) -> Option<MediaJobSnapshot> {
        let state = lock(&self.inner.state);
        state.jobs.get(&id).map(|record| snapshot(&lock(record)))
    }

    pub fn cancel(&self, id: MediaJobId) -> bool {
        let state = lock(&self.inner.state);
        let Some(record) = state.jobs.get(&id) else {
            return false;
        };
        let record = lock(record);
        if matches!(record.state, MediaJobState::Queued | MediaJobState::Running) {
            record.cancellation.cancel();
            true
        } else {
            false
        }
    }

    pub async fn release(&self, id: MediaJobId) -> bool {
        let directory = self.inner.root.join(id.0.to_string());
        let removed = {
            let mut state = lock(&self.inner.state);
            let terminal = state.jobs.get(&id).is_some_and(|record| {
                matches!(
                    lock(record).state,
                    MediaJobState::Completed | MediaJobState::Failed | MediaJobState::Cancelled
                )
            });
            terminal.then(|| remove_record(&mut state, id)).is_some()
        };
        if removed {
            let _ = remove_entry(&directory).await;
        }
        removed
    }

    pub async fn open_output(
        &self,
        id: MediaJobId,
        name: &str,
    ) -> Result<MediaJobOutput, MediaJobOutputError> {
        let valid_name = Path::new(name).components().count() == 1
            && matches!(
                Path::new(name).components().next(),
                Some(std::path::Component::Normal(_))
            );
        if !valid_name {
            return Err(MediaJobOutputError::InvalidName);
        }
        {
            let state = lock(&self.inner.state);
            let record = state.jobs.get(&id).ok_or(MediaJobOutputError::UnknownJob)?;
            if lock(record).state != MediaJobState::Completed {
                return Err(MediaJobOutputError::NotReady);
            }
        }
        let path = self.inner.root.join(id.0.to_string()).join(name);
        let entry = tokio::fs::symlink_metadata(&path)
            .await
            .map_err(|_| MediaJobOutputError::Unavailable)?;
        if !entry.is_file() || entry.file_type().is_symlink() {
            return Err(MediaJobOutputError::Unavailable);
        }
        let file = tokio::fs::File::open(&path)
            .await
            .map_err(|_| MediaJobOutputError::Unavailable)?;
        let metadata = file
            .metadata()
            .await
            .map_err(|_| MediaJobOutputError::Unavailable)?;
        if !metadata.is_file() {
            return Err(MediaJobOutputError::Unavailable);
        }
        Ok(MediaJobOutput {
            file,
            size_bytes: metadata.len(),
        })
    }

    #[cfg(test)]
    pub(crate) fn test_output_path(&self, id: MediaJobId, name: &str) -> PathBuf {
        self.inner.root.join(id.0.to_string()).join(name)
    }
}

async fn worker(inner: Arc<Inner>, receiver: Arc<AsyncMutex<mpsc::Receiver<QueuedJob>>>) {
    loop {
        let queued = receiver.lock().await.recv().await;
        let Some(queued) = queued else { return };
        run_job(&inner, queued).await;
    }
}

async fn run_job(inner: &Arc<Inner>, queued: QueuedJob) {
    let (id, cancellation) = {
        let mut record = lock(&queued.record);
        if record.cancellation.is_cancelled() {
            record.state = MediaJobState::Cancelled;
            finish_active(inner, &record);
            return;
        }
        record.state = MediaJobState::Running;
        (record.id, record.cancellation.clone())
    };
    let directory = inner.root.join(id.0.to_string());
    let created = tokio::fs::create_dir(&directory).await;
    let result = if created.is_err() {
        Err(MediaJobFailureKind::TemporaryStorageUnavailable)
    } else {
        let context = MediaJobContext {
            directory: directory.clone(),
            cancellation: cancellation.clone(),
            progress: MediaJobProgress {
                record: Arc::clone(&queued.record),
            },
        };
        match (queued.work)(context).await {
            Ok(()) if cancellation.is_cancelled() => Err(MediaJobFailureKind::Transform),
            Ok(()) => match directory_size(&directory).await {
                Ok(size) if size <= lock(&queued.record).reserved_bytes => Ok(()),
                Ok(_) => Err(MediaJobFailureKind::TemporaryQuotaExceeded),
                Err(_) => Err(MediaJobFailureKind::TemporaryStorageUnavailable),
            },
            Err(_) => Err(MediaJobFailureKind::Transform),
        }
    };

    let (final_state, failure) = if cancellation.is_cancelled() {
        (MediaJobState::Cancelled, None)
    } else if let Err(failure) = result {
        (MediaJobState::Failed, Some(failure))
    } else {
        (MediaJobState::Completed, None)
    };
    if final_state != MediaJobState::Completed {
        let _ = remove_entry(&directory).await;
    }
    {
        let mut record = lock(&queued.record);
        record.state = final_state;
        record.failure = failure;
        if final_state == MediaJobState::Completed {
            record.progress_permille = 1000;
        }
        finish_active(inner, &record);
    }
}

fn finish_active(inner: &Inner, record: &JobRecord) {
    lock(&inner.state).active_keys.remove(&record.key);
}

fn snapshot(record: &JobRecord) -> MediaJobSnapshot {
    MediaJobSnapshot {
        id: record.id,
        state: record.state,
        progress_permille: record.progress_permille,
        failure: record.failure,
    }
}

fn remove_record(state: &mut ManagerState, id: MediaJobId) {
    if let Some(record) = state.jobs.remove(&id) {
        let record = lock(&record);
        state.active_keys.remove(&record.key);
        state.reserved_bytes = state.reserved_bytes.saturating_sub(record.reserved_bytes);
    }
}

async fn cleanup_children(root: &Path) -> std::io::Result<()> {
    let mut entries = tokio::fs::read_dir(root).await?;
    while let Some(entry) = entries.next_entry().await? {
        remove_entry(&entry.path()).await?;
    }
    Ok(())
}

async fn remove_entry(path: &Path) -> std::io::Result<()> {
    let metadata = tokio::fs::symlink_metadata(path).await?;
    if metadata.is_dir() && !metadata.file_type().is_symlink() {
        tokio::fs::remove_dir_all(path).await
    } else {
        tokio::fs::remove_file(path).await
    }
}

async fn directory_size(root: &Path) -> std::io::Result<u64> {
    let root = root.to_owned();
    tokio::task::spawn_blocking(move || {
        let mut total = 0_u64;
        let mut pending = vec![root];
        while let Some(directory) = pending.pop() {
            for entry in std::fs::read_dir(directory)? {
                let entry = entry?;
                let metadata = entry.path().symlink_metadata()?;
                if metadata.is_dir() && !metadata.file_type().is_symlink() {
                    pending.push(entry.path());
                } else if metadata.is_file() {
                    total = total.saturating_add(metadata.len());
                }
            }
        }
        Ok(total)
    })
    .await
    .map_err(std::io::Error::other)?
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::PathBuf, sync::Arc, time::Duration};

    use tempfile::TempDir;
    use tokio::sync::Notify;

    use super::*;
    use crate::media_tools::{ProcessError, ProcessRequest, ProcessRunner};

    fn long_running_process() -> ProcessRequest {
        #[cfg(windows)]
        return ProcessRequest::new(PathBuf::from(r"C:\Windows\System32\cmd.exe")).args([
            OsString::from("/C"),
            OsString::from("for /L %i in (1,0,2) do @rem"),
        ]);
        #[cfg(not(windows))]
        return ProcessRequest::new(PathBuf::from("/bin/sh"))
            .args([OsString::from("-c"), OsString::from("sleep 30")]);
    }

    async fn manager(
        temp: &TempDir,
        concurrent: usize,
        queued: usize,
        quota: u64,
    ) -> MediaJobManager {
        MediaJobManager::start(MediaJobConfig {
            work_root: temp.path().join("jobs"),
            max_concurrent: concurrent,
            max_queued: queued,
            temporary_byte_quota: quota,
        })
        .await
        .unwrap()
    }

    fn job_root(temp: &TempDir) -> PathBuf {
        temp.path().join("jobs").join(OWNED_WORK_DIRECTORY)
    }

    async fn wait_for(manager: &MediaJobManager, id: MediaJobId, state: MediaJobState) {
        tokio::time::timeout(Duration::from_secs(3), async {
            loop {
                if manager.snapshot(id).unwrap().state == state {
                    return;
                }
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("job should reach expected state");
    }

    #[tokio::test]
    async fn bounds_concurrency_queue_and_deduplicates_active_work() {
        let temp = TempDir::new().unwrap();
        let manager = manager(&temp, 1, 1, 100).await;
        let gate = Arc::new(Notify::new());
        let first_gate = Arc::clone(&gate);
        let first_key = MediaJobKey::new("same-transform").unwrap();
        let first = manager
            .submit(first_key.clone(), 10, move |_| async move {
                first_gate.notified().await;
                Ok(())
            })
            .unwrap();
        wait_for(&manager, first.id, MediaJobState::Running).await;
        let duplicate = manager.submit(first_key, 10, |_| async { Ok(()) }).unwrap();
        assert_eq!(duplicate.id, first.id);
        assert_eq!(duplicate.disposition, SubmitDisposition::Deduplicated);

        let second = manager
            .submit(MediaJobKey::new("second").unwrap(), 10, |_| async {
                Ok(())
            })
            .unwrap();
        assert_eq!(
            manager.snapshot(second.id).unwrap().state,
            MediaJobState::Queued
        );
        assert_eq!(
            manager.submit(MediaJobKey::new("third").unwrap(), 10, |_| async { Ok(()) }),
            Err(MediaJobSubmitError::QueueFull)
        );
        gate.notify_one();
        wait_for(&manager, second.id, MediaJobState::Completed).await;
    }

    #[tokio::test]
    async fn cancellation_cleans_output_releases_dedup_and_bounds_progress() {
        let temp = TempDir::new().unwrap();
        let manager = manager(&temp, 1, 2, 100).await;
        let key = MediaJobKey::new("cancel-me").unwrap();
        let submission = manager
            .submit(key.clone(), 50, |context| async move {
                tokio::fs::write(context.directory().join("partial"), b"partial")
                    .await
                    .unwrap();
                context.progress().set_permille(u16::MAX);
                context.cancellation().cancelled().await;
                Err(JobFailure)
            })
            .unwrap();
        wait_for(&manager, submission.id, MediaJobState::Running).await;
        tokio::time::timeout(Duration::from_secs(1), async {
            while manager.snapshot(submission.id).unwrap().progress_permille != 999 {
                tokio::task::yield_now().await;
            }
        })
        .await
        .expect("progress should be reported");
        assert!(manager.cancel(submission.id));
        wait_for(&manager, submission.id, MediaJobState::Cancelled).await;
        assert!(!job_root(&temp).join(submission.id.0.to_string()).exists());
        assert_eq!(
            manager
                .submit(key, 10, |_| async { Ok(()) })
                .unwrap()
                .disposition,
            SubmitDisposition::Admitted
        );
    }

    #[tokio::test]
    async fn enforces_reserved_and_actual_quota_then_releases_completed_output() {
        let temp = TempDir::new().unwrap();
        let manager = manager(&temp, 1, 2, 10).await;
        let job = manager
            .submit(
                MediaJobKey::new("oversize").unwrap(),
                5,
                |context| async move {
                    tokio::fs::write(context.directory().join("output"), [0_u8; 6])
                        .await
                        .unwrap();
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(
            manager.submit(MediaJobKey::new("quota").unwrap(), 6, |_| async { Ok(()) }),
            Err(MediaJobSubmitError::TemporaryQuotaExceeded)
        );
        wait_for(&manager, job.id, MediaJobState::Failed).await;
        assert_eq!(
            manager.snapshot(job.id).unwrap().failure,
            Some(MediaJobFailureKind::TemporaryQuotaExceeded)
        );
        assert!(manager.release(job.id).await);
        manager
            .submit(MediaJobKey::new("after-release").unwrap(), 10, |_| async {
                Ok(())
            })
            .unwrap();
    }

    #[tokio::test]
    async fn startup_removes_stale_files_directories_and_symlinks() {
        let temp = TempDir::new().unwrap();
        let parent = temp.path().join("jobs");
        let root = parent.join(OWNED_WORK_DIRECTORY);
        std::fs::create_dir_all(root.join("stale-dir")).unwrap();
        std::fs::write(root.join("stale-dir").join("part"), b"x").unwrap();
        std::fs::write(root.join("stale-file"), b"x").unwrap();
        std::fs::write(parent.join("unowned"), b"keep").unwrap();
        let _manager = manager(&temp, 1, 1, 10).await;
        assert_eq!(std::fs::read_dir(root).unwrap().count(), 0);
        assert!(parent.join("unowned").exists());
    }

    #[tokio::test]
    async fn dropping_last_manager_cancels_running_work_and_releases_workers() {
        let temp = TempDir::new().unwrap();
        let manager = manager(&temp, 1, 1, 10).await;
        let cancelled = Arc::new(Notify::new());
        let observed = Arc::clone(&cancelled);
        let submission = manager
            .submit(
                MediaJobKey::new("manager-drop").unwrap(),
                10,
                |context| async move {
                    context.cancellation().cancelled().await;
                    observed.notify_one();
                    Err(JobFailure)
                },
            )
            .unwrap();
        wait_for(&manager, submission.id, MediaJobState::Running).await;
        drop(manager);
        tokio::time::timeout(Duration::from_secs(1), cancelled.notified())
            .await
            .expect("dropping the manager should cancel running work");
    }

    #[tokio::test]
    async fn opens_only_completed_single_component_output_names() {
        use tokio::io::AsyncReadExt;

        let temp = TempDir::new().unwrap();
        let manager = manager(&temp, 1, 1, 10).await;
        let gate = Arc::new(Notify::new());
        let worker_gate = Arc::clone(&gate);
        let submission = manager
            .submit(
                MediaJobKey::new("output").unwrap(),
                10,
                move |context| async move {
                    worker_gate.notified().await;
                    tokio::fs::write(context.directory().join("output.mp4"), b"media")
                        .await
                        .unwrap();
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(
            manager
                .open_output(submission.id, "output.mp4")
                .await
                .unwrap_err(),
            MediaJobOutputError::NotReady
        );
        gate.notify_one();
        wait_for(&manager, submission.id, MediaJobState::Completed).await;
        assert_eq!(
            manager
                .open_output(submission.id, "../outside")
                .await
                .unwrap_err(),
            MediaJobOutputError::InvalidName
        );
        let mut output = manager
            .open_output(submission.id, "output.mp4")
            .await
            .unwrap();
        assert_eq!(output.size_bytes, 5);
        let mut bytes = Vec::new();
        output.file.read_to_end(&mut bytes).await.unwrap();
        assert_eq!(bytes, b"media");
    }

    #[tokio::test]
    async fn cancellation_reaches_process_boundary_before_terminal_state() {
        let temp = TempDir::new().unwrap();
        let manager = manager(&temp, 1, 1, 10).await;
        let process_stopped = Arc::new(Notify::new());
        let observed = Arc::clone(&process_stopped);
        let submission = manager
            .submit(
                MediaJobKey::new("process-cancel").unwrap(),
                10,
                |context| async move {
                    let result =
                        ProcessRunner::run(long_running_process(), context.cancellation()).await;
                    assert!(matches!(result, Err(ProcessError::Cancelled)));
                    observed.notify_one();
                    Err(JobFailure)
                },
            )
            .unwrap();
        wait_for(&manager, submission.id, MediaJobState::Running).await;
        assert!(manager.cancel(submission.id));
        tokio::time::timeout(Duration::from_secs(2), process_stopped.notified())
            .await
            .expect("the child process should be killed and reaped");
        wait_for(&manager, submission.id, MediaJobState::Cancelled).await;
    }
}
