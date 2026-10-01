//! The Wizard engine and the trained bots in the browser (the app in `app/`).
//!
//! JavaScript loads the networks once (`load_brain` with `ppo5.wznet`, `load_winprob` with
//! `winprob.wzwp`), then drives a `WizardGame`: a full game of rounds 1, 2, 3 ... with any mix
//! of human and bot seats. Everything crosses the boundary as JSON strings and action codes
//! (`wizard::encode::action_index`: trump 0..3, bid 4 + b, card 25 + card).
//!
//! Bot levels, weakest to strongest (from the strength curve in the README):
//!
//! | level      | how it picks                                                   |
//! | ---------- | -------------------------------------------------------------- |
//! | `beginner` | the counting bot (adds up rough card values, plays greedily)    |
//! | `casual`   | the network's options, sampled with softmax(points / 10)        |
//! | `club`     | ... softmax(points / 4)                                         |
//! | `strong`   | ... softmax(points / 2)                                         |
//! | `expert`   | ... softmax(points / 1)                                         |
//! | `master`   | ppo5: the policy's most likely move                             |
//!
//! Any level can carry a habit (`overbid`, `underbid`, `early-wizard`, `wild`; see
//! `wizard::style`).

use std::cell::RefCell;
use std::rc::Rc;

use serde_json::{json, Value};
use wasm_bindgen::prelude::*;
use wizard::bots::{Bot, CountingBot};
use wizard::card::{cards, Card};
use wizard::encode::{self, ACTIONS, FEATURES};
use wizard::net::Mlp;
use wizard::rng::Rng;
use wizard::round::{Action, Event, Phase, Round, TrumpSource};
use wizard::rules::Rules;
use wizard::style::Style;
use wizard::view::{SeatHistory, View};
use wizard::winprob::WinProb;

/// The networks: a policy that chooses (ppo5) and a points network that scores (its evaluator).
pub struct Brain {
    policy: Option<Mlp>,
    eval: Mlp,
}

thread_local! {
    static BRAIN: RefCell<Option<Rc<Brain>>> = const { RefCell::new(None) };
    static WINPROB: RefCell<Option<Rc<WinProb>>> = const { RefCell::new(None) };
}

fn brain() -> Option<Rc<Brain>> {
    BRAIN.with(|b| b.borrow().clone())
}

fn winprob() -> Option<Rc<WinProb>> {
    WINPROB.with(|w| w.borrow().clone())
}

/// Load the bot network (a `.wznet`, plain or two-brain). Call once before making games.
#[wasm_bindgen]
pub fn load_brain(bytes: &[u8]) -> Result<(), JsValue> {
    let (policy, eval) = Mlp::pair_from_bytes(bytes).map_err(|e| JsValue::from_str(&e))?;
    BRAIN.with(|b| *b.borrow_mut() = Some(Rc::new(Brain { policy, eval })));
    Ok(())
}

/// Load the win-probability model (`winprob.wzwp`), for the "chance to win" readouts.
#[wasm_bindgen]
pub fn load_winprob(bytes: &[u8]) -> Result<(), JsValue> {
    let wp = WinProb::from_bytes(bytes).map_err(|e| JsValue::from_str(&e))?;
    WINPROB.with(|w| *w.borrow_mut() = Some(Rc::new(wp)));
    Ok(())
}

#[wasm_bindgen]
pub fn brain_loaded() -> bool {
    brain().is_some()
}

// ------------------------------------------------------------------------------------- bots

#[derive(Clone, Copy, Debug, PartialEq)]
enum Level {
    Beginner,
    Casual,
    Club,
    Strong,
    Expert,
    Master,
}

impl Level {
    fn parse(s: &str) -> Option<Level> {
        Some(match s {
            "beginner" => Level::Beginner,
            "casual" => Level::Casual,
            "club" => Level::Club,
            "strong" => Level::Strong,
            "expert" => Level::Expert,
            "master" => Level::Master,
            _ => return None,
        })
    }
    /// Softmax temperature in points, for the sampling levels.
    fn temp(self) -> Option<f32> {
        match self {
            Level::Casual => Some(10.0),
            Level::Club => Some(4.0),
            Level::Strong => Some(2.0),
            Level::Expert => Some(1.0),
            _ => None,
        }
    }
}

/// The network's view of every legal option: (action, points, chance of making the bid),
/// best first, and the move the bot would play.
struct Options {
    rows: Vec<(Action, f32, Option<f32>)>,
    pick: Action,
}

