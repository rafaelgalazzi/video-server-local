<script setup lang="ts">
import type { NodeIdentitySummary } from '../composables/useNodeIdentity'

defineProps<{
  error: string | null
  identity: NodeIdentitySummary | null
  isConfirmingReset: boolean
  isExporting: boolean
  isResetting: boolean
  notice: string | null
  statusLabel: string
}>()

defineEmits<{
  cancelReset: []
  confirmReset: []
  exportCertificate: []
  reset: []
}>()
</script>

<template>
  <section class="identity-panel" aria-labelledby="node-identity-title">
    <div>
      <p class="section-label">{{ statusLabel }}</p>
      <h2 id="node-identity-title">This LocalStream node</h2>
      <p class="identity-panel__summary">
        Compare this fingerprint only on this trusted desktop during future device onboarding.
      </p>
    </div>
    <dl v-if="identity" class="identity-panel__details">
      <div>
        <dt>Node ID</dt>
        <dd>{{ identity.nodeId }}</dd>
      </div>
      <div>
        <dt>Root fingerprint</dt>
        <dd>{{ identity.fingerprint }}</dd>
      </div>
    </dl>
    <div v-if="identity && !isConfirmingReset" class="identity-panel__actions">
      <button type="button" :disabled="isExporting" @click="$emit('exportCertificate')">
        {{ isExporting ? 'Exporting…' : 'Export trust certificate' }}
      </button>
      <button type="button" @click="$emit('reset')">Reset node identity</button>
    </div>
    <p v-if="identity" class="identity-panel__trust-guidance">
      Installing this certificate grants your browser or operating system trust in this node. Export
      it only from this desktop, verify the full fingerprint above during installation, and remove
      it from the trust store if this node is reset or compromised. LocalStream never installs trust
      automatically.
    </p>
    <div v-if="identity && isConfirmingReset" class="identity-panel__confirmation" role="alert">
      <p>
        This removes the node identity and revokes every trusted device. All devices must pair again
        after LocalStream restarts.
      </p>
      <div class="identity-panel__actions">
        <button type="button" :disabled="isResetting" @click="$emit('cancelReset')">Cancel</button>
        <button
          type="button"
          class="danger-action"
          :disabled="isResetting"
          @click="$emit('confirmReset')"
        >
          {{ isResetting ? 'Resetting…' : 'Reset identity and revoke devices' }}
        </button>
      </div>
    </div>
    <p v-if="notice" class="feedback" role="status">{{ notice }}</p>
    <p
      v-if="!identity && !notice"
      class="feedback"
      :class="{ 'feedback--error': error }"
      role="status"
    >
      {{ error ?? 'Waiting for the protected node identity.' }}
    </p>
  </section>
</template>
