<script setup lang="ts">
import { ref } from 'vue'
import type {
  BrowserBootstrapState,
  BrowserPairingReceipt,
} from '../composables/useRuntimeBootstrap'

defineProps<{
  error: string | null
  isPairing: boolean
  pairing: BrowserPairingReceipt | null
  state: BrowserBootstrapState
}>()
defineEmits<{ beginPairing: [displayName: string]; finishPairing: []; retry: [] }>()
const displayName = ref('Browser')
</script>

<template>
  <section class="status-card" aria-labelledby="browser-status-title" aria-live="polite">
    <div>
      <p id="browser-status-title" class="status-card__label">Remote browser</p>
      <p v-if="state === 'bootstrapping'" class="status-card__detail">
        Connecting securely to this LocalStream node…
      </p>
      <p v-else-if="state === 'pairing-required'" class="status-card__detail">
        Pairing is required. Start pairing on this device, then compare and approve the code on the
        LocalStream desktop.
      </p>
      <p v-else-if="state === 'authenticated'" class="status-card__detail">
        Secure browser session authenticated.
      </p>
      <p v-else-if="state === 'disconnected'" class="status-card__detail" role="alert">
        {{ error ?? 'The LocalStream node could not be reached.' }}
      </p>
      <p v-else class="status-card__detail">Preparing secure browser access…</p>
      <div v-if="state === 'pairing-required' && !pairing">
        <label>Device name <input v-model="displayName" maxlength="80" /></label>
        <button
          type="button"
          :disabled="isPairing || !displayName.trim()"
          @click="$emit('beginPairing', displayName.trim())"
        >
          {{ isPairing ? 'Starting…' : 'Start pairing' }}
        </button>
      </div>
      <div v-if="pairing" role="status">
        <p>Compare this code on the trusted desktop:</p>
        <strong class="pairing-request__code">{{ pairing.verificationCode }}</strong>
        <p>Expires in {{ pairing.expiresInSeconds }} seconds.</p>
        <button type="button" :disabled="isPairing" @click="$emit('finishPairing')">
          {{ isPairing ? 'Checking…' : 'I approved this code' }}
        </button>
      </div>
    </div>
    <button v-if="state === 'disconnected'" type="button" @click="$emit('retry')">Retry</button>
  </section>
</template>