fn options(brain: &Brain, v: &View, obs: &mut [f32], mask: &mut [bool]) -> Options {
    encode::observe(v, obs);
    encode::legal_mask(v, mask);
    let q = brain.eval.forward(obs);
    let head = brain.eval.make_head;
    let mut rows: Vec<(Action, f32, Option<f32>)> = (0..ACTIONS)
        .filter(|&i| mask[i])
        .map(|i| {
            let p = head.then(|| 1.0 / (1.0 + (-q[ACTIONS + i]).exp()));
            (
                encode::action_from_index(i).unwrap(),
                q[i] * brain.eval.scale,
                p,
            )
        })
        .collect();
    rows.sort_by(|a, b| b.1.total_cmp(&a.1));
    let pick = match &brain.policy {
        Some(pol) => {
            let logits = pol.forward(obs);
            let best = (0..ACTIONS)
                .filter(|&i| mask[i])
                .max_by(|&a, &b| logits[a].total_cmp(&logits[b]))
                .expect("a legal action");
            encode::action_from_index(best).unwrap()
        }
        None => rows[0].0,
    };
    Options { rows, pick }
}

struct SeatBot {
    level: Level,
    style: Option<Style>,
    obs: Vec<f32>,
    mask: Vec<bool>,
}

impl SeatBot {
    fn new(level: Level, style: Option<Style>) -> SeatBot {
        SeatBot {
            level,
            style,
            obs: vec![0.0; FEATURES],
            mask: vec![false; ACTIONS],
        }
    }

    fn base(&mut self, v: &View, rng: &mut Rng) -> Action {
        let b = match (self.level, brain()) {
            (Level::Beginner, _) | (_, None) => return CountingBot.act(v, rng),
            (_, Some(b)) => b,
        };
        let opts = options(&b, v, &mut self.obs, &mut self.mask);
        match self.level.temp() {
            None => opts.pick,
            Some(t) => {
                // softmax(points / t): usually the best move or a near-tie, now and then a slip
                let top = opts.rows[0].1;
                let w: Vec<f64> = opts
                    .rows
                    .iter()
                    .map(|r| (((r.1 - top) / t) as f64).exp())
                    .collect();
                let mut x = rng.unit() * w.iter().sum::<f64>();
                for (r, wi) in opts.rows.iter().zip(&w) {
                    x -= wi;
                    if x <= 0.0 {
                        return r.0;
                    }
                }
                opts.rows[0].0
            }
        }
    }
}

impl Bot for SeatBot {
    fn name(&self) -> String {
        format!("{:?}", self.level)
    }
    fn act(&mut self, v: &View, rng: &mut Rng) -> Action {
        let a = self.base(v, rng);
        match (self.style, v.phase()) {
            (Some(s), Phase::Bid { .. } | Phase::Play { .. }) => s.apply(a, v, rng),
            _ => a,
        }
    }
}

// ------------------------------------------------------------------------------------- game

fn code(a: Action) -> i32 {
    encode::action_index(a) as i32
}

fn card_json(c: Card) -> Value {
    json!(c)
}

fn phase_json(p: Phase) -> Value {
    match p {
        Phase::PickTrump { seat } => json!({"kind": "trump", "seat": seat}),
        Phase::Bid { seat } => json!({"kind": "bid", "seat": seat}),
        Phase::Play { seat } => json!({"kind": "play", "seat": seat}),
        Phase::Done => json!({"kind": "done"}),
    }
}

/// A full game: rounds of 1, 2, 3 ... cards, the deal moving left, scores adding up.
#[wasm_bindgen]
pub struct WizardGame {
    rules: Rules,
    n: u8,
    seats: Vec<Option<SeatBot>>,
    rng: Rng,
    deal_rng: Rng,
    round: Round,
    dealer: u8,
    totals: Vec<i32>,
    history: Vec<SeatHistory>,
    past: Vec<Value>,
    over: bool,
    log: Vec<i32>,
    chances: Vec<f64>,
    swing: Vec<f64>,
}

