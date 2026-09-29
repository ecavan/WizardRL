//! Wizard, the trick-taking card game: a rules engine for self-play reinforcement learning.
//!
//! - [`card`]: the 60-card deck and card sets
//! - [`rules`]: the official rules (3 to 6 players)
//! - [`round`]: one round as a state machine (trump, bids, tricks, score)
//! - [`game`]: full games of rounds 1, 2, 3 … cards
//! - [`view`]: what one seat may see
//! - [`bots`]: baseline players
//! - [`chart`]: a bot that bids from a bid chart made from a trained network
//! - [`encode`]: what the network sees, and its action numbering
//! - [`env`]: a batch of tables for training, driven from Python
//! - [`net`]: a trained network running in Rust (`NetBot`)
//! - [`scenario`]: build a bidding situation to ask a network about
//! - [`search`]: look-ahead search for bids, with a network playing out imagined rounds
//! - [`style`]: players with habits (overbidding, early Wizards, ...)

pub mod bots;
pub mod card;
pub mod chart;
pub mod encode;
pub mod env;
pub mod game;
pub mod net;
pub mod rng;
pub mod round;
pub mod rules;
pub mod scenario;
pub mod search;
pub mod style;
pub mod view;

pub use card::{Card, CardSet, Suit};
pub use round::{Action, Event, Phase, Round};
pub use rules::Rules;
