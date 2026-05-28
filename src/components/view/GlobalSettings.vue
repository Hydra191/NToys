<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ToggleSwitch from "../settingcomponents/ToggleSwitch.vue";
import ShortcutInput from "../settingcomponents/ShortcutInput.vue";

const autostart = ref(false);
const showShortcut = ref("Alt+Shift+N");
const loading = ref(true);

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
</script>

<template>
  <div class="global-settings-panel" v-if="!loading">
    <div class="settings-section">
      <ToggleSwitch :modelValue="autostart" @update:modelValue="onAutostartChange" label="开机自启动" />
    </div>
    <div class="settings-section">
      <ShortcutInput :modelValue="showShortcut" @update:modelValue="onShortcutChange" label="显示主界面" />
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
</style>
