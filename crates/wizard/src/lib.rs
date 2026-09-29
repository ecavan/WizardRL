//! Wizard, the trick-taking card game: a rules engine for self-play reinforcement learning.
//!
//! - [`card`]: the 60-card deck and card sets
//! - [`rules`]: official rules plus house options
//! - [`round`]: one round as a state machine (trump, bids, tricks, score)
//! - [`game`]: full games of rounds 1, 2, 3 … cards
//! - [`view`]: what one seat may see
//! - [`bots`]: baseline players
//!
//! Official rules are the default everywhere. See the project plan for why.

pub mod bots;
pub mod card;
pub mod game;
pub mod rng;
pub mod round;
pub mod rules;
pub mod view;

pub use card::{Card, CardSet, Suit};
pub use round::{Action, Event, Phase, Round};
pub use rules::Rules;
