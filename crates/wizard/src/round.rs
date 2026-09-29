//! One round of Wizard: deal, trump, bids, tricks, score.
//!
//! Seats are numbered `0..n` going to the left (clockwise). The player left of the dealer
//! (`dealer + 1`) bids first and leads the first trick; each trick's winner leads the next.

use crate::card::{
    self, bit, cards, suit_of, Card, CardSet, Kind, Suit, ALL_CARDS, DECK_SIZE, JESTER_MASK,
    WIZARD_MASK,
};
use crate::rng::Rng;
use crate::rules::Rules;
use std::fmt;

pub const MAX_SEATS: usize = crate::rules::MAX_PLAYERS as usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// The dealer names the trump suit (a Wizard was turned up).
    PickTrump {
        seat: u8,
    },
    Bid {
        seat: u8,
    },
    Play {
        seat: u8,
    },
    Done,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    PickTrump(Suit),
    Bid(u8),
    Play(Card),
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::PickTrump(s) => write!(f, "trump {s}"),
            Action::Bid(b) => write!(f, "bid {b}"),
            Action::Play(c) => write!(f, "{}", card::name(*c)),
        }
    }
}

/// Why this round has the trump it has.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrumpSource {
    /// A standard card was turned up.
    TurnedCard(Card),
    /// A Wizard was turned up; the dealer names trump.
    WizardTurned(Card),
    /// A Jester was turned up: no trump.
    JesterTurned(Card),
    /// No card left to turn up (the last round): no trump.
    NoCardTurned,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IllegalAction {
    pub action: Action,
    pub phase: Phase,
    pub reason: &'static str,
}

impl fmt::Display for IllegalAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "illegal {} in {:?}: {}",
            self.action, self.phase, self.reason
        )
    }
}

impl std::error::Error for IllegalAction {}

/// What an accepted action caused.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    None,
    TrickWon { winner: u8 },
    RoundOver,
}

#[derive(Clone, Debug)]
pub struct Trick {
    /// `(seat, card)` in the order played.
    pub plays: Vec<(u8, Card)>,
    pub winner: u8,
}

#[derive(Clone, Debug)]
pub struct Round {
    rules: Rules,
    n: u8,
    size: u8,
    dealer: u8,
    hands: [CardSet; MAX_SEATS],
    trump_source: TrumpSource,
    trump: Option<Suit>,
    phase: Phase,
    bids: [Option<u8>; MAX_SEATS],
    won: [u8; MAX_SEATS],
    trick: Vec<(u8, Card)>,
    tricks: Vec<Trick>,
    played: CardSet,
    /// Suits each seat has shown it holds none of (a standard card of another suit when that
    /// suit was to be followed). Public information every attentive player has.
    voids: [u8; MAX_SEATS],
    history: Vec<(u8, Action)>,
}

/// The suit a card must follow in this trick, if any.
///
/// Wizard led: none (everyone plays anything). Jester led: the first standard card sets it,
/// unless a Wizard comes first, in which case Wizard rules apply and nothing is to be followed.
pub fn led_suit(plays: &[(u8, Card)]) -> Option<Suit> {
    for &(_, c) in plays {
        match card::kind(c) {
            Kind::Jester => continue,
            Kind::Wizard => return None,
            Kind::Normal { suit, .. } => return Some(suit),
        }
    }
    None
}

/// Index (into `plays`) of the card that wins the trick.
///
/// The first Wizard played; else the highest trump; else the highest card of the suit led;
/// if every card is a Jester, the first Jester.
pub fn trick_winner(plays: &[(u8, Card)], trump: Option<Suit>) -> usize {
    assert!(!plays.is_empty());
    if let Some(i) = plays.iter().position(|&(_, c)| card::is_wizard(c)) {
        return i;
    }
    let best_of = |s: Suit| {
        plays
            .iter()
            .enumerate()
            .filter(|(_, &(_, c))| suit_of(c) == Some(s))
            .max_by_key(|(_, &(_, c))| card::rank_of(c))
            .map(|(i, _)| i)
    };
    if let Some(t) = trump {
        if let Some(i) = best_of(t) {
            return i;
        }
    }
    match led_suit(plays) {
        Some(s) => best_of(s).expect("the led suit is on the table"),
        None => 0, // all Jesters
    }
}

