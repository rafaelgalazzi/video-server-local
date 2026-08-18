import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface NodeIdentitySummary {
  nodeId: string
  fingerprint: string
}

type NodeIdentityLoader = () => Promise<NodeIdentitySummary>
type NodeIdentityResetter = () => Promise<number>
type NodeCertificateExporter = () => Promise<boolean>

const loadFromTauri: NodeIdentityLoader = () => invoke<NodeIdentitySummary>('node_identity')
const resetThroughTauri: NodeIdentityResetter = () => invoke<number>('reset_node_identity')
const exportThroughTauri: NodeCertificateExporter = () =>
  invoke<boolean>('export_node_root_certificate')

export function useNodeIdentity(
  loader: NodeIdentityLoader = loadFromTauri,
  resetter: NodeIdentityResetter = resetThroughTauri,
  exporter: NodeCertificateExporter = exportThroughTauri,
) {
  const identity = ref<NodeIdentitySummary | null>(null)
  const error = ref<string | null>(null)
  const isLoading = ref(false)
  const isResetting = ref(false)
  const isConfirmingReset = ref(false)
  const isExporting = ref(false)
  const notice = ref<string | null>(null)

  const statusLabel = computed(() => {
    if (isLoading.value) return 'Loading node identity…'
    if (identity.value) return 'Node identity ready'
    return 'Node identity unavailable'
  })

  async function load() {
    isLoading.value = true
    error.value = null
    try {
      identity.value = await loader()
    } catch (reason) {
      identity.value = null
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isLoading.value = false
    }
  }

  function requestReset() {
    isConfirmingReset.value = true
    error.value = null
    notice.value = null
  }

  function cancelReset() {
    isConfirmingReset.value = false
  }

  async function confirmReset() {
    if (!isConfirmingReset.value || isResetting.value) return
    isResetting.value = true
    error.value = null
    notice.value = null
    try {
      const revoked = await resetter()
      identity.value = null
      isConfirmingReset.value = false
      notice.value = `Node identity removed and ${revoked} trusted device${revoked === 1 ? '' : 's'} revoked. Restart LocalStream to create a new identity.`
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isResetting.value = false
    }
  }

  async function exportRootCertificate() {
    if (!identity.value || isExporting.value || isResetting.value) return
    isExporting.value = true
    error.value = null
    notice.value = null
    try {
      const exported = await exporter()
      notice.value = exported
        ? 'Root certificate exported. Compare its fingerprint here before installing it as trusted.'
        : 'Certificate export canceled.'
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isExporting.value = false
    }
  }

  return {
    cancelReset,
    confirmReset,
    error,
    exportRootCertificate,
    identity,
    isConfirmingReset,
    isLoading,
    isExporting,
    isResetting,
    load,
    notice,
    requestReset,
    statusLabel,
  }
}
