import { reactive } from "vue";

export const musicState = reactive({
  currentSong: null,
  playing: false,
  currentLyric: "",
});
