<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import UpdateCheck from "../settings/components/UpdateCheck.vue";

defineEmits(["close"]);

const version = ref("");
const debugMsg = ref("");

onMounted(async () => {
  try {
    version.value = await invoke("get_app_version");
  } catch (e) { /* ignore */ }
});

async function onDebugCheck() {
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const u = await check();
    debugMsg.value = u
      ? `found: v${u.version}`
      : "no update found";
  } catch (e) {
    debugMsg.value = `err: ${e}`;
  }
}
</script>

<template>
  <div class="about-overlay" @click.self="$emit('close')">
    <div class="about-modal">
      <svg class="about-logo" viewBox="0 0 16 16" fill="#A78BFA" xmlns="http://www.w3.org/2000/svg">
        <path d="M6 0.278a0.77 0.77 0 0 1 0.08 0.858 7.2 7.2 0 0 0-0.878 3.46c0 4.021 3.278 7.277 7.318 7.277q0.792-1e-3 1.533-0.16a0.79 0.79 0 0 1 0.81 0.316 0.73 0.73 0 0 1-0.031 0.893A8.35 8.35 0 0 1 8.344 16C3.734 16 0 12.286 0 7.71 0 4.266 2.114 1.312 5.124 0.06A0.75 0.75 0 0 1 6 0.278" />
        <path transform="rotate(-33 12.052125 5.611427)" d="M10.2951 4.8033a0.217 0.217 0 0 1 0.412 0l0.387 1.162c0.173 0.518 0.579 0.924 1.097 1.097l1.162 0.387a0.217 0.217 0 0 1 0 0.412l-1.162 0.387a1.73 1.73 0 0 0-1.097 1.097l-0.387 1.162a0.217 0.217 0 0 1-0.412 0l-0.387-1.162a1.73 1.73 0 0 0-1.097-1.097l-1.162-0.387a0.217 0.217 0 0 1 0-0.412l1.162-0.387a1.73 1.73 0 0 0 1.097-1.097zM13.3641 1.7543a0.145 0.145 0 0 1 0.274 0l0.258 0.774c0.115 0.346 0.386 0.617 0.732 0.732l0.774 0.258a0.145 0.145 0 0 1 0 0.274l-0.774 0.258a1.16 1.16 0 0 0-0.732 0.732l-0.258 0.774a0.145 0.145 0 0 1-0.274 0l-0.258-0.774a1.16 1.16 0 0 0-0.732-0.732l-0.774-0.258a0.145 0.145 0 0 1 0-0.274l0.774-0.258c0.346-0.115 0.617-0.386 0.732-0.732z" />
      </svg>
      <h1 class="about-title">Luna Toys</h1>
      <p class="about-version" v-if="version">v{{ version }}</p>
      <UpdateCheck />
      <button class="debug-btn" @click="onDebugCheck">debug: check()</button>
      <p class="debug-msg" v-if="debugMsg">{{ debugMsg }}</p>
      <button class="about-close-btn" @click="$emit('close')">关闭</button>
    </div>
  </div>
</template>

<style>
.about-overlay {
  position: fixed; inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex; align-items: center; justify-content: center;
  z-index: 200;
}
.about-modal {
  background: rgb(36, 36, 36);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 32px 36px 24px;
  text-align: center;
  min-width: 280px;
}
.about-logo {
  width: 48px;
  height: 48px;
  margin-bottom: 8px;
}
.about-title {
  font-size: 28px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.9);
  margin-bottom: 4px;
}
.about-version {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.35);
  margin-bottom: 20px;
}
.about-close-btn {
  margin-top: 16px;
  padding: 6px 20px;
  border: none; border-radius: 6px;
  background: rgba(255, 255, 255, 0.12);
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px; cursor: pointer;
}
.about-close-btn:hover { background: rgba(255, 255, 255, 0.2); }
.debug-btn {
  margin-top: 8px;
  padding: 4px 12px;
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 4px;
  background: transparent;
  color: rgba(255, 255, 255, 0.3);
  font-size: 11px;
  cursor: pointer;
}
.debug-msg {
  margin-top: 4px;
  font-size: 12px;
  color: #fbbf24;
}
</style>
