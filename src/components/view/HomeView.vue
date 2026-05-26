<script setup>
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

const stats = ref({ cpu: 0, memory_total: 0, memory_used: 0, self_memory_mb: 0 });
let timer = null;

async function refresh() {
  try {
    stats.value = await invoke("get_system_stats");
  } catch (e) { /* ignore */ }
}

onMounted(() => {
  refresh();
  timer = setInterval(refresh, 2000);
});

onUnmounted(() => {
  clearInterval(timer);
});

function circleDash(percent) {
  const r = 38;
  const c = 2 * Math.PI * r;
  return `${(percent / 100) * c} ${c}`;
}

function memPercent() {
  if (!stats.value.memory_total) return 0;
  return Math.round((stats.value.memory_used / stats.value.memory_total) * 100);
}

function memUsedGB() {
  return (stats.value.memory_used / 1024 / 1024 / 1024).toFixed(1);
}

function memTotalGB() {
  return (stats.value.memory_total / 1024 / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="home-panel">
    <div class="cards-row">
      <div class="stat-card">
        <div class="ring-wrap">
          <svg class="ring" viewBox="0 0 96 96">
            <circle class="ring-bg" cx="48" cy="48" r="38" />
            <circle
              class="ring-fill cpu-ring"
              cx="48"
              cy="48"
              r="38"
              :stroke-dasharray="circleDash(stats.cpu)"
            />
          </svg>
          <div class="stat-info">
            <span class="stat-value">{{ stats.cpu }}<small>%</small></span>
          </div>
        </div>
        <span class="stat-label">CPU</span>
      </div>

      <div class="stat-card">
        <div class="ring-wrap">
          <svg class="ring" viewBox="0 0 96 96">
            <circle class="ring-bg" cx="48" cy="48" r="38" />
            <circle
              class="ring-fill mem-ring"
              cx="48"
              cy="48"
              r="38"
              :stroke-dasharray="circleDash(memPercent())"
            />
          </svg>
          <div class="stat-info">
            <span class="stat-value">{{ memPercent() }}<small>%</small></span>
          </div>
        </div>
        <span class="stat-label">内存 {{ memUsedGB() }} / {{ memTotalGB() }} GB</span>
      </div>

      <div class="stat-card">
        <div class="ring-wrap">
          <svg class="ring" viewBox="0 0 96 96">
            <circle class="ring-bg" cx="48" cy="48" r="38" />
            <circle
              class="ring-fill self-ring"
              cx="48"
              cy="48"
              r="38"
              :stroke-dasharray="circleDash(Math.min(stats.self_memory_mb, 100))"
            />
          </svg>
          <div class="stat-info">
            <span class="stat-value">{{ stats.self_memory_mb }}<small>MB</small></span>
          </div>
        </div>
        <span class="stat-label">NToys</span>
      </div>
    </div>
  </div>
</template>

<style>
.home-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.home-panel h2 {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 24px;
}

.cards-row {
  display: flex;
  gap: 16px;
  flex-wrap: wrap;
}

.stat-card {
  background: rgba(255, 255, 255, 0.04);
  border-radius: 12px;
  padding: 20px;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  width: 140px;
  flex-shrink: 0;
}

.ring-wrap {
  position: relative;
  width: 80px;
  height: 80px;
  flex-shrink: 0;
}

.ring {
  width: 100%;
  height: 100%;
  transform: rotate(-90deg);
}

.ring-bg {
  fill: none;
  stroke: rgba(255, 255, 255, 0.08);
  stroke-width: 6;
}

.ring-fill {
  fill: none;
  stroke-width: 6;
  stroke-linecap: round;
  transition: stroke-dasharray 0.6s ease;
}

.cpu-ring { stroke: #60a5fa; }
.mem-ring { stroke: #34d399; }
.self-ring { stroke: #a78bfa; }

.stat-info {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  align-items: baseline;
}

.stat-value {
  font-size: 22px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.9);
}

.stat-value small {
  font-size: 13px;
  font-weight: 500;
  color: rgba(255, 255, 255, 0.45);
  margin-left: 1px;
}

.stat-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.4);
  margin-top: 2px;
  text-align: center;
}
</style>
