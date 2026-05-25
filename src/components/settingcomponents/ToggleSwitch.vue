<script setup>
defineProps({
  modelValue: Boolean,
  label: String,
});

defineEmits(["update:modelValue"]);
</script>

<template>
  <label class="toggle-row">
    <span class="toggle-label">{{ label }}</span>
    <input
      type="checkbox"
      class="toggle"
      :checked="modelValue"
      @change="$emit('update:modelValue', $event.target.checked)"
    />
  </label>
</template>

<style>
.toggle-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  cursor: pointer;
}

.toggle-label {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.7);
  white-space: nowrap;
}

.toggle {
  -webkit-appearance: none;
  appearance: none;
  width: 40px;
  height: 22px;
  border-radius: 11px;
  background: rgba(255, 255, 255, 0.15);
  cursor: pointer;
  position: relative;
  transition: background 0.2s;
  flex-shrink: 0;
}

.toggle::after {
  content: "";
  position: absolute;
  top: 3px;
  left: 3px;
  width: 16px;
  height: 16px;
  border-radius: 50%;
  background: #fff;
  transition: transform 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.toggle:checked {
  background: rgb(140, 94, 212);
}

.toggle:checked::after {
  transform: translateX(18px);
}
</style>
