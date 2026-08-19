import { describe, expect, it } from 'vitest'
import { useTrustOnboarding } from './useTrustOnboarding'

describe('useTrustOnboarding', () => {
  it('requires a fresh explicit fingerprint-comparison acknowledgement', () => {
    const subject = useTrustOnboarding()
    expect(subject.canExport.value).toBe(false)
    subject.fingerprintAcknowledged.value = true
    expect(subject.canExport.value).toBe(true)
    subject.resetAcknowledgement()
    expect(subject.canExport.value).toBe(false)
  })
})
