<script setup>
import { ref, onMounted, onUnmounted, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";

const windowVisible = inject("windowVisible");
const stats = ref({ cpu: 0, memory_total: 0, memory_used: 0, self_memory_mb: 0 });
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

// ── weather ─────────────────────────────
const weather = ref(null);
const city = ref("");

async function loadWeather() {
  try {
    const ipRes = await fetch("http://ip-api.com/json/?lang=zh-CN&fields=city,lat,lon");
    const ipData = await ipRes.json();
    city.value = ipData.city || "";

    const wRes = await fetch(
      `https://api.open-meteo.com/v1/forecast?latitude=${ipData.lat}&longitude=${ipData.lon}&current_weather=true&timezone=auto`
    );
    const wData = await wRes.json();
    weather.value = wData.current_weather;
  } catch (e) { /* ignore */ }
}

function weatherIcon(code) {
  if (code <= 1) return "☀";       // clear
  if (code <= 3) return "⛅";       // partly cloudy
  if (code <= 48) return "☁";      // cloudy/fog
  if (code <= 57) return "🌧"; // drizzle
  if (code <= 67) return "🌧"; // rain
  if (code <= 77) return "❄";      // snow
  if (code <= 82) return "🌪"; // rain showers
  return "🌩";                 // thunderstorm
}

onMounted(() => {
  refresh();
  timer = setInterval(refresh, 2000);
  now.value = new Date();
  timeTimer = setInterval(() => { if (windowVisible.value) now.value = new Date(); }, 1000);
  loadWeather();
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
          <span class="stat-label">NToys</span>
          <span class="stat-value">{{ stats.self_memory_mb }} MB</span>
        </div>
        <div class="progress-track">
          <div class="progress-fill self-fill" :style="{ width: Math.min(stats.self_memory_mb, 100) + '%' }" />
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
</style>
