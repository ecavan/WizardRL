/**
 * Play: you against the bots. Pick who sits at the table (levels, characters, or a random
 * table), then play a full game. The coach can show the best bot's view of any decision and
 * says what each of your moves cost. A game in progress survives a reload.
 */
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import { Game, useEngine } from '../lib/engine.js';
import { usePrefs, PACE } from '../lib/prefs.js';
import { TABLES, byId, pickRandom, seatNames, seatSpec } from '../lib/players.js';
import { load, save, drop } from '../lib/store.js';
import { codeKind, codeBid, codeCard, bidCode, SUIT_NAME } from '../lib/cards.js';
import { Seat, Trick, TrumpInfo, YourHand, Scoreboard, History } from '../ui/Table.jsx';
import { CardText, SuitSym } from '../ui/Card.jsx';
import { Seg, Sheet, Toggle, fmtPts, pct } from '../ui/kit.jsx';
import { EngineGate } from './EngineGate.jsx';
import { SeatPickers } from './Watch.jsx';

const KEY = 'wizard.game.v1';
const YOU = { id: 'you', name: 'You', label: 'You', icon: '⭐' };
const personaOf = (id) => (id === 'you' ? YOU : byId(id));

export default function Play() {
  const eng = useEngine();
  const [table, setTable] = useState(null);
  if (eng.loading || eng.error) return <EngineGate eng={eng} />;
  if (!table) return <PlaySetup onStart={setTable} />;
  return <PlayGame key={table.seed} table={table} onExit={() => setTable(null)}
    onRematch={() => { drop(KEY); setTable({ ...table, seed: Math.floor(Math.random() * 2 ** 31), log: [] }); }} />;
}

// ------------------------------------------------------------------------------- setup

function PlaySetup({ onStart }) {
  const [prefs, setPrefs] = usePrefs();
  const saved = load(KEY, null);
  const [cfg, setCfg] = useState(() => load('wizard.setup.v1', { ids: ['you', 'club', 'strong', 'overbid'], simultaneous: true, preset: 'mixed' }));
  const ids = cfg.ids;
  const set = (patch) => setCfg((c) => { const n = { ...c, ...patch }; save('wizard.setup.v1', n); return n; });
  const setN = (k) => set({ ids: k > ids.length ? [...ids, ...Array(k - ids.length).fill(0).map(() => pickRandom(cfg.preset))] : ids.slice(0, k) });
  const begin = () => {
    drop(KEY);
    onStart({ ids, simultaneous: cfg.simultaneous, seed: Math.floor(Math.random() * 2 ** 31), log: [] });
  };
  const resumable = saved && !saved.over && saved.log?.length > 0;
  return (
    <div className="page fade-up">
      <h1 className="h-title font-display">Play</h1>
      <p className="muted mt-1 mb-5 max-w-2xl">Choose who sits at the table, or deal in a random table at a difficulty you like. You're seat 1; the deal moves left each round.</p>

      {resumable && (
        <button className="panel panel-pad w-full text-left mb-5 flex items-center justify-between gap-4 !border-violet-600/60 hover:bg-ink-850 transition"
          onClick={() => onStart(saved)}>
          <div>
            <div className="h-sec !text-violet-300">Game in progress</div>
            <div className="text-white font-semibold mt-1">{seatNames(saved.ids).slice(1).join(', ')}</div>
            <div className="text-xs text-ink-300 mt-0.5">{saved.log.filter((x) => x < 0).length + 1} round{saved.log.filter((x) => x < 0).length ? 's' : ''} in</div>
          </div>
          <span className="btn btn-primary">Continue</span>
        </button>
      )}

      <div className="grid lg:grid-cols-[1fr_340px] gap-5 items-start">
        <section className="panel panel-pad space-y-4">
          <div className="flex items-center justify-between gap-3 flex-wrap">
            <span className="h-sec">Players</span>
            <Seg value={ids.length} onChange={setN} options={[3, 4, 5, 6].map((k) => [k, String(k)])} label="Players" />
          </div>
          <SeatPickers ids={ids} onChange={(x) => set({ ids: x })} fixed={{ 0: true }} />
          <div className="flex items-center gap-2 flex-wrap">
            <span className="text-xs text-ink-300 mr-1">Random table</span>
            {TABLES.map((t) => (
              <button key={t.id} className={`btn btn-sm ${cfg.preset === t.id ? '!border-violet-500' : ''}`}
                onClick={() => set({ preset: t.id, ids: ids.map((x, i) => (i === 0 ? 'you' : pickRandom(t.id))) })}>{t.label}</button>
            ))}
          </div>
        </section>
        <aside className="panel panel-pad space-y-4">
          <div>
            <div className="h-sec mb-2">Bidding</div>
            <Seg value={cfg.simultaneous} onChange={(v) => set({ simultaneous: v })} options={[[true, 'All at once'], [false, 'In turn']]} className="w-full [&>button]:flex-1" />
            <p className="text-xs text-ink-300 mt-1.5">{cfg.simultaneous ? "Everyone shows their bid together: nobody sees another bid first." : 'The printed rules: bids go round the table, starting left of the dealer.'}</p>
          </div>
          <Toggle on={prefs.coach} onChange={(v) => setPrefs({ coach: v })} label="Coach" sub="A hint button for every decision, and after each move, what it cost next to the best bot's choice." />
          <div className="flex items-center justify-between gap-2">
            <span className="text-sm font-semibold text-ink-100">Bot speed</span>
            <Seg value={prefs.pace} onChange={(v) => setPrefs({ pace: v })} options={[['slow', 'Slow'], ['normal', 'Normal'], ['fast', 'Fast']]} />
          </div>
          <button className="btn btn-primary btn-lg btn-block" onClick={begin}>{resumable ? 'New game' : 'Deal'}</button>
        </aside>
      </div>
    </div>
  );
}

