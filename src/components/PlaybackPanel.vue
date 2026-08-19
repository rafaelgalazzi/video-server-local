<script setup lang="ts">
import type { MediaItem, SubtitleTrack } from '../composables/useMediaLibrary'
import type { PlaybackStatus } from '../composables/usePlayback'
import type { AudioOption } from '../composables/usePlayback'
import type { SubtitleOption } from '../composables/usePlayback'

defineProps<{
  error: string | null
  item: MediaItem
  status: PlaybackStatus
  progress: number | null
  streamUrl: string
  audioOptions: AudioOption[]
  audioSelectionError: string | null
  canSelectAudio: boolean
  isSavingAudio: boolean
  selectedAudioTrackId: string | null
  activeSubtitleTrack: SubtitleTrack | null
  subtitleDeliveryNotice: string | null
  subtitleOptions: SubtitleOption[]
  subtitleSelectionError: string | null
  subtitleSelectionValue: string
  subtitleTrackUrl: string | null
  isSavingSubtitle: boolean
}>()

defineEmits<{
  close: []
  failed: []
  playing: []
  selectAudio: [trackId: string]
  selectSubtitle: [value: string]
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
      Preparing playback<span v-if="progress !== null"> ({{ Math.floor(progress / 10) }}%)</span>â€¦
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

    <div v-if="subtitleOptions.length > 2" class="playback-panel__track-control">
      <label for="subtitle-track">Subtitles</label>
      <select
        id="subtitle-track"
        :disabled="!canSelectAudio || isSavingSubtitle"
        :value="subtitleSelectionValue"
        @change="$emit('selectSubtitle', ($event.target as HTMLSelectElement).value)"
      >
        <option v-for="option in subtitleOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
      <p v-if="subtitleDeliveryNotice" class="feedback feedback--error" role="alert">
        {{ subtitleDeliveryNotice }}
      </p>
      <p v-if="subtitleSelectionError" class="feedback feedback--error" role="alert">
        {{ subtitleSelectionError }}
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
    >
      <track
        v-if="subtitleTrackUrl && activeSubtitleTrack"
        :key="subtitleTrackUrl"
        :src="subtitleTrackUrl"
        :srclang="activeSubtitleTrack.language ?? 'und'"
        :label="activeSubtitleTrack.title ?? activeSubtitleTrack.language ?? 'Subtitles'"
        kind="subtitles"
        default
      />
    </video>
  </section>
</template>
