/**
 * Auto-update for Tauri 2.
 *
 * Flow:
 *   checkForUpdate()  →  returns { version, notes } or null
 *   downloadUpdate()  →  downloads in background, calls onProgress({ downloaded, total })
 *   installUpdate()   →  runs the MSI installer, app restarts
 */

let _update = null;

/** Check for updates. Returns { version, notes } if newer version found, else null. */
export async function checkForUpdate() {
  try {
    const { check } = await import("@tauri-apps/plugin-updater");
    const update = await check();
    if (!update) return null;
    _update = update;
    return { version: update.version, notes: update.body || "", date: update.date };
  } catch (e) {
    console.warn("Update check failed:", e);
    return null;
  }
}

/**
 * Download the previously-checked update.
 * Calls onProgress({ downloaded, total }) for progress percentage.
 */
export async function downloadUpdate(onProgress) {
  if (!_update) throw new Error("No pending update. Call checkForUpdate() first.");
  await _update.download((e) => {
    if (e?.event === "Progress" && onProgress) {
      onProgress({ downloaded: e.data.downloaded || 0, total: e.data.total || 0 });
    }
  });
}

/** Install the downloaded update. The app will restart. */
export async function installUpdate() {
  if (!_update) throw new Error("No pending update. Call checkForUpdate() first.");
  await _update.install();
}

/** Convenience: check → download (with progress) → install */
export async function checkAndInstall(onProgress) {
  const info = await checkForUpdate();
  if (!info) return null;
  await downloadUpdate(onProgress);
  await installUpdate();
  return info;
}
