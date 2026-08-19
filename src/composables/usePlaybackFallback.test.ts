import { ref } from 'vue'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { MediaItem } from './useMediaLibrary'

const { invoke } = vi.hoisted(() => ({ invoke: vi.fn() }))
vi.mock('@tauri-apps/api/core', () => ({ invoke }))

import { usePlayback } from './usePlayback'

const item: MediaItem = {
  id: 'mkv/id',
  title: 'Movie',
  extension: 'mkv',
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

    await vi.waitFor(() => expect(subject.playbackProgress.value).toBe(1000))
    expect(subject.streamUrl.value).toBe(
      'http://127.0.0.1:49152/api/v1/playback/jobs/job-id/output/output.webm',
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
})
