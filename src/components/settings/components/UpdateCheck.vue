<script setup>
import { ref } from "vue";
import { checkForUpdate, downloadUpdate, installUpdate } from "../../../scripts/updater.js";

const status = ref(""); // '' | 'checking' | 'downloading' | 'installing' | 'up-to-date' | 'error'
const progress = ref(0);

async function onCheck() {
  if (status.value === "checking" || status.value === "downloading") return;
  status.value = "checking";
  progress.value = 0;
  try {
    const info = await checkForUpdate();
    if (!info) {
      status.value = "up-to-date";
      setTimeout(() => { status.value = ""; }, 3000);
      return;
    }
    status.value = "downloading";
    await downloadUpdate(({ downloaded, total }) => {
      progress.value = total > 0 ? Math.round((downloaded / total) * 100) : 0;
    });
    status.value = "installing";
    await installUpdate();
  } catch (e) {
    status.value = "error";
    setTimeout(() => { status.value = ""; }, 3000);
  }
}
</script>

<template>
  <div class="update-row">
    <span class="update-label">版本更新</span>
    <button
      class="update-btn"
      :class="{ checking: status === 'checking' || status === 'downloading' || status === 'installing' }"
      :disabled="status === 'checking' || status === 'downloading' || status === 'installing'"
      @click="onCheck"
    >
      <span v-if="status === 'checking'">检查中...</span>
      <span v-else-if="status === 'downloading'">下载中 {{ progress }}%</span>
      <span v-else-if="status === 'installing'">安装中...</span>
      <span v-else-if="status === 'up-to-date'">已是最新</span>
      <span v-else-if="status === 'error'">检查失败</span>
      <span v-else>检查更新</span>
    </button>
  </div>
</template>

<style>
.update-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 12px 0;
}
.update-label { font-size: 14px; color: rgba(255, 255, 255, 0.7); }
.update-btn {
  padding: 6px 16px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  color: rgba(255, 255, 255, 0.65);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
  min-width: 100px;
  text-align: center;
}
.update-btn:hover { background: rgba(255, 255, 255, 0.12); color: rgba(255, 255, 255, 0.85); }
.update-btn:disabled { cursor: default; opacity: 0.6; }
.update-btn.checking { color: rgb(140, 94, 212); border-color: rgba(140, 94, 212, 0.3); }
</style>
