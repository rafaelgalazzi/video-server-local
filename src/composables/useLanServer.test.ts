import { describe, expect, it, vi } from 'vitest'
import { useLanServer, type LanServerAdapter, type LanServerConfig } from './useLanServer'

const disabled: LanServerConfig = { enabled: false, address: null, port: 8443, dnsName: null }
function adapter(): LanServerAdapter {
  return {
    loadConfig: vi.fn(() => Promise.resolve(disabled)),
    loadStatus: vi.fn(() =>
      Promise.resolve({
        configured: false,
        active: false,
        endpoint: null,
        failure: null,
      }),
    ),
    addresses: vi.fn(() => Promise.resolve(['192.168.1.4'])),
    save: vi.fn(() => Promise.resolve()),
  }
}
describe('useLanServer', () => {
  it('loads disabled defaults and safe public addresses', async () => {
    const subject = useLanServer(adapter())
    await subject.load()
    expect(subject.config.value).toEqual(disabled)
    expect(subject.addresses.value).toEqual(['192.168.1.4'])
    expect(subject.statusLabel.value).toBe('LAN server disabled')
  })
  it('saves explicit configuration and reports restart requirement', async () => {
    const backend = adapter()
    const subject = useLanServer(backend)
    await subject.load()
    const enabled = { enabled: true, address: '192.168.1.4', port: 8443, dnsName: 'media.home' }
    await subject.save(enabled)
    expect(backend.save).toHaveBeenCalledWith(enabled)
    expect(subject.notice.value).toContain('Restart')
    expect(subject.statusLabel.value).toBe('LAN server pending restart')
  })
  it('contains load and save failures', async () => {
    const backend = adapter()
    backend.loadConfig = vi.fn(() => Promise.reject(new Error('unavailable')))
    const subject = useLanServer(backend)
    await subject.load()
    expect(subject.error.value).toBe('unavailable')
    backend.save = vi.fn(() => Promise.reject(new Error('invalid')))
    await subject.save(disabled)
    expect(subject.error.value).toBe('invalid')
  })
})
