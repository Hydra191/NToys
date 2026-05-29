<script setup>
import { ref, watch, onMounted, onUnmounted, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

let ITEM_H = 32;
let headerH = 40;
let dividerH = 9;
let spacerH = 8;
let itemGap = 4;

function measureLayout() {
  const sb = document.querySelector('.search-bar');
  if (sb) {
    const s = getComputedStyle(sb);
    headerH = sb.offsetHeight + parseInt(s.marginTop) + parseInt(s.marginBottom);
  }
  const div = document.querySelector('.results-divider');
  if (div) {
    const s = getComputedStyle(div);
    dividerH = div.offsetHeight + parseInt(s.marginTop) + parseInt(s.marginBottom);
  }
  const sp = document.querySelector('.results-spacer');
  if (sp) spacerH = sp.offsetHeight;
  const item = document.querySelector('.result-item');
  if (item) {
    ITEM_H = item.offsetHeight;
    itemGap = parseInt(getComputedStyle(item).marginBottom) || 0;
  }
}
const MAX_VISIBLE = ref(8);
const preventHideOnText = ref(true);
const appWindow = getCurrentWindow();

function clearIfNeeded() {
  loadIconsGeneration++;
  query.value = "";
  results.value = [];
}

function hideLauncher() {
  invoke("hide_launcher");
  clearIfNeeded();
}

function onContainerMouseDown(e) {
  if (e.target.closest('.launcher-input') || e.target.closest('.result-item') || e.target.closest('.results-list')) {
    return;
  }
  appWindow.startDragging();
}

function onWindowKeydown(e) {
  if (e.key === "Escape") {
    hideLauncher();
  }
}

async function loadSettings() {
  try {
    const s = await invoke("get_settings");
    MAX_VISIBLE.value = s.runner.max_visible;
    preventHideOnText.value = s.runner.prevent_hide_on_text ?? true;
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
}

onMounted(async () => {
  await loadSettings();
  await nextTick();
  measureLayout();
  invoke("set_launcher_size", { height: headerH });
  window.addEventListener("keydown", onWindowKeydown);
  const unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
    if (focused) {
      (function tryFocus(n) {
        const input = document.querySelector(".launcher-input");
        if (input) { input.focus(); input.select(); }
        if (document.activeElement !== input && n < 30) {
          requestAnimationFrame(() => tryFocus(n + 1));
        }
      })(0);
    }
  });
  const unlistenSettings = await listen("settings-changed", () => {
    loadSettings();
  });
  const unlistenHidden = await listen("launcher-hidden", () => {
    clearIfNeeded();
  });
  window._focusUnlisten = unlistenFocus;
  window._settingsUnlisten = unlistenSettings;
  window._hiddenUnlisten = unlistenHidden;
});

onUnmounted(() => {
  clearTimeout(debounceTimer);
  window.removeEventListener("keydown", onWindowKeydown);
  if (window._focusUnlisten) window._focusUnlisten();
  if (window._settingsUnlisten) window._settingsUnlisten();
  if (window._hiddenUnlisten) window._hiddenUnlisten();
});
const query = ref("");
const results = ref([]);
const selectedIndex = ref(-1);
const MAX_ICON_CACHE = 200;
const iconCache = new Map();

let debounceTimer = null;
let loadIconsGeneration = 0;

watch(query, (val) => {
  invoke("set_prevent_hide", { prevent: preventHideOnText.value && !!val.trim() });
  clearTimeout(debounceTimer);
  selectedIndex.value = -1;
  loadIconsGeneration++; // cancel stale icon loads
  debounceTimer = setTimeout(() => search(val), 60);
});

async function search(q) {
  if (!q.trim()) {
    results.value = [];
    resizeWindow(0);
    return;
  }
  const apps = await invoke("search_apps", { query: q });
  results.value = apps.map(a => ({ ...a, icon: "" }));
  await nextTick();
  resizeWindow(results.value.length);
  loadIcons(apps);
}

function loadIcons(apps) {
  const gen = loadIconsGeneration;
  // Serve cache hits first, collect cache misses
  const missing = [];
  for (const app of apps) {
    if (iconCache.has(app.path)) {
      const idx = results.value.findIndex(r => r.path === app.path);
      if (idx >= 0) results.value[idx].icon = iconCache.get(app.path);
    } else {
      missing.push(app);
    }
  }
  if (missing.length === 0) return;
  // Batch processing: 5 concurrent max to limit memory spikes
  const BATCH = 5;
  let i = 0;
  async function next() {
    while (i < missing.length) {
      if (gen !== loadIconsGeneration) return;
      const app = missing[i++];
      const dataUrl = await invoke("get_icon", { path: app.path });
      if (gen !== loadIconsGeneration || !dataUrl) continue;
      iconCache.set(app.path, dataUrl);
      if (iconCache.size > MAX_ICON_CACHE) {
        const oldest = iconCache.keys().next().value;
        iconCache.delete(oldest);
      }
      if (gen !== loadIconsGeneration) return;
      const idx = results.value.findIndex(r => r.path === app.path);
      if (idx >= 0) results.value[idx].icon = dataUrl;
    }
  }
  Array.from({ length: Math.min(BATCH, missing.length) }, () => next());
}

