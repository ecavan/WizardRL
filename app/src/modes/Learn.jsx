/**
 * Learn: how to bid (a value for every card, and a counter for your hand), how the best bot
 * plays its cards, what common mistakes cost, and the rules. The data comes from the charts in
 * rl/charts (made from ppo5's self-play; see the README).
 */
import { useEffect, useMemo, useState } from 'react';
import { Seg } from '../ui/kit.jsx';
import { load, save } from '../lib/store.js';
import { SuitSym } from '../ui/Card.jsx';
import { Card } from '../ui/Card.jsx';

const useData = (name) => {
  const [d, set] = useState(null);
  useEffect(() => { fetch(`/data/${name}.json`).then((r) => r.json()).then(set).catch(() => set(false)); }, [name]);
  return d;
};

export default function Learn() {
  const [view, setView] = useState(() => load('wizard.learn.view', 'bid'));
  const [players, setPlayers] = useState(() => load('wizard.learn.players', 4));
  const pick = (v) => { setView(v); save('wizard.learn.view', v); };
  const pickP = (v) => { setPlayers(v); save('wizard.learn.players', v); };
  return (
    <div className="page fade-up space-y-5">
      <header className="space-y-1">
        <h1 className="h-title font-display">Learn</h1>
        <p className="muted max-w-3xl">{{
          bid: <>How many tricks each card is worth, learned by the best bot playing itself 200,000 rounds at each table size. <b className="text-white">Add up your cards, round to the nearest whole number, and bid that.</b></>,
          play: <>Bidding is the pre-flop; this is the post-flop: how the bot plays its cards once the bids are in, by where you stand against your bid, your seat in the trick and what's winning it.</>,
          mistakes: <>Which departures from the bot's play cost the most, in points and in chance of winning the game.</>,
          rules: <>The rules of Wizard, as the bots play them.</>,
        }[view]}</p>
      </header>
      <div className="flex flex-wrap gap-x-6 gap-y-3 items-end">
        <div className="space-y-1"><div className="h-sec">Show</div>
          <Seg value={view} onChange={pick} options={[['bid', 'Bidding'], ['play', 'Card play'], ['mistakes', 'Mistakes'], ['rules', 'Rules']]} /></div>
        {view !== 'rules' && <div className="space-y-1"><div className="h-sec">Players</div>
          <Seg value={players} onChange={pickP} options={[3, 4, 5, 6].map((n) => [n, String(n)])} /></div>}
      </div>
      {view === 'bid' && <BidChart players={players} />}
      {view === 'play' && <PlayChart players={players} />}
      {view === 'mistakes' && <Mistakes players={players} />}
      {view === 'rules' && <Rules />}
    </div>
  );
}

// ------------------------------------------------------------------------------- bidding

const SUITS4 = [[3, 'spades'], [2, 'hearts'], [1, 'diamonds'], [0, 'clubs']]; // engine suit index, name
const RANKS = ['A', 'K', 'Q', 'J', '10', '9', '8', '7', '6', '5', '4', '3', '2'];
const tenth = (v) => Math.round(v * 10) / 10;
const LABEL = (k) => { const m = k.match(/^(trump|off-suit) (.+)$/); return m ? [m[2], m[1]] : [k, '']; };

