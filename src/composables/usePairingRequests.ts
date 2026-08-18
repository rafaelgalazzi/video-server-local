import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export interface PendingPairing {
  requestId: string
  displayName: string
  verificationCode: string
  expiresInSeconds: number
}

type PairingLoader = () => Promise<PendingPairing[]>
type PairingApprover = (requestId: string, verificationCode: string) => Promise<void>
type PairingRejecter = (requestId: string) => Promise<void>

const loadFromTauri: PairingLoader = () => invoke<PendingPairing[]>('pending_pairings')
const approveThroughTauri: PairingApprover = (requestId, verificationCode) =>
  invoke('approve_pairing', { requestId, verificationCode })
const rejectThroughTauri: PairingRejecter = (requestId) => invoke('reject_pairing', { requestId })

export function usePairingRequests(
  loader: PairingLoader = loadFromTauri,
  approver: PairingApprover = approveThroughTauri,
  rejecter: PairingRejecter = rejectThroughTauri,
  pollIntervalMilliseconds = 5_000,
) {
  const requests = ref<PendingPairing[]>([])
  const error = ref<string | null>(null)
  const notice = ref<string | null>(null)
  const isLoading = ref(false)
  const decidingIds = ref(new Set<string>())
  let pollingTimer: ReturnType<typeof setInterval> | null = null

  const hasRequests = computed(() => requests.value.length > 0)

  async function load(silent = false) {
    if (!silent) isLoading.value = true
    try {
      requests.value = await loader()
      error.value = null
      return true
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
      return false
    } finally {
      if (!silent) isLoading.value = false
    }
  }

  async function approve(request: PendingPairing) {
    await decide(request, 'approve')
  }

  async function reject(request: PendingPairing) {
    await decide(request, 'reject')
  }

  async function decide(request: PendingPairing, decision: 'approve' | 'reject') {
    decidingIds.value.add(request.requestId)
    error.value = null
    notice.value = null
    try {
      if (decision === 'approve') {
        await approver(request.requestId, request.verificationCode)
      } else {
        await rejecter(request.requestId)
      }
      requests.value = requests.value.filter(({ requestId }) => requestId !== request.requestId)
      notice.value =
        decision === 'approve'
          ? `${request.displayName} was approved.`
          : `${request.displayName} was rejected.`
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      decidingIds.value.delete(request.requestId)
    }
  }

  async function startPolling() {
    stopPolling()
    if (!(await load())) return
    pollingTimer = setInterval(() => void load(true), pollIntervalMilliseconds)
  }

  function stopPolling() {
    if (pollingTimer !== null) {
      clearInterval(pollingTimer)
      pollingTimer = null
    }
  }

  function isDeciding(requestId: string) {
    return decidingIds.value.has(requestId)
  }

  return {
    approve,
    error,
    hasRequests,
    isDeciding,
    isLoading,
    load,
    notice,
    reject,
    requests,
    startPolling,
    stopPolling,
  }
}
