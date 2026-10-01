/** Per-device storage (localStorage). Every access is wrapped: private mode or a full disk never breaks the app. */
export function load(key, fallback) {
  try {
    const v = localStorage.getItem(key);
    if (v == null) return fallback;
    const x = JSON.parse(v);
    return fallback && typeof fallback === 'object' && !Array.isArray(fallback) ? { ...fallback, ...x } : x;
  } catch {
    return fallback;
  }
}
export function save(key, value) {
  try { localStorage.setItem(key, JSON.stringify(value)); return true; } catch { return false; }
}
export function drop(key) {
  try { localStorage.removeItem(key); } catch { /* ignore */ }
}
