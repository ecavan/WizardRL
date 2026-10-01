/**
 * Watch: bots play a full game with every hand face up. The commentary says what each bot did,
 * what the best bot (ppo5) expects from it, and when it would have done something else.
 */
import { useCallback, useEffect, useRef, useState } from 'react';
import { Game, useEngine } from '../lib/engine.js';
import { usePrefs, PACE } from '../lib/prefs.js';
import { ALL, TABLES, byId, pickRandom, seatNames, seatSpec } from '../lib/players.js';
import { codeKind, codeBid, codeCard, SUIT_NAME } from '../lib/cards.js';
import { Seat, Trick, TrumpInfo, Scoreboard, History } from '../ui/Table.jsx';
import { CardText } from '../ui/Card.jsx';
import { Seg, Sheet, fmtPts, pct } from '../ui/kit.jsx';
import { EngineGate } from './EngineGate.jsx';

const DEFAULT = ['master', 'overbid', 'club', 'early-wizard'];

export default function Watch() {
  const eng = useEngine();
  const [prefs, setPrefs] = usePrefs();
  const [setup, setSetup] = useState(DEFAULT);
  const [table, setTable] = useState(null); // { game, ids, names, personas }
  const [st, setSt] = useState(null);
  const [running, setRunning] = useState(false);
  const [feed, setFeed] = useState([]);
  const [shown, setShown] = useState(null); // the trick on the felt (stays a moment after it's taken)
  const [hist, setHist] = useState(false);
  const timer = useRef(null);

  const start = () => {
    table?.game.free();
    const ids = setup;
    const game = new Game({ players: ids.length, seats: ids.map(seatSpec), seed: Math.floor(Math.random() * 2 ** 31) });
    const t = { game, ids, names: seatNames(ids), personas: ids.map(byId) };
    setTable(t);
    const s = game.state(-1);
    setSt(s);
    setShown({ plays: [] });
    setFeed([{ k: 0, text: `New game: ${t.names.join(', ')}. ${s.rounds} rounds.` }]);
    setRunning(true);
  };
  useEffect(() => () => { clearTimeout(timer.current); }, []);

  const step = useCallback(() => {
    if (!table) return;
    const { game, names } = table;
    if (game.gameOver()) { setRunning(false); return; }
    if (game.roundOver()) {
      game.nextRound();
      const s = game.state(-1);
      setSt(s);
      setShown({ plays: [] });
      setFeed((f) => [{ k: Date.now(), round: true, text: `Round ${s.round}: ${s.round} card${s.round > 1 ? 's' : ''} each, ${names[s.dealer]} deals.` }, ...f].slice(0, 80));
      return;
    }
    const seat = game.toAct();
    const adv = game.advise(seat);
    const ev = game.botStep();
    const s = game.state(-1);
    setSt(s);
    const line = describe(ev, adv, names, s);
    const lines = [line];
    if (ev.trick) {
      setShown(ev.trick);
      lines.unshift({ k: Date.now() + 1, text: <>{names[ev.trick.winner]} takes the trick.</>, sub: true });
    } else setShown({ plays: s.trick });
    if (ev.roundOver) {
      const r = s.past[s.past.length - 1];
      const made = r.bids.map((b, i) => (b === r.won[i] ? names[i] : null)).filter(Boolean);
      lines.unshift({ k: Date.now() + 2, round: true, text: <>Round {r.round} over. {made.length ? `${made.join(', ')} made ${made.length > 1 ? 'their bids' : 'the bid'}.` : 'Nobody made their bid.'}</> });
    }
    if (ev.gameOver) {
      const best = Math.max(...s.totals);
      const winners = s.totals.map((t, i) => (t === best ? names[i] : null)).filter(Boolean);
      lines.unshift({ k: Date.now() + 3, round: true, text: <b>{winners.join(' and ')} win{winners.length > 1 ? '' : 's'} with {best} points.</b> });
      setRunning(false);
    }
    setFeed((f) => [...lines, ...f].slice(0, 80));
  }, [table]);

  // the loop: one move per beat, a longer pause after a trick or a round
  useEffect(() => {
    if (!running || !table) return undefined;
    const base = PACE[prefs.pace] || PACE.normal;
    const pause = table.game.roundOver() ? base * 3 : shown?.winner != null ? base * 1.8 : base;
    timer.current = setTimeout(step, pause);
    return () => clearTimeout(timer.current);
  }, [running, st, table, step, prefs.pace]); // eslint-disable-line react-hooks/exhaustive-deps

  if (eng.loading || eng.error) return <EngineGate eng={eng} />;

  if (!table) return <WatchSetup setup={setup} setSetup={setSetup} onStart={start} />;

  const { names, personas } = table;
  const n = st.players;
  const top = [...Array(Math.ceil(n / 2)).keys()];
  const bottom = [...Array(n).keys()].slice(top.length);
  const turn = st.phase.kind !== 'done' ? st.phase.seat : -1;
  const narrow = typeof window !== 'undefined' && window.innerWidth < 640;
  const seatEl = (s) => <Seat key={s} st={st} s={s} names={names} personas={personas} showHand turn={turn === s} cardW={narrow ? 28 : n >= 5 ? 34 : 40} />;

  return (
    <div className="page !max-w-[1280px]">
      <div className="grid lg:grid-cols-[minmax(0,1fr)_360px] gap-4 items-start">
        <div className="felt p-3 sm:p-4 space-y-3">
          <div className="flex items-center justify-between gap-3 flex-wrap relative z-[1]">
            <TrumpInfo st={st} names={names} />
            <div className="flex items-center gap-2">
              <button className="btn btn-primary btn-sm" onClick={() => setRunning((r) => !r)} disabled={st.gameOver}>{running ? 'Pause' : 'Play'}</button>
              <button className="btn btn-sm" onClick={() => { setRunning(false); step(); }} disabled={running || st.gameOver}>Step</button>
            </div>
          </div>
          <div className={`grid grid-cols-2 gap-2 sm:gap-2.5 relative z-[1] ${top.length >= 3 ? 'sm:grid-cols-3' : ''}`}>{top.map(seatEl)}</div>
          <div className="relative z-[1]"><Trick st={st} names={names} shown={shown} cardW={narrow ? 44 : n >= 5 ? 50 : 58} /></div>
          <div className={`grid grid-cols-2 gap-2 sm:gap-2.5 relative z-[1] ${bottom.length >= 3 ? 'sm:grid-cols-3' : ''}`}>{bottom.map(seatEl)}</div>
        </div>

        <aside className="space-y-3">
          <div className="panel panel-pad space-y-3">
            <div className="flex items-center justify-between gap-2">
              <span className="h-sec">Pace</span>
              <Seg value={prefs.pace} onChange={(v) => setPrefs({ pace: v })} options={[['slow', 'Slow'], ['normal', 'Normal'], ['fast', 'Fast']]} />
            </div>
            <div className="flex gap-2">
              <button className="btn btn-sm flex-1" onClick={() => setHist(true)}>Score sheet</button>
              <button className="btn btn-sm flex-1" onClick={() => { setRunning(false); table.game.free(); setTable(null); }}>New game</button>
            </div>
          </div>
          <div className="panel panel-pad">
            <div className="h-sec mb-2">Standings</div>
            <Scoreboard st={st} names={names} personas={personas} />
          </div>
          <div className="panel panel-pad">
            <div className="h-sec mb-2">Commentary</div>
            <div className="scroll-y max-h-[420px] space-y-1.5 text-sm">
              {feed.map((l, i) => (
                <div key={l.k + '-' + i} className={`${l.round ? 'text-white font-semibold pt-1' : l.sub ? 'text-ink-300 text-xs' : 'text-ink-100'} ${i === 0 ? 'fade-up' : ''}`}>
                  {l.text}{l.note && <div className="text-xs text-amber-300 mt-0.5">{l.note}</div>}
                </div>
              ))}
            </div>
          </div>
        </aside>
      </div>
      <Sheet open={hist} onClose={() => setHist(false)} title="Score sheet" wide>
        <History st={st} names={names} />
      </Sheet>
    </div>
  );
}

