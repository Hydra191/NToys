<script setup>
import { ref, watch, computed, onMounted, onUnmounted } from "vue";
import { musicState } from "../../stores/music.js";

// 部署到服务器后修改这里
const API_BASE = "http://119.23.60.130:3000";
const COOKIE_KEY = "ncm_cookie";
const VOLUME_KEY = "ncm_volume";
const PLAYBACK_KEY = "ncm_playback";

const activeTab = ref("search");
const query = ref("");
const results = ref([]);
const loading = ref(false);
const currentSong = ref(null);
const playing = ref(false);
const volume = ref(Number(localStorage.getItem(VOLUME_KEY)) || 0.7);
const audioEl = ref(null);
const currentTime = ref(0);
const duration = ref(0);
const seeking = ref(false);
const progressSliderEl = ref(null);
const lyrics = ref([]);
let debounceTimer = null;
const showVolume = ref(false);
let volumeTimer = null;

function onVolumeEnter() {
  clearTimeout(volumeTimer);
  showVolume.value = true;
}

function onVolumeLeave() {
  volumeTimer = setTimeout(() => {
    showVolume.value = false;
  }, 250);
}

watch(currentSong, (s) => { musicState.currentSong = s; });
watch(playing, (p) => { musicState.playing = p; });
watch(volume, (v) => { localStorage.setItem(VOLUME_KEY, v); if (audioEl.value) audioEl.value.volume = v; });
onMounted(() => { if (audioEl.value) audioEl.value.volume = volume.value; });

// ── playback persistence ─────────────────
let savePlaybackTimer = null;
function savePlayback() {
  clearTimeout(savePlaybackTimer);
  savePlaybackTimer = setTimeout(() => {
    if (!currentSong.value) return;
    const state = {
      song: currentSong.value,
      time: currentTime.value,
      tab: activeTab.value,
      playlist: currentPlaylist.value,
    };
    localStorage.setItem(PLAYBACK_KEY, JSON.stringify(state));
  }, 1000);
}

function restorePlayback() {
  try {
    const raw = localStorage.getItem(PLAYBACK_KEY);
    if (!raw) return;
    const state = JSON.parse(raw);
    if (state.tab) activeTab.value = state.tab;
    if (state.playlist) currentPlaylist.value = state.playlist;
    if (state.song) {
      currentSong.value = state.song;
      musicState.currentSong = state.song;
      currentTime.value = state.time || 0;
      fetchLyrics(state.song);
    }
  } catch (e) { /* ignore */ }
}

// ── login state ──────────────────────────
const cookie = ref(localStorage.getItem(COOKIE_KEY) || "");
const userInfo = ref(null);
const loginVisible = ref(false);
const showUserMenu = ref(false);
const showLoginInMenu = ref(false);
const qrKey = ref("");
const qrImg = ref("");
const qrStatus = ref("");
let qrTimer = null;

function logout() {
  cookie.value = "";
  localStorage.removeItem(COOKIE_KEY);
  userInfo.value = null;
  dailySongs.value = [];
  playlists.value = [];
  playlistSongs.value = [];
  currentPlaylist.value = null;
  showUserMenu.value = false;
}

async function logoutAndShowQR() {
  cookie.value = "";
  localStorage.removeItem(COOKIE_KEY);
  userInfo.value = null;
  dailySongs.value = [];
  playlists.value = [];
  playlistSongs.value = [];
  currentPlaylist.value = null;
  showLoginInMenu.value = true;
  await startMenuQr();
}

async function startMenuQr() {
  qrStatus.value = "获取二维码...";
  try {
    const kd = await api("/login/qr/key?timestamp=" + Date.now());
    qrKey.value = kd.data?.unikey;
    if (!qrKey.value) { qrStatus.value = "获取key失败"; return; }
    const qd = await api(`/login/qr/create?key=${qrKey.value}&qrimg=true`);
    qrImg.value = qd.data?.qrimg;
    qrStatus.value = "请用网易云音乐APP扫码";
    pollMenuQr();
  } catch (e) {
    qrStatus.value = "网络错误";
  }
}

