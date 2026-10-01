import { useEffect, useState } from 'react';
import { useRoute } from './lib/router.js';
import { usePrefs } from './lib/prefs.js';
import { engineReady } from './lib/engine.js';
import { Seg, Sheet, Toggle } from './ui/kit.jsx';
import Learn from './modes/Learn.jsx';
import Watch from './modes/Watch.jsx';
import Play from './modes/Play.jsx';

const TABS = [
  ['learn', 'Learn', LearnIcon],
  ['watch', 'Watch', WatchIcon],
  ['play', 'Play', PlayIcon],
];

export default function App() {
  const route = useRoute();
  const [prefs, setPrefs] = usePrefs();
  const [settings, setSettings] = useState(false);
  useEffect(() => { engineReady().catch(() => {}); }, []);
  useEffect(() => {
    const mq = window.matchMedia?.('(prefers-color-scheme: light)');
    const apply = () => {
      const t = prefs.theme === 'system' ? (mq?.matches ? 'light' : 'dark') : prefs.theme;
      document.documentElement.dataset.theme = t;
      document.querySelector('meta[name="theme-color"]')?.setAttribute('content', t === 'light' ? '#ffffff' : '#07090d');
    };
    apply();
    mq?.addEventListener?.('change', apply);
    return () => mq?.removeEventListener?.('change', apply);
  }, [prefs.theme]);
  const sec = TABS.some(([k]) => k === route.parts[0]) ? route.parts[0] : 'play';
  const page = sec === 'learn' ? <Learn /> : sec === 'watch' ? <Watch /> : <Play />;
  return (
    <div className="shell">
      <header className="topbar">
        <div className="flex items-center justify-between gap-3 px-4 sm:px-5 h-[52px] md:h-14 max-w-[1280px] mx-auto">
          <a href="#/play" className="flex items-center gap-2.5 shrink-0">
            <img src="/icon.svg" alt="" className="w-8 h-8 rounded-[9px] shadow-card" />
            <span className="font-display text-xl text-white tracking-tight">Wizard</span>
          </a>
          <nav className="tabbar-top" aria-label="Sections">
            {TABS.map(([k, label]) => <a key={k} href={`#/${k}`} className={sec === k ? 'on' : ''}>{label}</a>)}
          </nav>
          <button className="ibtn !w-10 !h-10" aria-label="Settings" onClick={() => setSettings(true)}><GearIcon /></button>
        </div>
      </header>
      <main className="flex-1" key={sec}>{page}</main>
      <nav className="tabbar-bottom" aria-label="Sections">
        {TABS.map(([k, label, Icon]) => <a key={k} href={`#/${k}`} className={sec === k ? 'on' : ''}><Icon /><span>{label}</span></a>)}
      </nav>
      <Sheet open={settings} onClose={() => setSettings(false)} title="Settings">
        <div className="space-y-4">
          <div className="flex items-center justify-between gap-3">
            <span className="text-sm font-semibold text-ink-100">Appearance</span>
            <Seg value={prefs.theme} onChange={(v) => setPrefs({ theme: v })} options={[['system', 'Auto'], ['light', 'Light'], ['dark', 'Dark']]} />
          </div>
          <Toggle on={prefs.fourColor} onChange={(v) => setPrefs({ fourColor: v })} label="Four-colour deck" sub="Clubs green and diamonds blue, so suits are easy to tell apart." />
          <Toggle on={prefs.coach} onChange={(v) => setPrefs({ coach: v })} label="Coach in Play" sub="Hints on request, and the cost of each of your moves." />
          <div className="flex items-center justify-between gap-3">
            <span className="text-sm font-semibold text-ink-100">Bot speed</span>
            <Seg value={prefs.pace} onChange={(v) => setPrefs({ pace: v })} options={[['slow', 'Slow'], ['normal', 'Normal'], ['fast', 'Fast']]} />
          </div>
          <p className="text-xs text-ink-400 leading-relaxed border-t border-ink-700 pt-3">The bots are networks trained by self-play (Wizard RL). The best, “The Wizard”, is ppo5: everything runs on this device, and works offline once loaded.</p>
        </div>
      </Sheet>
    </div>
  );
}

const I = (d) => function Icon() { return <svg width="22" height="22" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" strokeLinejoin="round">{d}</svg>; };
function LearnIcon() { return I(<><path d="M4 5h7a3 3 0 013 3v11a2 2 0 00-2-2H4z" /><path d="M20 5h-5a3 3 0 00-1 .2" /><path d="M20 5v12h-6" /></>)(); }
function WatchIcon() { return I(<><rect x="3" y="5" width="18" height="12" rx="2" /><path d="M10 9l4 2-4 2z" /><path d="M8 21h8" /></>)(); }
function PlayIcon() { return I(<><rect x="4" y="3" width="11" height="15" rx="2" /><path d="M9 21h8a2 2 0 002-2V7" /></>)(); }
function GearIcon() { return I(<><circle cx="12" cy="12" r="3" /><path d="M19.4 15a1.7 1.7 0 00.3 1.8l.1.1a2 2 0 11-2.8 2.8l-.1-.1a1.7 1.7 0 00-1.8-.3 1.7 1.7 0 00-1 1.5V21a2 2 0 11-4 0v-.1a1.7 1.7 0 00-1.1-1.5 1.7 1.7 0 00-1.8.3l-.1.1a2 2 0 11-2.8-2.8l.1-.1a1.7 1.7 0 00.3-1.8 1.7 1.7 0 00-1.5-1H3a2 2 0 110-4h.1a1.7 1.7 0 001.5-1.1 1.7 1.7 0 00-.3-1.8l-.1-.1a2 2 0 112.8-2.8l.1.1a1.7 1.7 0 001.8.3H9a1.7 1.7 0 001-1.5V3a2 2 0 114 0v.1a1.7 1.7 0 001 1.5 1.7 1.7 0 001.8-.3l.1-.1a2 2 0 112.8 2.8l-.1.1a1.7 1.7 0 00-.3 1.8V9a1.7 1.7 0 001.5 1H21a2 2 0 110 4h-.1a1.7 1.7 0 00-1.5 1z" /></>)(); }
