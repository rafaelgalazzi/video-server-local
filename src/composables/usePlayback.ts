import { computed, ref, type Ref } from 'vue'
import type { MediaItem } from './useMediaLibrary'
import type { ServerInfo } from './useServerStatus'

export type PlaybackStatus = 'idle' | 'loading' | 'playing' | 'error'

export function usePlayback(server: Ref<ServerInfo | null>) {
  const selectedItem = ref<MediaItem | null>(null)
  const status = ref<PlaybackStatus>('idle')
  const error = ref<string | null>(null)

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
  }

  return { canPlay, clear, error, markError, markPlaying, play, selectedItem, status, streamUrl }
}