#[wasm_bindgen]
impl WizardGame {
    /// `seats`: a JSON list, one per seat, of `{"human": true}` or
    /// `{"level": "club", "style": "overbid"}` (style optional).
    #[wasm_bindgen(constructor)]
    pub fn new(
        players: u8,
        simultaneous: bool,
        seats: &str,
        seed: f64,
    ) -> Result<WizardGame, JsValue> {
        let rules = if simultaneous {
            Rules::simultaneous(players)
        } else {
            Rules::official(players)
        };
        rules.validate().map_err(|e| JsValue::from_str(&e))?;
        let spec: Vec<Value> =
            serde_json::from_str(seats).map_err(|e| JsValue::from_str(&e.to_string()))?;
        if spec.len() != players as usize {
            return Err(JsValue::from_str("one seat spec per player"));
        }
        let mut list = Vec::new();
        for s in &spec {
            if s.get("human").and_then(Value::as_bool) == Some(true) {
                list.push(None);
                continue;
            }
            let level = s
                .get("level")
                .and_then(Value::as_str)
                .and_then(Level::parse)
                .ok_or_else(|| JsValue::from_str("each bot needs a level"))?;
            let style = s.get("style").and_then(Value::as_str).and_then(Style::parse);
            list.push(Some(SeatBot::new(level, style)));
        }
        let seed = seed as u64;
        let mut deal_rng = Rng::new(seed ^ 0x9E37_79B9_7F4A_7C15);
        let dealer = deal_rng.below(players as u64) as u8;
        let round = Round::deal(rules, 1, dealer, &mut deal_rng);
        let n = players as usize;
        Ok(WizardGame {
            rules,
            n: players,
            seats: list,
            rng: Rng::new(seed),
            deal_rng,
            round,
            dealer,
            totals: vec![0; n],
            history: vec![SeatHistory::default(); n],
            past: Vec::new(),
            over: false,
            log: Vec::new(),
            chances: vec![1.0 / n as f64; n],
            swing: vec![0.0; n],
        })
    }

    /// Seat to act, or -1 (between rounds, or the game is over).
    pub fn to_act(&self) -> i32 {
        self.round.to_act().map(|s| s as i32).unwrap_or(-1)
    }

    pub fn is_bot_turn(&self) -> bool {
        match self.round.to_act() {
            Some(s) => self.seats[s as usize].is_some(),
            None => false,
        }
    }

    pub fn round_over(&self) -> bool {
        self.round.is_done()
    }

    pub fn game_over(&self) -> bool {
        self.over
    }

    /// The bot whose turn it is moves. Returns the event (see `apply`).
    pub fn bot_step(&mut self) -> Result<String, JsValue> {
        let seat = self
            .round
            .to_act()
            .ok_or_else(|| JsValue::from_str("nobody to act"))?;
        let v = View::with_history(&self.round, seat, &self.totals, &self.history);
        let bot = self.seats[seat as usize]
            .as_mut()
            .ok_or_else(|| JsValue::from_str("it's a human's turn"))?;
        let a = bot.act(&v, &mut self.rng);
        self.apply(seat, a)
    }

    /// A human (or a replay) plays `code` for the seat to act.
    pub fn act(&mut self, code: i32) -> Result<String, JsValue> {
        let seat = self
            .round
            .to_act()
            .ok_or_else(|| JsValue::from_str("nobody to act"))?;
        let a = encode::action_from_index(code as usize)
            .filter(|&a| self.round.is_legal(a))
            .ok_or_else(|| JsValue::from_str("illegal move"))?;
        self.apply(seat, a)
    }

    /// Deal the next round (after the UI has shown the last one).
    pub fn next_round(&mut self) -> Result<(), JsValue> {
        if !self.round.is_done() || self.over {
            return Err(JsValue::from_str("the round isn't over, or the game is"));
        }
        self.dealer = (self.dealer + 1) % self.n;
        let size = self.round.size() + 1;
        self.round = Round::deal(self.rules, size, self.dealer, &mut self.deal_rng);
        self.log.push(-1);
        Ok(())
    }

    /// Every move so far (-1 = next round), for saving a game and replaying it with `replay`.
    pub fn log(&self) -> String {
        serde_json::to_string(&self.log).unwrap()
    }

    /// Replay a saved log on a fresh game made with the same settings and seed.
    pub fn replay(&mut self, log: &str) -> Result<(), JsValue> {
        let codes: Vec<i32> =
            serde_json::from_str(log).map_err(|e| JsValue::from_str(&e.to_string()))?;
        for c in codes {
            if c < 0 {
                self.next_round()?;
            } else {
                self.act(c)?;
            }
        }
        Ok(())
    }

