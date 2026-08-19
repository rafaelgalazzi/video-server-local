<script setup lang="ts">
import { computed, onMounted, onUnmounted, watch } from 'vue'
import BrowserBootstrapPanel from './components/BrowserBootstrapPanel.vue'
import LanServerPanel from './components/LanServerPanel.vue'
import FoundationStatus from './components/FoundationStatus.vue'
import MediaLibraryPanel from './components/MediaLibraryPanel.vue'
import NodeIdentityPanel from './components/NodeIdentityPanel.vue'
import PairingRequestsPanel from './components/PairingRequestsPanel.vue'
import PlaybackPanel from './components/PlaybackPanel.vue'
import ServerStatus from './components/ServerStatus.vue'
import TrustedPeersPanel from './components/TrustedPeersPanel.vue'
import { useAppInfo } from './composables/useAppInfo'
import { useMediaLibrary } from './composables/useMediaLibrary'
import { useNodeIdentity } from './composables/useNodeIdentity'
import { usePlayback } from './composables/usePlayback'
import { usePairingRequests } from './composables/usePairingRequests'
import { useServerStatus } from './composables/useServerStatus'
import { useTrustedPeers } from './composables/useTrustedPeers'
import { useRuntimeBootstrap } from './composables/useRuntimeBootstrap'
import { useLanServer } from './composables/useLanServer'

const { appInfo, error, isLoading, load, runtimeLabel } = useAppInfo()
const mediaLibrary = useMediaLibrary()
const nodeIdentity = useNodeIdentity()
const serverStatus = useServerStatus()
const pairing = usePairingRequests()
const trustedPeers = useTrustedPeers()
const runtime = useRuntimeBootstrap()
const lanServer = useLanServer()
const activeServer = computed(() =>
  runtime.isNative.value ? serverStatus.server.value : runtime.server.value,
)
const activeLibrary = computed(() =>
  runtime.isNative.value ? mediaLibrary.library.value : runtime.library.value,
)
const activeItemCountLabel = computed(() => {
  const count = activeLibrary.value?.items.length ?? 0
  return `${count} ${count === 1 ? 'video' : 'videos'}`
})
const playback = usePlayback(activeServer, runtime.isNative)

watch(
  () => mediaLibrary.library.value?.items,
  (items) => {
    const selectedId = playback.selectedItem.value?.id
    if (selectedId && !items?.some((item) => item.id === selectedId)) playback.clear()
  },
)

async function selectLibrary() {
  playback.clear()
  await mediaLibrary.selectLibrary()
}

onMounted(() => {
  if (!runtime.isNative.value) {
    void runtime.loadBrowser()
    return
  }
  void load()
  void mediaLibrary.loadCurrentLibrary()
  void nodeIdentity.load()
  void serverStatus.load()
  void pairing.startPolling()
  void trustedPeers.load()
  void lanServer.load()
})

onUnmounted(() => {
  if (runtime.isNative.value) pairing.stopPolling()
})
</script>

