<script setup lang="ts">
import { onMounted } from 'vue'
import FoundationStatus from './components/FoundationStatus.vue'
import { useAppInfo } from './composables/useAppInfo'

const { appInfo, error, isLoading, load, runtimeLabel } = useAppInfo()

onMounted(load)
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