    /// What `seat` should do now in the network's eyes: every option with its expected points
    /// and chance of making the bid, best first, and the move the best bot would play.
    pub fn advise(&mut self, seat: u8) -> Result<String, JsValue> {
        let b = brain().ok_or_else(|| JsValue::from_str("the network isn't loaded"))?;
        if self.round.to_act() != Some(seat) {
            return Err(JsValue::from_str("not that seat's turn"));
        }
        let v = View::with_history(&self.round, seat, &self.totals, &self.history);
        let mut obs = vec![0.0; FEATURES];
        let mut mask = vec![false; ACTIONS];
        let o = options(&b, &v, &mut obs, &mut mask);
        let rows: Vec<Value> = o
            .rows
            .iter()
            .map(|(a, x, p)| json!({"code": code(*a), "label": a.to_string(), "points": x, "make": p}))
            .collect();
        Ok(json!({"options": rows, "pick": code(o.pick)}).to_string())
    }

    /// The table as `viewer` sees it (-1 = everything, for watching bots).
    pub fn state(&self, viewer: i32) -> String {
        let r = &self.round;
        let n = self.n as usize;
        let phase = r.phase();
        let bidding = matches!(phase, Phase::Bid { .. } | Phase::PickTrump { .. });
        let hide_bids = viewer >= 0 && self.rules.simultaneous_bids && bidding;
        let hands: Vec<Value> = (0..n)
            .map(|s| {
                if viewer < 0 || viewer as usize == s || r.is_done() {
                    json!(cards(r.hand(s as u8)).map(card_json).collect::<Vec<_>>())
                } else {
                    Value::Null
                }
            })
            .collect();
        let counts: Vec<u32> = (0..n).map(|s| r.hand(s as u8).count_ones()).collect();
        let bids: Vec<Value> = (0..n)
            .map(|s| match r.bid(s as u8) {
                Some(b) if !hide_bids || viewer as usize == s => json!(b),
                _ => Value::Null,
            })
            .collect();
        let has_bid: Vec<bool> = (0..n).map(|s| r.bid(s as u8).is_some()).collect();
        let won: Vec<u8> = (0..n).map(|s| r.tricks_won(s as u8)).collect();
        let (src_kind, src_card) = match r.trump_source() {
            TrumpSource::TurnedCard(c) => ("card", json!(c)),
            TrumpSource::WizardTurned(c) => ("wizard", json!(c)),
            TrumpSource::JesterTurned(c) => ("jester", json!(c)),
            TrumpSource::NoCardTurned => ("none", Value::Null),
        };
        let to_act = r.to_act();
        let legal: Vec<i32> = match to_act {
            Some(s) if viewer < 0 || viewer == s as i32 => {
                r.legal_actions().into_iter().map(code).collect()
            }
            _ => Vec::new(),
        };
        let last = r
            .completed_tricks()
            .last()
            .map(|t| json!({"plays": t.plays, "winner": t.winner}));
        let seats: Vec<Value> = self
            .seats
            .iter()
            .map(|s| match s {
                None => json!({"human": true}),
                Some(b) => json!({"level": format!("{:?}", b.level).to_lowercase(),
                                  "style": b.style.map(|s| s.name())}),
            })
            .collect();
        json!({
            "players": n,
            "simultaneous": self.rules.simultaneous_bids,
            "rounds": self.rules.rounds(),
            "round": r.size(),
            "dealer": r.dealer(),
            "phase": phase_json(phase),
            "trump": r.trump().map(|s| s.index()),
            "trumpSource": {"kind": src_kind, "card": src_card},
            "hands": hands,
            "handCounts": counts,
            "bids": bids,
            "hasBid": has_bid,
            "won": won,
            "trick": r.current_trick(),
            "lastTrick": last,
            "tricksDone": r.completed_tricks().len(),
            "totals": self.totals,
            "roundScores": r.scores(),
            "roundOver": r.is_done(),
            "gameOver": self.over,
            "legal": legal,
            "seats": seats,
            "chances": self.chances,
            "swing": self.swing,
            "past": self.past,
        })
        .to_string()
    }
}

