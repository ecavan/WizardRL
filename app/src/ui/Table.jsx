/** The card table: seats around a felt, the trick in the middle, your hand at the bottom. */
import { Card, CardText, SuitSym } from './Card.jsx';
import { sortHand, winningIndex, cardCode, codeCard, SUIT_NAME } from '../lib/cards.js';
import { pct } from './kit.jsx';

/** Width for cards in a hand of `n` cards inside `room` pixels (wrapping is fine on phones). */
export function useCardWidth(n, room, max = 74, min = 40) {
  const w = Math.floor((room - (n - 1) * 6) / Math.max(1, n));
  return Math.max(min, Math.min(max, w));
}

export function TrumpInfo({ st, names }) {
  const src = st.trumpSource;
  return (
    <div className="flex items-center gap-3 min-w-0">
      {src.card != null ? <Card c={src.card} w={38} /> : <div className="pc back" style={{ '--w': '38px', opacity: 0.35 }} />}
      <div className="min-w-0 text-sm leading-tight">
        <div className="text-ink-300 text-xs">Round {st.round} of {st.rounds} · {names[st.dealer] === 'You' ? 'you deal' : `${names[st.dealer]} deals`}</div>
        <div className="font-semibold text-white">
          {st.trump != null ? <>Trump: <SuitSym s={st.trump} /> {SUIT_NAME[st.trump]}</>
            : src.kind === 'wizard' ? <>Wizard turned up: {names[st.dealer] === 'You' ? 'you name' : `${names[st.dealer]} names`} trump</>
              : src.kind === 'jester' ? 'Jester turned up: no trump'
                : 'Last round: no trump'}
        </div>
      </div>
    </div>
  );
}

function BidChip({ st, s, viewer }) {
  const bid = st.bids[s];
  const won = st.won[s];
  if (bid == null) {
    if (st.hasBid[s]) return <span className="bidchip" title="Bid made (hidden until everyone has bid)">bid hidden</span>;
    return <span className="bidchip opacity-60">no bid</span>;
  }
  const playing = st.phase.kind === 'play' || st.roundOver;
  const cls = playing ? (won > bid ? 'over' : won === bid ? 'made' : '') : '';
  return <span className={`bidchip ${cls}`} title={`bid ${bid}, won ${won} so far`}>won {won}/{bid}</span>;
}

/** One seat: who, bid/tricks, score, chance to win, and (when visible) the hand. */
export function Seat({ st, s, names, personas, showHand, cardW = 34, turn, isYou, chances = true }) {
  const p = personas[s];
  const hand = st.hands[s];
  return (
    <div className={`seatbox ${turn ? 'turn' : ''}`}>
      <div className="flex items-center gap-2.5 min-w-0">
        <span className="avatar">{p.icon}</span>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-1.5 min-w-0">
            <span className="text-sm font-semibold text-white truncate">{names[s]}</span>
            {st.dealer === s && <span className="tag shrink-0" title="Dealer">D</span>}
          </div>
          <div className="text-[11px] text-ink-300 truncate">{isYou ? 'you' : p.label || p.name}</div>
        </div>
      </div>
      <div className="flex items-center justify-between gap-2 mt-2">
        <BidChip st={st} s={s} />
        <span className="text-[11px] text-ink-300 num whitespace-nowrap">
          <b className="text-sm text-white">{st.totals[s]}</b>{chances && <span title="Chance to win the game"> · {pct(st.chances[s])}<span className="hidden sm:inline"> win</span></span>}{!showHand && <span className="hidden sm:inline"> · {st.handCounts[s]} card{st.handCounts[s] === 1 ? '' : 's'}</span>}
        </span>
      </div>
      {showHand && hand && hand.length > 0 && (
        <div className="hand mt-2" style={{ gap: 3 }}>
          {sortHand(hand, st.trump).map((c) => <Card key={c} c={c} w={cardW} trump={st.trump} />)}
        </div>
      )}
    </div>
  );
}

