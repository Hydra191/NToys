<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ToggleSwitch from "../settingcomponents/ToggleSwitch.vue";
import ShortcutInput from "../settingcomponents/ShortcutInput.vue";
import { checkForUpdate, downloadUpdate, installUpdate } from "../../scripts/updater.js";

const autostart = ref(false);
const showShortcut = ref("Alt+Shift+N");
const loading = ref(true);
const updateStatus = ref(""); // '' | 'checking' | 'up-to-date' | 'downloading' | 'installing' | 'error'
const updateProgress = ref(0); // 0-100
const updateVersion = ref("");

onMounted(async () => {
  try {
    const settings = await invoke("get_settings");
    autostart.value = settings.autostart;
    showShortcut.value = settings.show_window_shortcut;
  } catch (e) { /* ignore */ }
  loading.value = false;
});

async function onAutostartChange(val) {
  try {
    await invoke("set_autostart", { enable: val });
    autostart.value = val;
  } catch (e) { /* ignore */ }
}

async function onShortcutChange(val) {
  try {
    await invoke("set_show_window_shortcut", { shortcut: val });
    showShortcut.value = val;
  } catch (e) { /* ignore */ }
}

async function onCheckUpdate() {
  if (updateStatus.value === "checking" || updateStatus.value === "downloading") return;
  updateStatus.value = "checking";
  updateProgress.value = 0;
  try {
    const info = await checkForUpdate();
    if (!info) {
      updateStatus.value = "up-to-date";
      setTimeout(() => { updateStatus.value = ""; }, 3000);
      return;
    }
    updateVersion.value = info.version;
    updateStatus.value = "downloading";
    await downloadUpdate(({ downloaded, total }) => {
      updateProgress.value = total > 0 ? Math.round((downloaded / total) * 100) : 0;
    });
    updateStatus.value = "installing";
    await installUpdate();
  } catch (e) {
    updateStatus.value = "error";
    setTimeout(() => { updateStatus.value = ""; }, 3000);
  }
}
</script>

<template>
  <div class="global-settings-panel" v-if="!loading">
    <div class="settings-section">
      <ToggleSwitch :modelValue="autostart" @update:modelValue="onAutostartChange" label="开机自启动" />
    </div>
    <div class="settings-section">
      <ShortcutInput :modelValue="showShortcut" @update:modelValue="onShortcutChange" label="显示主界面" />
    </div>
    <div class="settings-section update-section">
      <div class="update-row">
        <span class="update-label">版本更新</span>
        <button
          class="update-btn"
          :class="{ checking: updateStatus === 'checking' || updateStatus === 'downloading' || updateStatus === 'installing' }"
          :disabled="updateStatus === 'checking' || updateStatus === 'downloading' || updateStatus === 'installing'"
          @click="onCheckUpdate"
        >
          <span v-if="updateStatus === 'checking'">检查中...</span>
          <span v-else-if="updateStatus === 'downloading'">下载中 {{ updateProgress }}%</span>
          <span v-else-if="updateStatus === 'installing'">安装中...</span>
          <span v-else-if="updateStatus === 'up-to-date'">已是最新</span>
          <span v-else-if="updateStatus === 'error'">检查失败</span>
          <span v-else>检查更新</span>
        </button>
      </div>
    </div>
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
.global-settings-panel h2 {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 24px;
}
.settings-section {
  padding: 16px 0;
  border-bottom: 1px solid rgba(255, 255, 255, 0.06);
}

.update-section {
  margin-top: 8px;
}

.update-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.update-label {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.7);
}

.update-btn {
  padding: 6px 16px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.65);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
  min-width: 100px;
  text-align: center;
}

.update-btn:hover {
  background: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.85);
}

.update-btn:disabled {
  cursor: default;
  opacity: 0.6;
}

.update-btn.checking {
  color: rgb(140, 94, 212);
  border-color: rgba(140, 94, 212, 0.3);
}
</style>
