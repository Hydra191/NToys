<script setup>
import ShortcutInput from "../settings/components/ShortcutInput.vue";
import RangeSlider from "../settings/components/RangeSlider.vue";
import ToggleSwitch from "../settings/components/ToggleSwitch.vue";

defineProps({
  sections: { type: Array, required: true },
  models: { type: Object, required: true },
});

const emit = defineEmits(["change"]);

function onChange(key, val) {
  emit("change", key, val);
}
</script>

<template>
  <div class="setting-view">
    <template v-for="section in sections" :key="section.name">
      <div class="section-header">{{ section.name }}</div>
      <div
        v-for="(item, i) in section.items"
        :key="item.key"
        class="settings-item"
        :class="{ 'last-in-section': i === section.items.length - 1 }"
      >
        <ShortcutInput
          v-if="item.type === 'shortcut'"
          :modelValue="models[item.key]"
          :label="item.label"
          @update:modelValue="onChange(item.key, $event)"
        />
        <RangeSlider
          v-else-if="item.type === 'slider'"
          :modelValue="models[item.key]"
          :label="item.label"
          :min="item.min"
          :max="item.max"
          :step="item.step"
          @update:modelValue="onChange(item.key, $event)"
        />
        <ToggleSwitch
          v-else-if="item.type === 'toggle'"
          :modelValue="models[item.key]"
          :label="item.label"
          @update:modelValue="onChange(item.key, $event)"
        />
        <span v-else class="unknown-type">unknown type: {{ item.type }}</span>
      </div>
    </template>
  </div>
</template>

<style>
.setting-view {
  width: 100%;
  flex: 1;
  min-height: 0;
  overflow-y: auto;
}

.section-header {
  font-size: 15px;
  font-weight: 600;
  color: rgb(183, 110, 226);
  text-transform: uppercase;
  letter-spacing: 0.5px;
  padding: 12px 0 0 6px;
  margin-bottom: 4px;
}
.section-header:first-child { padding-top: 0; }

.settings-item {
  display: flex;
  align-items: center;
  height: 80px;
  padding: 0 14px;
  margin-bottom: 4px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.1);
}

.last-in-section {
  border-bottom: none;
}

.settings-item > * {
  width: 100%;
}

.unknown-type {
  font-size: 12px;
  color: rgba(255, 80, 80, 0.6);
}

.setting-view::-webkit-scrollbar { width: 4px; }
.setting-view::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.12);
  border-radius: 2px;
}
</style>
