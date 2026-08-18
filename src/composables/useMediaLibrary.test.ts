import { describe, expect, it } from 'vitest'
import { useMediaLibrary } from './useMediaLibrary'

describe('useMediaLibrary', () => {
  it('stores a safely shaped library scan', async () => {
    const subject = useMediaLibrary(() =>
      Promise.resolve({
        libraryName: 'Videos',
        items: [
          {
            id: 'f3e97c24-8e02-56e1-824e-750c21c43291',
            title: 'Night Drive',
            extension: 'mp4',
            sizeBytes: 42,
          },
        ],
        skippedEntries: 0,
      }),
    )

    await subject.selectLibrary()

    expect(subject.library.value?.libraryName).toBe('Videos')
    expect(subject.itemCountLabel.value).toBe('1 video')
    expect(subject.error.value).toBeNull()
    expect(subject.isScanning.value).toBe(false)
  })

  it('keeps the current library when selection is cancelled', async () => {
    const subject = useMediaLibrary(() => Promise.resolve(null))

    await subject.selectLibrary()

    expect(subject.library.value).toBeNull()
    expect(subject.notice.value).toBe('Folder selection cancelled.')
  })

  it('contains adapter failures', async () => {
    const subject = useMediaLibrary(() => Promise.reject(new Error('scan failed')))

    await subject.selectLibrary()

    expect(subject.library.value).toBeNull()
    expect(subject.error.value).toBe('scan failed')
    expect(subject.isScanning.value).toBe(false)
  })
})