function pollMenuQr() {
  clearInterval(qrTimer);
  qrTimer = setInterval(async () => {
    try {
      const data = await api(`/login/qr/check?key=${qrKey.value}&timestamp=${Date.now()}`);
      const code = data.code;
      if (code === 800) { qrStatus.value = "二维码已过期，重新获取..."; clearInterval(qrTimer); startMenuQr(); }
      else if (code === 803) {
        qrStatus.value = "登录成功!";
        clearInterval(qrTimer);
        if (data.cookie) { cookie.value = data.cookie; localStorage.setItem(COOKIE_KEY, data.cookie); }
        setTimeout(async () => {
          try {
            const ud = await api("/user/account");
            userInfo.value = ud.profile || ud.account;
            if (!userInfo.value?.nickname) userInfo.value = { nickname: (ud.account?.id || "用户") };
          } catch (e) { /* ignore */ }
          showUserMenu.value = false;
          showLoginInMenu.value = false;
          qrImg.value = "";
          qrKey.value = "";
        }, 500);
      }
      else if (code === 802) { qrStatus.value = "扫描成功，请在手机上确认..."; }
      else if (code === 801) { qrStatus.value = "请用网易云音乐APP扫码"; }
    } catch (e) { /* ignore */ }
  }, 2000);
}

// ── daily / playlist state ───────────────
const dailySongs = ref([]);
const playlists = ref([]);
const playlistSongs = ref([]);
const currentPlaylist = ref(null);

const displaySongs = computed(() => {
  if (activeTab.value === "search") return results.value;
  if (activeTab.value === "daily") return dailySongs.value;
  if (activeTab.value === "playlist" && currentPlaylist.value) return playlistSongs.value;
  return [];
});

// Restore session and playback on mount
(async () => {
  restorePlayback();
  if (!cookie.value) return;
  try {
    const status = await api("/login/status");
    if (!status.data?.account) {
      cookie.value = "";
      localStorage.removeItem(COOKIE_KEY);
      return;
    }
    const ud = await api("/user/account");
    userInfo.value = ud.profile || ud.account || null;
    if (!userInfo.value) {
      cookie.value = "";
      localStorage.removeItem(COOKIE_KEY);
    }
  } catch (e) { /* ignore */ }
})();

async function api(path, options = {}) {
  const url = new URL(`${API_BASE}${path}`);
  if (cookie.value) url.searchParams.set("cookie", cookie.value);
  const res = await fetch(url.toString(), { ...options, headers: options.headers });
  const data = await res.json();
  if (data.cookie) {
    cookie.value = data.cookie;
    localStorage.setItem(COOKIE_KEY, data.cookie);
  }
  return data;
}

// ── QR login ─────────────────────────────
async function openLogin() {
  loginVisible.value = true;
  qrStatus.value = "获取二维码...";
  try {
    const kd = await api("/login/qr/key?timestamp=" + Date.now());
    qrKey.value = kd.data?.unikey;
    if (!qrKey.value) { qrStatus.value = "获取key失败"; return; }
    const qd = await api(`/login/qr/create?key=${qrKey.value}&qrimg=true`);
    qrImg.value = qd.data?.qrimg;
    qrStatus.value = "请用网易云音乐APP扫码";
    pollQr();
  } catch (e) {
    qrStatus.value = "网络错误";
    loginVisible.value = false;
  }
}

function pollQr() {
  qrTimer = setInterval(async () => {
    try {
      const data = await api(`/login/qr/check?key=${qrKey.value}&timestamp=${Date.now()}`);
      const code = data.code;
      if (code === 800) { qrStatus.value = "二维码已过期，重新获取..."; clearInterval(qrTimer); openLogin(); }
      else if (code === 803) {
        qrStatus.value = "登录成功!";
        clearInterval(qrTimer);
        if (data.cookie) { cookie.value = data.cookie; localStorage.setItem(COOKIE_KEY, data.cookie); }
        setTimeout(async () => {
          try {
            const ud = await api("/user/account");
            userInfo.value = ud.profile || ud.account;
            if (!userInfo.value?.nickname) userInfo.value = { nickname: (ud.account?.id || "用户") };
          } catch (e) { /* ignore */ }
          loginVisible.value = false;
          qrImg.value = "";
          qrKey.value = "";
        }, 500);
      }
      else if (code === 802) { qrStatus.value = "扫描成功，请在手机上确认..."; }
      else if (code === 801) { qrStatus.value = "请用网易云音乐APP扫码"; }
    } catch (e) { /* ignore */ }
  }, 2000);
}

onUnmounted(() => { clearInterval(qrTimer); });

// ── tab switching ─────────────────────────
function switchTab(tab) {
  activeTab.value = tab;
  if (tab === "daily") loadDaily();
  else if (tab === "playlist") loadPlaylists();
}

// ── search ────────────────────────────────
watch(query, (val) => {
  clearTimeout(debounceTimer);
  if (!val.trim()) { results.value = []; return; }
  debounceTimer = setTimeout(() => search(val), 400);
});

