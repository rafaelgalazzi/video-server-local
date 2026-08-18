import { describe, expect, it } from 'vitest'
import { useAppInfo } from './useAppInfo'

describe('useAppInfo', () => {
  it('loads application information from its adapter', async () => {
    const subject = useAppInfo(() =>
      Promise.resolve({
        name: 'LocalStream',
        version: '0.1.0',
        localFirst: true,
      }),
    )

    await subject.load()

    expect(subject.appInfo.value).toEqual({
      name: 'LocalStream',
      version: '0.1.0',
      localFirst: true,
    })
    expect(subject.runtimeLabel.value).toBe('LocalStream native core ready')
    expect(subject.error.value).toBeNull()
    expect(subject.isLoading.value).toBe(false)
  })

  it('contains adapter failures without throwing', async () => {
    const subject = useAppInfo(() => Promise.reject(new Error('adapter unavailable')))

    await subject.load()

    expect(subject.appInfo.value).toBeNull()
    expect(subject.error.value).toBe('adapter unavailable')
    expect(subject.isLoading.value).toBe(false)
  })
})
