<script setup lang="ts">
import type { AppInfo } from '../composables/useAppInfo'
import type { PortableRuntimeState } from '../composables/usePortableRuntime'

defineProps<{
  appInfo: AppInfo | null
  error: string | null
  isLoading: boolean
  portableFailure: string | null
  portableState: PortableRuntimeState | null
  runtimeLabel: string
}>()

defineEmits<{
  retry: []
}>()
</script>

<template>
  <div class="status-card" aria-live="polite">
    <div class="status-card__signal" :class="{ 'status-card__signal--ready': appInfo }" />
    <div>
      <p class="status-card__label">{{ runtimeLabel }}</p>
      <p v-if="isLoading" class="status-card__detail">Connecting to the native core…</p>
      <p v-else-if="appInfo" class="status-card__detail">
        Core {{ appInfo.version }} · {{ appInfo.localFirst ? 'Local-first mode' : 'Network mode' }}
      </p>
      <p v-else class="status-card__detail">
        {{ error ?? 'Web preview active. Open with Tauri to connect to the Rust core.' }}
      </p>
      <p v-if="portableState === 'failed'" class="feedback feedback--error" role="alert">
        Portable runtime unavailable{{ portableFailure ? ` (${portableFailure})` : '' }}. LAN
        browser access and media fallback are disabled; copy a fresh LocalStream.exe.
      </p>
      <p v-else-if="portableState === 'ready'" class="status-card__detail">
        Portable runtime verified
      </p>
    </div>
    <button v-if="error" type="button" @click="$emit('retry')">Retry</button>
  </div>
</template>
