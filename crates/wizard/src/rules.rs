//! Which version of the rules a table plays. `Rules::official` is the default everywhere;
//! the house options exist so we can train on either and compare.

/// What happens when the card turned up for trump is a Wizard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WizardTurned {
    /// Official: the dealer names the trump suit.
    DealerPicks,
    /// House: the player on the dealer's right names it.
    RightOfDealerPicks,
    /// House: a random suit.
    RandomSuit,
}

/// What happens when the card turned up for trump is a Jester.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum JesterTurned {
    /// Official: no trump this round.
    NoTrump,
    /// House: the player on the dealer's right names it.
    RightOfDealerPicks,
    /// House: a random suit.
    RandomSuit,
}

/// Trump in the last round.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LastRound {
    /// Official: no trump only when no card is left to turn up (always true with 3 to 6 players,
    /// since 60 divides evenly; with 7 or 8 players a few cards remain and one is turned up).
    Official,
    /// House: the last round never has trump.
    AlwaysNoTrump,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// 3 to 6 officially; the engine allows up to 8 (house option).
    pub players: u8,
    pub wizard_turned: WizardTurned,
    pub jester_turned: JesterTurned,
    pub last_round: LastRound,
}

pub const MIN_PLAYERS: u8 = 3;
pub const MAX_PLAYERS: u8 = 8;
pub const MAX_OFFICIAL_PLAYERS: u8 = 6;

impl Rules {
    pub fn official(players: u8) -> Rules {
        Rules {
            players,
            wizard_turned: WizardTurned::DealerPicks,
            jester_turned: JesterTurned::NoTrump,
            last_round: LastRound::Official,
        }
    }

    /// Eli's family table: the player on the dealer's right picks trump when a Wizard or a
    /// Jester turns up, and the last round has no trump.
    pub fn house(players: u8) -> Rules {
        Rules {
            players,
            wizard_turned: WizardTurned::RightOfDealerPicks,
            jester_turned: JesterTurned::RightOfDealerPicks,
            last_round: LastRound::AlwaysNoTrump,
        }
    }

    /// Rounds in a full game: round r deals r cards to each player until the deck can't give
    /// everyone one more (20 rounds with 3 players, 15 with 4, 12 with 5, 10 with 6).
    pub fn rounds(&self) -> u8 {
        (crate::card::DECK_SIZE as u8) / self.players
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(MIN_PLAYERS..=MAX_PLAYERS).contains(&self.players) {
            return Err(format!("{} players: Wizard needs {MIN_PLAYERS} to {MAX_PLAYERS}", self.players));
        }
        Ok(())
    }

    pub fn is_official(&self) -> bool {
        *self == Rules::official(self.players) && self.players <= MAX_OFFICIAL_PLAYERS
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_per_player_count() {
        let got: Vec<u8> = (3..=8).map(|n| Rules::official(n).rounds()).collect();
        assert_eq!(got, vec![20, 15, 12, 10, 8, 7]);
    }

    #[test]
    fn validation() {
        assert!(Rules::official(2).validate().is_err());
        assert!(Rules::official(9).validate().is_err());
        assert!((3..=8).all(|n| Rules::official(n).validate().is_ok()));
        assert!(Rules::official(4).is_official());
        assert!(!Rules::official(7).is_official());
        assert!(!Rules::house(4).is_official());
    }
}
