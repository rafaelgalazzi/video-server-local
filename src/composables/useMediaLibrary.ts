import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface MediaItem {
  id: string
  title: string
  extension: string
  sizeBytes: number
}

export interface LibraryScan {
  libraryName: string
  items: MediaItem[]
  skippedEntries: number
}

type LibrarySelector = () => Promise<LibraryScan | null>

const selectFromTauri: LibrarySelector = () => invoke<LibraryScan | null>('select_and_scan_library')

export function useMediaLibrary(selector: LibrarySelector = selectFromTauri) {
  const library = ref<LibraryScan | null>(null)
  const error = ref<string | null>(null)
  const notice = ref<string | null>(null)
  const isScanning = ref(false)

  const itemCountLabel = computed(() => {
    const count = library.value?.items.length ?? 0
    return `${count} ${count === 1 ? 'video' : 'videos'}`
  })

  async function selectLibrary() {
    isScanning.value = true
    error.value = null
    notice.value = null

    try {
      const result = await selector()
      if (result === null) {
        notice.value = 'Folder selection cancelled.'
        return
      }

      library.value = result
      notice.value =
        result.skippedEntries > 0
          ? `${result.skippedEntries} inaccessible ${result.skippedEntries === 1 ? 'entry was' : 'entries were'} skipped.`
          : null
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isScanning.value = false
    }
  }

  return {
    error,
    isScanning,
    itemCountLabel,
    library,
    notice,
    selectLibrary,
  }
}
