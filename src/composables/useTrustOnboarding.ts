import { computed, ref } from 'vue'

export function useTrustOnboarding() {
  const fingerprintAcknowledged = ref(false)
  const canExport = computed(() => fingerprintAcknowledged.value)
  function resetAcknowledgement() {
    fingerprintAcknowledged.value = false
  }
  return { canExport, fingerprintAcknowledged, resetAcknowledgement }
}