<template>
  <main class="app-shell">
    <section class="hero" aria-labelledby="page-title">
      <p class="eyebrow">Private media. Your network.</p>
      <h1 id="page-title">LocalStream</h1>
      <p class="lede">
        A local-first home for the media you choose to share designed to work across your LAN
        without a cloud account.
      </p>

      <FoundationStatus
        v-if="runtime.isNative.value"
        :app-info="appInfo"
        :error="error"
        :is-loading="isLoading"
        :runtime-label="runtimeLabel"
        @retry="load"
      />

      <BrowserBootstrapPanel
        v-else
        :error="runtime.error.value"
        :is-pairing="runtime.isPairing.value"
        :pairing="runtime.pairing.value"
        :state="runtime.state.value"
        @begin-pairing="runtime.beginPairing"
        @finish-pairing="runtime.finishPairing"
        @retry="runtime.loadBrowser"
      />

      <MediaLibraryPanel
        :can-play="playback.canPlay.value"
        :can-select="runtime.isNative.value"
        :error="runtime.isNative.value ? mediaLibrary.error.value : runtime.error.value"
        :is-scanning="mediaLibrary.isScanning.value"
        :is-restoring="mediaLibrary.isRestoring.value"
        :item-count-label="activeItemCountLabel"
        :library="activeLibrary"
        :notice="mediaLibrary.notice.value"
        @play="playback.play"
        @select="selectLibrary"
      />

      <PlaybackPanel
        v-if="playback.selectedItem.value && playback.streamUrl.value"
        :error="playback.error.value"
        :item="playback.selectedItem.value"
        :status="playback.status.value"
        :stream-url="playback.streamUrl.value"
        :audio-options="playback.audioOptions.value"
        :audio-selection-error="playback.audioSelectionError.value"
        :can-select-audio="playback.canPersistAudio.value"
        :is-saving-audio="playback.isSavingAudio.value"
        :selected-audio-track-id="playback.selectedAudioTrackId.value"
        :active-subtitle-track="playback.activeSubtitleTrack.value"
        :subtitle-delivery-notice="playback.subtitleDeliveryNotice.value"
        :subtitle-options="playback.subtitleOptions.value"
        :subtitle-selection-error="playback.subtitleSelectionError.value"
        :subtitle-selection-value="playback.subtitleSelectionValue.value"
        :subtitle-track-url="playback.subtitleTrackUrl.value"
        :is-saving-subtitle="playback.isSavingSubtitle.value"
        @close="playback.clear"
        @failed="playback.markError"
        @playing="playback.markPlaying"
        @select-audio="playback.selectAudioTrack"
        @select-subtitle="playback.selectSubtitle"
      />

      <ServerStatus
        v-if="runtime.isNative.value"
        :error="serverStatus.error.value"
        :server="serverStatus.server.value"
        :status-label="serverStatus.statusLabel.value"
      />

      <NodeIdentityPanel
        v-if="runtime.isNative.value"
        :error="nodeIdentity.error.value"
        :identity="nodeIdentity.identity.value"
        :is-confirming-reset="nodeIdentity.isConfirmingReset.value"
        :is-exporting="nodeIdentity.isExporting.value"
        :is-resetting="nodeIdentity.isResetting.value"
        :notice="nodeIdentity.notice.value"
        :status-label="nodeIdentity.statusLabel.value"
        @cancel-reset="nodeIdentity.cancelReset"
        @confirm-reset="nodeIdentity.confirmReset"
        @export-certificate="nodeIdentity.exportRootCertificate"
        @reset="nodeIdentity.requestReset"
      />

      <PairingRequestsPanel
        v-if="runtime.isNative.value"
        :error="pairing.error.value"
        :is-deciding="pairing.isDeciding"
        :is-loading="pairing.isLoading.value"
        :notice="pairing.notice.value"
        :requests="pairing.requests.value"
        @approve="pairing.approve"
        @reject="pairing.reject"
        @retry="pairing.startPolling"
      />

      <TrustedPeersPanel
        v-if="runtime.isNative.value"
        :confirming-peer="trustedPeers.confirmingPeer.value"
        :error="trustedPeers.error.value"
        :is-loading="trustedPeers.isLoading.value"
        :is-revoking="trustedPeers.isRevoking.value"
        :notice="trustedPeers.notice.value"
        :peers="trustedPeers.peers.value"
        @cancel="trustedPeers.cancelRevocation"
        @confirm="trustedPeers.confirmRevocation"
        @refresh="trustedPeers.load"
        @revoke="trustedPeers.requestRevocation"
      />

      <LanServerPanel
        v-if="runtime.isNative.value"
        :addresses="lanServer.addresses.value"
        :config="lanServer.config.value"
        :error="lanServer.error.value"
        :is-saving="lanServer.isSaving.value"
        :notice="lanServer.notice.value"
        :status="lanServer.status.value"
        :status-label="lanServer.statusLabel.value"
        @save="lanServer.save"
      />
    </section>

    <aside class="roadmap" aria-label="Initial roadmap">
      <span class="roadmap__index">01</span>
      <div>
        <p class="roadmap__label">Foundation milestone</p>
        <h2>From local folder to living-room screen.</h2>
        <ol>
          <li><span>Approve</span> a media folder</li>
          <li><span>Index</span> it locally</li>
          <li><span>Play</span> it anywhere on your LAN</li>
        </ol>
      </div>
    </aside>
  </main>
</template>