/// Official scoring: exact bid scores 20 + 10 per trick; otherwise −10 per trick off.
pub fn round_score(bid: u8, won: u8) -> i32 {
    if bid == won {
        20 + 10 * won as i32
    } else {
        -10 * (bid as i32 - won as i32).abs()
    }
}

/// Cards a hand may play to the current trick.
pub fn legal_cards(hand: CardSet, plays: &[(u8, Card)]) -> CardSet {
    if let Some(s) = led_suit(plays) {
        let follow = hand & s.mask();
        if follow != 0 {
            return follow | (hand & (WIZARD_MASK | JESTER_MASK));
        }
    }
    hand
}

impl Round {
    /// Shuffle and deal a round of `size` cards each.
    pub fn deal(rules: Rules, size: u8, dealer: u8, rng: &mut Rng) -> Round {
        rules.validate().expect("valid rules");
        let n = rules.players;
        assert!(
            size >= 1 && size <= rules.rounds(),
            "round size {size} with {n} players"
        );
        assert!(dealer < n);
        let mut deck: Vec<Card> = (0..DECK_SIZE as Card).collect();
        rng.shuffle(&mut deck);
        let mut hands = vec![Vec::new(); n as usize];
        // Deal one at a time starting left of the dealer, like at the table (order doesn't
        // change the odds, but it keeps the deck order meaningful for replays).
        let mut k = 0;
        for _ in 0..size {
            for i in 1..=n {
                hands[((dealer + i) % n) as usize].push(deck[k]);
                k += 1;
            }
        }
        let turned = deck.get(k).copied();
        Round::from_hands(rules, size, dealer, &hands, turned)
    }

    /// Start a round from given hands (for tests and replays). `turned` is the card turned up
    /// for trump, or `None` if none is left.
    pub fn from_hands(
        rules: Rules,
        size: u8,
        dealer: u8,
        hands: &[Vec<Card>],
        turned: Option<Card>,
    ) -> Round {
        rules.validate().expect("valid rules");
        let n = rules.players;
        assert_eq!(hands.len(), n as usize, "one hand per seat");
        assert!(dealer < n);
        let mut sets = [0u64; MAX_SEATS];
        let mut seen: CardSet = 0;
        for (i, h) in hands.iter().enumerate() {
            assert_eq!(
                h.len(),
                size as usize,
                "seat {i} has {} cards, not {size}",
                h.len()
            );
            for &c in h {
                assert!((c as usize) < DECK_SIZE);
                assert_eq!(seen & bit(c), 0, "card {} dealt twice", card::name(c));
                seen |= bit(c);
                sets[i] |= bit(c);
            }
        }
        if let Some(t) = turned {
            assert_eq!(seen & bit(t), 0, "the turned card is in a hand");
        }
        let mut phase = Phase::Bid {
            seat: (dealer + 1) % n,
        };
        let (trump_source, trump) = match turned {
            None => (TrumpSource::NoCardTurned, None),
            Some(c) => match card::kind(c) {
                Kind::Normal { suit, .. } => (TrumpSource::TurnedCard(c), Some(suit)),
                Kind::Wizard => {
                    phase = Phase::PickTrump { seat: dealer };
                    (TrumpSource::WizardTurned(c), None)
                }
                Kind::Jester => (TrumpSource::JesterTurned(c), None),
            },
        };

        Round {
            rules,
            n,
            size,
            dealer,
            hands: sets,
            trump_source,
            trump,
            phase,
            bids: [None; MAX_SEATS],
            won: [0; MAX_SEATS],
            trick: Vec::with_capacity(MAX_SEATS),
            tricks: Vec::with_capacity(size as usize),
            played: 0,
            voids: [0; MAX_SEATS],
            history: Vec::with_capacity(size as usize * (n as usize + 1) + 1),
        }
    }

    // ---------------------------------------------------------------- public state

