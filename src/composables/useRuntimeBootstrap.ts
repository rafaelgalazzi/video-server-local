import { computed, ref } from 'vue'
import type { LibraryScan } from './useMediaLibrary'
import type { ServerInfo } from './useServerStatus'

export type RuntimeMode = 'native' | 'browser'
export type BrowserBootstrapState =
  'idle' | 'bootstrapping' | 'pairing-required' | 'authenticated' | 'disconnected'

export interface BrowserBootstrapAdapter {
  origin: string
  request: (path: string, init?: RequestInit) => Promise<Response>
}

export interface BrowserPairingReceipt {
  requestId: string
  claimSecret: string
  verificationCode: string
  expiresInSeconds: number
}

function browserAdapter(): BrowserBootstrapAdapter {
  return {
    origin: window.location.origin,
    request: (path, init) => fetch(path, { ...init, credentials: 'same-origin' }),
  }
}

export function detectRuntime(global: unknown = globalThis): RuntimeMode {
  const candidate = global as { __TAURI_INTERNALS__?: unknown }
  return candidate.__TAURI_INTERNALS__ ? 'native' : 'browser'
}

export function useRuntimeBootstrap(
  mode: RuntimeMode = detectRuntime(),
  adapter: BrowserBootstrapAdapter | null = mode === 'browser' ? browserAdapter() : null,
) {
  const state = ref<BrowserBootstrapState>('idle')
  const error = ref<string | null>(null)
  const library = ref<LibraryScan | null>(null)
  const server = ref<ServerInfo | null>(
    mode === 'browser' && adapter
      ? { baseUrl: adapter.origin, bindScope: 'lan', lanAvailable: true }
      : null,
  )
  const pairing = ref<BrowserPairingReceipt | null>(null)
  const isPairing = ref(false)

  const isNative = computed(() => mode === 'native')
  const canRetry = computed(() => state.value === 'disconnected')

  async function loadBrowser() {
    if (mode !== 'browser' || !adapter) return
    state.value = 'bootstrapping'
    error.value = null
    try {
      const health = await adapter.request('/api/v1/health')
      if (!health.ok) throw new Error('The LocalStream server is unavailable.')
      const response = await adapter.request('/api/v1/library')
      if (response.status === 401) {
        library.value = null
        state.value = 'pairing-required'
        return
      }
      if (!response.ok) throw new Error('The media library could not be loaded.')
      library.value = (await response.json()) as LibraryScan | null
      state.value = 'authenticated'
    } catch (reason) {
      library.value = null
      state.value = 'disconnected'
      error.value = reason instanceof Error ? reason.message : String(reason)
    }
  }

  async function beginPairing(displayName: string) {
    if (mode !== 'browser' || !adapter || isPairing.value) return
    isPairing.value = true
    error.value = null
    try {
      const response = await adapter.request('/api/v1/pairing/requests', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({ displayName }),
      })
      if (!response.ok) throw new Error('Pairing could not be started.')
      pairing.value = (await response.json()) as BrowserPairingReceipt
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isPairing.value = false
    }
  }

  async function finishPairing() {
    if (mode !== 'browser' || !adapter || !pairing.value || isPairing.value) return
    isPairing.value = true
    error.value = null
    try {
      const response = await adapter.request('/api/v1/pairing/browser-claims', {
        method: 'POST',
        headers: { 'content-type': 'application/json' },
        body: JSON.stringify({
          requestId: pairing.value.requestId,
          claimSecret: pairing.value.claimSecret,
        }),
      })
      if (!response.ok) {
        throw new Error(
          'Pairing is not approved yet. Compare the code on the desktop and try again.',
        )
      }
      pairing.value = null
      await loadBrowser()
    } catch (reason) {
      error.value = reason instanceof Error ? reason.message : String(reason)
    } finally {
      isPairing.value = false
    }
  }

  return {
    beginPairing,
    canRetry,
    error,
    finishPairing,
    isNative,
    isPairing,
    library,
    loadBrowser,
    mode,
    pairing,
    server,
    state,
  }
}
