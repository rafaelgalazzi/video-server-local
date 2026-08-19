import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface LanServerConfig {
  enabled: boolean
  address: string | null
  port: number
  dnsName: string | null
}
export interface LanServerStatus {
  configured: boolean
  active: boolean
  endpoint: string | null
  failure: string | null
}
export interface LanServerAdapter {
  loadConfig: () => Promise<LanServerConfig>
  loadStatus: () => Promise<LanServerStatus>
  addresses: () => Promise<string[]>
  save: (config: LanServerConfig) => Promise<void>
}

const tauriAdapter: LanServerAdapter = {
  loadConfig: () => invoke('lan_server_config'),
  loadStatus: () => invoke('lan_server_status'),
  addresses: () => invoke('suggested_lan_addresses'),
  save: (config) => invoke('save_lan_server_config', { config }),
}

export function useLanServer(adapter: LanServerAdapter = tauriAdapter) {
  const config = ref<LanServerConfig | null>(null)
  const status = ref<LanServerStatus | null>(null)
  const addresses = ref<string[]>([])
  const error = ref<string | null>(null)
  const notice = ref<string | null>(null)
  const isSaving = ref(false)
  const statusLabel = computed(() =>
    status.value?.active
      ? 'Secure LAN server active'
      : config.value?.enabled
        ? 'LAN server pending restart'
        : 'LAN server disabled',
  )

  async function load() {
    error.value = null
    try {
      ;[config.value, status.value, addresses.value] = await Promise.all([
        adapter.loadConfig(),
        adapter.loadStatus(),
        adapter.addresses(),
      ])
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    }
  }
  async function save(next: LanServerConfig) {
    isSaving.value = true
    error.value = null
    notice.value = null
    try {
      await adapter.save(next)
      config.value = next
      notice.value = 'LAN configuration saved. Restart LocalStream to apply it.'
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isSaving.value = false
    }
  }
  return { addresses, config, error, isSaving, load, notice, save, status, statusLabel }
}
