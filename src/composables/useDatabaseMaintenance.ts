import { ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

type DatabaseClearer = () => Promise<void>

const clearThroughTauri: DatabaseClearer = () => invoke<void>('clear_local_database')

export function useDatabaseMaintenance(clearer: DatabaseClearer = clearThroughTauri) {
  const error = ref<string | null>(null)
  const isClearing = ref(false)
  const notice = ref<string | null>(null)

  async function clear() {
    isClearing.value = true
    error.value = null
    notice.value = null

    try {
      await clearer()
      notice.value = 'Local database cleared.'
      return true
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
      return false
    } finally {
      isClearing.value = false
    }
  }

  return { clear, error, isClearing, notice }
}
