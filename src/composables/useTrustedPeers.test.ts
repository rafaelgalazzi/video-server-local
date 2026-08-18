import { describe, expect, it, vi } from 'vitest'
import { useTrustedPeers, type TrustedPeerSummary } from './useTrustedPeers'

const peer: TrustedPeerSummary = {
  id: 'peer-1',
  displayName: 'Living Room TV',
  capability: 'library_read',
  createdAt: 1_777_000_000,
}

describe('useTrustedPeers', () => {
  it('loads safe active peer summaries', async () => {
    const subject = useTrustedPeers(() => Promise.resolve([peer]))

    await subject.load()

    expect(subject.peers.value).toEqual([peer])
    expect(subject.hasPeers.value).toBe(true)
    expect(subject.error.value).toBeNull()
  })

  it('requires confirmation before revoking and supports cancellation', async () => {
    const revoke = vi.fn(() => Promise.resolve(true))
    const subject = useTrustedPeers(() => Promise.resolve([peer]), revoke)
    await subject.load()

    subject.requestRevocation(peer)
    expect(subject.confirmingPeer.value).toEqual(peer)
    expect(revoke).not.toHaveBeenCalled()

    subject.cancelRevocation()
    expect(subject.confirmingPeer.value).toBeNull()
  })

  it('revokes the confirmed peer and removes it locally', async () => {
    const revoke = vi.fn(() => Promise.resolve(true))
    const subject = useTrustedPeers(() => Promise.resolve([peer]), revoke)
    await subject.load()
    subject.requestRevocation(peer)

    await subject.confirmRevocation()

    expect(revoke).toHaveBeenCalledWith('peer-1')
    expect(subject.peers.value).toEqual([])
    expect(subject.confirmingPeer.value).toBeNull()
    expect(subject.notice.value).toContain('can no longer access')
  })

  it('retains the peer and confirmation when revocation fails', async () => {
    const subject = useTrustedPeers(
      () => Promise.resolve([peer]),
      () => Promise.reject(new Error('credential store unavailable')),
    )
    await subject.load()
    subject.requestRevocation(peer)

    await subject.confirmRevocation()

    expect(subject.peers.value).toEqual([peer])
    expect(subject.confirmingPeer.value).toEqual(peer)
    expect(subject.error.value).toBe('credential store unavailable')
    expect(subject.isRevoking.value).toBe(false)
  })
})
