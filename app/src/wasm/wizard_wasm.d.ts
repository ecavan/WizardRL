/* tslint:disable */
/* eslint-disable */
/**
 * Load the win-probability model (`winprob.wzwp`), for the "chance to win" readouts.
 */
export function load_winprob(bytes: Uint8Array): void;
export function brain_loaded(): boolean;
/**
 * Load the bot network (a `.wznet`, plain or two-brain). Call once before making games.
 */
export function load_brain(bytes: Uint8Array): void;
/**
 * A full game: rounds of 1, 2, 3 ... cards, the deal moving left, scores adding up.
 */
export class WizardGame {
  free(): void;
  /**
   * Deal the next round (after the UI has shown the last one).
   */
  next_round(): void;
  round_over(): boolean;
  is_bot_turn(): boolean;
  /**
   * A human (or a replay) plays `code` for the seat to act.
   */
  act(code: number): string;
  /**
   * Every move so far (-1 = next round), for saving a game and replaying it with `replay`.
   */
  log(): string;
  /**
   * `seats`: a JSON list, one per seat, of `{"human": true}` or
   * `{"level": "club", "style": "overbid"}` (style optional).
   */
  constructor(players: number, simultaneous: boolean, seats: string, seed: number);
  /**
   * The table as `viewer` sees it (-1 = everything, for watching bots).
   */
  state(viewer: number): string;
  /**
   * What `seat` should do now in the network's eyes: every option with its expected points
   * and chance of making the bid, best first, and the move the best bot would play.
   */
  advise(seat: number): string;
  /**
   * Replay a saved log on a fresh game made with the same settings and seed.
   */
  replay(log: string): void;
  /**
   * Seat to act, or -1 (between rounds, or the game is over).
   */
  to_act(): number;
  /**
   * The bot whose turn it is moves. Returns the event (see `apply`).
   */
  bot_step(): string;
  game_over(): boolean;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_wizardgame_free: (a: number, b: number) => void;
  readonly brain_loaded: () => number;
  readonly load_brain: (a: number, b: number) => [number, number];
  readonly load_winprob: (a: number, b: number) => [number, number];
  readonly wizardgame_act: (a: number, b: number) => [number, number, number, number];
  readonly wizardgame_advise: (a: number, b: number) => [number, number, number, number];
  readonly wizardgame_bot_step: (a: number) => [number, number, number, number];
  readonly wizardgame_game_over: (a: number) => number;
  readonly wizardgame_is_bot_turn: (a: number) => number;
  readonly wizardgame_log: (a: number) => [number, number];
  readonly wizardgame_new: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
  readonly wizardgame_next_round: (a: number) => [number, number];
  readonly wizardgame_replay: (a: number, b: number, c: number) => [number, number];
  readonly wizardgame_round_over: (a: number) => number;
  readonly wizardgame_state: (a: number, b: number) => [number, number];
  readonly wizardgame_to_act: (a: number) => number;
  readonly __wbindgen_export_0: WebAssembly.Table;
  readonly __externref_table_dealloc: (a: number) => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
