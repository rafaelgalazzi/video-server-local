<script setup lang="ts">
import { reactive, watch } from 'vue'
import type { LanServerConfig, LanServerStatus } from '../composables/useLanServer'
const props = defineProps<{
  addresses: string[]
  config: LanServerConfig | null
  error: string | null
  isSaving: boolean
  notice: string | null
  status: LanServerStatus | null
  statusLabel: string
}>()
const emit = defineEmits<{ save: [config: LanServerConfig] }>()
const draft = reactive<LanServerConfig>({
  enabled: false,
  address: null,
  port: 8443,
  dnsName: null,
})
watch(
  () => props.config,
  (value) => {
    if (value) Object.assign(draft, value)
  },
  { immediate: true },
)
</script>
<template>
  <section class="identity-panel" aria-labelledby="lan-server-title">
    <p class="section-label">{{ statusLabel }}</p>
    <h2 id="lan-server-title">Secure LAN endpoint</h2>
    <p class="identity-panel__summary">
      Disabled by default. Select one explicit private address; wildcard and loopback binding are
      rejected.
    </p>
    <label><input v-model="draft.enabled" type="checkbox" /> Enable after restart</label>
    <label
      >Address<select v-model="draft.address" :disabled="!draft.enabled">
        <option :value="null">Select an address</option>
        <option v-for="address in addresses" :key="address">{{ address }}</option>
      </select></label
    >
    <label
      >HTTPS port<input
        v-model.number="draft.port"
        type="number"
        min="1024"
        max="65534"
        :disabled="!draft.enabled"
    /></label>
    <label
      >Optional DNS name<input
        v-model="draft.dnsName"
        type="text"
        :disabled="!draft.enabled"
        placeholder="media.home"
    /></label>
    <button
      type="button"
      :disabled="isSaving || (draft.enabled && !draft.address)"
      @click="emit('save', { ...draft })"
    >
      {{ isSaving ? 'Saving…' : 'Save LAN configuration' }}
    </button>
    <p v-if="status?.endpoint" class="feedback">{{ status.endpoint }}</p>
    <p v-if="notice" class="feedback">{{ notice }}</p>
    <p v-if="error" class="feedback feedback--error" role="alert">{{ error }}</p>
  </section>
</template>
