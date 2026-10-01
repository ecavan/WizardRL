/** A playing card: standard cards, Wizards, Jesters and the card back. */
import { isJester, isWizard, rankOf, suitOf, RANKS, SUIT_SYM, SUITS, cardName } from '../lib/cards.js';
import { usePrefs } from '../lib/prefs.js';

export function Card({ c, w = 64, back = false, dim, hint, win, onClick, trump, className = '', title, style }) {
  const [prefs] = usePrefs();
  const css = { '--w': `${w}px`, ...style };
  const tap = !!onClick;
  const base = `pc ${dim ? 'dim' : ''} ${tap ? 'tap' : ''} ${hint ? 'hint' : ''} ${win ? 'win' : ''} ${className}`;
  const Tag = tap ? 'button' : 'div';
  if (back) return <div className={`${base} back`} style={css} aria-label="face-down card" />;
  if (isWizard(c)) {
    return (
      <Tag type={tap ? 'button' : undefined} className={`${base} wiz`} style={css} onClick={onClick} title={title || 'Wizard'} aria-label="Wizard">
        <span className="big"><i>✦</i><b>W</b><span>Wizard</span></span>
      </Tag>
    );
  }
  if (isJester(c)) {
    return (
      <Tag type={tap ? 'button' : undefined} className={`${base} jes`} style={css} onClick={onClick} title={title || 'Jester'} aria-label="Jester">
        <span className="big"><i>☾</i><b>J</b><span>Jester</span></span>
      </Tag>
    );
  }
  const s = suitOf(c), r = rankOf(c);
  const red = s === 1 || s === 2;
  const isTrump = trump != null && s === trump;
  return (
    <Tag type={tap ? 'button' : undefined}
      className={`${base} ${red ? 'red' : ''} ${prefs.fourColor ? 'c4' : ''} s-${SUITS[s]} ${isTrump ? 'trumpmark' : ''}`}
      style={css} onClick={onClick} title={title || cardName(c) + (isTrump ? ' (trump)' : '')} aria-label={cardName(c) + (isTrump ? ', trump' : '')}>
      <span className="ix"><b>{RANKS[r]}</b><i>{SUIT_SYM[s]}</i></span>
      <span className="pip">{SUIT_SYM[s]}</span>
    </Tag>
  );
}

/** A suit symbol in its colour (for text). */
export function SuitSym({ s, className = '' }) {
  const [prefs] = usePrefs();
  const color = prefs.fourColor ? ['#16a34a', '#2563eb', '#dc2626', 'currentColor'][s] : s === 1 || s === 2 ? '#dc2626' : 'currentColor';
  return <span className={className} style={{ color }}>{SUIT_SYM[s]}</span>;
}

/** Card name with a coloured suit (for text). */
export function CardText({ c }) {
  if (isWizard(c)) return <b className="text-violet-300">Wizard</b>;
  if (isJester(c)) return <b className="text-ink-200">Jester</b>;
  return <b>{RANKS[rankOf(c)]}<SuitSym s={suitOf(c)} /></b>;
}
