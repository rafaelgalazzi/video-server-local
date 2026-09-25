import { describe, expect, it, vi } from 'vitest'
import { usePortableRuntime, type PortableRuntimeStatus } from './usePortableRuntime'

const ready: PortableRuntimeStatus = { state: 'ready', failure: null }

describe('usePortableRuntime', () => {
  it('reports a verified portable payload', async () => {
    const subject = usePortableRuntime(() => Promise.resolve(ready))
    await subject.load()
    expect(subject.isAvailable.value).toBe(true)
    expect(subject.hasFailed.value).toBe(false)
    expect(subject.error.value).toBeNull()
  })

  it('retains a safe portable failure code', async () => {
    const subject = usePortableRuntime(() =>
      Promise.resolve({ state: 'failed', failure: 'integrity_check_failed' }),
    )
    await subject.load()
    expect(subject.hasFailed.value).toBe(true)
    expect(subject.status.value?.failure).toBe('integrity_check_failed')
  })

  it('contains adapter failures', async () => {
    const loader = vi.fn().mockRejectedValue(new Error('native unavailable'))
    const subject = usePortableRuntime(loader)
    await subject.load()
    expect(subject.error.value).toBe('native unavailable')
    expect(subject.status.value).toBeNull()
  })
})
