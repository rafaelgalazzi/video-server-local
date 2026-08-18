import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface AppInfo {
  name: string
  version: string
  localFirst: boolean
}

type AppInfoLoader = () => Promise<AppInfo>

const loadFromTauri: AppInfoLoader = () => invoke<AppInfo>('app_info')

export function useAppInfo(loader: AppInfoLoader = loadFromTauri) {
  const appInfo = ref<AppInfo | null>(null)
  const error = ref<string | null>(null)
  const isLoading = ref(false)

  const runtimeLabel = computed(() =>
    appInfo.value ? `${appInfo.value.name} native core ready` : 'LocalStream foundation',
  )

  async function load() {
    isLoading.value = true
    error.value = null

    try {
      appInfo.value = await loader()
    } catch (reason) {
      appInfo.value = null
      error.value = reason instanceof Error ? reason.message : 'Native core is unavailable.'
    } finally {
      isLoading.value = false
    }
  }

  return { appInfo, error, isLoading, load, runtimeLabel }
}