async function search(q) {
  loading.value = true;
  try {
    const data = await api(`/search?keywords=${encodeURIComponent(q)}&limit=30`);
    results.value = (data.result?.songs || []).map(formatSong);
  } catch (e) { results.value = []; }
  finally { loading.value = false; }
}

// ── daily recommendations ─────────────────
async function loadDaily() {
  if (dailySongs.value.length) return;
  if (!cookie.value) return;
  loading.value = true;
  try {
    const data = await api("/recommend/songs");
    dailySongs.value = (data.data?.dailySongs || []).map(formatSong);
  } catch (e) { dailySongs.value = []; }
  finally { loading.value = false; }
}

// ── playlists ─────────────────────────────
async function loadPlaylists() {
  if (playlists.value.length) return;
  if (!cookie.value) return;
  loading.value = true;
  try {
    const data = await api("/user/playlist?uid=" + (userInfo.value?.userId || ""));
    playlists.value = (data.playlist || []).filter(p => p.trackCount > 0);
  } catch (e) { playlists.value = []; }
  finally { loading.value = false; }
}

async function openPlaylist(pl) {
  currentPlaylist.value = pl;
  loading.value = true;
  try {
    const data = await api(`/playlist/track/all?id=${pl.id}&limit=200`);
    playlistSongs.value = (data.songs || []).map(formatSong);
  } catch (e) { playlistSongs.value = []; }
  finally { loading.value = false; }
}

function backToPlaylists() {
  currentPlaylist.value = null;
  playlistSongs.value = [];
}

// ── helpers ───────────────────────────────
function formatSong(s) {
  return {
    id: s.id,
    name: s.name,
    artists: (s.ar || s.artists || []).map((a) => a.name).join(" / "),
    album: (s.al || s.album)?.name || "",
    cover: (s.al || s.album)?.picUrl || "",
    duration: formatDuration(s.dt || s.duration),
  };
}

function formatDuration(ms) {
  const total = Math.floor(ms / 1000);
  const m = Math.floor(total / 60);
  const s = String(total % 60).padStart(2, "0");
  return `${m}:${s}`;
}

function parseLRC(lrcStr) {
  const lines = [];
  if (!lrcStr) return lines;
  const re = /\[(\d{2}):(\d{2})(?:[.:](\d{2,3}))?\]/g;
  const textRe = /\[(\d{2}):(\d{2})(?:[.:](\d{2,3}))?\]/;
  for (const raw of lrcStr.split("\n")) {
    const match = raw.match(textRe);
    if (!match) continue;
    const text = raw.replace(textRe, "").trim();
    if (!text) continue;
    const mins = parseInt(match[1]);
    const secs = parseInt(match[2]);
    let ms = 0;
    if (match[3]) {
      ms = parseInt(match[3].length === 2 ? match[3] + "0" : match[3]);
    }
    const time = mins * 60 + secs + ms / 1000;
    lines.push({ time, text });
  }
  return lines;
}

async function fetchLyrics(song) {
  try {
    const data = await api(`/lyric?id=${song.id}`);
    const lrc = data.lrc?.lyric || "";
    lyrics.value = parseLRC(lrc);
  } catch (e) {
    lyrics.value = [];
  }
}

async function playSong(song) {
  try {
    const data = await api(`/song/url?id=${song.id}&level=standard`);
    const url = data.data?.[0]?.url;
    if (!url) return;
    currentSong.value = song;
    musicState.currentSong = song;
    savePlayback();
    fetchLyrics(song);
    if (audioEl.value) { audioEl.value.src = url; audioEl.value.play(); playing.value = true; }
  } catch (e) { /* ignore */ }
}

function togglePlay() {
  if (!audioEl.value) return;
  if (playing.value) {
    audioEl.value.pause();
    playing.value = false;
    musicState.playing = false;
  } else if (currentSong.value) {
    if (!audioEl.value.src) {
      playSong(currentSong.value);
      return;
    }
    audioEl.value.play();
    playing.value = true;
    musicState.playing = true;
  }
}

function onAudioPlay() { playing.value = true; musicState.playing = true; }
function onAudioPause() { playing.value = false; musicState.playing = false; }

function onTimeUpdate() {
  if (!seeking.value && audioEl.value) {
    currentTime.value = audioEl.value.currentTime;
    savePlayback();
  }
  const t = audioEl.value?.currentTime || 0;
  let line = "";
  for (let i = 0; i < lyrics.value.length; i++) {
    if (t >= lyrics.value[i].time) {
      line = lyrics.value[i].text;
    } else {
      break;
    }
  }
  musicState.currentLyric = line;
}

function onLoadedMeta() {
  if (audioEl.value) duration.value = audioEl.value.duration || 0;
}

