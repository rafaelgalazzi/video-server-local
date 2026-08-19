<script setup lang="ts">
import type { MediaItem } from '../composables/useMediaLibrary'
import type { PlaybackStatus } from '../composables/usePlayback'
import type { AudioOption } from '../composables/usePlayback'

defineProps<{
  error: string | null
  item: MediaItem
  status: PlaybackStatus
  streamUrl: string
  audioOptions: AudioOption[]
  audioSelectionError: string | null
  canSelectAudio: boolean
  isSavingAudio: boolean
  selectedAudioTrackId: string | null
}>()

defineEmits<{
  close: []
  failed: []
  playing: []
  selectAudio: [trackId: string]
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

    <div v-if="audioOptions.length > 1" class="playback-panel__track-control">
      <label for="audio-track">Audio track</label>
      <select
        id="audio-track"
        :disabled="!canSelectAudio || isSavingAudio"
        :value="selectedAudioTrackId ?? ''"
        @change="$emit('selectAudio', ($event.target as HTMLSelectElement).value)"
      >
        <option v-for="option in audioOptions" :key="option.id" :value="option.id">
          {{ option.label }}
        </option>
      </select>
      <p v-if="!canSelectAudio" class="playback-panel__status">
        Audio preferences can currently be saved from the desktop app.
      </p>
      <p v-if="audioSelectionError" class="feedback feedback--error" role="alert">
        {{ audioSelectionError }}
      </p>
    </div>

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