impl WizardGame {
    /// Apply `a` for `seat`. The event: `{"seat", "code", "label", "trick": {plays, winner}?,
    /// "roundOver", "gameOver"}`.
    fn apply(&mut self, seat: u8, a: Action) -> Result<String, JsValue> {
        let ev = self
            .round
            .apply(a)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.log.push(code(a));
        let mut out = json!({"seat": seat, "code": code(a), "label": a.to_string(),
                             "roundOver": false, "gameOver": false});
        if matches!(ev, Event::TrickWon { .. } | Event::RoundOver) {
            let t = self.round.completed_tricks().last().unwrap();
            out["trick"] = json!({"plays": t.plays, "winner": t.winner});
        }
        if ev == Event::RoundOver {
            let scores = self.round.scores().unwrap();
            let n = self.n as usize;
            for s in 0..n {
                self.totals[s] += scores[s];
                self.history[s].record(&self.round, s as u8);
            }
            let size = self.round.size();
            let left = (self.rules.rounds() - size) as u32;
            if let Some(wp) = winprob() {
                let now = wp.chances(&self.totals, left);
                self.swing = now.iter().zip(&self.chances).map(|(a, b)| a - b).collect();
                self.chances = now;
            } else if left == 0 {
                let best = *self.totals.iter().max().unwrap();
                let k = self.totals.iter().filter(|&&t| t == best).count() as f64;
                self.chances = self
                    .totals
                    .iter()
                    .map(|&t| if t == best { 1.0 / k } else { 0.0 })
                    .collect();
            }
            self.past.push(json!({
                "round": size,
                "dealer": self.round.dealer(),
                "bids": (0..n).map(|s| self.round.bid(s as u8).unwrap()).collect::<Vec<_>>(),
                "won": (0..n).map(|s| self.round.tricks_won(s as u8)).collect::<Vec<_>>(),
                "scores": scores,
                "totals": self.totals,
            }));
            self.over = size == self.rules.rounds();
            out["roundOver"] = json!(true);
            out["gameOver"] = json!(self.over);
        }
        Ok(out.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bots(n: usize, level: &str) -> String {
        serde_json::to_string(&vec![json!({"level": level}); n]).unwrap()
    }

    #[test]
    fn bots_finish_a_game_and_replay_matches() {
        for n in 3..=6u8 {
            let mut g = WizardGame::new(n, true, &bots(n as usize, "beginner"), 7.0).unwrap();
            let mut steps = 0;
            while !g.game_over() {
                if g.round_over() {
                    g.next_round().unwrap();
                } else {
                    g.bot_step().unwrap();
                }
                steps += 1;
                assert!(steps < 100_000);
            }
            let st: Value = serde_json::from_str(&g.state(-1)).unwrap();
            assert_eq!(st["past"].as_array().unwrap().len(), 60 / n as usize);
            let mut h = WizardGame::new(n, true, &bots(n as usize, "beginner"), 7.0).unwrap();
            h.replay(&g.log()).unwrap();
            assert_eq!(h.state(-1), g.state(-1));
        }
    }

    #[test]
    fn network_levels_play_and_advise() {
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../rl/models/");
        let (Ok(net), Ok(wp)) = (
            std::fs::read(format!("{dir}ppo5.wznet")),
            std::fs::read(format!("{dir}winprob.wzwp")),
        ) else {
            return; // models not present
        };
        load_brain(&net).unwrap();
        load_winprob(&wp).unwrap();
        let seats = json!([{"level": "master"}, {"level": "casual"}, {"level": "strong", "style": "wild"},
                           {"level": "expert", "style": "early-wizard"}]);
        let mut g = WizardGame::new(4, true, &seats.to_string(), 11.0).unwrap();
        let t0 = std::time::Instant::now();
        while !g.game_over() {
            if g.round_over() {
                g.next_round().unwrap();
            } else {
                let seat = g.to_act() as u8;
                let adv: Value = serde_json::from_str(&g.advise(seat).unwrap()).unwrap();
                assert!(!adv["options"].as_array().unwrap().is_empty());
                g.bot_step().unwrap();
            }
        }
        let st: Value = serde_json::from_str(&g.state(-1)).unwrap();
        let ch: f64 = st["chances"].as_array().unwrap().iter().map(|x| x.as_f64().unwrap()).sum();
        assert!((ch - 1.0).abs() < 1e-6);
        eprintln!("a 4-player game with advice at every move: {:?}", t0.elapsed());
    }

    #[test]
    fn humans_see_only_their_hand_and_no_bids_while_bidding() {
        let mut seats = vec![json!({"human": true})];
        seats.extend(vec![json!({"level": "beginner", "style": "overbid"}); 3]);
        let mut g = WizardGame::new(4, true, &serde_json::to_string(&seats).unwrap(), 3.0).unwrap();
        // play bots until it's the human's turn
        while g.is_bot_turn() {
            g.bot_step().unwrap();
        }
        let st: Value = serde_json::from_str(&g.state(0)).unwrap();
        assert!(st["hands"][0].is_array());
        assert!(st["hands"][1].is_null());
        if st["phase"]["kind"] == "bid" {
            for s in 1..4 {
                assert!(st["bids"][s].is_null());
            }
            assert!(!st["legal"].as_array().unwrap().is_empty());
        }
    }
}
