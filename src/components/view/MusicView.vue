<script setup>
import { ref, watch, computed, onMounted, onUnmounted, inject } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { musicState } from "../../scripts/music.js";
import { useMediaSession } from "../../scripts/useMediaSession.js";

// 部署到服务器后修改这里
const API_BASE = "http://119.23.60.130:3000";
const VOLUME_KEY = "ncm_volume";
const windowVisible = inject("windowVisible");

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
let lastLyricIdx = 0;
let debounceTimer = null;
const showVolume = ref(false);
const showQueue = ref(false);
let volumeTimer = null;
let queueTimer = null;

function onVolumeEnter() {
  clearTimeout(volumeTimer);
  showVolume.value = true;
}

function onVolumeLeave() {
  volumeTimer = setTimeout(() => {
    showVolume.value = false;
  }, 250);
}

function onQueueEnter() {
  clearTimeout(queueTimer);
  showQueue.value = true;
}

function onQueueLeave() {
  queueTimer = setTimeout(() => {
    showQueue.value = false;
  }, 250);
}

function jumpToQueueSong(song) {
  const idx = playQueue.value.findIndex(s => s.id === song.id);
  if (idx < 0) return;
  queueIndex.value = idx;
  savePlaybackNow();
  playQueueSong(song);
}

watch(currentSong, (s) => { musicState.currentSong = s; });
watch(playing, (p) => { musicState.playing = p; });
watch(volume, (v) => { localStorage.setItem(VOLUME_KEY, v); if (audioEl.value) audioEl.value.volume = v; });
onMounted(() => { if (audioEl.value) audioEl.value.volume = volume.value; });

// ── playback persistence ─────────────────
let savePlaybackTimer = null;
let lastSavedTime = 0;

function buildPlaybackState() {
  return JSON.stringify({
    song: currentSong.value,
    time: currentTime.value,
    tab: activeTab.value,
    playlist: currentPlaylist.value,
    queue: playQueue.value,
    queueIdx: queueIndex.value,
    shuffle: shuffle.value,
  });
}

/** Save progress at most every 15 seconds (avoids flooding disk) */
function savePlayback() {
  const t = currentTime.value;
  if (Math.abs(t - lastSavedTime) < 15) return;
  lastSavedTime = t;
  clearTimeout(savePlaybackTimer);
  savePlaybackTimer = setTimeout(() => {
    if (!currentSong.value) return;
    invoke("set_playback_state", { state: buildPlaybackState() });
  }, 500);
}

/** Immediate save on song change, close, tab switch */
function savePlaybackNow() {
  lastSavedTime = currentTime.value;
  clearTimeout(savePlaybackTimer);
  if (!currentSong.value) return;
  invoke("set_playback_state", { state: buildPlaybackState() });
}

async function restorePlayback() {
  try {
    const raw = await invoke("get_playback_state");
    if (!raw) return;
    const state = JSON.parse(raw);
    if (state.tab) activeTab.value = state.tab;
    if (state.playlist) currentPlaylist.value = state.playlist;
    if (state.queue?.length) playQueue.value = state.queue;
    if (state.queueIdx != null) queueIndex.value = state.queueIdx;
    if (state.shuffle != null) shuffle.value = state.shuffle;
    if (state.song) {
      currentSong.value = state.song;
      musicState.currentSong = state.song;
      currentTime.value = state.time || 0;
      fetchLyrics(state.song);
    }
  } catch (e) { /* ignore */ }
}

// ── login state ──────────────────────────
const cookie = ref("");
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
  invoke("set_ncm_cookie", { cookie: "" });
  userInfo.value = null;
  dailySongs.value = [];
  playlists.value = [];
  playlistSongs.value = [];
  currentPlaylist.value = null;
  showUserMenu.value = false;
}

