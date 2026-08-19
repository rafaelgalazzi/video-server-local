<script setup lang="ts">
import type { LibraryScan, MediaItem } from '../composables/useMediaLibrary'

defineProps<{
  canPlay: boolean
  canSelect: boolean
  error: string | null
  isScanning: boolean
  isRestoring: boolean
  itemCountLabel: string
  library: LibraryScan | null
  notice: string | null
}>()

defineEmits<{
  configure: [item: MediaItem]
  play: [item: MediaItem]
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
        v-if="canSelect"
        class="library-action library-action--primary"
        type="button"
        :disabled="isScanning || isRestoring"
        @click="$emit('select')"
      >
        {{ isScanning ? 'Scanning…' : library ? 'Change folder' : 'Choose folder' }}
      </button>
    </div>

    <p v-if="error" class="feedback feedback--error" role="alert">{{ error }}</p>
    <p v-else-if="notice" class="feedback">{{ notice }}</p>
    <p v-if="library?.items.length && !canPlay" class="feedback" role="status">
      Playback will be available when the private API is ready.
    </p>

    <div v-if="library && library.items.length > 0" class="media-list">
      <article v-for="(item, index) in library.items" :key="item.id" class="media-row">
        <span class="media-row__index">{{ String(index + 1).padStart(2, '0') }}</span>
        <div class="media-row__identity">
          <h3>{{ item.title }}</h3>
          <p>{{ item.extension.toUpperCase() }} · {{ formatSize(item.sizeBytes) }}</p>
        </div>
        <div class="media-row__actions">
          <button
            class="library-action"
            type="button"
            :disabled="!canPlay"
            :title="canPlay ? `Configure preview for ${item.title}` : 'Playback API unavailable'"
            @click="$emit('configure', item)"
          >
            Configure
          </button>
          <button
            class="library-action library-action--primary"
            type="button"
            :disabled="!canPlay"
            :title="canPlay ? `Play ${item.title}` : 'Playback API unavailable'"
            @click="$emit('play', item)"
          >
            Play
          </button>
        </div>
      </article>
    </div>
    <p v-else-if="library" class="empty-library">
      No supported videos were found. Try a folder containing MP4, MKV, WebM, MOV, or M4V files.
    </p>
  </section>
</template>
