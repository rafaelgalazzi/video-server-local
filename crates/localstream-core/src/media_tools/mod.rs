use std::{
    ffi::{OsStr, OsString},
    path::{Path, PathBuf},
    process::Stdio,
    time::Duration,
};

use thiserror::Error;
use tokio::{io::AsyncReadExt, process::Command};
use tokio_util::sync::CancellationToken;

mod probe;
pub(crate) use probe::probe_media;
pub use probe::ProbeError;

pub const FFPROBE_PATH_ENV: &str = "LOCALSTREAM_FFPROBE_PATH";
pub const FFMPEG_PATH_ENV: &str = "LOCALSTREAM_FFMPEG_PATH";
const VERSION_OUTPUT_LIMIT: usize = 64 * 1024;
const VERSION_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaToolPaths {
    pub ffprobe: PathBuf,
    pub ffmpeg: PathBuf,
}

impl MediaToolPaths {
    pub async fn discover_ffprobe() -> Result<PathBuf, ToolDiscoveryError> {
        let ffprobe = configured_tool(std::env::var_os(FFPROBE_PATH_ENV), "ffprobe")?;
        validate_tool(&ffprobe, "ffprobe").await?;
        Ok(ffprobe)
    }

    pub async fn discover_ffmpeg() -> Result<PathBuf, ToolDiscoveryError> {
        let ffmpeg = configured_tool(std::env::var_os(FFMPEG_PATH_ENV), "ffmpeg")?;
        validate_tool(&ffmpeg, "ffmpeg").await?;
        Ok(ffmpeg)
    }

    pub async fn discover() -> Result<Self, ToolDiscoveryError> {
        Self::discover_with(
            std::env::var_os(FFPROBE_PATH_ENV),
            std::env::var_os(FFMPEG_PATH_ENV),
        )
        .await
    }

    async fn discover_with(
        ffprobe_override: Option<OsString>,
        ffmpeg_override: Option<OsString>,
    ) -> Result<Self, ToolDiscoveryError> {
        let ffprobe = configured_tool(ffprobe_override, "ffprobe")?;
        let ffmpeg = configured_tool(ffmpeg_override, "ffmpeg")?;
        validate_tool(&ffprobe, "ffprobe").await?;
        validate_tool(&ffmpeg, "ffmpeg").await?;
        Ok(Self { ffprobe, ffmpeg })
    }
}

fn configured_tool(value: Option<OsString>, fallback: &str) -> Result<PathBuf, ToolDiscoveryError> {
    match value {
        Some(value) if value.is_empty() => Err(ToolDiscoveryError::EmptyConfiguration {
            variable: if fallback == "ffprobe" {
                FFPROBE_PATH_ENV
            } else {
                FFMPEG_PATH_ENV
            },
        }),
        Some(value) => {
            let path = PathBuf::from(value);
            if !path.is_absolute() {
                return Err(ToolDiscoveryError::RelativeConfiguredPath);
            }
            Ok(path)
        }
        None => Ok(PathBuf::from(fallback)),
    }
}

async fn validate_tool(path: &Path, expected_name: &'static str) -> Result<(), ToolDiscoveryError> {
    let request = ProcessRequest::new(path)
        .arg("-version")
        .timeout(VERSION_TIMEOUT)
        .output_limit(VERSION_OUTPUT_LIMIT);
    let output = ProcessRunner::run(request, CancellationToken::new())
        .await
        .map_err(|source| ToolDiscoveryError::Unavailable {
            tool: expected_name,
            source,
        })?;
    if !output.success {
        return Err(ToolDiscoveryError::Rejected {
            tool: expected_name,
        });
    }
    let first_line = String::from_utf8_lossy(&output.stdout)
        .lines()
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !first_line.starts_with(expected_name) {
        return Err(ToolDiscoveryError::WrongExecutable {
            expected: expected_name,
        });
    }
    Ok(())
}

