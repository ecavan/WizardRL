//! The official rules, plus one choice: whether bids are made in turn (the printed rules) or all
//! at once (everyone shows their bid together, so nobody sees another bid before making theirs).

pub const MIN_PLAYERS: u8 = 3;
pub const MAX_PLAYERS: u8 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// 3 to 6.
    pub players: u8,
    /// Everyone bids at the same time: while bidding, no one sees anyone else's bid.
    pub simultaneous_bids: bool,
}

impl Rules {
    /// The printed rules: bids go round the table in turn, starting left of the dealer, and
    /// each player hears the bids before theirs.
    pub fn official(players: u8) -> Rules {
        Rules {
            players,
            simultaneous_bids: false,
        }
    }

    /// Everyone bids at once (how Eli's family plays, and the default for training).
    pub fn simultaneous(players: u8) -> Rules {
        Rules {
            players,
            simultaneous_bids: true,
        }
    }

    /// Rounds in a full game: round r deals r cards to each player until the deck runs out
    /// (20 rounds with 3 players, 15 with 4, 12 with 5, 10 with 6).
    pub fn rounds(&self) -> u8 {
        (crate::card::DECK_SIZE as u8) / self.players
    }

    pub fn validate(&self) -> Result<(), String> {
        if !(MIN_PLAYERS..=MAX_PLAYERS).contains(&self.players) {
            return Err(format!(
                "{} players: Wizard is for {MIN_PLAYERS} to {MAX_PLAYERS}",
                self.players
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_per_player_count() {
        let got: Vec<u8> = (3..=6).map(|n| Rules::official(n).rounds()).collect();
        assert_eq!(got, vec![20, 15, 12, 10]);
        // 60 cards divide evenly, so the last round always deals the whole deck.
        assert!((3..=6).all(|n| Rules::official(n).rounds() as usize * n as usize == 60));
    }

    #[test]
    fn validation() {
        assert!(Rules::official(2).validate().is_err());
        assert!(Rules::official(7).validate().is_err());
        assert!((3..=6).all(|n| Rules::official(n).validate().is_ok()));
    }
}
