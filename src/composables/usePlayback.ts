import { computed, ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MediaItem } from './useMediaLibrary'
import type { ServerInfo } from './useServerStatus'

export type PlaybackStatus = 'idle' | 'loading' | 'playing' | 'error'
interface PlaybackPreparation {
  method: 'direct_play' | 'remux' | 'transcode'
  jobId: string | null
  outputName: string | null
}
interface PlaybackJob {
  state: 'queued' | 'running' | 'completed' | 'failed' | 'cancelled'
  progressPermille: number
}
interface HlsPreparation {
  jobId: string
  playlistName: string
  videoMode: 'copy' | 'transcode'
}
export interface AudioOption {
  id: string
  label: string
}
export interface SubtitleOption {
  value: string
  label: string
}
type AudioSelectionSaver = (mediaId: string, trackId: string | null) => Promise<void>
type SubtitleSelectionSaver = (
  mediaId: string,
  mode: 'automatic' | 'off' | 'track',
  trackId: string | null,
) => Promise<void>

const saveWithTauri: AudioSelectionSaver = async (mediaId, trackId) => {
  await invoke('select_audio_track', { mediaId, trackId })
}
const saveSubtitleWithTauri: SubtitleSelectionSaver = async (mediaId, mode, trackId) => {
  await invoke('select_subtitle', { mediaId, mode, trackId })
}

