<script setup>
import { ref, watch, nextTick, onMounted } from "vue";

const props = defineProps({
  modelValue: String,
  label: { type: String, default: "快捷键" },
});

const emit = defineEmits(["update:modelValue"]);

const display = ref(props.modelValue);
const inputEl = ref(null);
const mirrorEl = ref(null);

watch(
  () => props.modelValue,
  (v) => {
    display.value = v;
  }
);

function syncWidth() {
  nextTick(() => {
    if (inputEl.value && mirrorEl.value) {
      inputEl.value.style.width = mirrorEl.value.offsetWidth + "px";
    }
  });
}

watch(display, syncWidth);
onMounted(syncWidth);

function onKeyCapture(e) {
  const key = e.key;
  if (["Control", "Alt", "Shift", "Meta"].includes(key)) return;

  const parts = [];
  if (e.ctrlKey) parts.push("Ctrl");
  if (e.altKey) parts.push("Alt");
  if (e.shiftKey) parts.push("Shift");
  if (e.metaKey) parts.push("Meta");

  let keyName;
  if (key === " ") {
    keyName = "Space";
  } else if (key.length === 1) {
    keyName = key.toUpperCase();
  } else {
    keyName = key;
  }

  if (!keyName) return;
  if (parts.length === 0) return;

  parts.push(keyName);
  const combo = parts.join("+");
  display.value = combo;
  emit("update:modelValue", combo);
}
</script>

<template>
  <div class="shortcut-row">
    <label class="shortcut-label">{{ label }}</label>
    <div class="shortcut-input-wrap">
      <input
        ref="inputEl"
        class="shortcut-input"
        :value="display"
        @keydown.prevent="onKeyCapture"
        placeholder="点击后按键..."
        readonly
      />
      <span ref="mirrorEl" class="shortcut-mirror" aria-hidden="true">{{ display || "点击后按键..." }}</span>
    </div>
  </div>
</template>

<style>
.shortcut-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.shortcut-input-wrap {
  position: relative;
  display: inline-block;
  margin-left: auto;
}

.shortcut-label {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.7);
  white-space: nowrap;
  flex-shrink: 0;
}

.shortcut-input {
  padding: 8px 12px;
  background: rgba(255, 255, 255, 0.08);
  border: 1px solid rgba(255, 255, 255, 0.12);
  border-radius: 6px;
  color: #fff;
  font-size: 14px;
  font-family: inherit;
  outline: none;
  cursor: text;
  box-sizing: border-box;
}

.shortcut-input:focus {
  border-color: rgba(255, 255, 255, 0.3);
}

.shortcut-mirror {
  position: absolute;
  top: 0;
  left: 0;
  padding: 8px 12px;
  font-size: 14px;
  font-family: inherit;
  white-space: nowrap;
  visibility: hidden;
  pointer-events: none;
  border: 1px solid transparent;
}
</style>
