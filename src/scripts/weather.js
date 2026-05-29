import { reactive } from "vue";

const REFRESH_INTERVAL = 30 * 60 * 1000; // 30 minutes

export const weatherState = reactive({
  current: null,
  forecast: [],
  city: "",
  loading: false,
});

let timer = null;
let lastLoad = 0;

function weatherIcon(code) {
  if (code <= 1) return "☀";
  if (code <= 3) return "⛅";
  if (code <= 48) return "☁";
  if (code <= 57) return "🌧";
  if (code <= 67) return "🌧";
  if (code <= 77) return "❄";
  if (code <= 82) return "🌪";
  return "🌩";
}

async function loadWeather() {
  const now = Date.now();
  if (now - lastLoad < REFRESH_INTERVAL && weatherState.current) return;
  lastLoad = now;
  weatherState.loading = true;
  try {
    const ipRes = await fetch("http://ip-api.com/json/?lang=zh-CN&fields=city,lat,lon");
    const ipData = await ipRes.json();
    weatherState.city = ipData.city || "";

    const wRes = await fetch(
      `https://api.open-meteo.com/v1/forecast?latitude=${ipData.lat}&longitude=${ipData.lon}&current_weather=true&daily=temperature_2m_max,temperature_2m_min,weathercode&forecast_days=4&timezone=auto`
    );
    const wData = await wRes.json();
    weatherState.current = wData.current_weather;
    if (wData.daily) {
      const days = ["周日", "周一", "周二", "周三", "周四", "周五", "周六"];
      weatherState.forecast = wData.daily.time.map((t, i) => ({
        date: t,
        dayName: i === 0 ? "今天" : i === 1 ? "明天" : days[new Date(t).getDay()],
        max: Math.round(wData.daily.temperature_2m_max[i]),
        min: Math.round(wData.daily.temperature_2m_min[i]),
        code: wData.daily.weathercode[i],
      }));
    }
  } catch (e) { /* ignore */ }
  weatherState.loading = false;
}

/** Start periodic refresh. Call once at app init. */
export function startWeather() {
  loadWeather();
  timer = setInterval(loadWeather, REFRESH_INTERVAL);
}

/** Stop periodic refresh. Call on app teardown. */
export function stopWeather() {
  clearInterval(timer);
}

export { weatherIcon };
