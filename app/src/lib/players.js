/**
 * Who can sit at the table. Levels come from the strength curve in the README: each is the
 * trained bot picking its moves with more or less noise (softmax over expected points), from
 * the counting bot up to ppo5, the best bot. Characters are a level with a habit.
 */
export const LEVELS = [
  { id: 'beginner', label: 'Beginner', name: 'Rookie Rae', icon: '🐣', blurb: 'Adds up rough card values and plays greedily. Easy to beat.' },
  { id: 'casual', label: 'Casual', name: 'Casual Cal', icon: '🙂', blurb: 'Knows the game, but picks loosely: plenty of slips.' },
  { id: 'club', label: 'Club', name: 'Club Clara', icon: '🃏', blurb: 'A solid family-table player. Gives away a point or so a move.' },
  { id: 'strong', label: 'Strong', name: 'Sharp Sam', icon: '🎯', blurb: 'Usually finds the best move; the odd small slip.' },
  { id: 'expert', label: 'Expert', name: 'Expert Eve', icon: '🦉', blurb: 'Almost always plays the best move.' },
  { id: 'master', label: 'Master', name: 'The Wizard', icon: '🧙', blurb: 'ppo5, the strongest bot we trained. Reads habits and exploits them.' },
];

export const CHARACTERS = [
  { id: 'overbid', style: 'overbid', level: 'strong', label: 'Overbids', name: 'Optimistic Olly', icon: '📈', blurb: 'Bids one more than his cards are worth, 70% of the time.' },
  { id: 'underbid', style: 'underbid', level: 'strong', label: 'Underbids', name: 'Careful Cleo', icon: '🐢', blurb: 'Bids one fewer than her cards are worth, 70% of the time.' },
  { id: 'early-wizard', style: 'early-wizard', level: 'strong', label: 'Early Wizards', name: 'Wizard-First Wendy', icon: '⚡', blurb: 'Throws her Wizards on the first trick of a round.' },
  { id: 'wild', style: 'wild', level: 'strong', label: 'Wild', name: 'Wild Wes', icon: '🌪️', blurb: 'One move in five is anything at all.' },
];

export const ALL = [...LEVELS.map((l) => ({ ...l, kind: 'level', level: l.id, style: null })), ...CHARACTERS.map((c) => ({ ...c, kind: 'character' }))];
export const byId = (id) => ALL.find((p) => p.id === id) || ALL[2];

/** Table presets for "random" seats. */
export const TABLES = [
  { id: 'easy', label: 'Easy', pool: ['beginner', 'casual', 'club'] },
  { id: 'mixed', label: 'Mixed', pool: ['casual', 'club', 'strong', 'overbid', 'underbid', 'early-wizard', 'wild'] },
  { id: 'hard', label: 'Hard', pool: ['strong', 'expert', 'master'] },
  { id: 'any', label: 'Anyone', pool: ALL.map((p) => p.id) },
];

export function pickRandom(tableId, rnd = Math.random) {
  const pool = (TABLES.find((t) => t.id === tableId) || TABLES[1]).pool;
  return pool[Math.floor(rnd() * pool.length)];
}

/** Display names for a table of persona ids (numbered when one appears twice). */
export function seatNames(ids) {
  const seen = {};
  return ids.map((id) => {
    if (id === 'you') return 'You';
    const p = byId(id);
    seen[id] = (seen[id] || 0) + 1;
    const total = ids.filter((x) => x === id).length;
    return total > 1 ? `${p.name} ${seen[id]}` : p.name;
  });
}

export const seatSpec = (id) => (id === 'you' ? { human: true } : { level: byId(id).level, style: byId(id).style || undefined });
