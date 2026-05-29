import { watch, onMounted } from "vue";

/**
 * Wires reactive music state to the browser's MediaSession API.
 * Enables: media keys (keyboard), OS media overlay (Windows 10/11),
 * and Bluetooth headset controls.
 */
export function useMediaSession(song, playing, { onPlay, onPause, onPrev, onNext }) {
  onMounted(() => {
    if (!("mediaSession" in navigator)) return;

    navigator.mediaSession.setActionHandler("play", () => onPlay?.());
    navigator.mediaSession.setActionHandler("pause", () => onPause?.());
    navigator.mediaSession.setActionHandler("previoustrack", () => onPrev?.());
    navigator.mediaSession.setActionHandler("nexttrack", () => onNext?.());
  });

  watch(song, (s) => {
    if (!("mediaSession" in navigator)) return;
    if (!s) {
      navigator.mediaSession.metadata = null;
      return;
    }
    navigator.mediaSession.metadata = new MediaMetadata({
      title: s.name || "",
      artist: s.artists || "",
      album: s.album || "",
      artwork: s.cover
        ? [{ src: s.cover, sizes: "80x80", type: "image/jpeg" }]
        : [],
    });
  });

  watch(playing, (p) => {
    if (!("mediaSession" in navigator)) return;
    navigator.mediaSession.playbackState = p ? "playing" : "paused";
  });
}
