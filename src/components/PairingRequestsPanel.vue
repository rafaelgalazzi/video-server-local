<script setup lang="ts">
import type { PendingPairing } from '../composables/usePairingRequests'

defineProps<{
  error: string | null
  isDeciding: (requestId: string) => boolean
  isLoading: boolean
  notice: string | null
  requests: PendingPairing[]
}>()

defineEmits<{
  approve: [request: PendingPairing]
  reject: [request: PendingPairing]
  retry: []
}>()

function formatCode(code: string) {
  return `${code.slice(0, 3)} ${code.slice(3)}`
}

function expiryLabel(seconds: number) {
  if (seconds <= 1) return 'Expires very soon'
  return `Expires in about ${seconds} seconds`
}
</script>

<template>
  <section class="pairing-panel" aria-labelledby="pairing-title">
    <div class="pairing-panel__heading">
      <div>
        <p class="section-label">Trusted devices</p>
        <h2 id="pairing-title">Pairing requests</h2>
        <p class="pairing-panel__summary">
          Approve only when this code matches the code shown on the requesting device.
        </p>
      </div>
      <button v-if="error" type="button" @click="$emit('retry')">Retry</button>
    </div>

    <p v-if="isLoading" class="feedback" role="status">Checking for pairing requestsâ€¦</p>
    <p v-else-if="error" class="feedback feedback--error" role="alert">{{ error }}</p>
    <p v-else-if="notice" class="feedback" role="status">{{ notice }}</p>

    <div v-if="requests.length" class="pairing-list">
      <article v-for="request in requests" :key="request.requestId" class="pairing-request">
        <div>
          <h3>{{ request.displayName }}</h3>
          <p>{{ expiryLabel(request.expiresInSeconds) }}</p>
        </div>
        <output
          class="pairing-request__code"
          :aria-label="`Verification code ${request.verificationCode}`"
        >
          {{ formatCode(request.verificationCode) }}
        </output>
        <div class="pairing-request__actions">
          <button
            type="button"
            :disabled="isDeciding(request.requestId)"
            @click="$emit('reject', request)"
          >
            Reject
          </button>
          <button
            class="primary-action"
            type="button"
            :disabled="isDeciding(request.requestId)"
            @click="$emit('approve', request)"
          >
            {{ isDeciding(request.requestId) ? 'Decidingâ€¦' : 'Allow' }}
          </button>
        </div>
      </article>
    </div>
    <p v-else-if="!isLoading && !error" class="pairing-panel__empty">
      No devices are waiting for approval.
    </p>
  </section>
</template>