    pub fn rules(&self) -> &Rules {
        &self.rules
    }
    pub fn players(&self) -> u8 {
        self.n
    }
    /// Cards dealt to each player this round (= tricks in the round).
    pub fn size(&self) -> u8 {
        self.size
    }
    pub fn dealer(&self) -> u8 {
        self.dealer
    }
    pub fn phase(&self) -> Phase {
        self.phase
    }
    pub fn trump(&self) -> Option<Suit> {
        self.trump
    }
    pub fn trump_source(&self) -> TrumpSource {
        self.trump_source
    }
    pub fn bid(&self, seat: u8) -> Option<u8> {
        self.bids[seat as usize]
    }
    pub fn bids(&self) -> &[Option<u8>] {
        &self.bids[..self.n as usize]
    }
    pub fn tricks_won(&self, seat: u8) -> u8 {
        self.won[seat as usize]
    }
    /// Cards played so far to the trick in progress.
    pub fn current_trick(&self) -> &[(u8, Card)] {
        &self.trick
    }
    pub fn completed_tricks(&self) -> &[Trick] {
        &self.tricks
    }
    /// Every card played this round, including the trick in progress.
    pub fn played(&self) -> CardSet {
        self.played
    }
    /// Suits `seat` has shown it's out of (bit `s` set for suit `s`).
    pub fn known_voids(&self, seat: u8) -> u8 {
        self.voids[seat as usize]
    }
    pub fn history(&self) -> &[(u8, Action)] {
        &self.history
    }
    pub fn is_done(&self) -> bool {
        self.phase == Phase::Done
    }

    /// Whose turn it is.
    pub fn to_act(&self) -> Option<u8> {
        match self.phase {
            Phase::PickTrump { seat } | Phase::Bid { seat } | Phase::Play { seat } => Some(seat),
            Phase::Done => None,
        }
    }

    // ---------------------------------------------------------------- private state

    /// A seat's hand. Only the engine, the seat itself, and tests should look at this.
    pub fn hand(&self, seat: u8) -> CardSet {
        self.hands[seat as usize]
    }

    // ---------------------------------------------------------------- moves

    /// Cards `seat` may play now (empty unless it is `seat`'s turn to play).
    pub fn legal_plays(&self) -> CardSet {
        match self.phase {
            Phase::Play { seat } => legal_cards(self.hands[seat as usize], &self.trick),
            _ => 0,
        }
    }

    pub fn legal_actions(&self) -> Vec<Action> {
        match self.phase {
            Phase::PickTrump { .. } => Suit::ALL.iter().map(|&s| Action::PickTrump(s)).collect(),
            Phase::Bid { .. } => (0..=self.size).map(Action::Bid).collect(),
            Phase::Play { .. } => cards(self.legal_plays()).map(Action::Play).collect(),
            Phase::Done => Vec::new(),
        }
    }

    pub fn is_legal(&self, a: Action) -> bool {
        match (self.phase, a) {
            (Phase::PickTrump { .. }, Action::PickTrump(_)) => true,
            (Phase::Bid { .. }, Action::Bid(b)) => b <= self.size,
            (Phase::Play { .. }, Action::Play(c)) => {
                (c as usize) < DECK_SIZE && self.legal_plays() & bit(c) != 0
            }
            _ => false,
        }
    }

