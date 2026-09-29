//! The 60-card Wizard deck.
//!
//! Encoding (one `u8` per card, so a hand fits in a `u64` bitmask):
//! - `0..52`: the standard cards, `13 * suit + rank`, rank 0 = deuce … 12 = ace
//! - `52..56`: the four Wizards
//! - `56..60`: the four Jesters

use std::fmt;

pub type Card = u8;
/// A set of cards, one bit per card.
pub type CardSet = u64;

pub const DECK_SIZE: usize = 60;
pub const WIZARD_BASE: Card = 52;
pub const JESTER_BASE: Card = 56;
pub const ALL_CARDS: CardSet = (1u64 << DECK_SIZE) - 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Suit {
    Clubs = 0,
    Diamonds = 1,
    Hearts = 2,
    Spades = 3,
}

impl Suit {
    pub const ALL: [Suit; 4] = [Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades];

    pub fn from_index(i: u8) -> Suit {
        Suit::ALL[(i & 3) as usize]
    }

    pub fn index(self) -> u8 {
        self as u8
    }

    pub fn symbol(self) -> char {
        ['♣', '♦', '♥', '♠'][self as usize]
    }

    pub fn letter(self) -> char {
        ['c', 'd', 'h', 's'][self as usize]
    }

    pub fn from_char(c: char) -> Option<Suit> {
        match c.to_ascii_lowercase() {
            'c' | '♣' => Some(Suit::Clubs),
            'd' | '♦' => Some(Suit::Diamonds),
            'h' | '♥' => Some(Suit::Hearts),
            's' | '♠' => Some(Suit::Spades),
            _ => None,
        }
    }