export function usePlayback(
  server: Ref<ServerInfo | null>,
  canPersistAudio: Ref<boolean> = ref(true),
  saveAudioSelection: AudioSelectionSaver = saveWithTauri,
  saveSubtitleSelection: SubtitleSelectionSaver = saveSubtitleWithTauri,
  enableFallback: Ref<boolean> = ref(false),
) {
  const selectedItem = ref<MediaItem | null>(null)
  const status = ref<PlaybackStatus>('idle')
  const error = ref<string | null>(null)
  const audioSelectionError = ref<string | null>(null)
  const isSavingAudio = ref(false)
  const subtitleSelectionError = ref<string | null>(null)
  const isSavingSubtitle = ref(false)
  const playbackProgress = ref<number | null>(null)
  const preparationNotice = ref<string | null>(null)
  const activeJobId = ref<string | null>(null)
  const preparedStreamUrl = ref<string | null>(null)
  const preparationResolved = ref(false)

  const audioOptions = computed<AudioOption[]>(() =>
    (selectedItem.value?.metadata?.audioTracks ?? []).map((track, index) => ({
      id: track.id,
      label: audioTrackLabel(track, index),
    })),
  )
  const selectedAudioTrackId = computed(() => {
    const item = selectedItem.value
    const tracks = item?.metadata?.audioTracks ?? []
    if (
      item?.selectedAudioTrackId &&
      tracks.some((track) => track.id === item.selectedAudioTrackId)
    ) {
      return item.selectedAudioTrackId
    }
    return tracks.find((track) => track.isDefault)?.id ?? tracks[0]?.id ?? null
  })
  const subtitleOptions = computed<SubtitleOption[]>(() => [
    { value: 'automatic', label: 'Automatic (forced or default)' },
    { value: 'off', label: 'Off' },
    ...(selectedItem.value?.metadata?.subtitleTracks ?? []).map((track, index) => ({
      value: `track:${track.id}`,
      label: subtitleTrackLabel(track, index),
    })),
  ])
  const subtitleSelectionValue = computed(() => {
    const item = selectedItem.value
    if (!item || item.subtitleMode === 'automatic') return 'automatic'
    if (item.subtitleMode === 'off') return 'off'
    return item.selectedSubtitleTrackId ? `track:${item.selectedSubtitleTrackId}` : 'automatic'
  })
  const activeSubtitleTrack = computed(() => {
    const item = selectedItem.value
    const tracks = item?.metadata?.subtitleTracks ?? []
    if (!item || item.subtitleMode === 'off') return null
    if (item.subtitleMode === 'track') {
      return tracks.find((track) => track.id === item.selectedSubtitleTrackId) ?? null
    }
    return tracks.find((track) => track.isForced) ?? tracks.find((track) => track.isDefault) ?? null
  })
  const subtitleTrackUrl = computed(() => {
    const track = activeSubtitleTrack.value
    if (!server.value || !selectedItem.value || track?.kind !== 'text') return null
    const baseUrl = server.value.baseUrl.replace(/\/$/, '')
    return `${baseUrl}/api/v1/media/${encodeURIComponent(selectedItem.value.id)}/subtitles/${encodeURIComponent(track.id)}`
  })
  const subtitleDeliveryNotice = computed(() => {
    const track = activeSubtitleTrack.value
    if (track?.kind === 'bitmap') return 'This bitmap subtitle requires video conversion.'
    if (track?.kind === 'unknown') return 'This subtitle format is not supported.'
    return null
  })

  const streamUrl = computed(() => {
    if (!server.value || !selectedItem.value) return null
    if (preparedStreamUrl.value) return preparedStreamUrl.value
    if (enableFallback.value && !preparationResolved.value) return null
    const baseUrl = server.value.baseUrl.replace(/\/$/, '')
    return `${baseUrl}/api/v1/media/${encodeURIComponent(selectedItem.value.id)}/stream`
  })

  const canPlay = computed(() => server.value !== null)

  function select(item: MediaItem) {
    if (!server.value) {
      selectedItem.value = null
      status.value = 'error'
      error.value = 'The private playback API is unavailable.'
      return
    }
    cleanupJob()
    selectedItem.value = item
    status.value = 'idle'
    error.value = null
    audioSelectionError.value = null
    subtitleSelectionError.value = null
    preparedStreamUrl.value = null
    preparationResolved.value = false
    playbackProgress.value = null
    preparationNotice.value = null
  }

  function play(item: MediaItem) {
    select(item)
    start()
  }

  function start() {
    const item = selectedItem.value
    if (!item || !server.value) return
    cleanupJob()
    status.value = 'loading'
    error.value = null
    preparedStreamUrl.value = null
    preparationResolved.value = !enableFallback.value
    playbackProgress.value = null
    preparationNotice.value = null
    if (enableFallback.value) void prepare(item)
  }

  function markPlaying() {
    if (!selectedItem.value) return
    status.value = 'playing'
    error.value = null
  }

  function markError() {
    if (!selectedItem.value) return
    status.value = 'error'
    error.value = 'This video could not be played in the current browser.'
  }

  function clear() {
    cleanupJob()
    selectedItem.value = null
    status.value = 'idle'
    error.value = null
    audioSelectionError.value = null
    subtitleSelectionError.value = null
    preparedStreamUrl.value = null
    preparationResolved.value = false
    playbackProgress.value = null
    preparationNotice.value = null
  }

  async function selectAudioTrack(trackId: string) {
    const item = selectedItem.value
    if (
      !item ||
      !canPersistAudio.value ||
      !item.metadata?.audioTracks.some((track) => track.id === trackId)
    ) {
      audioSelectionError.value = 'Audio selection is unavailable in this playback mode.'
      return
    }
    isSavingAudio.value = true
    audioSelectionError.value = null
    try {
      await saveAudioSelection(item.id, trackId)
      item.selectedAudioTrackId = trackId
      stopForConfiguration()
    } catch (reason) {
      audioSelectionError.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isSavingAudio.value = false
    }
  }

  async function selectSubtitle(value: string) {
    const item = selectedItem.value
    if (!item || !canPersistAudio.value) {
      subtitleSelectionError.value = 'Subtitle selection is unavailable in this playback mode.'
      return
    }
    const mode = value === 'automatic' ? 'automatic' : value === 'off' ? 'off' : 'track'
    const trackId = mode === 'track' && value.startsWith('track:') ? value.slice(6) : null
    if (
      mode === 'track' &&
      (!trackId || !item.metadata?.subtitleTracks.some((track) => track.id === trackId))
    ) {
      subtitleSelectionError.value = 'The selected subtitle track is invalid.'
      return
    }
    isSavingSubtitle.value = true
    subtitleSelectionError.value = null
    try {
      await saveSubtitleSelection(item.id, mode, trackId)
      item.subtitleMode = mode
      item.selectedSubtitleTrackId = trackId
      stopForConfiguration()
    } catch (reason) {
      subtitleSelectionError.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isSavingSubtitle.value = false
    }
  }

  return {
    audioOptions,
    audioSelectionError,
    activeSubtitleTrack,
    canPersistAudio,
    canPlay,
    clear,
    error,
    isSavingAudio,
    isSavingSubtitle,
    markError,
    markPlaying,
    play,
    select,
    playbackProgress,
    preparationNotice,
    selectAudioTrack,
    selectSubtitle,
    selectedAudioTrackId,
    selectedItem,
    subtitleDeliveryNotice,
    subtitleOptions,
    subtitleSelectionError,
    subtitleSelectionValue,
    subtitleTrackUrl,
    status,
    start,
    streamUrl,
  }

  function stopForConfiguration() {
    cleanupJob()
    status.value = 'idle'
    error.value = null
    preparedStreamUrl.value = null
    preparationResolved.value = false
    playbackProgress.value = null
    preparationNotice.value = 'Configuration saved. Press Play preview when you are ready.'
  }

  async function prepare(item: MediaItem) {
    cleanupJob()
    try {
      if (item.extension.toLowerCase() === 'mkv') {
        await prepareHls(item)
        return
      }
      const result = await invoke<PlaybackPreparation>('prepare_playback', {
        mediaId: item.id,
        capabilities: browserCapabilities(),
      })
      if (result.method === 'direct_play') {
        preparationResolved.value = true
        return
      }
      if (!result.jobId || !result.outputName || !server.value)
        throw new Error('Invalid playback job')
      activeJobId.value = result.jobId
      while (activeJobId.value === result.jobId) {
        const job = await invoke<PlaybackJob>('playback_job', { jobId: result.jobId })
        playbackProgress.value = job.progressPermille
        if (job.state === 'completed') {
          const baseUrl = server.value.baseUrl.replace(/\/$/, '')
          preparedStreamUrl.value = `${baseUrl}/api/v1/playback/jobs/${encodeURIComponent(result.jobId)}/output/${encodeURIComponent(result.outputName)}`
          preparationResolved.value = true
          return
        }
        if (job.state === 'failed' || job.state === 'cancelled')
          throw new Error('Playback conversion failed')
        await new Promise((resolve) => window.setTimeout(resolve, 100))
      }
    } catch (reason) {
      if (!selectedItem.value) return
      status.value = 'error'
      error.value = reason instanceof Error ? reason.message : 'Playback preparation failed.'
    }
  }

  async function prepareHls(item: MediaItem) {
    const result = canPersistAudio.value
      ? await invoke<HlsPreparation>('prepare_hls', { mediaId: item.id })
      : await prepareBrowserHls(item.id)
    if (!result.jobId || !server.value) throw new Error('Invalid HLS playback session')
    activeJobId.value = result.jobId
    preparationNotice.value =
      result.videoMode === 'copy'
        ? 'Fast start: copying video and converting only audio. The native seek range expands as segments become ready.'
        : 'This video codec requires full software conversion and may use significant CPU.'
    const baseUrl = server.value.baseUrl.replace(/\/$/, '')
    const playlistUrl = `${baseUrl}/api/v1/playback/hls/${encodeURIComponent(result.jobId)}/${encodeURIComponent(result.playlistName)}`
    const deadline = Date.now() + 60_000
    while (activeJobId.value === result.jobId) {
      const response = await fetch(playlistUrl, {
        cache: 'no-store',
        credentials: canPersistAudio.value ? 'omit' : 'same-origin',
      })
      if (response.ok) {
        preparedStreamUrl.value = playlistUrl
        preparationResolved.value = true
        return
      }
      const job = canPersistAudio.value
        ? await invoke<PlaybackJob>('playback_job', { jobId: result.jobId })
        : await browserHlsJob(result.jobId)
      playbackProgress.value = job.progressPermille
      if (job.state === 'failed' || job.state === 'cancelled') {
        throw new Error('HLS playback preparation failed')
      }
      if (job.state === 'completed') {
        throw new Error('HLS conversion completed, but its playlist is unavailable.')
      }
      if (Date.now() >= deadline) {
        throw new Error('Browser-compatible playback did not start within 60 seconds.')
      }
      await new Promise((resolve) => window.setTimeout(resolve, 150))
    }
  }

  function cleanupJob() {
    const jobId = activeJobId.value
    activeJobId.value = null
    if (!jobId) return
    if (canPersistAudio.value) {
      void invoke('cancel_playback', { jobId }).finally(() => invoke('release_playback', { jobId }))
    } else {
      void releaseBrowserHls(jobId)
    }
  }
}

