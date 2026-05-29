<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import SettingView from "../view/SettingView.vue";
import UpdateCheck from "./components/UpdateCheck.vue";
import { settingsSections, settingsState, keyToSection } from "./settingsRegistry.js";

const loading = ref(true);

onMounted(async () => {
  try {
    const s = await invoke("get_settings");
    for (const sec of settingsSections) {
      const data = s[sec.section];
      if (!data) continue;
      for (const item of sec.items) {
        if (data[item.key] !== undefined) {
          settingsState[item.key] = data[item.key];
        }
      }
    }
  } catch (e) { /* ignore */ }
  loading.value = false;
});

async function onChange(key, val) {
  settingsState[key] = val;

  const sec = keyToSection(key);
  if (!sec) return;

  if (sec === "runner") {
    const parts = settingsState.shortcut.split("+");
    if (parts.length < 2 || parts[parts.length - 1].length === 0) {
      settingsState.shortcut = "Alt+Space";
    }
    settingsState.max_visible = Math.round(settingsState.max_visible);
  }

  const data = {};
  for (const s of settingsSections) {
    if (s.section !== sec) continue;
    for (const item of s.items) data[item.key] = settingsState[item.key];
  }
  try {
    await invoke("update_settings_section", { section: sec, data });
  } catch (e) { /* ignore */ }
}
</script>

<template>
  <div class="global-settings-panel" v-if="!loading">
    <SettingView :sections="settingsSections" :models="settingsState" @change="onChange" />
    <div class="section-header">更新</div>
    <UpdateCheck />
  </div>
</template>

<style>
.global-settings-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}
</style>
