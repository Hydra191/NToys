<script setup>
import { ref, watch, onMounted } from "vue";

const props = defineProps({
  modelValue: Number,
  min: { type: Number, default: 1 },
  max: { type: Number, default: 20 },
  step: { type: Number, default: 0.01 },
  label: String,
});

const emit = defineEmits(["update:modelValue"]);

const sliderEl = ref(null);

function pct(v) {
  return ((v - props.min) / (props.max - props.min)) * 100;
}

function onInput(e) {
  const v = Number(e.target.value);
  e.target.style.setProperty("--fill", pct(v) + "%");
  emit("update:modelValue", v);
}

watch(
  () => props.modelValue,
  (v) => {
    if (sliderEl.value) {
      sliderEl.value.style.setProperty("--fill", pct(v) + "%");
    }
  },
  { immediate: true }
);
onMounted(() => {
  if (sliderEl.value) {
    sliderEl.value.style.setProperty("--fill", pct(props.modelValue) + "%");
  }
});
</script>

<template>
  <div class="slider-group">
    <div class="slider-row">
      <label class="slider-label">{{ label }}</label>
      <span class="slider-value-badge">{{ Math.round(modelValue) }}</span>
      <div class="slider-track-group">
        <span class="slider-minmax">{{ min }}</span>
        <input
          ref="sliderEl"
          type="range"
          class="slider"
          :value="modelValue"
          :min="min"
          :max="max"
          :step="step"
          @input="onInput"
        />
        <span class="slider-minmax">{{ max }}</span>
      </div>
    </div>
  </div>
</template>

<style>
@property --fill {
  syntax: "<percentage>";
  inherits: true;
  initial-value: 50%;
}

.slider-group {
  margin-bottom: 0px;
}

.slider-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.slider-label {
  font-size: 14px;
  font-weight: 500;
  color: rgba(255, 255, 255,0.8);
  white-space: nowrap;
  flex-shrink: 0;
}

.slider-value-badge {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 28px;
  height: 22px;
  padding: 0 6px;
  font-size: 12px;
  font-weight: 600;
  color: #fff;
  background: rgba(140, 94, 212, 0.3);
  border-radius: 6px;
  flex-shrink: 0;
}

.slider {
  -webkit-appearance: none;
  appearance: none;
  width: 100%;
  flex-shrink: 1;
  height: 12px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.1);
  outline: none;
  cursor: pointer;
}

.slider::-webkit-slider-runnable-track {
  height: 12px;
  border-radius: 6px;
  background: linear-gradient(
    to right,
    rgb(140, 94, 212) 0%,
    rgb(140, 94, 212) var(--fill, 50%),
    rgba(255, 255, 255, 0.1) var(--fill, 50%),
    rgba(255, 255, 255, 0.1) 100%
  );
}

.slider::-moz-range-track {
  height: 6px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.1);
}

.slider::-moz-range-progress {
  height: 6px;
  border-radius: 3px;
  background: rgb(140, 94, 212);
}

.slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 18px;
  height: 18px;
  border-radius: 50%;
  background: rgb(140, 94, 212);
  border: 2px solid rgba(255, 255, 255);
  margin-top: -3px;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
  transition: transform 0.15s, box-shadow 0.15s;
}

.slider::-webkit-slider-thumb:hover {
  transform: scale(1.25);
  box-shadow: 0 2px 8px rgba(140, 94, 212, 0.5);
}

.slider::-webkit-slider-thumb:active {
  transform: scale(1.1);
  background: rgb(140, 94, 212);
  box-shadow: 0 0 0 6px rgba(140, 94, 212, 0.25);
}

.slider::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  border: none;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
  transition: transform 0.15s, box-shadow 0.15s;
}

.slider::-moz-range-thumb:hover {
  transform: scale(1.25);
  box-shadow: 0 2px 8px rgba(140, 94, 212, 0.5);
}

.slider::-moz-range-thumb:active {
  transform: scale(1.1);
  background: rgb(140, 94, 212);
  box-shadow: 0 0 0 6px rgba(140, 94, 212, 0.25);
}

.slider-track-group {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: auto;
  width: 50%;
}

.slider-minmax {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.35);
  flex-shrink: 0;
}
</style>