// ------------------------------------------------------------------------------- the game

function PlayGame({ table, onExit, onRematch }) {
  const [prefs] = usePrefs();
  const you = 0;
  const { game, names, personas } = useMemo(() => {
    const g = new Game({ players: table.ids.length, simultaneous: table.simultaneous, seats: table.ids.map(seatSpec), seed: table.seed });
    if (table.log?.length) { try { g.replay(table.log); } catch { /* a stale save: start fresh */ } }
    return { game: g, names: seatNames(table.ids), personas: table.ids.map(personaOf) };
  }, [table]);
  useEffect(() => () => game.free(), [game]);

  const [st, setSt] = useState(() => game.state(you));
  const [shown, setShown] = useState(() => ({ plays: game.state(you).trick }));
  const [review, setReview] = useState(null); // the coach's note on your last move
  const [hint, setHint] = useState(null); // the bot's view of the current decision
  const [sheet, setSheet] = useState(null); // 'round' | 'history' | 'menu'
  const timer = useRef(null);

  const persist = useCallback(() => {
    save(KEY, { ids: table.ids, simultaneous: table.simultaneous, seed: table.seed, log: game.log(), over: game.gameOver() });
  }, [game, table]);

  const after = useCallback((ev) => {
    const s = game.state(you);
    setSt(s);
    setShown(ev.trick ? ev.trick : { plays: s.trick });
    if (ev.roundOver) setSheet('round');
    persist();
  }, [game, persist]);

  // bots move one at a time, with a beat between moves and a longer one after a trick
  useEffect(() => {
    if (st.roundOver || st.gameOver || !game.isBotTurn()) return undefined;
    const base = PACE[prefs.pace] || PACE.normal;
    const wait = shown?.winner != null ? base * 1.8 + 300 : base;
    timer.current = setTimeout(() => after(game.botStep()), wait);
    return () => clearTimeout(timer.current);
  }, [st, shown, game, after, prefs.pace]);

  // reopen the round summary after a reload that lands between rounds
  useEffect(() => { if (st.roundOver) setSheet('round'); }, []); // eslint-disable-line react-hooks/exhaustive-deps

  const myTurn = !st.roundOver && st.phase.kind !== 'done' && st.phase.seat === you;
  useEffect(() => { setHint(null); }, [st.round, st.tricksDone, st.phase.kind, st.trick.length]);

  function act(code) {
    if (!myTurn) return;
    const adv = prefs.coach ? game.advise(you) : null;
    const ev = game.act(code);
    if (adv) setReview(reviewOf(code, adv));
    setHint(null);
    after(ev);
  }

  function next() {
    setSheet(null);
    if (game.gameOver()) return;
    game.nextRound();
    const s = game.state(you);
    setSt(s);
    setShown({ plays: [] });
    setReview(null);
    persist();
  }

  const n = st.players;
  const others = [...Array(n - 1).keys()].map((i) => (you + 1 + i) % n);
  const w = typeof window !== 'undefined' ? window.innerWidth : 1200;
  const handW = w < 420 ? 52 : w < 700 ? 60 : 70;
  const phase = st.phase.kind;

  return (
    <div className="page !max-w-[1240px]">
      <div className="grid lg:grid-cols-[minmax(0,1fr)_340px] gap-4 items-start">
        <div className="felt p-3 sm:p-4 space-y-3">
          <div className="flex items-center justify-between gap-3 flex-wrap relative z-[1]">
            <TrumpInfo st={st} names={names} />
            <button className="btn btn-sm" onClick={() => setSheet('menu')}>Menu</button>
          </div>
          <div className={`opps relative z-[1] ${others.length >= 3 ? 'sm:grid-cols-3' : 'sm:grid-cols-2'} ${others.length >= 5 ? 'xl:grid-cols-5' : ''}`}>
            {others.map((s) => <Seat key={s} st={st} s={s} names={names} personas={personas} showHand={false} turn={!st.roundOver && phase !== 'done' && st.phase.seat === s} />)}
          </div>
          <div className="relative z-[1]"><Trick st={st} names={names} shown={shown} cardW={w < 420 ? 46 : 58} /></div>

          <div className="relative z-[1] space-y-3">
            <div className={`seatbox ${myTurn ? 'turn' : ''}`}>
              <div className="flex items-center justify-between gap-3 flex-wrap">
                <div className="flex items-center gap-2.5">
                  <span className="avatar">⭐</span>
                  <div>
                    <div className="text-sm font-semibold text-white">You {st.dealer === you && <span className="tag ml-1">D</span>}</div>
                    <div className="text-[11px] text-ink-300 num">{st.totals[you]} points · {pct(st.chances[you])} to win</div>
                  </div>
                </div>
                <div className="text-sm text-ink-100">
                  {st.bids[you] != null ? <>bid <b className="num">{st.bids[you]}</b> · won <b className="num">{st.won[you]}</b></> : 'no bid yet'}
                </div>
              </div>
              {myTurn && <div className="mt-2 text-sm font-semibold text-violet-200">{phase === 'bid' ? `Your bid: how many tricks will you take with these ${st.round} card${st.round > 1 ? 's' : ''}?` : phase === 'trump' ? 'A Wizard was turned up: you name trump.' : 'Your turn: tap a card to play it.'}</div>}
              {myTurn && phase === 'bid' && <BidPicker st={st} hint={hint} onBid={(b) => act(bidCode(b))} />}
              {myTurn && phase === 'trump' && <TrumpPicker onPick={(s) => act(s)} />}
            </div>
            <YourHand st={st} seat={you} onPlay={act} hint={hint && phase === 'play' ? codeCard(hint.pick) : null} cardW={handW} />
          </div>
        </div>

        <aside className="space-y-3">
          {prefs.coach && (
            <div className="panel panel-pad space-y-2">
              <div className="flex items-center justify-between gap-2">
                <span className="h-sec">Coach</span>
                <button className="btn btn-sm" disabled={!myTurn || !!hint} onClick={() => setHint(game.advise(you))}>Hint</button>
              </div>
              {hint && myTurn && <HintView hint={hint} phase={phase} />}
              {review && !hint && <div className="fade-up text-sm">{review}</div>}
              {!hint && !review && <p className="text-sm text-ink-300">Ask for a hint on any decision. After each move I'll say how it compares with the best bot's choice.</p>}
            </div>
          )}
          <div className="panel panel-pad">
            <div className="flex items-center justify-between mb-2">
              <span className="h-sec">Standings</span>
              <button className="btn btn-quiet btn-sm" onClick={() => setSheet('history')}>Score sheet</button>
            </div>
            <Scoreboard st={st} names={names} personas={personas} />
          </div>
        </aside>
      </div>

      <Sheet open={sheet === 'round'} onClose={next} title={st.gameOver ? 'Final scores' : `Round ${st.round} of ${st.rounds}`}>
        <RoundSummary st={st} names={names} you={you} />
        <div className="grid grid-cols-2 gap-2 mt-4">
          {st.gameOver ? (
            <>
              <button className="btn" onClick={() => { drop(KEY); onExit(); }}>New table</button>
              <button className="btn btn-primary" onClick={onRematch}>Rematch</button>
            </>
          ) : (
            <>
              <button className="btn" onClick={() => setSheet('history')}>Score sheet</button>
              <button className="btn btn-primary" onClick={next}>Next round</button>
            </>
          )}
        </div>
      </Sheet>
      <Sheet open={sheet === 'history'} onClose={() => setSheet(st.roundOver && !st.gameOver ? 'round' : null)} title="Score sheet" wide>
        <History st={st} names={names} />
      </Sheet>
      <Sheet open={sheet === 'menu'} onClose={() => setSheet(null)} title="Game">
        <div className="space-y-2">
          <button className="btn btn-block" onClick={() => setSheet('history')}>Score sheet</button>
          <button className="btn btn-block" onClick={() => { onExit(); }}>Back to the table setup (keeps this game)</button>
          <button className="btn btn-block !text-rose-300" onClick={() => { drop(KEY); onExit(); }}>Abandon this game</button>
        </div>
      </Sheet>
    </div>
  );
}

