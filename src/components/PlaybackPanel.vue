<script setup lang="ts">
import Hls from 'hls.js'
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import type { MediaItem, SubtitleTrack } from '../composables/useMediaLibrary'
import type { PlaybackStatus } from '../composables/usePlayback'
import type { AudioOption } from '../composables/usePlayback'
import type { SubtitleOption } from '../composables/usePlayback'

const props = defineProps<{
  error: string | null
  item: MediaItem
  status: PlaybackStatus
  progress: number | null
  preparationNotice: string | null
  streamUrl: string | null
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

const emit = defineEmits<{
  close: []
  failed: []
  playing: []
  start: []
  selectAudio: [trackId: string]
  selectSubtitle: [value: string]
}>()

const video = ref<HTMLVideoElement | null>(null)
const isHlsStream = computed(() => props.streamUrl?.includes('/playback/hls/') ?? false)
let hls: Hls | null = null
let recoveryAttempts = 0
let playbackStarted = false

watch(
  () => props.streamUrl,
  async (url) => {
    destroyHls()
    await nextTick()
    const element = video.value
    if (!element || !url) return
    if (!isHlsStream.value) {
      element.src = url
      return
    }
    if (!Hls.isSupported()) {
      if (element.canPlayType('application/vnd.apple.mpegurl')) {
        element.src = url
      } else {
        emit('failed')
      }
      return
    }
    hls = new Hls({
      autoStartLoad: false,
      enableWorker: true,
      lowLatencyMode: false,
      startPosition: 0,
      xhrSetup: (request) => {
        request.withCredentials =
          new URL(url, window.location.href).origin === window.location.origin
      },
    })
    recoveryAttempts = 0
    playbackStarted = false
    hls.on(Hls.Events.ERROR, (_event, data) => {
      if (!data.fatal || !hls) return
      if (recoveryAttempts >= 2) {
        emit('failed')
        return
      }
      recoveryAttempts += 1
      if (data.type === Hls.ErrorTypes.NETWORK_ERROR) {
        hls.startLoad()
        return
      }
      if (data.type === Hls.ErrorTypes.MEDIA_ERROR) {
        hls.recoverMediaError()
        return
      }
      emit('failed')
    })
    hls.on(Hls.Events.MANIFEST_PARSED, () => {
      hls?.startLoad(0)
    })
    hls.on(Hls.Events.FRAG_BUFFERED, () => {
      if (playbackStarted) return
      playbackStarted = true
      element.currentTime = element.seekable.length > 0 ? element.seekable.start(0) : 0
      void element.play().catch(() => undefined)
    })
    hls.attachMedia(element)
    hls.loadSource(url)
  },
  { immediate: true },
)

onBeforeUnmount(destroyHls)

function destroyHls() {
  hls?.destroy()
  hls = null
  recoveryAttempts = 0
  playbackStarted = false
  if (video.value) video.value.removeAttribute('src')
}
</script>

<template>
  <section class="playback-panel" aria-labelledby="playback-title">
    <div class="playback-panel__heading">
      <div>
        <p class="section-label">{{ status === 'idle' ? 'Preview setup' : 'Now playing' }}</p>
        <h2 id="playback-title">{{ item.title }}</h2>
      </div>
      <button type="button" aria-label="Close player" @click="$emit('close')">Close</button>
    </div>

    <p v-if="status === 'loading'" class="playback-panel__status" role="status">
      Preparing playback<span v-if="progress !== null"> ({{ Math.floor(progress / 10) }}%)</span>â€¦
    </p>
    <p v-if="preparationNotice" class="playback-panel__notice">{{ preparationNotice }}</p>
    <div
      v-if="status === 'loading'"
      class="playback-panel__progress"
      :class="{ 'playback-panel__progress--indeterminate': progress === null }"
      role="progressbar"
      aria-label="Preparing compatible playback"
      aria-valuemin="0"
      aria-valuemax="100"
      :aria-valuenow="progress === null ? undefined : Math.floor(progress / 10)"
    >
      <span :style="progress === null ? undefined : { width: `${progress / 10}%` }" />
    </div>
    <p v-if="error" class="feedback feedback--error" role="alert">{{ error }}</p>

    <button
      v-if="status === 'idle'"
      type="button"
      class="playback-panel__start"
      @click="$emit('start')"
    >
      Play
    </button>

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
      v-if="streamUrl"
      ref="video"
      :key="streamUrl"
      class="playback-panel__video"
      :src="isHlsStream ? undefined : streamUrl"
      :aria-label="`Video player for ${item.title}`"
      controls
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
