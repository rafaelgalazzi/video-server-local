import { ref } from 'vue'
import { describe, expect, it } from 'vitest'
import type { MediaItem } from './useMediaLibrary'
import { usePlayback } from './usePlayback'

const item: MediaItem = {
  id: 'opaque/id with spaces',
  title: 'Night Drive',
  extension: 'mp4',
  sizeBytes: 42,
}

describe('usePlayback', () => {
  it('builds a versioned stream URL from the server and encoded opaque ID', () => {
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152/', bindScope: 'loopback', lanAvailable: false }),
    )

    subject.play(item)

    expect(subject.streamUrl.value).toBe(
      'http://127.0.0.1:49152/api/v1/media/opaque%2Fid%20with%20spaces/stream',
    )
    expect(subject.status.value).toBe('loading')
    expect(subject.selectedItem.value).toEqual(item)
  })

  it('reports unavailable playback without selecting media', () => {
    const subject = usePlayback(ref(null))

    subject.play(item)

    expect(subject.canPlay.value).toBe(false)
    expect(subject.selectedItem.value).toBeNull()
    expect(subject.status.value).toBe('error')
    expect(subject.error.value).toContain('unavailable')
  })

  it('tracks playing and browser compatibility failure states', () => {
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
    )
    subject.play(item)

    subject.markPlaying()
    expect(subject.status.value).toBe('playing')
    expect(subject.error.value).toBeNull()

    subject.markError()
    expect(subject.status.value).toBe('error')
    expect(subject.error.value).toContain('could not be played directly')
  })

  it('clears stale playback state', () => {
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
    )
    subject.play(item)

    subject.clear()

    expect(subject.selectedItem.value).toBeNull()
    expect(subject.streamUrl.value).toBeNull()
    expect(subject.status.value).toBe('idle')
    expect(subject.error.value).toBeNull()
  })
})