function BidPicker({ st, hint, onBid }) {
  const opts = hint ? Object.fromEntries(hint.options.map((o) => [codeBid(o.code), o])) : null;
  return (
    <div className="flex flex-wrap gap-2 mt-3">
      {[...Array(st.round + 1).keys()].map((b) => {
        const o = opts?.[b];
        const pick = hint && codeBid(hint.pick) === b;
        return (
          <button key={b} className={`btn !px-0 w-[58px] flex-col !gap-0 !py-1.5 ${pick ? '!border-amber-400 !bg-amber-950/40' : ''}`} onClick={() => onBid(b)} aria-label={`Bid ${b}`}>
            <span className="text-lg leading-tight num">{b}</span>
            {o && <span className="text-[10px] text-ink-300 num leading-tight">{fmtPts(o.points)}{o.make != null ? ` · ${pct(o.make)}` : ''}</span>}
          </button>
        );
      })}
    </div>
  );
}

function TrumpPicker({ onPick }) {
  return (
    <div className="flex flex-wrap gap-2 mt-3">
      {[0, 1, 2, 3].map((s) => (
        <button key={s} className="btn" onClick={() => onPick(s)}><SuitSym s={s} className="text-lg" /> {SUIT_NAME[s]}</button>
      ))}
    </div>
  );
}

