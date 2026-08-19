import { computed, ref, type Ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { MediaItem } from './useMediaLibrary'
import type { ServerInfo } from './useServerStatus'

export type PlaybackStatus = 'idle' | 'loading' | 'playing' | 'error'
export interface AudioOption {
  id: string
  label: string
}
type AudioSelectionSaver = (mediaId: string, trackId: string | null) => Promise<void>

const saveWithTauri: AudioSelectionSaver = async (mediaId, trackId) => {
  await invoke('select_audio_track', { mediaId, trackId })
}

export function usePlayback(
  server: Ref<ServerInfo | null>,
  canPersistAudio: Ref<boolean> = ref(true),
  saveAudioSelection: AudioSelectionSaver = saveWithTauri,
) {
  const selectedItem = ref<MediaItem | null>(null)
  const status = ref<PlaybackStatus>('idle')
  const error = ref<string | null>(null)
  const audioSelectionError = ref<string | null>(null)
  const isSavingAudio = ref(false)

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

  return {
    audioOptions,
    audioSelectionError,
    canPersistAudio,
    canPlay,
    clear,
    error,
    isSavingAudio,
    markError,
    markPlaying,
    play,
    selectAudioTrack,
    selectedAudioTrackId,
    selectedItem,
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
