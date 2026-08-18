<script setup lang="ts">
import type { MediaItem } from '../composables/useMediaLibrary'
import type { PlaybackStatus } from '../composables/usePlayback'

defineProps<{
  error: string | null
  item: MediaItem
  status: PlaybackStatus
  streamUrl: string
}>()

defineEmits<{
  close: []
  failed: []
  playing: []
}>()
</script>

<template>
  <section class="playback-panel" aria-labelledby="playback-title">
    <div class="playback-panel__heading">
      <div>
        <p class="section-label">Now playing</p>
        <h2 id="playback-title">{{ item.title }}</h2>
      </div>
      <button type="button" aria-label="Close player" @click="$emit('close')">Close</button>
    </div>

    <p v-if="status === 'loading'" class="playback-panel__status" role="status">
      Preparing direct playbackâ€¦
    </p>
    <p v-if="error" class="feedback feedback--error" role="alert">{{ error }}</p>

    <video
      :key="streamUrl"
      class="playback-panel__video"
      :src="streamUrl"
      :aria-label="`Video player for ${item.title}`"
      controls
      autoplay
      preload="metadata"
      @playing="$emit('playing')"
      @error="$emit('failed')"
    />
  </section>
</template>
