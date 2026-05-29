<script setup>
import { ref, computed, watch, onMounted, onUnmounted, provide } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Sidebar from "./components/Sidebar.vue";
import MusicView from "./components/view/MusicView.vue";
import VpnView from "./components/view/VpnView.vue";
import HomeView from "./components/view/HomeView.vue";
import AboutModal from "./components/view/AboutModal.vue";
import GlobalSettings from "./components/settings/SettingsController.vue";
import { musicState } from "./scripts/music.js";
import { useRafIdle } from "./scripts/useRafIdle.js";
import { startWeather, stopWeather } from "./scripts/weather.js";
import { settingsState, loadSettings } from "./components/settings/settingsRegistry.js";
import logoSrc from "./assets/icon/luna.svg";
const activePlugin = ref("home");
const showAbout = ref(false);
const tauriVisible = ref(true);
const { isIdle: rafIdle } = useRafIdle();
const windowVisible = computed(() => tauriVisible.value && !rafIdle.value);
const animating = ref(false);

watch(activePlugin, () => {
  animating.value = false;
  requestAnimationFrame(() => {
    animating.value = true;
  });
});
watch(() => settingsState.bga, (v) => {
  document.documentElement.style.setProperty("--bga", v);
}, { immediate: true });

provide("windowVisible", windowVisible);

// ── animated title ──
const fullTitle = "LUNA TOYS";
const titleIndex = ref(0);
let titleTimer = null;
let titleForward = true;

function startTitleAnimation() {
  titleTimer = setInterval(() => {
    if (!windowVisible.value) return;
    if (titleForward) {
      titleIndex.value++;
      if (titleIndex.value >= fullTitle.length) {
        titleForward = false;
      }
    } else {
      titleIndex.value--;
      if (titleIndex.value < 0) {
        titleForward = true;
      }
    }
  }, 800);
}

const animatedTitle = computed(() => fullTitle.slice(0, titleIndex.value + 1) || '\xa0');

onMounted(async () => {
  loadSettings();
  startWeather();
  startTitleAnimation();
  const unlistenShown = await listen("main-window-shown", () => {
    tauriVisible.value = true;
  });
  const unlistenHidden = await listen("main-window-hidden", () => {
    tauriVisible.value = false;
  });
  window._winShownUnlisten = unlistenShown;
  window._winHiddenUnlisten = unlistenHidden;
});

onUnmounted(() => {
  stopWeather();
  clearInterval(titleTimer);
  window._winShownUnlisten?.();
  window._winHiddenUnlisten?.();
});

const lyricText = computed(() => {
  return musicState.currentLyric || "";
});

async function minimize() {
  await invoke("hide_main");
}

async function close() {
  await invoke("exit_app");
}
</script>

<template>
  <div class="app-root" :class="{ 'window-hidden': !windowVisible }">

      <div class="titlebar">

        <div class="titlebar-title" @click.stop="showAbout = true">

          <img :src="logoSrc" class="titlebar-icon" />
        </div>

        <div class="titlebar-lyrics" v-if="lyricText">
          <span class="lyrics-text">{{ lyricText }}</span>
        </div>

      <div class="titlebar-controls">

        <button class="titlebar-btn" @click.stop="minimize" title="最小化到托盘">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14" /></svg>
        </button>
        <button class="titlebar-btn" @click.stop="close" title="关闭">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12" /></svg>
        </button>

      </div>

    </div>

    <div class="app-body">


      <Sidebar :activePlugin="activePlugin" @select="activePlugin = $event" />

      <div class="main-content" :class="{ 'content-switch': animating }">
        <HomeView v-if="activePlugin === 'home'" />
        <MusicView v-show="activePlugin === 'music'" />
        <VpnView v-if="activePlugin === 'vpn'" />
        <GlobalSettings v-if="activePlugin === 'settings'" />
      </div>

    </div>

  </div>
  <AboutModal v-if="showAbout" @close="showAbout = false" />
</template>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body {
  height: 100%;
  overflow: hidden;
  background: rgba(0,0,0, calc(var(--bga) / 100));
  color: rgba(255, 255, 255, 1);
  font-family: 'Google Sans', sans-serif;
  font-size: 14px;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: optimizeLegibility;
}

#app {
  height: 100%;
}

.app-root {
  height: 100%;
  display: flex;
  flex-direction: column;
}

.window-hidden *,
.window-hidden *::before,
.window-hidden *::after {
  animation-play-state: paused !important;
}

.titlebar {
  height: 48px;
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  -webkit-app-region: drag;
  position: relative;
  background: rgba(138, 138, 138, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 2px 16px rgba(0, 0, 0, 0.2);
  border-radius: 24px;
  margin: 12px;

}
.titlebar-title {
  background: rgba(138, 138, 138, 0.2);
  border-radius: 50%;
  padding: 4px;
  margin: 6px;
  display: flex;
  align-items: center;
  -webkit-app-region: no-drag;
  cursor: pointer;
}
.titlebar-icon {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
}
.titlebar-lyrics {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  max-width: 360px;
  background: rgba(134, 134, 134, 0.2);
  border-radius: 24px;
  padding: 10px 16px;
  overflow: hidden;
  -webkit-app-region: no-drag;
}
.lyrics-text {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.65);
  display: block;
  max-width: 360px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
.titlebar-controls {
  display: flex;
  flex-shrink: 0;
  -webkit-app-region: no-drag;
  margin-right: 6px;
}

.titlebar-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  border-radius: 50%;
  transition: background 0.15s, color 0.15s;
}

.titlebar-btn svg {
  width: 16px;
  height: 16px;
}

.titlebar-btn:hover {
  border-radius: 50%;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.85);
}


.app-body {
  flex: 1;
  min-height: 0;
  display: flex;
  padding: 0 12px 12px 12px;
  gap: 8px;
  -webkit-app-region: no-drag;
}

.main-content {
  flex: 1;
  min-width: 0;
  min-height: 0;
  padding: 12px;
  background: rgba(138, 138, 138, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.1);
  box-shadow: 0 2px 16px rgba(0, 0, 0, 0.2);
  border-radius: 24px;
}

.content-switch {
  animation: content-in 0.25s ease;
}

@keyframes content-in {
  0% {
    opacity: 0;
    transform: translateX(16px);
  }
  100% {
    opacity: 1;
    transform: translateX(0);
  }
}
</style>