function BidChart({ players }) {
  const data = useData('bid_chart');
  const [suit, setSuit] = useState(() => load('wizard.learn.suit', 2)); // engine suit index, or -1
  const [prec, setPrec] = useState(() => load('wizard.learn.prec', 'q'));
  const [size, setSize] = useState(5);
  const [cards, setCards] = useState(() => new Set(['2:A', '2:4', '3:K', '0:9']));
  const [wiz, setWiz] = useState(1);
  const [jes, setJes] = useState(0);
  const [example, setExample] = useState(true);
  if (data === false) return <p className="muted">Couldn't load the chart.</p>;
  if (!data) return null;
  const T = data.tables[String(players)];
  if (!T) return <p className="muted">No chart for {players} players.</p>;
  const maxCards = Math.floor(60 / players);
  const sz = Math.min(size, maxCards);
  const last = sz === maxCards;
  const key = last || suit < 0 ? 'no_trump' : 'trump';
  const kinds = key === 'trump' ? data.kinds : data.no_trump_kinds;
  const shown = (v) => (prec === 'q' ? tenth(v) : v);
  const fmt = (v) => { const x = shown(v); let t = Math.abs(x).toFixed(2); if (prec === 'q') t = t.replace(/\.?0+$/, '') || '0'; return (x < -0.001 ? '−' : '') + t; };
  const bands = T[key];
  const bandIdx = bands.findIndex((r) => sz >= r.lo && sz <= r.hi);
  let row = bands[bandIdx], borrowed = false;
  if (!row && key === 'no_trump') { row = T.trump.find((r) => sz >= r.lo && sz <= r.hi); borrowed = !!row; }
  const val = (k) => (row ? shown(row.values[k] ?? 0) : 0);
  const kindOf = (s, r) => (key === 'trump' && s === suit ? 'trump ' : 'off-suit ') + r;
  const count = cards.size + wiz + jes;
  const full = count >= sz;
  let total = wiz * val('Wizard') + jes * val('Jester');
  for (const id of cards) { const [s, r] = id.split(':'); total += val(kindOf(Number(s), r)); }
  const bid = Math.max(0, Math.min(sz, Math.round(total)));
  const pctf = (x) => Math.round(x * 100) + '%';
  const pp = (x) => (x == null ? '' : (x >= 0 ? '+' : '−') + Math.round(Math.abs(x) * 100) + '%');
  const touch = () => setExample(false);
  const trim = (k) => { // keep the hand no bigger than the round
    let c = new Set(cards), w = wiz, j = jes;
    while (c.size + w + j > k) { if (j) j--; else if (c.size) c.delete([...c].pop()); else w--; }
    setCards(c); setWiz(w); setJes(j);
  };
  const seat = T.seat || {};
  const order = SUITS4.slice().sort((a, b) => (b[0] === suit && key === 'trump') - (a[0] === suit && key === 'trump'));

  return (
    <div className="grid lg:grid-cols-[minmax(0,1fr)_380px] gap-5 items-start">
      <section className="panel panel-pad min-w-0">
        <div className="flex flex-wrap items-end justify-between gap-3 mb-3">
          <div>
            <h2 className="text-lg font-semibold text-white">Tricks per card</h2>
            <p className="text-xs text-ink-300">{players} players · {key === 'trump' ? 'a trump suit' : 'no trump (a Jester turned up, or the last round)'} · columns are cards in hand</p>
          </div>
          <div className="flex flex-wrap gap-2">
            <Seg value={key === 'no_trump' ? -1 : suit} onChange={(v) => { if (last && v >= 0) setSize(maxCards - 1); setSuit(v); save('wizard.learn.suit', v); }}
              options={[...SUITS4.map(([s]) => [s, <SuitSym key={s} s={s} />]), [-1, 'No trump']]} label="Trump" />
            <Seg value={prec} onChange={(v) => { setPrec(v); save('wizard.learn.prec', v); }} options={[['q', 'Simple'], ['x', 'Exact']]} label="Values" />
          </div>
        </div>
        <div className="overflow-x-auto -mx-1">
          <table className="chart">
            <thead><tr><th className="!text-left">Card</th>{bands.map((b, i) => <th key={b.name} className={i === bandIdx ? 'on' : ''}>{b.name.replace('-', '–')}</th>)}</tr></thead>
            <tbody>
              {kinds.map((k) => {
                const [r, s] = LABEL(k);
                return (
                  <tr key={k}>
                    <th className={k === 'Wizard' ? '!text-amber-300' : ''}>{r} {s && <span className="font-normal text-ink-300">{key === 'no_trump' && s === 'off-suit' ? 'any suit' : s}</span>}</th>
                    {bands.map((b, i) => {
                      const v = b.values[k];
                      const a = Math.max(0, Math.min(1, shown(v) / 1.5));
                      const bg = shown(v) < -0.01 ? 'rgb(244 63 94 / .18)' : `rgb(var(--heat) / ${(0.06 + a * 0.94).toFixed(2)})`;
                      return <td key={b.name} className={`v ${a > 0.5 ? 'hot' : ''} ${i === bandIdx ? 'on' : ''}`} style={{ background: bg }} title={`${k}, ${b.name}: ${v.toFixed(3)} tricks`}>{fmt(v)}</td>;
                    })}
                  </tr>
                );
              })}
              {[['Chart bid made', (b) => pctf(prec === 'q' ? b.q_hit : b.hit)], ["Bot's own bid made", (b) => pctf(b.bot_hit)], ['Average miss', (b) => b.mae.toFixed(2)],
                ['Each Jester: chance to make your bid', (b) => pp(b.make?.Jester)], ['Each Wizard: chance to make your bid', (b) => pp(b.make?.Wizard)]].map(([name, f], j) => (
                <tr key={name} className={`acc ${j === 0 ? 'first-acc' : ''}`}><th>{name}</th>{bands.map((b) => <td key={b.name}>{f(b)}</td>)}</tr>
              ))}
            </tbody>
          </table>
        </div>
        <p className="text-xs text-ink-300 mt-3">A Jester is worth about zero tricks, but it's far from useless: you can always duck a trick with it, so each one you hold raises your chance of making your bid (the "Each Jester" row, compared with holding another card instead).</p>
      </section>

      <section className="panel panel-pad space-y-3 lg:sticky lg:top-20">
        <div>
          <h2 className="text-lg font-semibold text-white">Count a hand</h2>
          <p className="text-xs text-ink-300">Set trump and how many cards you hold, then tap your cards. {example && <span className="tag !text-amber-300">Example hand</span>}</p>
        </div>
        <div className="flex items-center gap-3 flex-wrap">
          <span className="h-sec">Cards in hand</span>
          <span className="inline-flex items-center rounded-lg border border-ink-600 overflow-hidden">
            <button className="w-9 h-9 bg-ink-800 disabled:opacity-35" disabled={sz <= 1} onClick={() => { setSize(sz - 1); trim(sz - 1); touch(); }} aria-label="Fewer cards">−</button>
            <output className="w-10 text-center num font-semibold">{sz}</output>
            <button className="w-9 h-9 bg-ink-800 disabled:opacity-35" disabled={sz >= maxCards} onClick={() => { setSize(sz + 1); touch(); }} aria-label="More cards">+</button>
          </span>
          <button className="text-sm text-violet-300 underline" onClick={() => { setCards(new Set()); setWiz(0); setJes(0); touch(); }}>Clear</button>
        </div>
        {[['Wizards', wiz, setWiz, 'Wizard'], ['Jesters', jes, setJes, 'Jester']].map(([label, n, setN, k]) => (
          <div key={label} className="flex items-center gap-3">
            <span className={`flex-1 text-sm ${n ? 'font-semibold text-white' : 'text-ink-200'}`}>{label}</span>
            <span className="inline-flex items-center rounded-lg border border-ink-600 overflow-hidden">
              <button className="w-8 h-8 bg-ink-800 disabled:opacity-35" disabled={n === 0} onClick={() => { setN(n - 1); touch(); }} aria-label={`One fewer ${k}`}>−</button>
              <output className="w-8 text-center num">{n}</output>
              <button className="w-8 h-8 bg-ink-800 disabled:opacity-35" disabled={n >= 4 || full} onClick={() => { setN(n + 1); touch(); }} aria-label={`One more ${k}`}>+</button>
            </span>
            <span className="w-14 text-right text-xs text-ink-300 num">{row ? fmt(row.values[k]) + ' ea' : ''}</span>
          </div>
        ))}
        <div className="space-y-2">
          {order.map(([s, name]) => {
            const isT = key === 'trump' && s === suit;
            return (
              <div key={s} className="grid grid-cols-[78px_minmax(0,1fr)] items-center gap-2">
                <span className="text-sm"><SuitSym s={s} className="text-lg" /> <span className="text-xs text-ink-300">{isT ? 'trump' : name}</span></span>
                <span className="flex flex-wrap gap-1">
                  {RANKS.map((r) => {
                    const id = `${s}:${r}`, on = cards.has(id);
                    return (
                      <button key={r} className={`chip ${on ? 'on' : ''} ${isT ? 'trump' : ''}`} disabled={!on && full} aria-pressed={on}
                        title={`${fmt(val(kindOf(s, r)))} tricks`}
                        onClick={() => { const c = new Set(cards); on ? c.delete(id) : c.add(id); setCards(c); touch(); }}>{r}</button>
                    );
                  })}
                </span>
              </div>
            );
          })}
        </div>
        <div className="border-t border-ink-700 pt-3">
          <div className="flex items-baseline gap-3"><span className="font-display text-5xl text-white leading-none">{count ? bid : '–'}</span>
            <span className="text-sm text-ink-300">{count ? `bid · chart total ${total.toFixed(2)} tricks` : 'tap in your cards'}</span></div>
          <div className="text-xs text-rose-300 mt-1 min-h-[1.2em]">{count < sz ? `${count} of ${sz} cards entered` : borrowed ? "Using the trump chart's off-suit values for this round size." : ''}</div>
          {seat['1'] != null && <div className="text-xs text-ink-300">Seat: leading first takes {sgn(seat['1'])} tricks vs the chart on average; the dealer {sgn(seat[String(players)])}.</div>}
        </div>
      </section>
    </div>
  );
}
const sgn = (x) => (x == null ? '' : (x >= 0 ? '+' : '−') + Math.abs(x).toFixed(2));

