import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface ServerInfo {
  baseUrl: string
  bindScope: 'loopback' | 'lan'
  lanAvailable: boolean
}

type ServerInfoLoader = () => Promise<ServerInfo>

const loadFromTauri: ServerInfoLoader = () => invoke<ServerInfo>('server_info')

export function useServerStatus(loader: ServerInfoLoader = loadFromTauri) {
  const server = ref<ServerInfo | null>(null)
  const error = ref<string | null>(null)
  const isLoading = ref(false)

  const statusLabel = computed(() => {
    if (isLoading.value) return 'Starting local API…'
    if (server.value) return server.value.lanAvailable ? 'LAN API ready' : 'Private API ready'
    return 'API unavailable'
  })

  async function load() {
    isLoading.value = true
    error.value = null
    try {
      server.value = await loader()
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isLoading.value = false
    }
  }

  return { error, isLoading, load, server, statusLabel }
}
