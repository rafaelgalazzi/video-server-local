import { describe, expect, it, vi } from 'vitest'
import { useDatabaseMaintenance } from './useDatabaseMaintenance'

describe('useDatabaseMaintenance', () => {
  it('reports a successful reset', async () => {
    const clearer = vi.fn(() => Promise.resolve())
    const subject = useDatabaseMaintenance(clearer)

    await expect(subject.clear()).resolves.toBe(true)

    expect(clearer).toHaveBeenCalledOnce()
    expect(subject.notice.value).toBe('Local database cleared.')
    expect(subject.error.value).toBeNull()
    expect(subject.isClearing.value).toBe(false)
  })

  it('contains reset failures', async () => {
    const subject = useDatabaseMaintenance(() => Promise.reject(new Error('reset failed')))

    await expect(subject.clear()).resolves.toBe(false)

    expect(subject.error.value).toBe('reset failed')
    expect(subject.notice.value).toBeNull()
    expect(subject.isClearing.value).toBe(false)
  })
})
