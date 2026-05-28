<script setup>
import { ref, watch, nextTick, onMounted } from "vue";

const props = defineProps({
  activePlugin: String,
});

defineEmits(["select"]);

const expanded = ref(false);
const indicatorTop = ref(0);
const indicatorH = ref(40);
const itemRefs = {};

function setRef(id) {
  return (el) => {
    if (el) itemRefs[id] = el;
  };
}

function updateIndicator() {
  const el = itemRefs[props.activePlugin];
  if (!el) return;
  const sidebar = el.closest('.sidebar');
  if (!sidebar) return;
  let top = 0;
  let cur = el;
  while (cur && cur !== sidebar) {
    top += cur.offsetTop;
    cur = cur.offsetParent;
  }
  indicatorTop.value = top;
  indicatorH.value = el.offsetHeight;
}

onMounted(() => updateIndicator());
watch(() => props.activePlugin, async () => {
  await nextTick();
  updateIndicator();
});

const plugins = [
  { id: "home", name: "Home" },
  { id: "runner", name: "Runner" },
  { id: "vpn", name: "VPN" },
  { id: "music", name: "Music" },
];
</script>

<template>
  <div class="sidebar" :class="{ expanded }" @mouseenter="expanded = true" @mouseleave="expanded = false">
    <div class="indicator" :style="{ top: indicatorTop + 'px', height: indicatorH + 'px' }" />
    <div class="plugin-list">
      <div
        v-for="plugin in plugins"
        :key="plugin.id"
        class="plugin-item"
        :class="{ active: activePlugin === plugin.id }"
        :ref="setRef(plugin.id)"
        :title="plugin.name"
        @click="$emit('select', plugin.id)"
      >
        <!-- Home -->
        <svg v-if="plugin.id === 'home'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2z" />
          <polyline points="9 22 9 12 15 12 15 22" />
        </svg>
        <!-- Runner -->
        <svg v-if="plugin.id === 'runner'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="4 17 10 11 4 5" />
          <line x1="12" y1="19" x2="20" y2="19" />
        </svg>
        <!-- VPN -->
        <svg v-if="plugin.id === 'vpn'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <circle cx="12" cy="12" r="10" />
          <line x1="2" y1="12" x2="22" y2="12" />
          <path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z" />
        </svg>
        <!-- Music -->
        <svg v-if="plugin.id === 'music'" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round">
          <path d="M9 18V5l12-2v13" />
          <circle cx="6" cy="18" r="3" />
          <circle cx="18" cy="16" r="3" />
        </svg>
        <span class="plugin-label">{{ plugin.name }}</span>
      </div>
    </div>

    <div class="sidebar-bottom">
      <div
        class="plugin-item"
        :class="{ active: activePlugin === 'settings' }"
        :ref="setRef('settings')"
        title="Settings"
        @click="$emit('select', 'settings')"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
        </svg>
        <span class="plugin-label">Settings</span>
      </div>
    </div>
  </div>
</template>

<style>
.sidebar {
  width: 56px;
  font-family: 'Google Sans Bold';
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  padding: 12px 8px;
  background: rgba(138, 138, 138, 0.1);
  border: 1px solid rgba(255, 255, 255, 0.1);
  backdrop-filter: blur(10px);
  border-radius: 10px;
  transition: width 0.25s ease-in-out;
  overflow: hidden;
  position: relative;
}

.sidebar.expanded {
  width: 150px;
}

.indicator {
  position: absolute;
  left: 8px;
  right: 8px;
  border-radius: 8px;
  background: rgb(140, 94, 212);
  transition: top 0.15s ease-out, height 0.15s ease-out;
  z-index: 0;
}

.plugin-list {
  margin-top: 10px;
  display: flex;
  align-items: stretch;
  flex-direction: column;
  gap: 6px;
  position: relative;
  z-index: 1;
}

.plugin-item {
  height: 40px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  padding: 0 10px;
  gap: 10px;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.45);
  background: transparent;
  transition: color 0.15s;
  white-space: nowrap;
  overflow: hidden;
  position: relative;
  z-index: 1;
}

.plugin-item svg {
  width: 20px;
  height: 20px;
  flex-shrink: 0;
}

.plugin-item:hover {
  color: rgba(255, 255, 255, 0.7);
}

.plugin-item.active {
  color: rgba(255, 255, 255, 0.9);
}

.plugin-label {
  font-size: 13px;
  font-weight: 500;
  opacity: 0;
  transition: opacity 0.15s ease;
  overflow: hidden;
}

.sidebar.expanded .plugin-label {
  opacity: 1;
}

.sidebar-bottom {
  margin-top: auto;
  display: flex;
  align-items: stretch;
  flex-direction: column;
  position: relative;
  z-index: 1;
}
</style>
