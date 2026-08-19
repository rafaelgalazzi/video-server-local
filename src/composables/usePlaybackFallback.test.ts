import { ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { MediaItem } from './useMediaLibrary'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))

import { usePlayback } from './usePlayback'

const item: MediaItem = {
  id: 'mkv/id',
  title: 'Movie',
  extension: 'mov',
  sizeBytes: 10,
  metadata: null,
  probeStatus: 'available',
  selectedAudioTrackId: null,
  subtitleMode: 'off',
  selectedSubtitleTrackId: null,
}

describe('usePlayback fallback', () => {
  beforeEach(() => {
    invoke.mockReset()
    vi.stubGlobal('document', {
      createElement: () => ({ canPlayType: () => 'probably' }),
    })
    vi.stubGlobal(
      'fetch',
      vi.fn(() => Promise.resolve({ ok: true })),
    )
  })

  it('configures an item without starting playback until explicitly requested', async () => {
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
      ref(true),
      undefined,
      undefined,
      ref(true),
    )

    subject.select({ ...item, extension: 'mkv' })

    expect(subject.status.value).toBe('idle')
    expect(subject.streamUrl.value).toBeNull()
    expect(invoke).not.toHaveBeenCalledWith('prepare_hls', expect.anything())

    subject.start()
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith('prepare_hls', { mediaId: item.id }))
  })

  it('prepares and exposes a completed opaque fallback output', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'prepare_playback') {
        return Promise.resolve({ method: 'remux', jobId: 'job-id', outputName: 'output.webm' })
      }
      if (command === 'playback_job') {
        return Promise.resolve({ state: 'completed', progressPermille: 1000 })
      }
      return Promise.resolve(true)
    })
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
      ref(true),
      undefined,
      undefined,
      ref(true),
    )

    subject.play({ ...item })
    expect(subject.streamUrl.value).toBeNull()

    await vi.waitFor(() => expect(subject.playbackProgress.value).toBe(1000))
    expect(subject.streamUrl.value).toBe(
      'http://127.0.0.1:49152/api/v1/playback/jobs/job-id/output/output.webm',
    )
  })

  it('withholds the source URL until direct play is confirmed', async () => {
    let resolvePreparation: ((value: PlaybackPreparationResult) => void) | undefined
    type PlaybackPreparationResult = {
      method: 'direct_play'
      jobId: null
      outputName: null
    }
    invoke.mockImplementation((command: string) => {
      if (command === 'prepare_playback') {
        return new Promise((resolve) => {
          resolvePreparation = resolve
        })
      }
      return Promise.resolve(true)
    })
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
      ref(true),
      undefined,
      undefined,
      ref(true),
    )

    subject.play({ ...item })
    expect(subject.streamUrl.value).toBeNull()

    resolvePreparation?.({ method: 'direct_play', jobId: null, outputName: null })
    await vi.waitFor(() =>
      expect(subject.streamUrl.value).toBe('http://127.0.0.1:49152/api/v1/media/mkv%2Fid/stream'),
    )
  })

  it('cancels and releases an active fallback when closed', async () => {
    let resolveJob: ((value: unknown) => void) | undefined
    invoke.mockImplementation((command: string) => {
      if (command === 'prepare_playback') {
        return Promise.resolve({ method: 'transcode', jobId: 'job-id', outputName: 'output.mp4' })
      }
      if (command === 'playback_job') {
        return new Promise((resolve) => {
          resolveJob = resolve
        })
      }
      return Promise.resolve(true)
    })
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
      ref(true),
      undefined,
      undefined,
      ref(true),
    )
    subject.play({ ...item })
    await Promise.resolve()
    await Promise.resolve()

    subject.clear()
    resolveJob?.({ state: 'cancelled', progressPermille: 0 })
    await vi.waitFor(() =>
      expect(invoke).toHaveBeenCalledWith('release_playback', { jobId: 'job-id' }),
    )

    expect(invoke).toHaveBeenCalledWith('cancel_playback', { jobId: 'job-id' })
    expect(subject.status.value).toBe('idle')
  })

  it('starts MKV playback from the first available HLS playlist', async () => {
    invoke.mockImplementation((command: string) => {
      if (command === 'prepare_hls') {
        return Promise.resolve({
          jobId: 'hls-job',
          playlistName: 'index.m3u8',
          videoMode: 'copy',
        })
      }
      return Promise.resolve({ state: 'running', progressPermille: 0 })
    })
    const subject = usePlayback(
      ref({ baseUrl: 'http://127.0.0.1:49152', bindScope: 'loopback', lanAvailable: false }),
      ref(true),
      undefined,
      undefined,
      ref(true),
    )

    subject.play({ ...item, extension: 'mkv' })

    await vi.waitFor(() =>
      expect(subject.streamUrl.value).toBe(
        'http://127.0.0.1:49152/api/v1/playback/hls/hls-job/index.m3u8',
      ),
    )
    expect(invoke).toHaveBeenCalledWith('prepare_hls', { mediaId: item.id })
    expect(subject.preparationNotice.value).toContain('converting only audio')
  })
})
