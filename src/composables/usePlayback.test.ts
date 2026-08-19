import { ref } from 'vue'
import { describe, expect, it } from 'vitest'
import type { MediaItem } from './useMediaLibrary'
import { usePlayback } from './usePlayback'

const item: MediaItem = {
  id: 'opaque/id with spaces',
  title: 'Night Drive',
  extension: 'mp4',
  sizeBytes: 42,
  metadata: null,
  probeStatus: 'not_probed',
  selectedAudioTrackId: null,
  subtitleMode: 'automatic',
  selectedSubtitleTrackId: null,
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

  it('labels defaults and persists a validated audio choice', async () => {
    const saved: Array<[string, string | null]> = []
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
      ref(true),
      (mediaId, trackId) => {
        saved.push([mediaId, trackId])
        return Promise.resolve()
      },
    )
    const dualAudio: MediaItem = {
      ...item,
      id: 'dual-audio',
      metadata: {
        container: 'matroska',
        durationMillis: 1000,
        video: null,
        subtitleTracks: [],
        audioTracks: [
          {
            id: 'audio-eng',
            codec: 'aac',
            channels: 2,
            language: 'eng',
            title: null,
            isDefault: true,
          },
          {
            id: 'audio-por',
            codec: 'ac3',
            channels: 6,
            language: 'por',
            title: 'Brazilian Portuguese',
            isDefault: false,
          },
        ],
      },
    }
    subject.play(dualAudio)

    expect(subject.selectedAudioTrackId.value).toBe('audio-eng')
    expect(subject.audioOptions.value[0].label).toBe('ENG · AAC · 2 channels · Default')
    expect(subject.audioOptions.value[1].label).toBe('Brazilian Portuguese · AC3 · 6 channels')

    await subject.selectAudioTrack('audio-por')

    expect(saved).toEqual([['dual-audio', 'audio-por']])
    expect(subject.selectedAudioTrackId.value).toBe('audio-por')
  })

  it('rejects unavailable and unknown audio choices without persisting', async () => {
    let calls = 0
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
      ref(false),
      () => {
        calls += 1
        return Promise.resolve()
      },
    )
    subject.play({
      ...item,
      metadata: {
        container: 'matroska',
        durationMillis: null,
        video: null,
        subtitleTracks: [],
        audioTracks: [
          {
            id: 'known',
            codec: 'aac',
            channels: null,
            language: null,
            title: null,
            isDefault: false,
          },
        ],
      },
    })

    expect(subject.audioOptions.value[0].label).toBe('Audio track 1 · AAC')

    await subject.selectAudioTrack('unknown')

    expect(calls).toBe(0)
    expect(subject.audioSelectionError.value).toContain('unavailable')
  })

  it('selects forced text subtitles automatically and persists Off explicitly', async () => {
    const saved: Array<[string, string, string | null]> = []
    const subject = usePlayback(
      ref({ baseUrl: 'https://192.168.1.20:8443', bindScope: 'lan', lanAvailable: true }),
      ref(true),
      () => Promise.resolve(),
      (mediaId, mode, trackId) => {
        saved.push([mediaId, mode, trackId])
        return Promise.resolve()
      },
    )
    subject.play({
      ...item,
      id: 'mkv/subtitles',
      metadata: {
        container: 'matroska',
        durationMillis: 1000,
        video: null,
        audioTracks: [],
        subtitleTracks: [
          {
            id: 'sub-default',
            codec: 'subrip',
            language: 'por',
            title: null,
            isDefault: true,
            isForced: false,
            kind: 'text',
          },
          {
            id: 'sub-forced',
            codec: 'ass',
            language: 'eng',
            title: 'Signs',
            isDefault: false,
            isForced: true,
            kind: 'text',
          },
        ],
      },
    })

    expect(subject.activeSubtitleTrack.value?.id).toBe('sub-forced')
    expect(subject.subtitleOptions.value[2].label).toBe('POR · SUBRIP · Default')
    expect(subject.subtitleOptions.value[3].label).toBe('Signs · ASS · Forced')
    expect(subject.subtitleTrackUrl.value).toBe(
      'https://192.168.1.20:8443/api/v1/media/mkv%2Fsubtitles/subtitles/sub-forced',
    )

    await subject.selectSubtitle('off')

    expect(saved).toEqual([['mkv/subtitles', 'off', null]])
    expect(subject.activeSubtitleTrack.value).toBeNull()
    expect(subject.subtitleTrackUrl.value).toBeNull()
  })

  it('reports bitmap delivery requirements and rejects malformed subtitle IDs', async () => {
    let calls = 0
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
      ref(true),
      () => Promise.resolve(),
      () => {
        calls += 1
        return Promise.resolve()
      },
    )
    subject.play({
      ...item,
      subtitleMode: 'track',
      selectedSubtitleTrackId: 'pgs',
      metadata: {
        container: 'matroska',
        durationMillis: null,
        video: null,
        audioTracks: [],
        subtitleTracks: [
          {
            id: 'pgs',
            codec: 'hdmv_pgs_subtitle',
            language: null,
            title: null,
            isDefault: false,
            isForced: false,
            kind: 'bitmap',
          },
        ],
      },
    })

    expect(subject.subtitleOptions.value[2].label).toContain('Bitmap')
    expect(subject.subtitleDeliveryNotice.value).toContain('requires video conversion')
    expect(subject.subtitleTrackUrl.value).toBeNull()

    await subject.selectSubtitle('track:unknown')
    expect(calls).toBe(0)
    expect(subject.subtitleSelectionError.value).toContain('invalid')
  })
})
