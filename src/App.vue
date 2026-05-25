<script setup>
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import Sidebar from "./components/Sidebar.vue";
import RunnerSettings from "./components/view/RunnerSettings.vue";
import MusicView from "./components/view/MusicView.vue";
import VpnView from "./components/view/VpnView.vue";
import { musicState } from "./stores/music.js";

const activePlugin = ref("home");

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
  <div class="app-root">

      <div class="titlebar">

        <div class="titlebar-title">

          <span>NToys Beta</span>
        </div>

        <div class="titlebar-lyrics" v-if="lyricText">
          <span class="lyrics-text">{{ lyricText }}</span>
        </div>

      <div class="titlebar-controls">

        <button class="titlebar-btn" @click.stop="minimize" title="最小化到托盘">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M5 12h14" /></svg>
        </button>
        <button class="titlebar-btn titlebar-btn-close" @click.stop="close" title="关闭">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M18 6L6 18M6 6l12 12" /></svg>
        </button>

      </div>

    </div>  

    <div class="app-body">

      
      <Sidebar :activePlugin="activePlugin" @select="activePlugin = $event" />

      <div class="main-content">
        <RunnerSettings v-if="activePlugin === 'runner'" />
        <MusicView v-show="activePlugin === 'music'" />
        <VpnView v-if="activePlugin === 'vpn'" />
      </div>

    </div>

  </div>
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
  background: linear-gradient(135deg, rgb(20, 14, 30) 0%, rgb(30, 18, 42) 40%, rgb(18, 16, 28) 100%);
  color: rgba(255, 255, 255, 0.85);
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
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

.titlebar {
  height: 32px;
  flex-shrink: 0;
  display: flex;
  justify-content: space-between;
  align-items: center;
  -webkit-app-region: drag;
  position: relative;
}
.titlebar-title {
  width: 140px;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 8px;
  padding: 10px 30px;
  margin: 24px 0 0 12px;
}
.titlebar-lyrics {
  position: absolute;
  left: 50%;
  transform: translateX(-50%);
  margin-top: 24px;
  max-width: 360px;
  background: rgba(0, 0, 0, 0.6);
  border-radius: 8px;
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
  background: rgba(0, 0, 0, 0.6);
  border-radius: 8px;
  padding: 4px 4px;
  gap: 2px;
  margin:24px 12px 0px 0;
}

.titlebar-btn {
  width: 36px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  border: none;
  background: transparent;
  color: rgba(255, 255, 255, 0.5);
  cursor: pointer;
  border-radius: 8px;
  transition: background 0.15s, color 0.15s;
}

.titlebar-btn svg {
  width: 16px;
  height: 16px;
}

.titlebar-btn:hover {
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.85);
}

.titlebar-btn-close:hover {
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.1);
  color: #fff;
}

.app-body {
  flex: 1;
  min-height: 0;
  display: flex;
  padding: 24px 12px 12px;
  gap: 12px;
  -webkit-app-region: no-drag;
}

.main-content {
  flex: 1;
  min-width: 0;
  min-height: 0;
  padding: 24px;
  overflow: hidden;
  background: rgba(2, 2, 2, 0.753);
  backdrop-filter: blur(16px);
  -webkit-backdrop-filter: blur(16px);
  border-radius: 10px;
}
</style>