// ------------------------------------------------------------------------------- card play

const PLAYCOLOR = (k) => (k.startsWith('win big') ? 'rgb(var(--heat) / .5)' : k.startsWith('win') ? 'rgb(var(--heat))' : k === 'duck high' ? 'rgb(var(--ink-300))'
  : k === 'duck low' ? 'rgb(var(--ink-500))' : k === 'Wizard' ? '#f59e0b' : k === 'Jester' ? '#94a3b8' : k.includes('high') ? '#2563eb' : '#93c5fd');
const NEEDS = [['need all', 'Need every trick left'], ['need more', 'Still need tricks'], ['made it', 'Made my bid exactly'], ['over', 'Already over']];
const SUITCH = { s: 3, h: 2, d: 1, c: 0 };
const parseCard = (t) => {
  if (t === 'wiz') return 52;
  if (t === 'jes') return 56;
  const r = ['2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K', 'A'].indexOf(t.slice(0, -1));
  return SUITCH[t.slice(-1)] * 13 + r;
};
function PlayChart({ players }) {
  const play = useData('play_chart');
  const sits = useData('play_situations');
  const [need, setNeed] = useState('need more');
  if (!play) return null;
  const rows = (play.tables[String(players)] || []).filter((r) => r.situation.need === need);
  const total = (play.tables[String(players)] || []).reduce((a, r) => a + r.n, 0);
  return (
    <div className="space-y-5">
      <section className="panel panel-pad">
        <h2 className="text-lg font-semibold text-white">How the bot plays its cards</h2>
        <p className="text-xs text-ink-300 mb-3">{players} players · from {total.toLocaleString()} of the bot's own card plays where it had a real choice · each bar shows how often it makes each kind of play</p>
        <Seg value={need} onChange={setNeed} options={NEEDS} label="Where you stand against your bid" />
        <div className="overflow-x-auto mt-3">
          <table className="w-full text-sm">
            <thead><tr className="text-[11px] text-ink-300 text-left"><th className="font-semibold py-1 pr-3">Seat</th><th className="font-semibold pr-3">Trick so far</th><th className="font-semibold pr-3">Your cards</th><th className="font-semibold">What the bot does</th><th className="font-semibold text-right">Seen</th></tr></thead>
            <tbody>
              {rows.map((r, i) => {
                const s = r.situation, entries = Object.entries(r.plays);
                return (
                  <tr key={i} className="border-t border-ink-800 align-middle">
                    <td className="py-2 pr-3 text-ink-200">{s.seat}</td>
                    <td className="pr-3 text-ink-200">{s.trick}</td>
                    <td className="pr-3 text-ink-200">{s.follow}</td>
                    <td className="min-w-[220px]">
                      <div className="text-xs text-ink-300">{entries.filter(([, v]) => v >= 0.05).map(([k, v], j) => <span key={k}>{j ? ' · ' : ''}{j === 0 ? <b className="text-white">{k} {Math.round(v * 100)}%</b> : `${k} ${Math.round(v * 100)}%`}</span>)}</div>
                      <div className="pbar">{entries.map(([k, v]) => <span key={k} title={`${k} ${Math.round(v * 100)}%`} style={{ width: `${v * 100}%`, background: PLAYCOLOR(k) }} />)}</div>
                    </td>
                    <td className="text-right text-xs text-ink-300 num pl-2">{r.n.toLocaleString()}</td>
                  </tr>
                );
              })}
            </tbody>
          </table>
        </div>
        <p className="text-xs text-ink-300 mt-3"><b>win cheaply</b>: the lowest card that takes the trick · <b>win big</b>: a stronger winner than needed · <b>duck high</b>: the highest card that still loses · <b>duck low</b>: a lower losing card · <b>trump in</b>: winning with trump when you can't follow.</p>
      </section>
      {sits && (
        <section className="panel panel-pad">
          <h2 className="text-lg font-semibold text-white">Example decisions</h2>
          <p className="text-xs text-ink-300 mb-3">Expected points for the round and your chance of making your bid, for each card you could play (first trick of a round, 4 players). <b className="text-amber-300">Plays</b> marks the card the bot plays.</p>
          <div className="grid gap-3 [grid-template-columns:repeat(auto-fill,minmax(290px,1fr))]">
            {sits.situations.map((x) => {
              const best = x.plays[0].points;
              return (
                <div key={x.title} className="rounded-xl border border-ink-700 p-3 space-y-2 min-w-0">
                  <h3 className="text-sm font-semibold text-white leading-snug">{x.title}</h3>
                  <div className="flex items-center gap-1.5 flex-wrap">
                    {x.hand.map((c) => <Card key={c} c={parseCard(c)} w={30} trump={x.trump ? SUITCH[x.trump] : null} />)}
                    {x.trick.length > 0 && <span className="text-[11px] text-ink-300 mx-1">trick:</span>}
                    {x.trick.map((c) => <Card key={'t' + c} c={parseCard(c)} w={30} />)}
                  </div>
                  <table className="w-full text-sm num">
                    <tbody>
                      {x.plays.map((p, i) => (
                        <tr key={p.card} className={i === 0 ? 'text-white font-semibold' : 'text-ink-200'}>
                          <td className="py-0.5">{p.card.replace(/([♥♦])/g, '$1')}{p.pick && <span className="ml-1.5 tag !text-amber-300">plays</span>}</td>
                          <td className="text-right">{p.points >= 0 ? '+' : '−'}{Math.abs(p.points).toFixed(1)}</td>
                          <td className="text-right text-ink-300">{i === 0 ? 'best' : (p.points - best).toFixed(1)}</td>
                          <td className="text-right text-ink-300">{p.make == null ? '' : Math.round(p.make * 100) + '%'}</td>
                        </tr>
                      ))}
                    </tbody>
                  </table>
                </div>
              );
            })}
          </div>
        </section>
      )}
    </div>
  );
}