/** One commentary line for a bot's move, with the best bot's view of it. */
function describe(ev, adv, names, st) {
  const who = names[ev.seat];
  const kind = codeKind(ev.code);
  const opt = adv?.options.find((o) => o.code === ev.code);
  const pick = adv?.options.find((o) => o.code === adv.pick);
  const gap = opt && pick ? pick.points - opt.points : 0;
  let text, note = null;
  if (kind === 'trump') text = <>{who} names {SUIT_NAME[ev.code]} trump.</>;
  else if (kind === 'bid') {
    const b = codeBid(ev.code);
    text = <>{who} bids <b>{b}</b>{opt ? <span className="text-ink-300"> · expects {fmtPts(opt.points)}{opt.make != null ? `, ${pct(opt.make)} to make it` : ''}</span> : null}</>;
    if (pick && pick.code !== ev.code && gap >= 3) note = `The Wizard would bid ${codeBid(pick.code)} (about ${Math.round(gap)} points better).`;
  } else {
    const c = codeCard(ev.code);
    text = <>{who} plays <CardText c={c} />.</>;
    if (pick && pick.code !== ev.code && gap >= 4) note = <>The Wizard would play <CardText c={codeCard(pick.code)} /> (about {Math.round(gap)} points better).</>;
  }
  return { k: `${st.round}-${st.tricksDone}-${ev.seat}-${ev.code}`, text, note };
}

