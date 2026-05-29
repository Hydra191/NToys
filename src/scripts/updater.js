/**
 * Auto-update checker for Tauri 2.
 * Only checks — user downloads manually from GitHub Releases.
 */

export async function checkForUpdate() {
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (!update) return null;
    return { version: update.version, notes: update.body || "", date: update.date };
  } catch (e) {
    console.warn("Update check failed:", e);
    return null;
  }
}
