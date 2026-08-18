import { afterEach, describe, expect, it, vi } from 'vitest'
import { usePairingRequests, type PendingPairing } from './usePairingRequests'

const request: PendingPairing = {
  requestId: 'ls_pair_request',
  displayName: 'Living Room TV',
  verificationCode: '482914',
  expiresInSeconds: 120,
}

afterEach(() => vi.useRealTimers())

describe('usePairingRequests', () => {
  it('loads pending safe request metadata', async () => {
    const subject = usePairingRequests(() => Promise.resolve([request]))

    await subject.load()

    expect(subject.requests.value).toEqual([request])
    expect(subject.hasRequests.value).toBe(true)
    expect(subject.isLoading.value).toBe(false)
    expect(subject.error.value).toBeNull()
  })

  it('approves with the request ID and displayed code then removes the request', async () => {
    const approve = vi.fn(() => Promise.resolve())
    const subject = usePairingRequests(() => Promise.resolve([request]), approve)
    await subject.load()

    await subject.approve(request)

    expect(approve).toHaveBeenCalledWith('ls_pair_request', '482914')
    expect(subject.requests.value).toEqual([])
    expect(subject.notice.value).toBe('Living Room TV was approved.')
    expect(subject.isDeciding(request.requestId)).toBe(false)
  })

  it('retains a request when rejection fails', async () => {
    const reject = vi.fn(() => Promise.reject(new Error('request expired')))
    const subject = usePairingRequests(
      () => Promise.resolve([request]),
      () => Promise.resolve(),
      reject,
    )
    await subject.load()

    await subject.reject(request)

    expect(reject).toHaveBeenCalledWith('ls_pair_request')
    expect(subject.requests.value).toEqual([request])
    expect(subject.error.value).toBe('request expired')
  })

  it('polls only after a successful load and stops cleanly', async () => {
    vi.useFakeTimers()
    const loader = vi.fn(() => Promise.resolve<PendingPairing[]>([]))
    const subject = usePairingRequests(loader, undefined, undefined, 1_000)

    await subject.startPolling()
    expect(loader).toHaveBeenCalledTimes(1)

    await vi.advanceTimersByTimeAsync(2_000)
    expect(loader).toHaveBeenCalledTimes(3)

    subject.stopPolling()
    await vi.advanceTimersByTimeAsync(2_000)
    expect(loader).toHaveBeenCalledTimes(3)
  })

  it('does not poll when the initial native load fails', async () => {
    vi.useFakeTimers()
    const loader = vi.fn(() => Promise.reject(new Error('native unavailable')))
    const subject = usePairingRequests(loader, undefined, undefined, 1_000)

    await subject.startPolling()
    await vi.advanceTimersByTimeAsync(2_000)

    expect(loader).toHaveBeenCalledTimes(1)
    expect(subject.error.value).toBe('native unavailable')
  })
})