/** The cards in the middle. `shown`: { plays, winner? } (the current trick, or the one just taken). */
export function Trick({ st, names, shown, cardW = 58 }) {
  const plays = shown?.plays || [];
  const lead = winningIndex(plays, st.trump);
  const done = shown?.winner != null;
  return (
    <div className="flex flex-col items-center gap-2 py-2">
      <div className="flex flex-wrap justify-center gap-3 min-h-[calc(var(--tw)*1.42+18px)]" style={{ '--tw': `${cardW}px` }}>
        {plays.length === 0 && <div className="text-sm text-ink-300 self-center px-4 text-center">{st.phase.kind === 'play' ? (names[st.phase.seat] === 'You' ? 'You lead' : `${names[st.phase.seat]} leads`) : ''}</div>}
        {plays.map(([s, c], i) => (
          <div key={`${s}-${c}`} className="slot deal">
            <Card c={c} w={cardW} trump={st.trump} win={i === lead} />
            <span className="who">{names[s]}</span>
          </div>
        ))}
      </div>
      {done && plays.length > 0 && (
        <div className="text-xs font-semibold text-violet-200 fade-up">{names[shown.winner] === 'You' ? 'You take' : `${names[shown.winner]} takes`} it with <CardText c={plays[lead][1]} /></div>
      )}
    </div>
  );
}

/** Your hand: tap a legal card to play it. */
export function YourHand({ st, seat, onPlay, hint, cardW = 66 }) {
  const hand = sortHand(st.hands[seat] || [], st.trump);
  const myTurn = st.phase.kind === 'play' && st.phase.seat === seat;
  const legal = new Set(st.legal.filter((k) => k >= 25).map(codeCard));
  return (
    <div className="hand justify-center" style={{ gap: 6 }}>
      {hand.map((c) => {
        const ok = myTurn && legal.has(c);
        return <Card key={c} c={c} w={cardW} trump={st.trump} dim={myTurn && !ok} hint={hint === c}
          onClick={ok ? () => onPlay(cardCode(c)) : undefined} />;
      })}
    </div>
  );
}

/** Scores, this round's bids and each player's chance to win the game. */
export function Scoreboard({ st, names, personas }) {
  const order = [...Array(st.players).keys()].sort((a, b) => st.totals[b] - st.totals[a]);
  const lead = Math.max(...st.totals);
  return (
    <div className="space-y-1.5">
      {order.map((s) => {
        const ch = st.chances[s];
        const sw = st.swing?.[s] || 0;
        return (
          <div key={s} className="flex items-center gap-2.5">
            <span className="w-5 text-center">{personas[s].icon}</span>
            <div className="min-w-0 flex-1">
              <div className="flex items-center justify-between gap-2 text-sm">
                <span className={`truncate ${st.totals[s] === lead ? 'font-semibold text-white' : 'text-ink-100'}`}>{names[s]}</span>
                <span className="num font-semibold text-white">{st.totals[s]}</span>
              </div>
              <div className="flex items-center gap-2 mt-1">
                <div className="bar flex-1"><i style={{ width: `${Math.max(2, ch * 100)}%`, background: 'rgb(var(--heat))' }} /></div>
                <span className="text-[11px] text-ink-300 num w-[76px] text-right">{pct(ch)}{Math.abs(sw) >= 0.005 ? <span className={sw > 0 ? 'text-emerald-300' : 'text-rose-300'}> {sw > 0 ? '+' : '−'}{Math.round(Math.abs(sw) * 100)}</span> : ''}</span>
              </div>
            </div>
          </div>
        );
      })}
      <p className="text-[11px] text-ink-400 pt-1">Bars: each player's chance to win the game, from the scores and rounds left (and how much the last round moved it).</p>
    </div>
  );
}

/** The table of past rounds. */
export function History({ st, names }) {
  if (!st.past.length) return <p className="text-sm text-ink-300">No rounds finished yet.</p>;
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-sm num">
        <thead>
          <tr className="text-ink-300 text-xs">
            <th className="text-left font-semibold py-1 pr-2">Round</th>
            {names.map((n, s) => <th key={s} className="text-right font-semibold py-1 px-1.5 whitespace-nowrap">{n}</th>)}
          </tr>
        </thead>
        <tbody>
          {st.past.map((r) => (
            <tr key={r.round} className="border-t border-ink-800">
              <td className="py-1.5 pr-2 text-ink-300">{r.round}</td>
              {r.scores.map((x, s) => (
                <td key={s} className="text-right py-1.5 px-1.5">
                  <span className={r.bids[s] === r.won[s] ? 'text-emerald-300' : 'text-rose-300'}>{x > 0 ? '+' : ''}{x}</span>
                  <span className="text-[11px] text-ink-400"> {r.won[s]}/{r.bids[s]}</span>
                  <div className="text-[11px] text-ink-300">{r.totals[s]}</div>
                </td>
              ))}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}
