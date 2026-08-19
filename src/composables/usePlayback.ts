import { computed, ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MediaItem } from './useMediaLibrary'
import type { ServerInfo } from './useServerStatus'

export type PlaybackStatus = 'idle' | 'loading' | 'playing' | 'error'
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
) {
  const selectedItem = ref<MediaItem | null>(null)
  const status = ref<PlaybackStatus>('idle')
  const error = ref<string | null>(null)
  const audioSelectionError = ref<string | null>(null)
  const isSavingAudio = ref(false)
  const subtitleSelectionError = ref<string | null>(null)
  const isSavingSubtitle = ref(false)

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
    const baseUrl = server.value.baseUrl.replace(/\/$/, '')
    return `${baseUrl}/api/v1/media/${encodeURIComponent(selectedItem.value.id)}/stream`
  })

  const canPlay = computed(() => server.value !== null)

  function play(item: MediaItem) {
    if (!server.value) {
      selectedItem.value = null
      status.value = 'error'
      error.value = 'The private playback API is unavailable.'
      return
    }

    selectedItem.value = item
    status.value = 'loading'
    error.value = null
    audioSelectionError.value = null
    subtitleSelectionError.value = null
  }

  function markPlaying() {
    if (!selectedItem.value) return
    status.value = 'playing'
    error.value = null
  }

  function markError() {
    if (!selectedItem.value) return
    status.value = 'error'
    error.value = 'This video could not be played directly in the current browser.'
  }

  function clear() {
    selectedItem.value = null
    status.value = 'idle'
    error.value = null
    audioSelectionError.value = null
    subtitleSelectionError.value = null
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
    streamUrl,
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