#[derive(Debug, Error)]
pub enum ToolDiscoveryError {
    #[error("{variable} is configured but empty")]
    EmptyConfiguration { variable: &'static str },
    #[error("configured media tool paths must be absolute")]
    RelativeConfiguredPath,
    #[error("{tool} is unavailable or could not be validated")]
    Unavailable {
        tool: &'static str,
        #[source]
        source: ProcessError,
    },
    #[error("{tool} returned an unsuccessful version check")]
    Rejected { tool: &'static str },
    #[error("the configured executable is not {expected}")]
    WrongExecutable { expected: &'static str },
}

#[derive(Debug, Clone)]
pub struct ProcessRequest {
    executable: PathBuf,
    arguments: Vec<OsString>,
    timeout: Duration,
    output_limit: usize,
}

impl ProcessRequest {
    pub fn new(executable: impl Into<PathBuf>) -> Self {
        Self {
            executable: executable.into(),
            arguments: Vec::new(),
            timeout: Duration::from_secs(30),
            output_limit: 1024 * 1024,
        }
    }

    #[must_use]
    pub fn arg(mut self, argument: impl AsRef<OsStr>) -> Self {
        self.arguments.push(argument.as_ref().to_os_string());
        self
    }

    #[must_use]
    pub fn args<I, S>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        self.arguments.extend(
            arguments
                .into_iter()
                .map(|argument| argument.as_ref().to_os_string()),
        );
        self
    }

    #[must_use]
    pub const fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[must_use]
    pub const fn output_limit(mut self, output_limit: usize) -> Self {
        self.output_limit = output_limit;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    pub success: bool,
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("the process could not be started")]
    Spawn(#[source] std::io::Error),
    #[error("the process could not be monitored")]
    Wait(#[source] std::io::Error),
    #[error("the process timed out")]
    Timeout,
    #[error("the process was cancelled")]
    Cancelled,
    #[error("the process output exceeded its configured limit")]
    OutputLimitExceeded,
    #[error("the process output pipe failed")]
    Output(#[source] std::io::Error),
}

pub struct ProcessRunner;

impl ProcessRunner {
    pub async fn run(
        request: ProcessRequest,
        cancellation: CancellationToken,
    ) -> Result<ProcessOutput, ProcessError> {
        if request.output_limit == 0 {
            return Err(ProcessError::OutputLimitExceeded);
        }

        let mut command = Command::new(&request.executable);
        command
            .args(&request.arguments)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            command.as_std_mut().creation_flags(CREATE_NO_WINDOW);
        }
        let mut child = command.spawn().map_err(ProcessError::Spawn)?;
        let stdout = child.stdout.take().expect("piped stdout must exist");
        let stderr = child.stderr.take().expect("piped stderr must exist");
        let output_limit = request.output_limit;
        let stdout_task = tokio::spawn(read_bounded(stdout, output_limit));
        let stderr_task = tokio::spawn(read_bounded(stderr, output_limit));

        let status = tokio::select! {
            status = child.wait() => status.map_err(ProcessError::Wait)?,
            () = cancellation.cancelled() => {
                terminate(&mut child).await;
                stdout_task.abort();
                stderr_task.abort();
                return Err(ProcessError::Cancelled);
            }
            () = tokio::time::sleep(request.timeout) => {
                terminate(&mut child).await;
                stdout_task.abort();
                stderr_task.abort();
                return Err(ProcessError::Timeout);
            }
        };

        let stdout = join_output(stdout_task).await?;
        let stderr = join_output(stderr_task).await?;
        Ok(ProcessOutput {
            success: status.success(),
            exit_code: status.code(),
            stdout,
            stderr,
        })
    }
}

async fn terminate(child: &mut tokio::process::Child) {
    let _ = child.kill().await;
    let _ = child.wait().await;
}

async fn join_output(
    task: tokio::task::JoinHandle<Result<Vec<u8>, ProcessError>>,
) -> Result<Vec<u8>, ProcessError> {
    task.await.map_err(|error| {
        ProcessError::Output(std::io::Error::other(format!(
            "output task failed: {error}"
        )))
    })?
}

async fn read_bounded<R>(mut reader: R, limit: usize) -> Result<Vec<u8>, ProcessError>
where
    R: tokio::io::AsyncRead + Unpin,
{
    let mut captured = Vec::with_capacity(limit.min(8192));
    let mut buffer = [0_u8; 8192];
    let mut exceeded = false;
    loop {
        let count = reader
            .read(&mut buffer)
            .await
            .map_err(ProcessError::Output)?;
        if count == 0 {
            break;
        }
        let remaining = limit.saturating_sub(captured.len());
        captured.extend_from_slice(&buffer[..count.min(remaining)]);
        exceeded |= count > remaining;
    }
    if exceeded {
        Err(ProcessError::OutputLimitExceeded)
    } else {
        Ok(captured)
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsString, path::PathBuf, time::Duration};

    use tokio_util::sync::CancellationToken;

    use super::{MediaToolPaths, ProcessError, ProcessRequest, ProcessRunner};

    fn test_shell() -> PathBuf {
        #[cfg(windows)]
        return PathBuf::from(r"C:\Windows\System32\cmd.exe");
        #[cfg(not(windows))]
        return PathBuf::from("/bin/sh");
    }

    fn shell_request(script: &str) -> ProcessRequest {
        #[cfg(windows)]
        return ProcessRequest::new(test_shell()).args([OsString::from("/C"), script.into()]);
        #[cfg(not(windows))]
        return ProcessRequest::new(test_shell()).args([OsString::from("-c"), script.into()]);
    }

    #[test]
    fn hostile_media_names_remain_one_structured_argument() {
        let hostile = OsString::from("movie; echo injected && $(echo bad).mkv");
        let request = ProcessRequest::new("ffprobe").args([OsString::from("-i"), hostile.clone()]);

        assert_eq!(request.executable, PathBuf::from("ffprobe"));
        assert_eq!(request.arguments, vec![OsString::from("-i"), hostile]);
    }

    #[tokio::test]
    async fn enforces_timeout() {
        #[cfg(windows)]
        let script = "for /L %i in (1,0,2) do @rem";
        #[cfg(not(windows))]
        let script = "sleep 5";
        let error = ProcessRunner::run(
            shell_request(script).timeout(Duration::from_millis(20)),
            CancellationToken::new(),
        )
        .await
        .expect_err("slow process must time out");
        assert!(matches!(error, ProcessError::Timeout));
    }

    #[tokio::test]
    async fn supports_cancellation() {
        #[cfg(windows)]
        let script = "for /L %i in (1,0,2) do @rem";
        #[cfg(not(windows))]
        let script = "sleep 5";
        let token = CancellationToken::new();
        token.cancel();
        let error = ProcessRunner::run(shell_request(script), token)
            .await
            .expect_err("cancelled process must stop");
        assert!(matches!(error, ProcessError::Cancelled));
    }

    #[tokio::test]
    async fn bounds_each_output_pipe() {
        let error = ProcessRunner::run(
            shell_request("echo 123456789").output_limit(4),
            CancellationToken::new(),
        )
        .await
        .expect_err("large output must be rejected");
        assert!(matches!(error, ProcessError::OutputLimitExceeded));
    }

    #[tokio::test]
    async fn rejects_relative_explicit_paths_before_spawning() {
        let error = MediaToolPaths::discover_with(
            Some(OsString::from("relative/ffprobe")),
            Some(OsString::from("relative/ffmpeg")),
        )
        .await
        .expect_err("configured paths must be absolute");
        assert!(matches!(
            error,
            super::ToolDiscoveryError::RelativeConfiguredPath
        ));
    }

    #[tokio::test]
    async fn discovers_real_tools_when_available() {
        let result = MediaToolPaths::discover().await;
        if std::process::Command::new("ffprobe")
            .arg("-version")
            .output()
            .is_ok()
            && std::process::Command::new("ffmpeg")
                .arg("-version")
                .output()
                .is_ok()
        {
            result.expect("installed FFmpeg tools should validate");
        }
    }
}