async function prepareBrowserHls(mediaId: string): Promise<HlsPreparation> {
  const response = await fetch('/api/v1/playback/hls', {
    method: 'POST',
    credentials: 'same-origin',
    headers: {
      'content-type': 'application/json',
      'x-localstream-csrf': csrfToken(),
    },
    body: JSON.stringify({ mediaId }),
  })
  if (!response.ok) throw new Error('Browser-compatible playback could not be started.')
  return (await response.json()) as HlsPreparation
}

async function browserHlsJob(jobId: string): Promise<PlaybackJob> {
  const response = await fetch(`/api/v1/playback/hls/${encodeURIComponent(jobId)}/status`, {
    credentials: 'same-origin',
    cache: 'no-store',
  })
  if (!response.ok) throw new Error('Browser playback preparation was interrupted.')
  return (await response.json()) as PlaybackJob
}

async function releaseBrowserHls(jobId: string) {
  await fetch(`/api/v1/playback/hls/${encodeURIComponent(jobId)}`, {
    method: 'DELETE',
    credentials: 'same-origin',
    headers: { 'x-localstream-csrf': csrfToken() },
  })
}

function csrfToken() {
  const prefix = '__Host-localstream_csrf='
  const cookie = document.cookie.split('; ').find((value) => value.startsWith(prefix))
  return cookie?.slice(prefix.length) ?? ''
}

