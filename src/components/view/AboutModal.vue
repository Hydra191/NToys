<script setup>
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import UpdateCheck from "../settings/components/UpdateCheck.vue";
import logoSrc from "../../assets/icon/luna.svg";

defineEmits(["close"]);

const version = ref("");

onMounted(async () => {
  try {
    version.value = await invoke("get_app_version");
  } catch (e) { /* ignore */ }
});
</script>

<template>
  <div class="about-overlay" @click.self="$emit('close')">
    <div class="about-modal">
      <img :src="logoSrc" class="about-logo" />
      <h1 class="about-title">Luna Toys</h1>
      <p class="about-version" v-if="version">v{{ version }}</p>
      <UpdateCheck />
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
</style>
