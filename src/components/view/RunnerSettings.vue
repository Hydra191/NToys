<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import ToggleSwitch from "../settingcomponents/ToggleSwitch.vue";
import RangeSlider from "../settingcomponents/RangeSlider.vue";
import ShortcutInput from "../settingcomponents/ShortcutInput.vue";

const shortcut = ref("Alt+Space");
const maxVisible = ref(8);
const sliderValue = ref(8);
const preventHideOnText = ref(true);
const dirty = ref(false);
const saved = ref(false);
let savedTimer = null;

onMounted(async () => {
  try {
    const s = await invoke("get_settings");
    shortcut.value = s.shortcut;
    maxVisible.value = s.max_visible;
    sliderValue.value = s.max_visible;
    preventHideOnText.value = s.prevent_hide_on_text ?? true;
  } catch (e) {
    console.error("Failed to load settings:", e);
  }
});

onUnmounted(() => {
  clearTimeout(savedTimer);
});

function onShortcutChange(v) {
  shortcut.value = v;
  dirty.value = true;
  saved.value = false;
}

function onSliderChange() {
  maxVisible.value = Math.round(sliderValue.value);
  dirty.value = true;
  saved.value = false;
}

async function saveSettings() {
  // Prevent saving shortcuts with only modifiers
  const parts = shortcut.value.split("+");
  if (parts.length < 2 || parts[parts.length - 1].length === 0) {
    shortcut.value = "Alt+Space";
    dirty.value = true;
    return;
  }

  try {
    await invoke("update_settings", {
      shortcut: shortcut.value,
      maxVisible: maxVisible.value,
      preventHideOnText: preventHideOnText.value,
    });
    dirty.value = false;
    saved.value = true;
    savedTimer = setTimeout(() => {
      saved.value = false;
    }, 2000);
  } catch (e) {
    console.error("Failed to save settings:", e);
    shortcut.value = "Alt+Space";
    dirty.value = true;
  }
}
</script>

<template>
  <div class="settings-panel">
    <div class="settings-body">
      <h2>Runner</h2>
      <p class="plugin-desc">搜索软件然后快速启动.</p>

      <div class="settings-group">
        <ShortcutInput v-model="shortcut" label="快捷键" @update:model-value="onShortcutChange" />
      </div>

      <RangeSlider
        label="显示数量"
        v-model="sliderValue"
        :min="1"
        :max="20"
        :step="0.01"
        @update:model-value="onSliderChange"
      />
      
        <div class="settings-group">
          <ToggleSwitch v-model="preventHideOnText" label="输入框有内容时保持窗口" @update:model-value="dirty = true; saved = false" />
        </div>

    </div>



    <div class="save-bar">
      <button class="save-btn" :disabled="!dirty" @click="saveSettings">
        {{ saved ? "已保存" : "保存设置" }}
      </button>
    </div>
  </div>
</template>

<style>
.settings-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;

}

.settings-body {
  flex: 1;
  
}


.save-bar {
  display: flex;
  justify-content: flex-end;
}

.settings-panel h2 {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 4px;

}

.plugin-desc {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
  margin-bottom: 24px;
}

.settings-group {
  margin-bottom: 20px;
}


.save-btn {
  margin-top: 8px;
  padding: 8px 20px;
  background: rgba(255, 255, 255, 0.12);
  border: none;
  border-radius: 6px;
  color: #fff;
  font-size: 14px;
  cursor: pointer;
  transition: background 0.15s;
}

.save-btn:hover:not(:disabled) {
  background: rgba(255, 255, 255, 0.2);
}

.save-btn:disabled {
  opacity: 0.4;
  cursor: default;
}
</style>
