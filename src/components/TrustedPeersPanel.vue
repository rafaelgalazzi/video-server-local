<script setup lang="ts">
import type { TrustedPeerSummary } from '../composables/useTrustedPeers'

defineProps<{
  confirmingPeer: TrustedPeerSummary | null
  error: string | null
  isLoading: boolean
  isRevoking: boolean
  notice: string | null
  peers: TrustedPeerSummary[]
}>()

defineEmits<{
  cancel: []
  confirm: []
  refresh: []
  revoke: [peer: TrustedPeerSummary]
}>()

const dateFormatter = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium' })

function pairedDate(createdAt: number) {
  return dateFormatter.format(new Date(createdAt * 1_000))
}
</script>

<template>
  <section class="trusted-peers" aria-labelledby="trusted-peers-title">
    <div class="trusted-peers__heading">
      <div>
        <p class="section-label">Access control</p>
        <h2 id="trusted-peers-title">Trusted devices</h2>
        <p class="trusted-peers__summary">
          Revocation immediately invalidates that device's credential.
        </p>
      </div>
      <button type="button" :disabled="isLoading" @click="$emit('refresh')">Refresh</button>
    </div>

    <p v-if="isLoading" class="feedback" role="status">Loading trusted devices...</p>
    <p v-else-if="error" class="feedback feedback--error" role="alert">{{ error }}</p>
    <p v-else-if="notice" class="feedback" role="status">{{ notice }}</p>

    <div
      v-if="confirmingPeer"
      class="revocation-confirmation"
      role="alertdialog"
      aria-modal="false"
      aria-labelledby="revocation-title"
    >
      <div>
        <h3 id="revocation-title">Revoke {{ confirmingPeer.displayName }}?</h3>
        <p>The device will need to pair again before it can access your library.</p>
      </div>
      <div class="revocation-confirmation__actions">
        <button type="button" :disabled="isRevoking" @click="$emit('cancel')">Cancel</button>
        <button
          class="danger-action"
          type="button"
          :disabled="isRevoking"
          @click="$emit('confirm')"
        >
          {{ isRevoking ? 'Revoking...' : 'Confirm revoke' }}
        </button>
      </div>
    </div>

    <div v-if="peers.length" class="trusted-peer-list">
      <article v-for="peer in peers" :key="peer.id" class="trusted-peer-row">
        <div>
          <h3>{{ peer.displayName }}</h3>
          <p>Library access | Paired {{ pairedDate(peer.createdAt) }}</p>
        </div>
        <button type="button" :disabled="isRevoking" @click="$emit('revoke', peer)">
          Revoke access
        </button>
      </article>
    </div>
    <p v-else-if="!isLoading && !error" class="trusted-peers__empty">
      No devices are currently trusted.
    </p>
  </section>
</template>