function onProgressInput(e) {
  const t = Number(e.target.value);
  currentTime.value = t;
  const pct = duration.value > 0 ? (t / duration.value) * 100 : 0;
  e.target.style.setProperty("--prog-fill", pct + "%");
}

function onProgressChange(e) {
  const t = Number(e.target.value);
  if (audioEl.value) audioEl.value.currentTime = t;
  currentTime.value = t;
  seeking.value = false;
}

function onProgressDown() {
  seeking.value = true;
}

function onAudioEnded() {
  playing.value = false;
  musicState.playing = false;
  lyrics.value = [];
  musicState.currentLyric = "";
  playNext();
}

function formatTime(s) {
  if (!isFinite(s)) return "0:00";
  const m = Math.floor(s / 60);
  const sec = String(Math.floor(s % 60)).padStart(2, "0");
  return `${m}:${sec}`;
}

function playPrev() {
  const list = displaySongs.value;
  if (!list.length) return;
  const idx = list.findIndex(s => s.id === currentSong.value?.id);
  if (idx < 0) { playSong(list[0]); return; }
  const prev = idx <= 0 ? list.length - 1 : idx - 1;
  playSong(list[prev]);
}

function playNext() {
  const list = displaySongs.value;
  if (!list.length) return;
  const idx = list.findIndex(s => s.id === currentSong.value?.id);
  if (idx < 0) { playSong(list[0]); return; }
  const next = idx >= list.length - 1 ? 0 : idx + 1;
  playSong(list[next]);
}

const canPrev = computed(() => displaySongs.value.length > 0);
const canNext = computed(() => displaySongs.value.length > 0);

function onVolumeInput(e) {
  const v = Number(e.target.value);
  volume.value = v;
  e.target.style.setProperty("--vol-fill", (v * 100) + "%");
  if (audioEl.value) audioEl.value.volume = v;
}
</script>