// ------------------------------------------------------------------------------- mistakes

const NEEDTEXT = { 'need all': 'need every trick left', 'need more': 'need more tricks', 'made it': 'made your bid', over: 'already over' };
function misSituation(r) {
  if (r.kind === 'bid') return r.situation[0];
  const [need, seat, trick, follow] = r.situation;
  if (seat === 'lead') return `${NEEDTEXT[need]}, you lead`;
  return `${NEEDTEXT[need]}, ${seat === 'last' ? 'last to play' : 'mid-trick'}, ${trick}, ${follow}`;
}
const lost = (x, d) => (x >= 0 ? '−' : '+') + Math.abs(x).toFixed(d);

function Mistakes({ players }) {
  const data = useData('mistake_chart');
  const [kind, setKind] = useState('habits');
  const rows = useMemo(() => {
    const t = data?.tables[String(players)];
    if (!t) return [];
    const all = t.mistakes;
    const plays = all.filter((r) => r.kind === 'play' && !r.mistake.includes('a different card'));
    if (kind === 'bid') return all.filter((r) => r.kind === 'bid').slice(0, 16);
    if (kind === 'habits') return plays.slice(0, 20);
    const common = Math.max(50, Math.floor(plays.reduce((a, r) => a + r.count, 0) / 1000));
    return plays.filter((r) => r.count >= common).sort((a, b) => b.points - a.points).slice(0, 20);
  }, [data, players, kind]);
  if (!data) return null;
  const t = data.tables[String(players)];
  if (!t) return <p className="muted">No mistake chart for {players} players.</p>;
  const lp = t.lost_per_game;
  const max = Math.max(...rows.map((r) => (kind === 'habits' ? r.per_game : r.points)), 1e-9);
  return (
    <section className="panel panel-pad">
      <h2 className="text-lg font-semibold text-white">What mistakes cost</h2>
      <p className="text-xs text-ink-300 mb-3">{players} players · a human-like player gives up about <b className="text-white">{Math.round(lp.bid.points)} points a game in bidding</b> and <b className="text-white">{Math.round(lp.play.points)} in card play</b> against the bot's choices.</p>
      <Seg value={kind} onChange={setKind} options={[['habits', 'Costliest habits'], ['big', 'Biggest mistakes'], ['bid', 'Bidding']]} />
      <div className="mt-3 space-y-2">
        {rows.map((r, i) => {
          const v = kind === 'habits' ? r.per_game : r.points;
          const m = r.mistake.match(/^bot: (.*) \/ you: (.*)$/);
          return (
            <div key={i} className="rounded-xl border border-ink-800 px-3 py-2">
              <div className="text-xs text-ink-300">{misSituation(r)}</div>
              <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-0.5 sm:gap-3 mt-0.5">
                <div className="text-sm text-ink-100 min-w-0">{m ? <>bot: <b className="text-white">{m[1]}</b> · you: <b className="text-white">{m[2]}</b></> : r.mistake}</div>
                <div className="sm:text-right text-xs num shrink-0"><span className="text-white font-semibold">{lost(r.points, 1)} pts</span> <span className="text-ink-300">· {lost(r.win, 1)}% win · {lost(r.per_game, 1)}/game</span></div>
              </div>
              <div className="bar mt-1.5"><i style={{ width: `${Math.max(0, (100 * v) / max)}%`, background: 'rgb(var(--heat))' }} /></div>
            </div>
          );
        })}
      </div>
      <p className="text-xs text-ink-300 mt-3"><b>Points</b>: expected points lost that round. <b>Win</b>: chance of winning the game lost (a rough reading). <b>Per game</b>: what the habit costs a human-like player over a whole game. A Wizard played when a Wizard is already winning can't take the trick: it just throws your Wizard away.</p>
    </section>
  );
}

