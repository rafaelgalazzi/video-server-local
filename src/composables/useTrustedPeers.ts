import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface TrustedPeerSummary {
  id: string
  displayName: string
  capability: 'library_read'
  createdAt: number
}

type PeerLoader = () => Promise<TrustedPeerSummary[]>
type PeerRevoker = (peerId: string) => Promise<boolean>

const loadFromTauri: PeerLoader = () => invoke<TrustedPeerSummary[]>('trusted_peers')
const revokeThroughTauri: PeerRevoker = (peerId) =>
  invoke<boolean>('revoke_trusted_peer', { peerId })

export function useTrustedPeers(
  loader: PeerLoader = loadFromTauri,
  revoker: PeerRevoker = revokeThroughTauri,
) {
  const peers = ref<TrustedPeerSummary[]>([])
  const confirmingPeer = ref<TrustedPeerSummary | null>(null)
  const error = ref<string | null>(null)
  const notice = ref<string | null>(null)
  const isLoading = ref(false)
  const isRevoking = ref(false)

  const hasPeers = computed(() => peers.value.length > 0)

  async function load() {
    isLoading.value = true
    error.value = null
    try {
      peers.value = await loader()
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isLoading.value = false
    }
  }

  function requestRevocation(peer: TrustedPeerSummary) {
    confirmingPeer.value = peer
    error.value = null
    notice.value = null
  }

  function cancelRevocation() {
    confirmingPeer.value = null
  }

  async function confirmRevocation() {
    const peer = confirmingPeer.value
    if (!peer) return
    isRevoking.value = true
    error.value = null
    notice.value = null
    try {
      await revoker(peer.id)
      peers.value = peers.value.filter(({ id }) => id !== peer.id)
      confirmingPeer.value = null
      notice.value = `${peer.displayName} can no longer access this library.`
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isRevoking.value = false
    }
  }

  return {
    cancelRevocation,
    confirmingPeer,
    confirmRevocation,
    error,
    hasPeers,
    isLoading,
    isRevoking,
    load,
    notice,
    peers,
    requestRevocation,
  }
}
