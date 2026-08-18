<script setup lang="ts">
import type { ServerInfo } from '../composables/useServerStatus'

defineProps<{
  error: string | null
  server: ServerInfo | null
  statusLabel: string
}>()
</script>

<template>
  <section class="server-status" aria-labelledby="server-status-title">
    <span class="server-status__signal" :class="{ 'server-status__signal--ready': server }" />
    <div>
      <p id="server-status-title" class="section-label">{{ statusLabel }}</p>
      <p v-if="server" class="server-status__detail">
        {{ server.baseUrl }} · Loopback only until secure pairing is available
      </p>
      <p v-else class="server-status__detail">{{ error ?? 'Waiting for the embedded API.' }}</p>
    </div>
  </section>
</template>