// ------------------------------------------------------------------------------- rules

function Rules() {
  return (
    <div className="grid md:grid-cols-2 gap-4">
      <section className="panel panel-pad space-y-2 text-sm text-ink-100 leading-relaxed">
        <h2 className="text-lg font-semibold text-white">The deck and the deal</h2>
        <p>60 cards: the usual 52, plus 4 <b className="text-violet-300">Wizards</b> and 4 <b>Jesters</b>. 3 to 6 players.</p>
        <p>Round 1 deals one card each, round 2 two, and so on until the deck runs out: 20 rounds with 3 players, 15 with 4, 12 with 5, 10 with 6. The deal moves left each round.</p>
        <p>After the deal the next card is turned up for <b>trump</b>. A Jester (or no card left, in the last round) means no trump; a Wizard means the dealer names trump.</p>
        <h2 className="text-lg font-semibold text-white pt-2">Bidding</h2>
        <p>Everyone says how many tricks they'll take. In this app everyone bids at once by default, as many families play; the printed rules (bids in turn, starting left of the dealer) are an option in Play.</p>
      </section>
      <section className="panel panel-pad space-y-2 text-sm text-ink-100 leading-relaxed">
        <h2 className="text-lg font-semibold text-white">Tricks</h2>
        <p>The player left of the dealer leads; whoever takes a trick leads the next. You must follow the suit led if you can. <b className="text-violet-300">Wizards</b> and <b>Jesters</b> can always be played.</p>
        <p>The <b>first Wizard</b> played takes the trick. Otherwise the highest trump wins, otherwise the highest card of the suit led. Jesters never win (unless every card is a Jester: then the first one does).</p>
        <p>If a Wizard leads, nobody has to follow anything. If a Jester leads, the first standard card played sets the suit.</p>
        <h2 className="text-lg font-semibold text-white pt-2">Scoring</h2>
        <ul className="list-disc pl-5 space-y-1">
          <li>Take exactly your bid: <b className="text-emerald-300">20 points + 10 per trick</b>.</li>
          <li>Miss it either way: <b className="text-rose-300">−10 per trick</b> over or under.</li>
        </ul>
        <p>Most points after the last round wins.</p>
      </section>
    </div>
  );
}
