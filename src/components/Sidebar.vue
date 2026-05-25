<script setup>
defineProps({
  activePlugin: String,
});

defineEmits(["select"]);

const plugins = [
  { id: "home", name: "Home" },
  { id: "runner", name: "Runner" },
  { id: "vpn", name: "VPN" },
  { id: "music", name: "Music" },
];
</script>

<template>
  <div class="sidebar">
    <div class="plugin-list">
      <div
        v-for="plugin in plugins"
        :key="plugin.id"
        class="plugin-item"
        :class="{ active: activePlugin === plugin.id }"
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
      </div>
    </div>

    <div class="sidebar-bottom">
      <div
        class="plugin-item"
        :class="{ active: activePlugin === 'settings' }"
        title="Settings"
        @click="$emit('select', 'settings')"
      >
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z"/>
        </svg>
      </div>
    </div>
  </div>
</template>

<style>
.sidebar {
  width: 90px;
  flex-shrink: 0;
  display: flex;
  flex-direction: column;
  padding: 16px;
  background: rgba(2, 2, 2, 0.699);
  border-radius: 10px;
}

.plugin-list {
  margin-top: 10px;
  display: flex;
  align-items: center;
  flex-direction: column;
  gap: 10px;
}

.plugin-item {
  width: 45px;
  height: 45px;
  border-radius: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  color: rgba(255, 255, 255, 0.45);
  background: transparent;
  transition: background 0.15s, color 0.15s;
}

.plugin-item svg {
  width: 20px;
  height: 20px;
}

.plugin-item:hover {
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.7);
}

.plugin-item.active {
  background: rgb(140, 94, 212);
  color: rgba(255, 255, 255, 0.9);
}

.sidebar-bottom {
  margin-top: auto;
  display: flex;
  align-items: center;
  flex-direction: column;
}
</style>