function HintView({ hint, phase }) {
  const top = hint.options.slice(0, phase === 'bid' ? 4 : 5);
  const pick = hint.options.find((o) => o.code === hint.pick);
  const label = (o) => (codeKind(o.code) === 'bid' ? `Bid ${codeBid(o.code)}` : codeKind(o.code) === 'trump' ? SUIT_NAME[o.code] : <CardText c={codeCard(o.code)} />);
  return (
    <div className="fade-up space-y-2">
      <div className="text-sm">The Wizard plays <b className="text-amber-300">{label(pick)}</b> here.</div>
      <table className="w-full text-sm num">
        <thead><tr className="text-[11px] text-ink-300"><th className="text-left font-semibold">Option</th><th className="text-right font-semibold">Expected</th><th className="text-right font-semibold">Makes bid</th></tr></thead>
        <tbody>
          {top.map((o) => (
            <tr key={o.code} className={o.code === hint.pick ? 'text-amber-200' : 'text-ink-100'}>
              <td className="py-0.5">{label(o)}</td>
              <td className="text-right">{fmtPts(o.points)}</td>
              <td className="text-right">{o.make != null ? pct(o.make) : ''}</td>
            </tr>
          ))}
        </tbody>
      </table>
      <p className="text-[11px] text-ink-400">Expected points this round, as the bot sees it (including a small bonus for improving its chance to win the game).</p>
    </div>
  );
}

