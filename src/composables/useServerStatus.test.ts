import { describe, expect, it } from 'vitest'
import { useServerStatus } from './useServerStatus'

describe('useServerStatus', () => {
  it('loads loopback server information', async () => {
    const subject = useServerStatus(() =>
      Promise.resolve({
        baseUrl: 'http://127.0.0.1:49152',
        bindScope: 'loopback',
        lanAvailable: false,
      }),
    )

    await subject.load()

    expect(subject.server.value?.baseUrl).toBe('http://127.0.0.1:49152')
    expect(subject.statusLabel.value).toBe('Private API ready')
    expect(subject.error.value).toBeNull()
  })

  it('contains adapter failures', async () => {
    const subject = useServerStatus(() => Promise.reject(new Error('server unavailable')))

    await subject.load()

    expect(subject.server.value).toBeNull()
    expect(subject.error.value).toBe('server unavailable')
    expect(subject.statusLabel.value).toBe('API unavailable')
  })
})