function browserCapabilities() {
  const video = document.createElement('video')
  const supports = (type: string) => video.canPlayType(type) !== ''
  const containers: string[] = []
  const videoCodecs: string[] = []
  const audioCodecs: string[] = []
  if (supports('video/mp4')) containers.push('mp4')
  if (supports('video/webm')) containers.push('webm')
  if (supports('video/mp4; codecs="avc1.42E01E"')) videoCodecs.push('h264')
  if (supports('video/mp4; codecs="hvc1"')) videoCodecs.push('hevc')
  if (supports('video/webm; codecs="vp8"')) videoCodecs.push('vp8')
  if (supports('video/webm; codecs="vp9"')) videoCodecs.push('vp9')
  if (supports('video/mp4; codecs="mp4a.40.2"')) audioCodecs.push('aac')
  if (supports('video/webm; codecs="opus"')) audioCodecs.push('opus')
  if (supports('video/webm; codecs="vorbis"')) audioCodecs.push('vorbis')
  return {
    containers,
    videoCodecs,
    audioCodecs,
    embeddedTextSubtitleCodecs: [],
    externalWebvtt: true,
    embeddedAudioSelection: false,
    bitmapSubtitles: false,
    remuxTargets: [
      { container: 'mp4', videoCodecs: ['h264', 'hevc'], audioCodecs: ['aac'] },
      { container: 'webm', videoCodecs: ['vp8', 'vp9', 'av1'], audioCodecs: ['opus', 'vorbis'] },
    ],
  }
}

function audioTrackLabel(
  track: NonNullable<MediaItem['metadata']>['audioTracks'][number],
  index: number,
) {
  const parts = [track.title?.trim() || track.language?.toUpperCase() || `Audio track ${index + 1}`]
  parts.push(track.codec.toUpperCase())
  if (track.channels) parts.push(`${track.channels} channels`)
  if (track.isDefault) parts.push('Default')
  return parts.join(' · ')
}

function subtitleTrackLabel(
  track: NonNullable<MediaItem['metadata']>['subtitleTracks'][number],
  index: number,
) {
  const parts = [track.title?.trim() || track.language?.toUpperCase() || `Subtitle ${index + 1}`]
  parts.push(track.codec.toUpperCase())
  if (track.isForced) parts.push('Forced')
  if (track.isDefault) parts.push('Default')
  if (track.kind === 'bitmap') parts.push('Bitmap')
  return parts.join(' · ')
}
