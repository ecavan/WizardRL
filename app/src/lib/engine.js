/**
 * The Wizard engine (Rust, compiled to WebAssembly) and the trained bots.
 * `engineReady()` downloads the networks once (7 MB, then cached for offline use);
 * `new Game(...)` is a full game with any mix of human and bot seats.
 */
import { useEffect, useState } from 'react';
import init, { load_brain, load_winprob, WizardGame } from '../wasm/wizard_wasm.js';
import wasmUrl from '../wasm/wizard_wasm_bg.wasm?url';

let ready = null;
const listeners = new Set();
let status = { loading: true, progress: 0, error: null };
const setStatus = (s) => { status = { ...status, ...s }; for (const f of listeners) f(status); };

async function fetchBytes(url, onProgress) {
  const res = await fetch(url);
  if (!res.ok) throw new Error(`${url}: ${res.status}`);
  const total = Number(res.headers.get('content-length')) || 0;
  if (!res.body || !total) return new Uint8Array(await res.arrayBuffer());
  const reader = res.body.getReader();
  const out = new Uint8Array(total);
  let got = 0;
  for (;;) {
    const { done, value } = await reader.read();
    if (done) break;
    if (got + value.length > out.length) return new Uint8Array(await (await fetch(url)).arrayBuffer());
    out.set(value, got);
    got += value.length;
    onProgress?.(got / total);
  }
  return got === total ? out : out.slice(0, got);
}

export function engineReady() {
  if (!ready) {
    ready = (async () => {
      await init({ module_or_path: wasmUrl });
      const [net, wp] = await Promise.all([
        fetchBytes('/models/ppo5.wznet', (p) => setStatus({ progress: p })),
        fetchBytes('/models/winprob.wzwp'),
      ]);
      load_brain(net);
      load_winprob(wp);
      setStatus({ loading: false, progress: 1 });
    })().catch((e) => { setStatus({ loading: false, error: String(e?.message || e) }); ready = null; throw e; });
  }
  return ready;
}

/** { loading, progress (0..1), error } while the engine and networks load. */
export function useEngine() {
  const [s, set] = useState(status);
  useEffect(() => {
    listeners.add(set);
    engineReady().catch(() => {});
    set(status);
    return () => listeners.delete(set);
  }, []);
  return s;
}

/** A full game. seats: [{ human: true } | { level, style }]. */
export class Game {
  constructor({ players, simultaneous = true, seats, seed }) {
    this.cfg = { players, simultaneous, seats, seed };
    this.g = new WizardGame(players, simultaneous, JSON.stringify(seats), seed);
  }
  state(viewer = -1) { return JSON.parse(this.g.state(viewer)); }
  toAct() { return this.g.to_act(); }
  isBotTurn() { return this.g.is_bot_turn(); }
  roundOver() { return this.g.round_over(); }
  gameOver() { return this.g.game_over(); }
  botStep() { return JSON.parse(this.g.bot_step()); }
  act(code) { return JSON.parse(this.g.act(code)); }
  advise(seat) { try { return JSON.parse(this.g.advise(seat)); } catch { return null; } }
  nextRound() { this.g.next_round(); }
  log() { return JSON.parse(this.g.log()); }
  replay(log) { this.g.replay(JSON.stringify(log)); }
  free() { this.g.free(); }
}