<template>
  <div class="music-panel">
    <!-- Header -->
    <div class="music-header">
      <h2>Music</h2>
      <div class="header-right">
        <button v-if="!userInfo" class="login-btn" @click="openLogin">登录</button>
        <div v-else class="user-badge" @click.stop="showUserMenu = true; showLoginInMenu = false">
          <img v-if="userInfo.avatarUrl" class="user-avatar" :src="userInfo.avatarUrl"
            referrerpolicy="no-referrer" crossorigin="anonymous"
            @error="userInfo.avatarUrl = ''" />
          <span v-else class="user-avatar-placeholder">{{ (userInfo.nickname || 'U')[0] }}</span>
          <span class="user-nickname">{{ userInfo.nickname }}</span>
        </div>
      </div>
    </div>

    <!-- Nav tabs -->
    <div class="nav-tabs">
      <button class="nav-tab" :class="{ active: activeTab === 'search' }" @click="switchTab('search')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg>
        搜索歌曲
      </button>
      <button class="nav-tab" :class="{ active: activeTab === 'daily' }" @click="switchTab('daily')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
        日推歌曲
      </button>
      <button class="nav-tab" :class="{ active: activeTab === 'playlist' }" @click="switchTab('playlist')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
        个人歌单
      </button>
    </div>

    <!-- Search bar (search tab only) -->
    <div v-if="activeTab === 'search'" class="search-bar">
      <svg class="search-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <circle cx="11" cy="11" r="8" /><path d="M21 21l-4.35-4.35" />
      </svg>
      <input v-model="query" type="text" class="music-search-input" placeholder="搜索歌曲..." />
    </div>

    <!-- Playlist breadcrumb -->
    <div v-if="activeTab === 'playlist' && currentPlaylist" class="playlist-breadcrumb">
      <button class="back-btn" @click="backToPlaylists">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M19 12H5M12 19l-7-7 7-7"/></svg>
      </button>
      <span class="playlist-title">{{ currentPlaylist.name }}</span>
    </div>

    <!-- Playlist grid -->
    <div v-if="activeTab === 'playlist' && !currentPlaylist" class="playlist-grid">
      <div v-if="loading" class="loading">加载中...</div>
      <div v-for="pl in playlists" :key="pl.id" class="playlist-card" @click="openPlaylist(pl)">
        <img v-if="pl.coverImgUrl" class="playlist-cover" :src="pl.coverImgUrl" referrerpolicy="no-referrer" />
        <div v-else class="playlist-cover-placeholder">{{ pl.name[0] }}</div>
        <span class="playlist-card-name">{{ pl.name }}</span>
        <span class="playlist-card-count">{{ pl.trackCount }}首</span>
      </div>
      <div v-if="!loading && !playlists.length" class="empty">未登录或无法获取歌单</div>
    </div>

    <!-- Song list -->
    <div v-if="loading && !displaySongs.length" class="loading">加载中...</div>
    <ul v-if="displaySongs.length" class="music-results">
      <li v-for="song in displaySongs" :key="song.id"
        class="music-item" :class="{ active: currentSong?.id === song.id }" @click="playSong(song)">
        <div class="music-info">
          <span class="music-name">{{ song.name }}</span>
          <span class="music-artists">{{ song.artists }} · {{ song.album }}</span>
        </div>
        <span class="music-duration">{{ song.duration }}</span>
      </li>
    </ul>
    <div v-if="!loading && !displaySongs.length && activeTab !== 'playlist'" class="empty">
      {{ activeTab === 'daily' ? '请先登录后再获取日推歌曲' : '暂无内容' }}
    </div>

    <!-- Player bar -->
    <div class="player-bar">
      <img v-if="currentSong?.cover" class="player-cover" :src="currentSong.cover" referrerpolicy="no-referrer" />
      <div v-else class="player-cover-placeholder">
        <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55C7.79 13 6 14.79 6 17s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z"/></svg>
      </div>
      <div class="player-center">
        <div class="player-info">
          <template v-if="currentSong">
            <span class="player-name">{{ currentSong.name }}</span>
            <span class="player-artists">{{ currentSong.artists }}</span>
          </template>
          <span v-else class="player-empty">无正在播放歌曲</span>
        </div>
        <div class="player-progress-row">
          <span class="player-time">{{ formatTime(currentTime) }}</span>
          <input
            ref="progressSliderEl"
            type="range"
            class="progress-slider"
            :value="currentTime"
            :max="duration || 0"
            :style="{ '--prog-fill': duration > 0 ? (currentTime / duration) * 100 + '%' : '0%' }"
            step="0.01"
            @mousedown="onProgressDown"
            @touchstart="onProgressDown"
            @input="onProgressInput"
            @change="onProgressChange"
          />
          <span class="player-time">{{ formatTime(duration) }}</span>
        </div>
      </div>
      <div class="player-actions">
        <button class="player-btn player-btn-sm" :disabled="!canPrev" @click="playPrev">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 6h2v12H6zm3.5 6l8.5 6V6z"/></svg>
        </button>
        <button class="player-btn" :disabled="!currentSong" @click="togglePlay">
          <svg v-if="!playing" viewBox="0 0 24 24" fill="currentColor"><path d="M8 5v14l11-7z"/></svg>
          <svg v-else viewBox="0 0 24 24" fill="currentColor"><path d="M6 19h4V5H6v14zm8-14v14h4V5h-4z"/></svg>
        </button>
        <button class="player-btn player-btn-sm" :disabled="!canNext" @click="playNext">
          <svg viewBox="0 0 24 24" fill="currentColor"><path d="M6 18l8.5-6L6 6v12zM16 6v12h2V6h-2z"/></svg>
        </button>
        <div class="volume-wrap" @mouseenter="onVolumeEnter" @mouseleave="onVolumeLeave">
          <div class="volume-popup" :class="{ visible: showVolume }"
            @mouseenter="onVolumeEnter" @mouseleave="onVolumeLeave">
            <input ref="volumeSliderEl" type="range" class="volume-slider" orient="vertical"
              :value="volume" :style="{ '--vol-fill': (volume * 100) + '%' }"
              min="0" max="1" step="0.01" @input="onVolumeInput" />
          </div>
          <svg class="volume-icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
          </svg>
        </div>
      </div>
      <audio ref="audioEl"
        @timeupdate="onTimeUpdate"
        @loadedmetadata="onLoadedMeta"
        @ended="onAudioEnded"
        @pause="onAudioPause"
        @play="onAudioPlay" />
    </div>

    <!-- QR Login Modal -->
    <div v-if="loginVisible" class="login-overlay" @click.self="loginVisible = false; clearInterval(qrTimer)">
      <div class="login-modal">
        <h3>扫码登录</h3>
        <img v-if="qrImg" class="qr-img" :src="qrImg" alt="QR code" />
        <div class="qr-status">{{ qrStatus }}</div>
        <button class="login-close-btn" @click="loginVisible = false; clearInterval(qrTimer)">取消</button>
      </div>
    </div>

    <!-- User Menu Popup -->
    <div v-if="showUserMenu" class="user-menu-overlay" @click.self="showUserMenu = false; showLoginInMenu = false">
      <div class="user-menu-popup">
        <button class="menu-close-btn" @click="showUserMenu = false; showLoginInMenu = false">&times;</button>
        <template v-if="!showLoginInMenu">
          <img v-if="userInfo?.avatarUrl" class="menu-avatar-large" :src="userInfo.avatarUrl"
            referrerpolicy="no-referrer" crossorigin="anonymous"
            @error="userInfo.avatarUrl = ''" />
          <span v-else class="menu-avatar-placeholder">{{ (userInfo?.nickname || 'U')[0] }}</span>
          <div class="menu-nickname">{{ userInfo?.nickname }}</div>
          <button class="menu-btn menu-btn-logout" @click="logoutAndShowQR">登出</button>
        </template>
        <template v-else>
          <h3 class="menu-qr-title">扫码登录</h3>
          <img v-if="qrImg" class="qr-img" :src="qrImg" alt="QR code" />
          <div class="qr-status">{{ qrStatus }}</div>
          <button class="menu-btn menu-btn-back" @click="showLoginInMenu = false; clearInterval(qrTimer)">返回</button>
        </template>
      </div>
    </div>
  </div>
