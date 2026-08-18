<script setup lang="ts">
import { onMounted, onUnmounted, watch } from 'vue'
import FoundationStatus from './components/FoundationStatus.vue'
import MediaLibraryPanel from './components/MediaLibraryPanel.vue'
import PairingRequestsPanel from './components/PairingRequestsPanel.vue'
import PlaybackPanel from './components/PlaybackPanel.vue'
import ServerStatus from './components/ServerStatus.vue'
import TrustedPeersPanel from './components/TrustedPeersPanel.vue'
import { useAppInfo } from './composables/useAppInfo'
import { useMediaLibrary } from './composables/useMediaLibrary'
import { usePlayback } from './composables/usePlayback'
import { usePairingRequests } from './composables/usePairingRequests'
import { useServerStatus } from './composables/useServerStatus'
import { useTrustedPeers } from './composables/useTrustedPeers'

const { appInfo, error, isLoading, load, runtimeLabel } = useAppInfo()
const mediaLibrary = useMediaLibrary()
const serverStatus = useServerStatus()
const playback = usePlayback(serverStatus.server)
const pairing = usePairingRequests()
const trustedPeers = useTrustedPeers()

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
  void load()
  void mediaLibrary.loadCurrentLibrary()
  void serverStatus.load()
  void pairing.startPolling()
  void trustedPeers.load()
})

onUnmounted(() => pairing.stopPolling())
</script>

<template>
  <main class="app-shell">
    <section class="hero" aria-labelledby="page-title">
      <p class="eyebrow">Private media. Your network.</p>
      <h1 id="page-title">LocalStream</h1>
      <p class="lede">
        A local-first home for the media you choose to share—designed to work across your LAN
        without a cloud account.
      </p>

      <FoundationStatus
        :app-info="appInfo"
        :error="error"
        :is-loading="isLoading"
        :runtime-label="runtimeLabel"
        @retry="load"
      />

      <MediaLibraryPanel
        :can-play="playback.canPlay.value"
        :error="mediaLibrary.error.value"
        :is-scanning="mediaLibrary.isScanning.value"
        :is-restoring="mediaLibrary.isRestoring.value"
        :item-count-label="mediaLibrary.itemCountLabel.value"
        :library="mediaLibrary.library.value"
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
        @close="playback.clear"
        @failed="playback.markError"
        @playing="playback.markPlaying"
      />

      <ServerStatus
        :error="serverStatus.error.value"
        :server="serverStatus.server.value"
        :status-label="serverStatus.statusLabel.value"
      />

      <PairingRequestsPanel
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
