<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from 'vue'
import BrowserBootstrapPanel from './components/BrowserBootstrapPanel.vue'
import DatabaseMaintenancePanel from './components/DatabaseMaintenancePanel.vue'
import LanServerPanel from './components/LanServerPanel.vue'
import FoundationStatus from './components/FoundationStatus.vue'
import MediaLibraryPanel from './components/MediaLibraryPanel.vue'
import NodeIdentityPanel from './components/NodeIdentityPanel.vue'
import PairingRequestsPanel from './components/PairingRequestsPanel.vue'
import PlaybackPanel from './components/PlaybackPanel.vue'
import TrustedPeersPanel from './components/TrustedPeersPanel.vue'
import { useAppInfo } from './composables/useAppInfo'
import { useDatabaseMaintenance } from './composables/useDatabaseMaintenance'
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
const databaseMaintenance = useDatabaseMaintenance()
const isConfirmingDatabaseClear = ref(false)
type WorkspaceTab = 'library' | 'network' | 'access'
const activeTab = ref<WorkspaceTab>('library')
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
const playbackFallbackEnabled = computed(
  () => runtime.isNative.value || runtime.state.value === 'authenticated',
)
const playback = usePlayback(
  activeServer,
  runtime.isNative,
  undefined,
  undefined,
  playbackFallbackEnabled,
)

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

async function clearLocalDatabase() {
  if (!(await databaseMaintenance.clear())) return
  playback.clear()
  isConfirmingDatabaseClear.value = false
  await Promise.all([mediaLibrary.loadCurrentLibrary(), trustedPeers.load()])
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
    <header class="app-header" aria-labelledby="page-title">
      <div class="app-header__intro">
        <h1 id="page-title">LocalStream</h1>
        <p class="lede">Private media on your network.</p>
      </div>
    </header>

    <section class="workspace" aria-label="LocalStream workspace">
      <BrowserBootstrapPanel
        v-if="!runtime.isNative.value"
        :error="runtime.error.value"
        :is-pairing="runtime.isPairing.value"
        :pairing="runtime.pairing.value"
        :state="runtime.state.value"
        @begin-pairing="runtime.beginPairing"
        @finish-pairing="runtime.finishPairing"
        @retry="runtime.loadBrowser"
      />

      <section v-if="runtime.isNative.value" class="access-guide" aria-labelledby="setup-title">
        <div>
          <p class="section-label">Quick setup</p>
          <h2 id="setup-title">Connect another device</h2>
        </div>
        <ol>
          <li><span>1</span>Choose your media folder in Library & playback.</li>
          <li><span>2</span>Enable an address in Network and restart LocalStream.</li>
          <li><span>3</span>Open that address on your device and approve its code in Access.</li>
        </ol>
      </section>

      <nav
        v-if="runtime.isNative.value"
        class="workspace-tabs"
        aria-label="Settings sections"
        role="tablist"
      >
        <button
          v-for="tab in ['library', 'network', 'access'] as const"
          :id="`workspace-tab-${tab}`"
          :key="tab"
          type="button"
          role="tab"
          :aria-controls="`workspace-panel-${tab}`"
          :aria-selected="activeTab === tab"
          :class="{ 'workspace-tabs__tab--active': activeTab === tab }"
          @click="activeTab = tab"
        >
          {{ tab === 'library' ? 'Library & playback' : tab === 'network' ? 'Network' : 'Access' }}
        </button>
      </nav>

      <div
        v-show="!runtime.isNative.value || activeTab === 'library'"
        id="workspace-panel-library"
        class="tab-panel tab-panel--library"
        role="tabpanel"
        :aria-label="runtime.isNative.value ? undefined : 'Library and playback'"
        :aria-labelledby="runtime.isNative.value ? 'workspace-tab-library' : undefined"
      >
        <MediaLibraryPanel
          :can-play="playback.canPlay.value"
          :can-select="runtime.isNative.value"
          :error="runtime.isNative.value ? mediaLibrary.error.value : runtime.error.value"
          :is-scanning="mediaLibrary.isScanning.value"
          :is-restoring="mediaLibrary.isRestoring.value"
          :item-count-label="activeItemCountLabel"
          :library="activeLibrary"
          :notice="mediaLibrary.notice.value"
          @configure="playback.select"
          @play="playback.play"
          @select="selectLibrary"
        />

        <PlaybackPanel
          v-if="playback.selectedItem.value"
          :error="playback.error.value"
          :item="playback.selectedItem.value"
          :status="playback.status.value"
          :progress="playback.playbackProgress.value"
          :preparation-notice="playback.preparationNotice.value"
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
          @start="playback.start"
          @select-audio="playback.selectAudioTrack"
          @select-subtitle="playback.selectSubtitle"
        />
      </div>

      <div
        v-if="runtime.isNative.value"
        v-show="activeTab === 'network'"
        id="workspace-panel-network"
        class="tab-panel tab-panel--network"
        role="tabpanel"
        aria-labelledby="workspace-tab-network"
      >
        <LanServerPanel
          :addresses="lanServer.addresses.value"
          :config="lanServer.config.value"
          :error="lanServer.error.value"
          :is-saving="lanServer.isSaving.value"
          :notice="lanServer.notice.value"
          :status="lanServer.status.value"
          :status-label="lanServer.statusLabel.value"
          @save="lanServer.save"
        />
      </div>

      <div
        v-if="runtime.isNative.value"
        v-show="activeTab === 'access'"
        id="workspace-panel-access"
        class="tab-panel tab-panel--access"
        role="tabpanel"
        aria-labelledby="workspace-tab-access"
      >
        <NodeIdentityPanel
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
        <DatabaseMaintenancePanel
          :error="databaseMaintenance.error.value"
          :is-clearing="databaseMaintenance.isClearing.value"
          :is-confirming="isConfirmingDatabaseClear"
          :notice="databaseMaintenance.notice.value"
          @cancel="isConfirmingDatabaseClear = false"
          @clear="clearLocalDatabase"
          @request="isConfirmingDatabaseClear = true"
        />
      </div>

      <footer v-if="runtime.isNative.value" class="app-footer">
        <FoundationStatus
          :app-info="appInfo"
          :error="error"
          :is-loading="isLoading"
          :runtime-label="runtimeLabel"
          @retry="load"
        />
      </footer>
    </section>
  </main>
</template>