function WatchSetup({ setup, setSetup, onStart }) {
  const n = setup.length;
  const setN = (k) => setSetup((s) => (k > s.length ? [...s, ...Array(k - s.length).fill(0).map(() => pickRandom('mixed'))] : s.slice(0, k)));
  return (
    <div className="page fade-up">
      <h1 className="h-title font-display">Watch</h1>
      <p className="muted mt-1 mb-5 max-w-2xl">Pick who sits at the table and watch a full game with every hand face up. The commentary shows what the best bot expects from each bid and when it would have played something else.</p>
      <div className="grid lg:grid-cols-[1fr_340px] gap-5 items-start">
        <section className="panel panel-pad space-y-4">
          <div className="flex items-center justify-between gap-3 flex-wrap">
            <span className="h-sec">Players</span>
            <Seg value={n} onChange={setN} options={[3, 4, 5, 6].map((k) => [k, String(k)])} />
          </div>
          <SeatPickers ids={setup} onChange={setSetup} />
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-xs text-ink-300 mr-1">Fill with</span>
            {TABLES.map((t) => <button key={t.id} className="btn btn-sm" onClick={() => setSetup(setup.map(() => pickRandom(t.id)))}>{t.label}</button>)}
            <button className="btn btn-sm" onClick={() => setSetup(setup.map(() => 'master'))}>All Wizards</button>
          </div>
        </section>
        <aside className="panel panel-pad space-y-3">
          <button className="btn btn-primary btn-lg btn-block" onClick={onStart}>Start watching</button>
          <p className="text-xs text-ink-300 leading-relaxed">Everyone bids at once, as at your table. Pause any time, or step one move at a time. The bars beside the scores are each player's chance to win the game.</p>
        </aside>
      </div>
    </div>
  );
}

/** A picker per seat: levels and characters. `fixed`: seats that can't change (you). */
export function SeatPickers({ ids, onChange, fixed = {} }) {
  return (
    <div className="grid sm:grid-cols-2 gap-2.5">
      {ids.map((id, s) => {
        if (fixed[s]) return <div key={s} className="seatbox flex items-center gap-2.5"><span className="avatar">⭐</span><div><div className="text-sm font-semibold text-white">You</div><div className="text-[11px] text-ink-300">seat {s + 1}</div></div></div>;
        const p = byId(id);
        return (
          <label key={s} className="seatbox flex items-center gap-2.5">
            <span className="avatar">{p.icon}</span>
            <span className="min-w-0 flex-1">
              <select className="w-full" value={id} onChange={(e) => onChange(ids.map((x, i) => (i === s ? e.target.value : x)))} aria-label={`Seat ${s + 1}`}>
                <optgroup label="Levels">{ALL.filter((x) => x.kind === 'level').map((x) => <option key={x.id} value={x.id}>{x.icon} {x.name} · {x.label}</option>)}</optgroup>
                <optgroup label="Characters">{ALL.filter((x) => x.kind === 'character').map((x) => <option key={x.id} value={x.id}>{x.icon} {x.name}</option>)}</optgroup>
              </select>
              <span className="block text-[11px] text-ink-300 mt-1 leading-snug">{p.blurb}</span>
            </span>
          </label>
        );
      })}
    </div>
  );
}

