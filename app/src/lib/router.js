/** Hash routing: #/section/sub?x=y */
import { useSyncExternalStore } from 'react';

function parse() {
  const h = location.hash.replace(/^#/, '') || '/';
  const [path, qs = ''] = h.split('?');
  const parts = path.split('/').filter(Boolean);
  return { path, parts, query: Object.fromEntries(new URLSearchParams(qs)), key: h };
}
let cur = parse();
const subs = new Set();
window.addEventListener('hashchange', () => { cur = parse(); for (const f of subs) f(); window.scrollTo(0, 0); });

export function useRoute() {
  return useSyncExternalStore((f) => { subs.add(f); return () => subs.delete(f); }, () => cur);
}
export function go(path) {
  const h = '#' + path;
  if (location.hash !== h) location.hash = h;
}
