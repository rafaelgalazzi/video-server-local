import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export type PortableRuntimeState = 'not-portable' | 'ready' | 'failed'

export interface PortableRuntimeStatus {
  state: PortableRuntimeState
  failure: string | null
}

type PortableRuntimeLoader = () => Promise<PortableRuntimeStatus>

const loadFromTauri: PortableRuntimeLoader = () => invoke('portable_runtime_status')

export function usePortableRuntime(loader: PortableRuntimeLoader = loadFromTauri) {
  const status = ref<PortableRuntimeStatus | null>(null)
  const error = ref<string | null>(null)
  const isAvailable = computed(() => status.value?.state === 'ready')
  const hasFailed = computed(() => status.value?.state === 'failed')

  async function load() {
    error.value = null
    try {
      status.value = await loader()
    } catch (reason) {
      status.value = null
      error.value = reason instanceof Error ? reason.message : String(reason)
    }
  }

  return { error, hasFailed, isAvailable, load, status }
}
