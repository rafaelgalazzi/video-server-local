import { describe, expect, it, vi } from 'vitest'
import {
  detectRuntime,
  useRuntimeBootstrap,
  type BrowserBootstrapAdapter,
} from './useRuntimeBootstrap'

function adapter(responses: Response[]): BrowserBootstrapAdapter {
  return {
    origin: 'https://media.local:8443',
    request: vi.fn(() => Promise.resolve(responses.shift() ?? new Response(null, { status: 500 }))),
  }
}

describe('useRuntimeBootstrap', () => {
  it('detects native only from the Tauri runtime marker', () => {
    expect(detectRuntime({ __TAURI_INTERNALS__: {} })).toBe('native')
    expect(detectRuntime({})).toBe('browser')
  })

  it('keeps native mode away from browser requests', async () => {
    const backend = adapter([])
    const runtime = useRuntimeBootstrap('native', backend)
    await runtime.loadBrowser()
    expect(backend.request).not.toHaveBeenCalled()
    expect(runtime.server.value).toBeNull()
  })

  it('uses same-origin cookie requests and reaches authenticated state', async () => {
    const backend = adapter([
      Response.json({ status: 'ok' }),
      Response.json({ libraryName: 'Movies', items: [], skippedEntries: 0 }),
    ])
    const runtime = useRuntimeBootstrap('browser', backend)
    await runtime.loadBrowser()
    expect(backend.request).toHaveBeenNthCalledWith(1, '/api/v1/health')
    expect(backend.request).toHaveBeenNthCalledWith(2, '/api/v1/library')
    expect(runtime.state.value).toBe('authenticated')
    expect(runtime.library.value?.libraryName).toBe('Movies')
    expect(runtime.server.value?.baseUrl).toBe('https://media.local:8443')
  })

  it('distinguishes pairing-required from disconnected and retries', async () => {
    const backend = adapter([Response.json({ status: 'ok' }), new Response(null, { status: 401 })])
    const runtime = useRuntimeBootstrap('browser', backend)
    await runtime.loadBrowser()
    expect(runtime.state.value).toBe('pairing-required')
    expect(runtime.error.value).toBeNull()

    backend.request = vi
      .fn()
      .mockRejectedValueOnce(new Error('offline'))
      .mockResolvedValueOnce(Response.json({ status: 'ok' }))
      .mockResolvedValueOnce(Response.json(null))
    await runtime.loadBrowser()
    expect(runtime.state.value).toBe('disconnected')
    expect(runtime.canRetry.value).toBe(true)
    await runtime.loadBrowser()
    expect(runtime.state.value).toBe('authenticated')
  })

  it('keeps the browser claim transient and authenticates through the resulting cookie', async () => {
    const receipt = {
      requestId: 'request-1',
      claimSecret: 'temporary-secret',
      verificationCode: '123456',
      expiresInSeconds: 120,
    }
    const backend = adapter([
      Response.json(receipt),
      new Response(null, { status: 204 }),
      Response.json({ status: 'ok' }),
      Response.json(null),
    ])
    const runtime = useRuntimeBootstrap('browser', backend)
    await runtime.beginPairing('Living Room Browser')
    expect(runtime.pairing.value?.verificationCode).toBe('123456')
    await runtime.finishPairing()
    expect(runtime.pairing.value).toBeNull()
    expect(runtime.state.value).toBe('authenticated')
    expect(backend.request).toHaveBeenNthCalledWith(
      2,
      '/api/v1/pairing/browser-claims',
      expect.objectContaining({ method: 'POST' }),
    )
  })
})
