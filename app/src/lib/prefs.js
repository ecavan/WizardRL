/** App preferences: a tiny global store with a React hook. */
import { useSyncExternalStore } from 'react';
import { load, save } from './store.js';

const KEY = 'wizard.prefs.v1';
export const DEFAULT_PREFS = {
  theme: 'system', // 'system' | 'dark' | 'light'
  fourColor: false, // clubs green, diamonds blue
  coach: true, // Play: the coach's suggestion and the cost of your moves
  pace: 'normal', // how fast bots play: 'fast' | 'normal' | 'slow'
};

let prefs = load(KEY, DEFAULT_PREFS);
const subs = new Set();
export const getPrefs = () => prefs;
export function setPrefs(patch) {
  prefs = { ...prefs, ...(typeof patch === 'function' ? patch(prefs) : patch) };
  save(KEY, prefs);
  for (const f of subs) f();
}
export function usePrefs() {
  const p = useSyncExternalStore((f) => { subs.add(f); return () => subs.delete(f); }, () => prefs);
  return [p, setPrefs];
}
export const PACE = { fast: 260, normal: 650, slow: 1200 };
