<script setup>
import { ref, onMounted, onUnmounted, inject, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { weatherState, weatherIcon } from "../../scripts/weather.js";

const windowVisible = inject("windowVisible");
const stats = ref({ cpu: 0, memory_total: 0, memory_used: 0, self_memory_mb: 0, download_speed: 0, upload_speed: 0 });
let timer = null;

async function refresh() {
  if (!windowVisible.value) return;
  try {
    stats.value = await invoke("get_system_stats");
  } catch (e) { /* ignore */ }
}

// ── time ────────────────────────────────
const now = ref(new Date());
let timeTimer = null;

function pad(n) { return String(n).padStart(2, "0"); }
const timeStr = () => {
  const d = now.value;
  return `${d.getHours()}:${pad(d.getMinutes())}`;
};
const dateStr = () => {
  const d = now.value;
  const days = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
  return `${d.getFullYear()}年${d.getMonth() + 1}月${d.getDate()}日 ${days[d.getDay()]}`;
};

// ── weather (shared state, refreshed every 30min at app level) ──
const weather = computed(() => weatherState.current);
const forecast = computed(() => weatherState.forecast);
const city = computed(() => weatherState.city);

onMounted(() => {
  refresh();
  timer = setInterval(refresh, 2000);
  now.value = new Date();
  timeTimer = setInterval(() => { if (windowVisible.value) now.value = new Date(); }, 1000);
});

onUnmounted(() => {
  clearInterval(timer);
  clearInterval(timeTimer);
});

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

function formatSpeed(bytesPerSec) {
  if (!bytesPerSec || bytesPerSec < 0) return "0 B/s";
  if (bytesPerSec < 1024) return `${Math.round(bytesPerSec)} B/s`;
  if (bytesPerSec < 1024 * 1024) return `${(bytesPerSec / 1024).toFixed(1)} KB/s`;
  return `${(bytesPerSec / 1024 / 1024).toFixed(1)} MB/s`;
}

const MAX_NET_SPEED = 5 * 1024 * 1024; // 5 MB/s reference for bar width
function netDlPercent() {
  return Math.min((stats.value.download_speed / MAX_NET_SPEED) * 100, 100);
}
function netUlPercent() {
  return Math.min((stats.value.upload_speed / MAX_NET_SPEED) * 100, 100);
}
</script>

<template>
  <div class="home-panel">
    <div class="hero-row">
      <div class="time-block">
        <div class="time-text">{{ timeStr() }}</div>
        <div class="date-text">{{ dateStr() }}</div>
      </div>
      <div class="weather-block" v-if="weather">
        <span class="weather-icon">{{ weatherIcon(weather.weathercode) }}</span>
        <div class="weather-info">
          <span class="weather-temp">{{ weather.temperature }}°C</span>
          <span class="weather-city">{{ city }}</span>
        </div>
      </div>
    </div>

    <!-- forecast row -->
    <div class="forecast-row" v-if="forecast.length">
      <div v-for="day in forecast" :key="day.date" class="forecast-card">
        <span class="forecast-day">{{ day.dayName }}</span>
        <span class="forecast-icon">{{ weatherIcon(day.code) }}</span>
        <span class="forecast-temps">{{ day.max }}° / {{ day.min }}°</span>
      </div>
    </div>

    <div class="cards-row">
      <div class="stat-card">
        <div class="stat-header">
          <span class="stat-label">CPU</span>
          <span class="stat-value">{{ stats.cpu }}%</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill cpu-fill" :style="{ width: stats.cpu + '%' }" />
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-header">
          <span class="stat-label">内存</span>
          <span class="stat-value">{{ memUsedGB() }} / {{ memTotalGB() }} GB</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill mem-fill" :style="{ width: memPercent() + '%' }" />
        </div>
      </div>

      <div class="stat-card">
        <div class="stat-header">
          <span class="stat-label">Luna Toys</span>
          <span class="stat-value">{{ stats.self_memory_mb }} MB</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill self-fill" :style="{ width: Math.min(stats.self_memory_mb, 100) + '%' }" />
        </div>
      </div>

      <div class="stat-card net-card">
        <div class="stat-header net-header">
          <span class="stat-label">网络</span>
          <div class="net-speeds">
            <span class="net-label">↓</span>
            <span class="stat-value net-val">{{ formatSpeed(stats.download_speed) }}</span>
            <span class="net-label">↑</span>
            <span class="stat-value net-val">{{ formatSpeed(stats.upload_speed) }}</span>
          </div>
        </div>
        <div class="net-indicator-row">
          <div class="net-bar dl-bar" :style="{ width: netDlPercent() + '%' }" />
          <div class="net-bar ul-bar" :style="{ width: netUlPercent() + '%' }" />
        </div>
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
  justify-content: space-between;
  overflow: hidden;
}

/* ── hero row (time + weather) ── */
.hero-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 20px;
  padding: 16px 0;
}

.time-block {
  display: flex;
  flex-direction: column;
  padding: 16px 24px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
}

.time-text {
  font-size: 48px;
  font-weight: 700;
  line-height: 1;
  letter-spacing: -1px;
  color: rgba(255, 255, 255, 0.9);
}

.date-text {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.4);
  margin-top: 6px;
}

.weather-block {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 20px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.08);
  border-radius: 16px;
}

.weather-icon {
  font-size: 32px;
  line-height: 1;
}

.weather-info {
  display: flex;
  flex-direction: column;
}

.weather-temp {
  font-size: 22px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.85);
}

.weather-city {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.35);
  margin-top: 2px;
}

/* ── cards ── */
.cards-row {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stat-card {
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
  padding: 14px 18px;
}

.stat-header {
  display: flex;
  justify-content: space-between;
  align-items: baseline;
  margin-bottom: 10px;
}

.stat-label {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.5);
}

.stat-value {
  font-size: 14px;
  font-weight: 600;
  color: rgba(255, 255, 255, 0.8);
  font-variant-numeric: tabular-nums;
}

.progress-track {
  height: 6px;
  border-radius: 3px;
  background: rgba(255, 255, 255, 0.06);
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  border-radius: 3px;
  transition: width 0.6s ease;
}

.cpu-fill { background: #60a5fa; }
.mem-fill { background: #34d399; }
.self-fill { background: #a78bfa; }

/* ── forecast row ── */
.forecast-row {
  display: flex;
  gap: 10px;
  margin-bottom: 16px;
}

.forecast-card {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  padding: 10px 8px;
  background: rgba(255, 255, 255, 0.04);
  border: 1px solid rgba(255, 255, 255, 0.06);
  border-radius: 12px;
}

.forecast-day {
  font-size: 12px;
  color: rgba(255, 255, 255, 0.4);
}

.forecast-icon {
  font-size: 22px;
  line-height: 1;
}

.forecast-temps {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.6);
  white-space: nowrap;
}

/* ── network card ── */
.net-card .net-header {
  flex-wrap: wrap;
}

.net-speeds {
  display: flex;
  align-items: center;
  gap: 3px;
}

.net-label {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.3);
}

.net-val {
  min-width: 55px;
}

.net-indicator-row {
  display: flex;
  gap: 6px;
  margin-top: 4px;
}

.net-bar {
  height: 3px;
  border-radius: 2px;
  flex: 1;
  min-width: 0;
  transition: width 1.2s ease;
}

.dl-bar { background: #38bdf8; }
.ul-bar { background: #fb923c; }
</style>