    pub fn apply(&mut self, a: Action) -> Result<Event, IllegalAction> {
        let err = |reason| IllegalAction {
            action: a,
            phase: self.phase,
            reason,
        };
        match (self.phase, a) {
            (Phase::PickTrump { seat }, Action::PickTrump(s)) => {
                self.trump = Some(s);
                self.history.push((seat, a));
                self.phase = Phase::Bid {
                    seat: (self.dealer + 1) % self.n,
                };
                Ok(Event::None)
            }
            (Phase::Bid { seat }, Action::Bid(b)) => {
                if b > self.size {
                    return Err(err("bid above the number of cards"));
                }
                self.bids[seat as usize] = Some(b);
                self.history.push((seat, a));
                let next = (seat + 1) % self.n;
                self.phase = if next == (self.dealer + 1) % self.n {
                    Phase::Play { seat: next }
                } else {
                    Phase::Bid { seat: next }
                };
                Ok(Event::None)
            }
            (Phase::Play { seat }, Action::Play(c)) => {
                if (c as usize) >= DECK_SIZE || self.hands[seat as usize] & bit(c) == 0 {
                    return Err(err("card not in hand"));
                }
                if self.legal_plays() & bit(c) == 0 {
                    return Err(err("must follow the suit led"));
                }
                if let (Some(s), Some(cs)) = (led_suit(&self.trick), suit_of(c)) {
                    if cs != s {
                        self.voids[seat as usize] |= 1 << s.index();
                    }
                }
                self.hands[seat as usize] &= !bit(c);
                self.played |= bit(c);
                self.trick.push((seat, c));
                self.history.push((seat, a));
                if self.trick.len() < self.n as usize {
                    self.phase = Phase::Play {
                        seat: (seat + 1) % self.n,
                    };
                    return Ok(Event::None);
                }
                let w = self.trick[trick_winner(&self.trick, self.trump)].0;
                self.won[w as usize] += 1;
                let plays = std::mem::take(&mut self.trick);
                self.tricks.push(Trick { plays, winner: w });
                if self.tricks.len() == self.size as usize {
                    self.phase = Phase::Done;
                    Ok(Event::RoundOver)
                } else {
                    self.phase = Phase::Play { seat: w };
                    Ok(Event::TrickWon { winner: w })
                }
            }
            (Phase::Done, _) => Err(err("the round is over")),
            _ => Err(err("not that kind of decision now")),
        }
    }

    /// Each seat's score for the round, once it's over.
    pub fn scores(&self) -> Option<Vec<i32>> {
        if !self.is_done() {
            return None;
        }
        Some(
            (0..self.n as usize)
                .map(|i| round_score(self.bids[i].expect("everyone bid"), self.won[i]))
                .collect(),
        )
    }

    /// Internal consistency; used heavily by the tests.
    pub fn check_invariants(&self) -> Result<(), String> {
        let n = self.n as usize;
        let mut seen: CardSet = 0;
        for i in 0..n {
            if seen & self.hands[i] != 0 {
                return Err("a card is in two hands".into());
            }
            seen |= self.hands[i];
        }
        if seen & self.played != 0 {
            return Err("a played card is still in a hand".into());
        }
        let in_trick: CardSet = self.trick.iter().fold(0, |m, &(_, c)| m | bit(c));
        let in_tricks: CardSet = self
            .tricks
            .iter()
            .flat_map(|t| t.plays.iter())
            .fold(0, |m, &(_, c)| m | bit(c));
        if in_trick | in_tricks != self.played || in_trick & in_tricks != 0 {
            return Err("played set disagrees with the tricks".into());
        }
        if (seen | self.played) & !ALL_CARDS != 0 {
            return Err("card out of range".into());
        }
        let total_cards = seen.count_ones() + self.played.count_ones();
        if total_cards != (self.size as u32) * self.n as u32 {
            return Err(format!(
                "{total_cards} cards in play, expected {}",
                self.size as usize * n
            ));
        }
        // Everyone has played the same number of cards, give or take the trick in progress.
        let done = self.tricks.len() as u32;
        for i in 0..n {
            let in_hand = self.hands[i].count_ones();
            let played_now = self.trick.iter().any(|&(s, _)| s as usize == i) as u32;
            if in_hand + done + played_now != self.size as u32 {
                return Err(format!(
                    "seat {i} holds {in_hand} cards after {done} tricks"
                ));
            }
        }
        let won: u32 = self.won[..n].iter().map(|&w| w as u32).sum();
        if won != done {
            return Err(format!("{won} tricks won but {done} played"));
        }
        for t in &self.tricks {
            if t.plays.len() != n {
                return Err("a completed trick without a card from everyone".into());
            }
            if t.plays[trick_winner(&t.plays, self.trump)].0 != t.winner {
                return Err("recorded trick winner is wrong".into());
            }
        }
        Ok(())
    }
}
