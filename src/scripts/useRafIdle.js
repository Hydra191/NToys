import { ref, onMounted, onUnmounted } from "vue";

const IDLE_THRESHOLD = 300;

export function useRafIdle() {
  const isIdle = ref(false);

  let rafId = null;
  let idleWatchdog = null;

  function tick() {
    clearTimeout(idleWatchdog);
    if (isIdle.value) isIdle.value = false;
    idleWatchdog = setTimeout(() => { isIdle.value = true; }, IDLE_THRESHOLD);
    rafId = requestAnimationFrame(tick);
  }

  function start() {
    if (rafId) return;
    rafId = requestAnimationFrame(tick);
  }

  function stop() {
    clearTimeout(idleWatchdog);
    idleWatchdog = null;
    if (rafId) { cancelAnimationFrame(rafId); rafId = null; }
  }

  // Pause rAF when window hidden to save CPU/memory
  function onVisChange() {
    if (document.hidden) { stop(); isIdle.value = true; }
    else { start(); }
  }

  onMounted(() => {
    document.addEventListener("visibilitychange", onVisChange);
    start();
  });

  onUnmounted(() => {
    document.removeEventListener("visibilitychange", onVisChange);
    stop();
  });

  return { isIdle, start, stop };
}
