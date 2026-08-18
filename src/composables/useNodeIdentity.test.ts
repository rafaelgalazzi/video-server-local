import { describe, expect, it } from 'vitest'
import { useNodeIdentity } from './useNodeIdentity'

describe('useNodeIdentity', () => {
  it('loads only the safe public identity summary', async () => {
    const subject = useNodeIdentity(() =>
      Promise.resolve({
        nodeId: 'ls_node_example',
        fingerprint: 'AA:BB:CC:DD',
      }),
    )

    await subject.load()

    expect(subject.identity.value).toEqual({
      nodeId: 'ls_node_example',
      fingerprint: 'AA:BB:CC:DD',
    })
    expect(subject.statusLabel.value).toBe('Node identity ready')
    expect(subject.error.value).toBeNull()
  })

  it('contains adapter failures and clears stale identity', async () => {
    let fail = false
    const subject = useNodeIdentity(() => {
      if (fail) return Promise.reject(new Error('protected store unavailable'))
      return Promise.resolve({ nodeId: 'ls_node_example', fingerprint: 'AA:BB' })
    })
    await subject.load()
    fail = true

    await subject.load()

    expect(subject.identity.value).toBeNull()
    expect(subject.error.value).toBe('protected store unavailable')
    expect(subject.statusLabel.value).toBe('Node identity unavailable')
  })

  it('requires confirmation before resetting and reports revoked devices', async () => {
    let resets = 0
    const subject = useNodeIdentity(
      () => Promise.resolve({ nodeId: 'ls_node_example', fingerprint: 'AA:BB' }),
      () => {
        resets += 1
        return Promise.resolve(2)
      },
    )
    await subject.load()

    await subject.confirmReset()
    expect(resets).toBe(0)

    subject.requestReset()
    await subject.confirmReset()

    expect(resets).toBe(1)
    expect(subject.identity.value).toBeNull()
    expect(subject.notice.value).toContain('2 trusted devices revoked')
    expect(subject.isConfirmingReset.value).toBe(false)
  })

  it('supports cancellation and retains confirmation after reset failure', async () => {
    const subject = useNodeIdentity(
      () => Promise.resolve({ nodeId: 'ls_node_example', fingerprint: 'AA:BB' }),
      () => Promise.reject(new Error('protected store unavailable')),
    )
    await subject.load()

    subject.requestReset()
    subject.cancelReset()
    await subject.confirmReset()
    expect(subject.error.value).toBeNull()

    subject.requestReset()
    await subject.confirmReset()
    expect(subject.identity.value?.nodeId).toBe('ls_node_example')
    expect(subject.error.value).toBe('protected store unavailable')
    expect(subject.isConfirmingReset.value).toBe(true)
  })

  it('reports certificate export success and cancellation without receiving path data', async () => {
    let exported = true
    const subject = useNodeIdentity(
      () => Promise.resolve({ nodeId: 'ls_node_example', fingerprint: 'AA:BB' }),
      () => Promise.resolve(0),
      () => Promise.resolve(exported),
    )
    await subject.load()

    await subject.exportRootCertificate()
    expect(subject.notice.value).toContain('Root certificate exported')

    exported = false
    await subject.exportRootCertificate()
    expect(subject.notice.value).toBe('Certificate export canceled.')
  })

  it('contains certificate export failures and requires a loaded identity', async () => {
    let calls = 0
    const subject = useNodeIdentity(
      () => Promise.resolve({ nodeId: 'ls_node_example', fingerprint: 'AA:BB' }),
      () => Promise.resolve(0),
      () => {
        calls += 1
        return Promise.reject(new Error('certificate export unavailable'))
      },
    )

    await subject.exportRootCertificate()
    expect(calls).toBe(0)

    await subject.load()
    await subject.exportRootCertificate()
    expect(calls).toBe(1)
    expect(subject.error.value).toBe('certificate export unavailable')
    expect(subject.identity.value?.nodeId).toBe('ls_node_example')
  })
})
