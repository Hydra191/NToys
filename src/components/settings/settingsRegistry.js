/**
 * Declarative settings registry.
 *
 * Each section has a `name` and an array of `items`.
 * Each item: { type, key, label, default?, min?, max?, step? }
 *
 * To add a new setting, just push an item to the right section.
 * SettingView renders everything automatically.
 */


/*
  {
    name: "Module",
    section: "module",
    items: [
      { type: "toggle", key: "test1", label: "toggletest1", default: false },
      { type: "toggle", key: "test2", label: "toggletest2", default: true },
      { type: "slider", key: "slider1", label: "slider1", default: 10, min: 1, max: 200, step: 0.01 },
      { type: "slider", key: "slider2", label: "slider2", default: 20, min: 1, max: 100, step: 1 },
      { type: "shortcut", key: "shortcuttest", label: "显示主界面", default: "Alt+Shift+N" },
    ],
  },
*/
export const settingsSections = [
  {
    name: "启动器",
    section: "runner",
    items: [
      { type: "shortcut", key: "shortcut", label: "快捷键", default: "Alt+Space" },
      { type: "slider", key: "max_visible", label: "显示数量", default: 8, min: 1, max: 20, step: 0.01 },
      { type: "toggle", key: "prevent_hide_on_text", label: "输入框有内容时保持窗口", default: true },
    ],
  },
  {
    name: "系统",
    section: "global",
    items: [
      { type: "toggle", key: "autostart", label: "开机自启动", default: false },
      { type: "shortcut", key: "show_window_shortcut", label: "显示主界面", default: "Alt+Shift+N" },
    ],
  },

];

/** Map key → section name for save dispatch */
export function keyToSection(key) {
  for (const sec of settingsSections) {
    if (sec.items.some((it) => it.key === key)) return sec.section;
  }
  return null;
}

/**
 * Build a reactive models object from registry defaults.
 * Returns: { runnerShortcut: ref("Alt+Space"), maxVisible: ref(8), ... }
 */
import { reactive } from "vue";

function buildDefaults() {
  const models = {};
  for (const sec of settingsSections) {
    for (const item of sec.items) {
      models[item.key] = item.default;
    }
  }
  return models;
}

/** Shared reactive state — import anywhere to read/write setting values. */
export const settingsState = reactive(buildDefaults());
