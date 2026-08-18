<script setup lang="ts">
import type { LibraryScan } from '../composables/useMediaLibrary'

defineProps<{
  error: string | null
  isScanning: boolean
  isRestoring: boolean
  itemCountLabel: string
  library: LibraryScan | null
  notice: string | null
}>()

defineEmits<{
  select: []
}>()

const sizeFormatter = new Intl.NumberFormat(undefined, {
  maximumFractionDigits: 1,
})

function formatSize(bytes: number) {
  if (bytes < 1024) return `${bytes} B`
  const kibibytes = bytes / 1024
  if (kibibytes < 1024) return `${sizeFormatter.format(kibibytes)} KiB`
  return `${sizeFormatter.format(kibibytes / 1024)} MiB`
}
</script>

<template>
  <section class="library-panel" aria-labelledby="library-title">
    <div class="library-panel__heading">
      <div>
        <p class="section-label">Local library</p>
        <h2 id="library-title">{{ library?.libraryName ?? 'Choose your media' }}</h2>
        <p v-if="isRestoring" class="library-panel__summary">Restoring your saved library…</p>
        <p v-else class="library-panel__summary">
          {{ library ? itemCountLabel : 'Only folders you approve will be scanned.' }}
        </p>
      </div>
      <button
        class="primary-action"
        type="button"
        :disabled="isScanning || isRestoring"
        @click="$emit('select')"
      >
        {{ isScanning ? 'Scanning…' : library ? 'Change folder' : 'Choose folder' }}
      </button>
    </div>

    <p v-if="error" class="feedback feedback--error" role="alert">{{ error }}</p>
    <p v-else-if="notice" class="feedback">{{ notice }}</p>

    <div v-if="library && library.items.length > 0" class="media-list">
      <article v-for="(item, index) in library.items" :key="item.id" class="media-row">
        <span class="media-row__index">{{ String(index + 1).padStart(2, '0') }}</span>
        <div class="media-row__identity">
          <h3>{{ item.title }}</h3>
          <p>{{ item.extension.toUpperCase() }} · {{ formatSize(item.sizeBytes) }}</p>
        </div>
        <span class="media-row__status">Indexed</span>
      </article>
    </div>
    <p v-else-if="library" class="empty-library">
      No supported videos were found. Try a folder containing MP4, MKV, WebM, MOV, or M4V files.
    </p>
  </section>
</template>
