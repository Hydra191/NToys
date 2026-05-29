import { ref, onMounted, onUnmounted } from "vue";

const IDLE_THRESHOLD = 300; // ms — rAF gap beyond this means the browser throttled

export function useRafIdle() {
  const isIdle = ref(false);

  let rafId = null;
  let idleWatchdog = null;
  let running = false;

  function tick() {
    // Clear the watchdog — rAF fired, so we're active
    clearTimeout(idleWatchdog);

    if (isIdle.value) {
      isIdle.value = false;
    }

    // Arm watchdog: if rAF doesn't fire again within THRESHOLD, we're idle
    idleWatchdog = setTimeout(() => {
      isIdle.value = true;
    }, IDLE_THRESHOLD);

    rafId = requestAnimationFrame(tick);
  }

  function start() {
    if (running) return;
    running = true;
    rafId = requestAnimationFrame(tick);
  }

  function stop() {
    running = false;
    clearTimeout(idleWatchdog);
    if (rafId) cancelAnimationFrame(rafId);
    rafId = null;
  }

  onMounted(start);
  onUnmounted(stop);

  return { isIdle, start, stop };
}