function resizeWindow(count) {
  if (count > 0) measureLayout();
  const visible = Math.min(count, MAX_VISIBLE.value);
  const d = count > 0 ? dividerH : 6;
  const sp = count > 0 ? spacerH : 6;
  const h = headerH + d + (visible * ITEM_H) + ((visible - 1) * itemGap) + sp;
  invoke("set_launcher_size", { height: h });
}

function selectApp(app) {
  invoke("launch_app", { path: app.path });
  query.value = "";
  results.value = [];
}

function onKeydown(e) {
  if (e.key === "ArrowDown") {
    e.preventDefault();
    selectedIndex.value = Math.min(selectedIndex.value + 1, results.value.length - 1);
  } else if (e.key === "ArrowUp") {
    e.preventDefault();
    selectedIndex.value = Math.max(selectedIndex.value - 1, -1);
  } else if (e.key === "Enter" && selectedIndex.value >= 0) {
    e.preventDefault();
    selectApp(results.value[selectedIndex.value]);
  } else if (e.key === "Escape") {
    hideLauncher();
  }
}
</script>

<template>
  <div class="launcher-container" @mousedown="onContainerMouseDown">
    <div class="search-bar">
      <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8" />
        <path d="M21 21l-4.35-4.35" />
      </svg>
      <input
        class="launcher-input"
        v-model="query"
        type="text"
        placeholder="Search apps..."
        autofocus
        @keydown="onKeydown"
      />
    </div>
    
    <hr v-if="results.length" class="results-divider" />
    <ul v-if="results.length" class="results-list">
      <li
        v-for="(app, i) in results"
        :key="app.path"
        class="result-item"
        :class="{ selected: i === selectedIndex }"
        @click="selectApp(app)"
        @mouseenter="selectedIndex = i"
      >
        <img
          v-if="app.icon"
          class="result-icon"
          :src="app.icon"
        />
        <span v-else class="result-icon-placeholder">{{ app.name[0] }}</span>
        <span class="result-name">{{ app.name }}</span>
      </li>
      <li class="results-spacer" />
    </ul>
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
  background: transparent;
  color: rgba(255, 255, 255, 1);
  font-family: Arial, Helvetica, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: optimizeLegibility;
}

#launcher-app {
  height: 100%;
}


.launcher-container {
  height: 100%;
  background: transparent;
  border-radius: 6px;
  -webkit-app-region: drag;
  display: flex;
  flex-direction: column;
}

.search-bar {
  display: flex;
  align-items: center;
  margin: 16px 20px;
  flex-shrink: 0;
}

.search-icon {
  width: 18px;
  height: 18px;
  color: rgba(255, 255, 255, 0.4);
  flex-shrink: 0;
  margin-right: 10px;
}

.launcher-input {
  height: 20px;
  padding: 0;
  border: none;
  font-size: 16px;
  color: #ffffff;
  background: rgba(255, 255, 255, 0);
  outline: none;
  -webkit-app-region: no-drag;
  flex: 1;
  flex-shrink: 1;
}

.results-divider {
  border: none;
  border-top: 1px solid rgba(255, 255, 255, 0.12);
  margin: 4px 0px;
  flex-shrink: 0;
}

.results-list {
  list-style: none;
  overflow-y: auto;
  flex: 1;
  padding: 0 8px;
  -webkit-app-region: no-drag;
}

.results-spacer {
  height: 8px;
}

.result-item {
  display: flex;
  align-items: center;
  height: 42px;
  padding: 0 12px;
  margin-bottom: 14px;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.85);
  cursor: pointer;
  transition: background 0.1s;
  gap: 10px;
}

.result-item:hover,
.result-item.selected {
  background: rgba(255, 255, 255, 0.12);
}

.result-icon {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  display: block;
  object-fit: contain;
  align-self: center;
}

.result-icon-placeholder {
  width: 24px;
  height: 24px;
  flex-shrink: 0;
  align-self: center;
  border-radius: 4px;
  background: rgba(255, 255, 255, 0.15);
  color: rgba(255, 255, 255, 0.7);
  font-size: 20px;
  font-weight: 600;
  display: flex;
  align-items: center;
  justify-content: center;
}



.result-name {
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  font-size: 16px;
}

.results-list::-webkit-scrollbar {
  width: 2px;
}

.results-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
}
</style>

a