/** What the coach says about the move you just made. */
function reviewOf(code, adv) {
  const mine = adv.options.find((o) => o.code === code);
  const pick = adv.options.find((o) => o.code === adv.pick);
  if (!mine || !pick) return null;
  const label = (o) => (codeKind(o.code) === 'bid' ? `bid ${codeBid(o.code)}` : codeKind(o.code) === 'trump' ? SUIT_NAME[o.code] : <CardText c={codeCard(o.code)} />);
  if (code === adv.pick) return <span className="text-emerald-300">✓ {codeKind(code) === 'bid' ? 'Bid' : 'Move'} matches the Wizard's choice.</span>;
  const gap = pick.points - mine.points;
  if (gap < 1.5) return <span className="text-ink-100">Fine: the Wizard prefers {label(pick)}, but it's a near tie.</span>;
  return (
    <span className={gap >= 8 ? 'text-rose-300' : 'text-amber-300'}>
      {gap >= 8 ? 'Costly: the' : 'The'} Wizard would {codeKind(code) === 'bid' ? '' : 'play '}{label(pick)} ({fmtPts(pick.points)} vs {fmtPts(mine.points)}): about {Math.round(gap)} points better.
    </span>
  );
}

function RoundSummary({ st, names, you }) {
  const r = st.past[st.past.length - 1];
  if (!r) return null;
  const order = [...Array(st.players).keys()].sort((a, b) => st.totals[b] - st.totals[a]);
  const best = Math.max(...st.totals);
  return (
    <div className="space-y-3">
      {st.gameOver && (
        <div className={`verdict ${st.totals[you] === best ? 'v-good' : 'v-info'}`}>
          <div className="text-lg font-semibold text-white">{st.totals[you] === best ? 'You win!' : `${names[order[0]]} wins`}</div>
          <div className="text-sm text-ink-200">You finished {ordinal(order.indexOf(you) + 1)} with {st.totals[you]} points.</div>
        </div>
      )}
      <table className="w-full text-sm num">
        <thead><tr className="text-[11px] text-ink-300"><th className="text-left font-semibold py-1">Player</th><th className="text-right font-semibold">Bid</th><th className="text-right font-semibold">Won</th><th className="text-right font-semibold">Round</th><th className="text-right font-semibold">Total</th><th className="text-right font-semibold">To win</th></tr></thead>
        <tbody>
          {order.map((s) => (
            <tr key={s} className={`border-t border-ink-800 ${s === you ? 'text-white font-semibold' : 'text-ink-100'}`}>
              <td className="py-1.5">{names[s]}</td>
              <td className="text-right">{r.bids[s]}</td>
              <td className="text-right">{r.won[s]}</td>
              <td className={`text-right ${r.scores[s] > 0 ? 'text-emerald-300' : 'text-rose-300'}`}>{r.scores[s] > 0 ? '+' : ''}{r.scores[s]}</td>
              <td className="text-right">{st.totals[s]}</td>
              <td className="text-right text-ink-300">{pct(st.chances[s])}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

const ordinal = (k) => k + (k === 1 ? 'st' : k === 2 ? 'nd' : k === 3 ? 'rd' : 'th');