</template>

<style>
@property --vol-fill {
  syntax: "<percentage>";
  inherits: true;
  initial-value: 70%;
}

@property --prog-fill {
  syntax: "<percentage>";
  inherits: true;
  initial-value: 0%;
}

.music-panel {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

/* header */
.music-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
}
.music-header h2 { font-size: 20px; font-weight: 600; }
.header-right { flex-shrink: 0; }

.login-btn {
  padding: 6px 16px;
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.7);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s;
}
.login-btn:hover { background: rgba(255, 255, 255, 0.14); }

.user-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 12px 4px 4px;
  border-radius: 8px;
  background: rgba(255, 255, 255, 0.06);
  cursor: pointer;
  transition: background 0.15s;
}
.user-badge:hover { background: rgba(255, 255, 255, 0.1); }
.user-avatar { width: 26px; height: 26px; border-radius: 50%; object-fit: cover; }
.user-avatar-placeholder {
  width: 26px; height: 26px; border-radius: 50%;
  background: rgba(255, 255, 255, 0.2); color: #fff;
  font-size: 13px; font-weight: 600;
  display: flex; align-items: center; justify-content: center; flex-shrink: 0;
}
.user-nickname { font-size: 13px; color: rgba(255, 255, 255, 0.8); }

/* nav tabs */
.nav-tabs {
  display: flex;
  gap: 4px;
  margin-bottom: 14px;
  background: rgba(255, 255, 255, 0.04);
  border-radius: 8px;
  padding: 3px;
}
.nav-tab {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  padding: 8px 0;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: rgba(255, 255, 255, 0.5);
  font-size: 13px;
  cursor: pointer;
  transition: background 0.15s, color 0.15s;
}
.nav-tab svg { width: 14px; height: 14px; flex-shrink: 0; }
.nav-tab:hover { color: rgba(255, 255, 255, 0.75); }
.nav-tab.active { background: rgb(140, 94, 212); color: #fff; }

/* search bar */
.search-bar {
  display: flex;
  align-items: center;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  padding: 10px 14px;
  margin-bottom: 12px;
}
.search-icon { width: 16px; height: 16px; color: rgba(255, 255, 255, 0.35); flex-shrink: 0; margin-right: 10px; }
.music-search-input {
  flex: 1; border: none; background: transparent; color: #fff; font-size: 14px; outline: none;
}
.music-search-input::placeholder { color: rgba(255, 255, 255, 0.3); }

/* playlist breadcrumb */
.playlist-breadcrumb {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 12px;
}
.back-btn {
  width: 28px; height: 28px;
  display: flex; align-items: center; justify-content: center;
  border: none; border-radius: 6px;
  background: rgba(255, 255, 255, 0.08);
  color: rgba(255, 255, 255, 0.6);
  cursor: pointer;
}
.back-btn svg { width: 16px; height: 16px; }
.back-btn:hover { background: rgba(255, 255, 255, 0.14); color: #fff; }
.playlist-title { font-size: 15px; font-weight: 600; color: rgba(255, 255, 255, 0.85); }

/* playlist grid */
.playlist-grid {
  flex: 1;
  min-height: 0;
  overflow-y: auto;
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(140px, 1fr));
  gap: 12px;
  align-content: start;
}
.playlist-card {
  display: flex;
  flex-direction: column;
  cursor: pointer;
  border-radius: 8px;
  padding: 8px;
  transition: background 0.12s;
}
.playlist-card:hover { background: rgba(255, 255, 255, 0.06); }
.playlist-cover {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 6px;
  object-fit: cover;
  margin-bottom: 8px;
}
.playlist-cover-placeholder {
  width: 100%;
  aspect-ratio: 1;
  border-radius: 6px;
  background: rgb(140, 94, 212);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 28px;
  color: rgba(255, 255, 255, 0.5);
  margin-bottom: 8px;
}
.playlist-card-name {
  font-size: 13px;
  color: rgba(255, 255, 255, 0.8);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}
.playlist-card-count {
  font-size: 11px;
  color: rgba(255, 255, 255, 0.35);
  margin-top: 2px;
}

/* song list */
.loading {
  font-size: 13px; color: rgba(255, 255, 255, 0.4);
  text-align: center; padding: 24px 0;
}
.music-results {
  list-style: none;
  flex: 1; min-height: 0; overflow-y: auto;
}
.music-item {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 14px; border-radius: 8px;
  cursor: pointer; transition: background 0.12s; gap: 12px;
}
.music-item:hover { background: rgba(255, 255, 255, 0.08); }
.music-item.active { background: rgb(140, 94, 212); }
.music-info { min-width: 0; flex: 1; }
.music-name {
  display: block; font-size: 14px; color: rgba(255, 255, 255, 0.85);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.music-artists {
  display: block; font-size: 12px; color: rgba(255, 255, 255, 0.4);
  margin-top: 2px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.music-duration { font-size: 13px; color: rgba(255, 255, 255, 0.35); flex-shrink: 0; }
.empty { font-size: 13px; color: rgba(255, 255, 255, 0.35); text-align: center; padding: 24px 0; }

/* player bar */
.player-bar {
  display: flex; align-items: center; gap: 12px;
  padding: 12px 14px;
  margin-top: auto;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 8px;
  box-shadow: 0 0 10px rgba(0, 0, 0, 0.2);
  flex-shrink: 0;
}
.player-cover {
  width: 40px; height: 40px;
  border-radius: 6px;
  object-fit: cover;
  flex-shrink: 0;
}
.player-cover-placeholder {
  width: 40px; height: 40px;
  border-radius: 6px;
  background: rgba(255, 255, 255, 0.06);
  display: flex; align-items: center; justify-content: center;
  color: rgba(255, 255, 255, 0.2);
  flex-shrink: 0;
}
.player-cover-placeholder svg { width: 20px; height: 20px; }
.player-center {
  flex: 1; min-width: 0;
  display: flex; flex-direction: column;
  justify-content: center; gap: 6px;
}
.player-info { min-width: 0; }
.player-name {
  display: block; font-size: 13px; font-weight: 600;
  color: rgba(255, 255, 255, 0.85);
  white-space: nowrap; overflow: hidden; text-overflow: ellipsis;
}
.player-artists {
  display: block; font-size: 11px; color: rgba(255, 255, 255, 0.4); margin-top: 1px;
}

.player-progress-row {
  display: flex; align-items: center; gap: 8px;
}
.player-time {
  font-size: 11px; color: rgba(255, 255, 255, 0.35);
  flex-shrink: 0; min-width: 32px;
  font-variant-numeric: tabular-nums;
}
.player-time:first-child { text-align: right; }
.player-time:last-child { text-align: left; }

.progress-slider {
  -webkit-appearance: none; appearance: none;
  flex: 1; height: 4px; border-radius: 2px;
  background: rgba(255, 255, 255, 0.12);
  outline: none; cursor: pointer;
}
.progress-slider::-webkit-slider-runnable-track {
  height: 4px; border-radius: 2px;
  background: linear-gradient(
    to right,
    rgb(140, 94, 212) 0%,
    rgb(140, 94, 212) var(--prog-fill, 0%),
    rgba(255, 255, 255, 0.12) var(--prog-fill, 0%),
    rgba(255, 255, 255, 0.12) 100%
  );
}
.progress-slider::-webkit-slider-thumb {
  -webkit-appearance: none; appearance: none;
  width: 10px; height: 10px; border-radius: 50%;
  background: #fff; margin-top: -3px;
  opacity: 0; transition: opacity 0.15s;
}
.progress-slider:hover::-webkit-slider-thumb { opacity: 1; }

.player-actions {
  display: flex; align-items: center; gap: 8px; flex-shrink: 0;
}
.player-btn {
  width: 32px; height: 32px;
  display: flex; align-items: center; justify-content: center;
  border: none; border-radius: 50%;
  background: rgb(140, 94, 212); color: #fff;
  cursor: pointer; flex-shrink: 0;
}
.player-btn svg { width: 16px; height: 16px; }
.player-btn:hover:not(:disabled) { opacity: 0.85; }
.player-btn:disabled { opacity: 0.3; cursor: default; }
.player-btn-sm {
  width: 28px; height: 28px;
  background: rgba(255, 255, 255, 0.08);
}
.player-btn-sm svg { width: 14px; height: 14px; }
.player-empty { font-size: 13px; color: rgba(255, 255, 255, 0.3); }

/* volume */
.volume-wrap {
  position: relative;
  display: flex; align-items: center; justify-content: center;
  padding: 0 12px;
  margin: 0 -12px;
}
.volume-icon {
  width: 16px; height: 16px;
  color: rgba(255, 255, 255, 0.4); flex-shrink: 0;
  cursor: pointer;
  position: relative;
  z-index: 1;
}
.volume-popup {
  position: absolute;
  bottom: 26px;
  left: 50%;
  transform: translateX(-50%);
  width: 36px;
  height: 96px;
  background: rgba(28, 28, 32, 0.95);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s;
  box-shadow: 0 4px 16px rgba(0, 0, 0, 0.4);
}
.volume-popup.visible {
  opacity: 1;
  pointer-events: auto;
}
.volume-slider {
  -webkit-appearance: slider-vertical;
  appearance: slider-vertical;
  accent-color: rgb(140, 94, 212);
  width: 4px;
  height: 76px;
  outline: none;
  cursor: pointer;
}

/* scrollbar */
.music-results::-webkit-scrollbar,
.playlist-grid::-webkit-scrollbar { width: 4px; }
.music-results::-webkit-scrollbar-thumb,
.playlist-grid::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.12);
  border-radius: 2px;
}

/* QR login modal */
.login-overlay {
  position: fixed; inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex; align-items: center; justify-content: center;
  z-index: 100;
}
.login-modal {
  background: rgb(36, 36, 36);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 24px; text-align: center;
  min-width: 260px;
}
.login-modal h3 { font-size: 16px; font-weight: 600; margin-bottom: 16px; }
.qr-img { width: 200px; height: 200px; display: block; margin: 0 auto 12px; }
.qr-status { font-size: 13px; color: rgba(255, 255, 255, 0.5); margin-bottom: 16px; }
.login-close-btn {
  padding: 6px 20px; border: none; border-radius: 6px;
  background: rgba(255, 255, 255, 0.12); color: rgba(255, 255, 255, 0.7);
  font-size: 13px; cursor: pointer;
}
.login-close-btn:hover { background: rgba(255, 255, 255, 0.2); }

/* user menu popup */
.user-menu-overlay {
  position: fixed; inset: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex; align-items: center; justify-content: center;
  z-index: 110;
}
.user-menu-popup {
  position: relative;
  background: rgb(36, 36, 36);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 12px;
  padding: 32px 28px 24px;
  text-align: center;
  min-width: 240px;
  max-width: 300px;
}
.menu-close-btn {
  position: absolute;
  top: 8px; right: 10px;
  width: 28px; height: 28px;
  border: none; border-radius: 6px;
  background: transparent;
  color: rgba(255, 255, 255, 0.4);
  font-size: 18px;
  cursor: pointer;
  display: flex; align-items: center; justify-content: center;
}
.menu-close-btn:hover { background: rgba(255, 255, 255, 0.08); color: #fff; }

.menu-avatar-large {
  width: 72px; height: 72px;
  border-radius: 50%;
  object-fit: cover;
  margin-bottom: 12px;
}
.menu-avatar-placeholder {
  width: 72px; height: 72px;
  border-radius: 50%;
  background: rgb(140, 94, 212);
  color: #fff;
  font-size: 28px; font-weight: 600;
  display: flex; align-items: center; justify-content: center;
  margin: 0 auto 12px;
}
.menu-nickname {
  font-size: 15px; font-weight: 600;
  color: rgba(255, 255, 255, 0.9);
  margin-bottom: 20px;
}
.menu-btn {
  padding: 8px 24px;
  border: none; border-radius: 6px;
  font-size: 13px; cursor: pointer;
  transition: background 0.15s;
}
.menu-btn-logout {
  background: rgba(255, 80, 80, 0.2);
  color: #ff6b6b;
}
.menu-btn-logout:hover { background: rgba(255, 80, 80, 0.35); }
.menu-btn-back {
  background: rgba(255, 255, 255, 0.1);
  color: rgba(255, 255, 255, 0.7);
  margin-top: 8px;
}
.menu-btn-back:hover { background: rgba(255, 255, 255, 0.18); }
.menu-qr-title {
  font-size: 15px; font-weight: 600;
  margin-bottom: 16px;
}
</style>
