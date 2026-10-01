/**
 * Cards and moves as the engine numbers them.
 * Card c: 0..51 = suit * 13 + rank (suits clubs, diamonds, hearts, spades; ranks 2..A),
 * 52..55 Wizards, 56..59 Jesters. A move code: 0..3 name trump, 4 + b bid b, 25 + c play card c.
 */
export const SUITS = ['c', 'd', 'h', 's'];
export const SUIT_SYM = ['♣', '♦', '♥', '♠'];
export const SUIT_NAME = ['clubs', 'diamonds', 'hearts', 'spades'];
export const RANKS = ['2', '3', '4', '5', '6', '7', '8', '9', '10', 'J', 'Q', 'K', 'A'];

export const isWizard = (c) => c >= 52 && c < 56;
export const isJester = (c) => c >= 56;
export const suitOf = (c) => (c < 52 ? Math.floor(c / 13) : null);
export const rankOf = (c) => (c < 52 ? c % 13 : null);
export const isRed = (c) => c < 52 && (suitOf(c) === 1 || suitOf(c) === 2);

export function cardName(c) {
  if (isWizard(c)) return 'Wizard';
  if (isJester(c)) return 'Jester';
  return RANKS[rankOf(c)] + SUIT_SYM[suitOf(c)];
}

export const ACT_BID = 4;
export const ACT_CARD = 25;
export const codeKind = (k) => (k < ACT_BID ? 'trump' : k < ACT_CARD ? 'bid' : 'card');
export const codeBid = (k) => k - ACT_BID;
export const codeCard = (k) => k - ACT_CARD;
export const bidCode = (b) => ACT_BID + b;
export const cardCode = (c) => ACT_CARD + c;

export function moveName(k) {
  const kind = codeKind(k);
  if (kind === 'trump') return `${SUIT_NAME[k]} as trump`;
  if (kind === 'bid') return `bids ${codeBid(k)}`;
  return cardName(codeCard(k));
}

/** Sort a hand for display: Wizards, then suits (trump first) high to low, then Jesters. */
export function sortHand(cards, trump) {
  const key = (c) => {
    if (isWizard(c)) return [0, 0, 0];
    if (isJester(c)) return [3, 0, 0];
    const s = suitOf(c);
    const order = trump == null ? s : s === trump ? -1 : s;
    return [1, order, -rankOf(c)];
  };
  return [...cards].sort((a, b) => { const x = key(a), y = key(b); return x[0] - y[0] || x[1] - y[1] || x[2] - y[2]; });
}

/** Which card is winning the trick so far (index into plays), as the engine decides it. */
export function winningIndex(plays, trump) {
  if (!plays.length) return -1;
  const wiz = plays.findIndex(([, c]) => isWizard(c));
  if (wiz >= 0) return wiz;
  const firstStd = plays.findIndex(([, c]) => !isJester(c));
  if (firstStd < 0) return 0; // only Jesters: the first one
  const led = suitOf(plays[firstStd][1]);
  let best = firstStd;
  for (let i = firstStd + 1; i < plays.length; i++) {
    const c = plays[i][1], b = plays[best][1];
    if (isJester(c)) continue;
    const cs = suitOf(c), bs = suitOf(b);
    if (cs === bs ? rankOf(c) > rankOf(b) : cs === trump && bs !== trump) best = i;
  }
  return best;
}
