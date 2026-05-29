/**
 * Check for updates by fetching update.json directly.
 * No Tauri updater plugin needed — just version comparison.
 */

const ENDPOINT = "https://raw.githubusercontent.com/Hydra191/NToys/main/update.json";

export async function checkForUpdate() {
  try {
    const res = await fetch(ENDPOINT);
    if (!res.ok) return null;
    const data = await res.json();
    if (!data?.version) return null;
    return { version: data.version, notes: data.notes || "", date: data.pub_date || "" };
  } catch (e) {
    console.warn("Update check failed:", e);
    return null;
  }
}