    /// All 13 standard cards of this suit.
    pub fn mask(self) -> CardSet {
        ((1u64 << 13) - 1) << (13 * self as u32)
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = ["clubs", "diamonds", "hearts", "spades"][*self as usize];
        write!(f, "{name}")
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// A standard card; rank 0 = deuce … 12 = ace.
    Normal { suit: Suit, rank: u8 },
    Wizard,
    Jester,
}

#[inline]
pub fn kind(c: Card) -> Kind {
    debug_assert!((c as usize) < DECK_SIZE);
    if c < WIZARD_BASE {
        Kind::Normal { suit: Suit::from_index(c / 13), rank: c % 13 }
    } else if c < JESTER_BASE {
        Kind::Wizard
    } else {
        Kind::Jester
    }
}

#[inline]
pub fn is_wizard(c: Card) -> bool {
    (WIZARD_BASE..JESTER_BASE).contains(&c)
}

#[inline]
pub fn is_jester(c: Card) -> bool {
    c >= JESTER_BASE && (c as usize) < DECK_SIZE
}

/// The suit of a standard card; `None` for Wizards and Jesters.
#[inline]
pub fn suit_of(c: Card) -> Option<Suit> {
    if c < WIZARD_BASE {
        Some(Suit::from_index(c / 13))
    } else {
        None
    }
}

/// Rank 0 = deuce … 12 = ace, for standard cards.
#[inline]
pub fn rank_of(c: Card) -> Option<u8> {
    if c < WIZARD_BASE {
        Some(c % 13)
    } else {
        None
    }
}

pub fn card(suit: Suit, rank: u8) -> Card {
    assert!(rank < 13);
    13 * suit as u8 + rank
}

pub const WIZARD_MASK: CardSet = 0b1111 << WIZARD_BASE;
pub const JESTER_MASK: CardSet = 0b1111 << JESTER_BASE;

#[inline]
pub fn bit(c: Card) -> CardSet {
    1u64 << c
}

/// Iterate the cards of a set, lowest encoding first.
pub fn cards(mut set: CardSet) -> impl Iterator<Item = Card> {
    std::iter::from_fn(move || {
        if set == 0 {
            None
        } else {
            let c = set.trailing_zeros() as Card;
            set &= set - 1;
            Some(c)
        }
    })
}

const RANK_NAMES: [&str; 13] = ["2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K", "A"];

/// Short name: `A♠`, `10♥`, `Wiz`, `Jes`.
pub fn name(c: Card) -> String {
    match kind(c) {
        Kind::Normal { suit, rank } => format!("{}{}", RANK_NAMES[rank as usize], suit.symbol()),
        Kind::Wizard => "Wiz".to_string(),
        Kind::Jester => "Jes".to_string(),
    }
}

/// Parse `As`, `10h`, `Th`, `2c`, `Jd` (a jack). The special cards are `Wiz1`..`Wiz4` and
/// `Jes1`..`Jes4` (`Wiz` / `Jes` alone mean the first one).
pub fn parse(s: &str) -> Option<Card> {
    let t = s.trim();
    let lower = t.to_ascii_lowercase();
    if let Some(rest) = lower.strip_prefix("wiz") {
        let i: u8 = if rest.is_empty() { 1 } else { rest.parse().ok()? };
        return (1..=4).contains(&i).then(|| WIZARD_BASE + i - 1);
    }
    if let Some(rest) = lower.strip_prefix("jes") {
        let i: u8 = if rest.is_empty() { 1 } else { rest.parse().ok()? };
        return (1..=4).contains(&i).then(|| JESTER_BASE + i - 1);
    }
    let chars: Vec<char> = t.chars().collect();
    if chars.len() < 2 {
        return None;
    }
    let suit = Suit::from_char(*chars.last()?)?;
    let rank_str: String = chars[..chars.len() - 1].iter().collect::<String>().to_ascii_uppercase();
    let rank = match rank_str.as_str() {
        "T" | "10" => 8,
        "J" => 9,
        "Q" => 10,
        "K" => 11,
        "A" => 12,
        r => {
            let v: u8 = r.parse().ok()?;
            if !(2..=9).contains(&v) {
                return None;
            }
            v - 2
        }
    };
    Some(card(suit, rank))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deck_layout() {
        let mut normals = 0;
        let mut wizards = 0;
        let mut jesters = 0;
        for c in 0..DECK_SIZE as Card {
            match kind(c) {
                Kind::Normal { .. } => normals += 1,
                Kind::Wizard => wizards += 1,
                Kind::Jester => jesters += 1,
            }
        }
        assert_eq!((normals, wizards, jesters), (52, 4, 4));
        assert_eq!(cards(ALL_CARDS).count(), 60);
        assert_eq!(cards(WIZARD_MASK).collect::<Vec<_>>(), vec![52, 53, 54, 55]);
        assert_eq!(cards(JESTER_MASK).collect::<Vec<_>>(), vec![56, 57, 58, 59]);
        for s in Suit::ALL {
            assert_eq!(cards(s.mask()).count(), 13);
            assert!(cards(s.mask()).all(|c| suit_of(c) == Some(s)));
        }
    }

    #[test]
    fn names_round_trip() {
        for c in 0..DECK_SIZE as Card {
            let n = name(c);
            if c < WIZARD_BASE {
                let ascii = n.replace('♣', "c").replace('♦', "d").replace('♥', "h").replace('♠', "s");
                assert_eq!(parse(&ascii), Some(c), "{n}");
            }
        }
        assert_eq!(parse("As"), Some(card(Suit::Spades, 12)));
        assert_eq!(parse("10h"), Some(card(Suit::Hearts, 8)));
        assert_eq!(parse("Th"), Some(card(Suit::Hearts, 8)));
        assert_eq!(parse("2c"), Some(card(Suit::Clubs, 0)));
        assert_eq!(parse("Jd"), Some(card(Suit::Diamonds, 9)));
        assert_eq!(parse("wiz"), Some(52));
        assert_eq!(parse("Wiz4"), Some(55));
        assert_eq!(parse("jes2"), Some(57));
        assert_eq!(parse("1h"), None);
        assert_eq!(parse("Wiz5"), None);
    }
}
