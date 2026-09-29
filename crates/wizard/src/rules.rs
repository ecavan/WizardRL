//! The official rules. The only thing a table chooses is how many play.

pub const MIN_PLAYERS: u8 = 3;
pub const MAX_PLAYERS: u8 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rules {
    /// 3 to 6.
    pub players: u8,
}

impl Rules {
    pub fn official(players: u8) -> Rules {
        Rules { players }
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
