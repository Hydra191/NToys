<script setup>
import { ref, watch, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { listen } from "@tauri-apps/api/event";

const ITEM_H = 52;
const MAX_VISIBLE = ref(8);
const preventHideOnText = ref(true);
const saveSearchHistory = ref(false);
const appWindow = getCurrentWindow();

function clearIfNeeded() {
  if (!saveSearchHistory.value) {
    query.value = "";
    results.value = [];
  }
}

function hideLauncher() {
  invoke("hide_launcher");
  clearIfNeeded();
}

function onWindowKeydown(e) {
  if (e.key === "Escape") {
    hideLauncher();
  }
}

async function loadSettings() {
  try {
    const s = await invoke("get_settings");
    MAX_VISIBLE.value = s.max_visible;
    preventHideOnText.value = s.prevent_hide_on_text ?? true;
    saveSearchHistory.value = s.save_search_history ?? false;
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
}

onMounted(async () => {
  await loadSettings();
  window.addEventListener("keydown", onWindowKeydown);
  const unlistenFocus = await appWindow.onFocusChanged(({ payload: focused }) => {
    if (focused) {
      const input = document.querySelector(".launcher-input");
      if (input) input.focus();
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

watch(query, (val) => {
  invoke("set_prevent_hide", { prevent: preventHideOnText.value && !!val.trim() });
  clearTimeout(debounceTimer);
  selectedIndex.value = -1;
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
  resizeWindow(results.value.length);
  loadIcons(apps);
}

function loadIcons(apps) {
  apps.forEach(async (app) => {
    if (iconCache.has(app.path)) {
      const idx = results.value.findIndex(r => r.path === app.path);
      if (idx >= 0) results.value[idx].icon = iconCache.get(app.path);
      return;
    }
    const icon = await invoke("get_icon", { path: app.path });
    if (!icon) return;
    iconCache.set(app.path, icon);
    if (iconCache.size > MAX_ICON_CACHE) {
      const oldest = iconCache.keys().next().value;
      iconCache.delete(oldest);
    }
    const idx = results.value.findIndex(r => r.path === app.path);
    if (idx >= 0) results.value[idx].icon = icon;
  });
}

function resizeWindow(count) {
  const visible = Math.min(count, MAX_VISIBLE.value);
  const spacer = count > 0 ? 8 : 0;
  const divider = count > 0 ? 9 : 0;
  const h = 60 + divider + (visible * ITEM_H) + spacer;
  invoke("set_launcher_size", { height: h });
}

function selectApp(app) {
  invoke("launch_app", { path: app.path });
  if (!saveSearchHistory.value) {
    query.value = "";
    results.value = [];
  }
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
  <div class="launcher-container" @mousedown="appWindow.setFocus()">
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
          :src="'data:image/png;base64,' + app.icon"
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
}

#launcher-app {
  height: 100%;
}

html, body {
  height: 100%;
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-rendering: optimizeLegibility;
}

.launcher-container {
  height: 100%;
  background: rgb(15, 15, 15);
  border-radius: 6px;
  -webkit-app-region: drag;
  display: flex;
  flex-direction: column;
}

.search-bar {
  display: flex;
  align-items: center;
  margin: 20px 20px;
  -webkit-app-region: no-drag;
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
  height: 52px;
  padding: 0 12px;
  border-radius: 6px;
  color: rgba(255, 255, 255, 0.85);
  font-size: 14px;
  cursor: pointer;
  -webkit-app-region: no-drag;
  transition: background 0.1s;
  gap: 10px;
}

.result-item:hover,
.result-item.selected {
  background: rgba(255, 255, 255, 0.12);
}

.result-icon {
  width: 28px;
  height: 28px;
  flex-shrink: 0;
  display: block;
  object-fit: contain;
  align-self: center;
}

.result-icon-placeholder {
  width: 28px;
  height: 28px;
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
}

.results-list::-webkit-scrollbar {
  width: 4px;
}

.results-list::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.15);
  border-radius: 2px;
}
</style>

a