async function logoutAndShowQR() {
  cookie.value = "";
  invoke("set_ncm_cookie", { cookie: "" });
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
    if (!windowVisible.value) return;
    try {
      const data = await api(`/login/qr/check?key=${qrKey.value}&timestamp=${Date.now()}`);
      const code = data.code;
      if (code === 800) { qrStatus.value = "二维码已过期，重新获取..."; clearInterval(qrTimer); startMenuQr(); }
      else if (code === 803) {
        qrStatus.value = "登录成功!";
        clearInterval(qrTimer);
        if (data.cookie) { cookie.value = data.cookie; invoke("set_ncm_cookie", { cookie: data.cookie }); }
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

// Pre-load saved song audio after cookie is ready
async function resumeFromSaved() {
  const song = currentSong.value;
  const t = currentTime.value;
  if (!song || !audioEl.value) return;
  try {
    const data = await api(`/song/url?id=${song.id}&level=standard`);
    const url = data.data?.[0]?.url;
    if (!url) return;
    audioEl.value.src = url;
    if (t > 0) {
      const seek = () => { audioEl.value.currentTime = t; };
      audioEl.value.addEventListener('loadedmetadata', seek, { once: true });
    }
  } catch (e) { /* ignore */ }
}

// Restore session and playback on mount
(async () => {
  await restorePlayback();
  const saved = await invoke("get_ncm_cookie");
  if (saved) cookie.value = saved;
  if (cookie.value) {
    try {
      const status = await api("/login/status");
      if (!status.data?.account) {
        cookie.value = "";
        invoke("set_ncm_cookie", { cookie: "" });
      } else {
        const ud = await api("/user/account");
        userInfo.value = ud.profile || ud.account || null;
        if (!userInfo.value) {
          cookie.value = "";
          invoke("set_ncm_cookie", { cookie: "" });
        }
        loadDaily();
        loadPlaylists();
      }
    } catch (e) { /* ignore */ }
  }
  // cookie is ready, restore playlist songs and audio
  if (activeTab.value === "playlist" && currentPlaylist.value) {
    openPlaylist(currentPlaylist.value);
  }
  resumeFromSaved();
})();

async function api(path, options = {}) {
  const url = new URL(`${API_BASE}${path}`);
  if (cookie.value) url.searchParams.set("cookie", cookie.value);
  const res = await fetch(url.toString(), { ...options, headers: options.headers });
  const data = await res.json();
  if (data.cookie) {
    cookie.value = data.cookie;
    invoke("set_ncm_cookie", { cookie: data.cookie });
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
  clearInterval(qrTimer);
  qrTimer = setInterval(async () => {
    if (!windowVisible.value) return;
    try {
      const data = await api(`/login/qr/check?key=${qrKey.value}&timestamp=${Date.now()}`);
      const code = data.code;
      if (code === 800) { qrStatus.value = "二维码已过期，重新获取..."; clearInterval(qrTimer); openLogin(); }
      else if (code === 803) {
        qrStatus.value = "登录成功!";
        clearInterval(qrTimer);
        if (data.cookie) { cookie.value = data.cookie; invoke("set_ncm_cookie", { cookie: data.cookie }); }
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

onUnmounted(() => {
  savePlaybackNow();
  clearTimeout(debounceTimer);
  clearTimeout(volumeTimer);
  clearTimeout(queueTimer);
  clearTimeout(savePlaybackTimer);
  clearInterval(qrTimer);
  if (audioEl.value) {
    audioEl.value.pause();
    audioEl.value.src = "";
    audioEl.value.load();
  }
});

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
function ncmThumb(url, size) {
  if (!url) return "";
  if (!url.includes("music.126.net")) return url;
  const sep = url.includes("?") ? "&" : "?";
  return `${url}${sep}param=${size}y${size}`;
}

function formatSong(s) {
  return {
    id: s.id,
    name: s.name,
    artists: (s.ar || s.artists || []).map((a) => a.name).join(" / "),
    album: (s.al || s.album)?.name || "",
    cover: ncmThumb((s.al || s.album)?.picUrl || "", 80),
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
    lastLyricIdx = 0;
  } catch (e) {
    lyrics.value = [];
    lastLyricIdx = 0;
  }
}

// ── play queue ──────────────────────────
const playQueue = ref([]);
const queueIndex = ref(-1);
const shuffle = ref(false);

/** Click a song in a list → build static queue starting from that song */
function playFromList(songs, startSong) {
  const idx = songs.findIndex(s => s.id === startSong.id);
  if (idx < 0) return;
  playQueue.value = [...songs.slice(idx), ...songs.slice(0, idx)];
  queueIndex.value = 0;
  playQueueSong(playQueue.value[0]);
}

/** Play the song at queueIndex */
async function playQueueSong(song) {
  if (!song) return;
  try {
    const data = await api(`/song/url?id=${song.id}&level=standard`);
    const url = data.data?.[0]?.url;
    if (!url) return;
    currentSong.value = song;
    musicState.currentSong = song;
    fetchLyrics(song);
    if (audioEl.value) { audioEl.value.src = url; audioEl.value.play(); playing.value = true; }
    savePlaybackNow(); // persist immediately
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
      resumeCurrent();
      return;
    }
    audioEl.value.play();
    playing.value = true;
    musicState.playing = true;
  }
}

/** Resume current song without queue manipulation */
async function resumeCurrent() {
  const song = currentSong.value;
  if (!song) return;
  try {
    const data = await api(`/song/url?id=${song.id}&level=standard`);
    const url = data.data?.[0]?.url;
    if (!url) return;
    if (audioEl.value) { audioEl.value.src = url; audioEl.value.play(); playing.value = true; }
  } catch (e) { /* ignore */ }
}

function onAudioPlay() { playing.value = true; musicState.playing = true; }
function onAudioPause() { playing.value = false; musicState.playing = false; }

function onTimeUpdate() {
  if (!seeking.value && audioEl.value) {
    currentTime.value = audioEl.value.currentTime;
    savePlayback();
  }
  if (!windowVisible.value) return;
  const t = audioEl.value?.currentTime || 0;
  let line = "";
  for (let i = lastLyricIdx; i < lyrics.value.length; i++) {
    if (t >= lyrics.value[i].time) {
      line = lyrics.value[i].text;
      lastLyricIdx = i;
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

function randomQueueIndex(exclude) {
  const len = playQueue.value.length;
  if (len < 2) return 0;
  let i;
  do { i = Math.floor(Math.random() * len); } while (i === exclude);
  return i;
}

function onAudioEnded() {
  playing.value = false;
  musicState.playing = false;
  lyrics.value = [];
  lastLyricIdx = 0;
  musicState.currentLyric = "";
  if (playQueue.value.length < 1) return;
  if (shuffle.value) {
    queueIndex.value = randomQueueIndex(queueIndex.value);
  } else if (queueIndex.value < playQueue.value.length - 1) {
    queueIndex.value++;
  }
  playQueueSong(playQueue.value[queueIndex.value]);
}

function formatTime(s) {
  if (!isFinite(s)) return "0:00";
  const m = Math.floor(s / 60);
  const sec = String(Math.floor(s % 60)).padStart(2, "0");
  return `${m}:${sec}`;
}

function playPrev() {
  const len = playQueue.value.length;
  if (len < 2) return;
  queueIndex.value = queueIndex.value > 0 ? queueIndex.value - 1 : len - 1;
  playQueueSong(playQueue.value[queueIndex.value]);
}

function playNext() {
  const len = playQueue.value.length;
  if (len < 2) return;
  if (shuffle.value) {
    queueIndex.value = randomQueueIndex(queueIndex.value);
  } else {
    queueIndex.value = queueIndex.value < len - 1 ? queueIndex.value + 1 : 0;
  }
  playQueueSong(playQueue.value[queueIndex.value]);
}

const canPrev = computed(() => playQueue.value.length > 1);
const canNext = computed(() => playQueue.value.length > 1);

// ── media keys / OS media controls ─────────
useMediaSession(currentSong, playing, {
  onPlay: () => { if (!playing.value) togglePlay(); },
  onPause: () => { if (playing.value) togglePlay(); },
  onPrev: () => playPrev(),
  onNext: () => playNext(),
});

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
      <div class="header-right">
        <button v-if="!userInfo" class="login-btn" @click="openLogin">登录</button>
        <div v-else class="user-badge" @click.stop="showUserMenu = true; showLoginInMenu = false">
          <img v-if="userInfo.avatarUrl" class="user-avatar" :src="ncmThumb(userInfo.avatarUrl, 100)"
            referrerpolicy="no-referrer" crossorigin="anonymous"
            @error="userInfo.avatarUrl = ''" />
          <span v-else class="user-avatar-placeholder">{{ (userInfo.nickname || 'U')[0] }}</span>
          <span class="user-nickname">{{ userInfo.nickname }}</span>
        </div>
      </div>
          <div class="nav-tabs">

      <button class="nav-tab" :class="{ active: activeTab === 'daily' }" @click="switchTab('daily')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"/></svg>
        日推歌曲
      </button>

      <button class="nav-tab" :class="{ active: activeTab === 'playlist' }" @click="switchTab('playlist')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M9 18V5l12-2v13"/><circle cx="6" cy="18" r="3"/><circle cx="18" cy="16" r="3"/></svg>
        个人歌单
      </button>

      <button class="nav-tab" :class="{ active: activeTab === 'search' }" @click="switchTab('search')">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="M21 21l-4.35-4.35"/></svg>
        搜索歌曲
      </button>
    </div>
    </div>

    <!-- Nav tabs -->


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
        <img v-if="pl.coverImgUrl" class="playlist-cover" :src="ncmThumb(pl.coverImgUrl, 200)" referrerpolicy="no-referrer" />
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
        class="music-item" :class="{ active: currentSong?.id === song.id }" @click="playFromList(displaySongs, song)">
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
      <img v-if="currentSong?.cover" class="player-cover" :class="{ spinning: playing }" :src="currentSong.cover" referrerpolicy="no-referrer" />
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
            <span class="volume-num">{{ Math.round(volume * 100) }}</span>
            <input ref="volumeSliderEl" type="range" class="volume-slider" orient="vertical"
              :value="volume" :style="{ '--vol-fill': (volume * 100) + '%' }"
              min="0" max="1" step="0.01" @input="onVolumeInput" />
          </div>
          <svg class="volume-icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M3 9v6h4l5 5V4L7 9H3zm13.5 3c0-1.77-1.02-3.29-2.5-4.03v8.05c1.48-.73 2.5-2.25 2.5-4.02zM14 3.23v2.06c2.89.86 5 3.54 5 6.71s-2.11 5.85-5 6.71v2.06c4.01-.91 7-4.49 7-8.77s-2.99-7.86-7-8.77z"/>
          </svg>
        </div>
        <!-- queue -->
        <div class="queue-wrap" @mouseenter="onQueueEnter" @mouseleave="onQueueLeave">
          <div class="queue-popup" :class="{ visible: showQueue }"
            @mouseenter="onQueueEnter" @mouseleave="onQueueLeave">
            <div class="queue-header">
              <span>播放队列 · {{ playQueue.length }}首</span>
              <button class="queue-shuffle-btn" :class="{ on: shuffle }" @click.stop="shuffle = !shuffle" title="随机播放">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                  <polyline points="16 3 21 3 21 8" /><line x1="4" y1="20" x2="21" y2="3" /><polyline points="21 16 21 21 16 21" /><line x1="15" y1="15" x2="21" y2="21" /><line x1="4" y1="4" x2="9" y2="9" />
                </svg>
              </button>
            </div>
            <ul class="queue-list">
              <li
                v-for="s in playQueue"
                :key="s.id"
                class="queue-item"
                :class="{ active: s.id === currentSong?.id }"
                @click="jumpToQueueSong(s)"
              >
                <img v-if="s.cover" class="queue-cover" :src="s.cover" loading="lazy" referrerpolicy="no-referrer" />
                <span v-else class="queue-cover-placeholder">{{ s.name[0] }}</span>
                <div class="queue-info">
                  <span class="queue-name">{{ s.name }}</span>
                  <span class="queue-artists">{{ s.artists }}</span>
                </div>
                <span class="queue-playing" v-if="s.id === currentSong?.id">♪</span>
              </li>
            </ul>
          </div>
          <svg class="queue-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
            <line x1="8" y1="6" x2="21" y2="6" /><line x1="8" y1="12" x2="21" y2="12" /><line x1="8" y1="18" x2="21" y2="18" /><line x1="3" y1="6" x2="3.01" y2="6" /><line x1="3" y1="12" x2="3.01" y2="12" /><line x1="3" y1="18" x2="3.01" y2="18" />
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
          <img v-if="userInfo?.avatarUrl" class="menu-avatar-large" :src="ncmThumb(userInfo.avatarUrl, 100)"
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
@import "../../css/music.css";
</style>
