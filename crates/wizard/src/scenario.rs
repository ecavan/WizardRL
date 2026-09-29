//! Build a bidding situation to ask a trained network about: "4 players, hearts trump, I hold
//! 7♥ 10♥ 3♥ and bid first — what should I bid?"
//!
//! Only what the bidder can see matters (their hand, trump, their seat, and the bids before them
//! when bidding in turn); the other hands are dealt at random from the rest of the deck.

use crate::card::{bit, cards, is_jester, suit_of, Card, CardSet, Suit, ALL_CARDS};
use crate::rng::Rng;
use crate::round::{Action, Phase, Round};
use crate::rules::Rules;

/// A round waiting for `me` to bid. `position` is the seat order: 1 = left of the dealer (bids
/// and leads first), `players` = the dealer. With `simultaneous` bids nobody sees another bid,
/// so `bids_before` must be empty; bidding in turn, it lists the earlier players' bids in order.
/// `trump = None` means no trump (a Jester was turned up, or it's the last round).
pub fn bid_scenario(
    simultaneous: bool,
    players: u8,
    hand: &[Card],
    trump: Option<Suit>,
    position: u8,
    bids_before: &[u8],
    rng: &mut Rng,
) -> Result<(Round, u8), String> {
    let rules = if simultaneous {
        Rules::simultaneous(players)
    } else {
        Rules::official(players)
    };
    rules.validate()?;
    let size = hand.len() as u8;
    if size == 0 || size > rules.rounds() {
        return Err(format!(
            "{size} cards: a {players}-player round has 1 to {}",
            rules.rounds()
        ));
    }
    if !(1..=players).contains(&position) {
        return Err(format!("position must be 1 to {players}"));
    }
    if simultaneous && !bids_before.is_empty() {
        return Err("with simultaneous bids nobody sees the other bids".into());
    }
    if !simultaneous && bids_before.len() != position as usize - 1 {
        return Err(format!(
            "position {position} means {} earlier bids",
            position - 1
        ));
    }
    let mut mine: CardSet = 0;
    for &c in hand {
        if mine & bit(c) != 0 {
            return Err("a card appears twice".into());
        }
        mine |= bit(c);
    }
    let mut rest: Vec<Card> = cards(ALL_CARDS & !mine).collect();
    rng.shuffle(&mut rest);
    let last_round = size == rules.rounds();
    let turned = match trump {
        Some(s) => {
            if last_round {
                return Err("the last round has no trump".into());
            }
            let i = rest
                .iter()
                .position(|&c| suit_of(c) == Some(s))
                .ok_or("no card of that suit left to turn up")?;
            Some(rest.remove(i))
        }
        None if last_round => None,
        None => {
            let i = rest
                .iter()
                .position(|&c| is_jester(c))
                .ok_or("no Jester left to turn up for 'no trump'")?;
            Some(rest.remove(i))
        }
    };
    let dealer = 0u8;
    let me = position % players; // position 1 = seat 1 (left of dealer 0) ... position n = seat 0
    let mut hands = vec![Vec::new(); players as usize];
    hands[me as usize] = hand.to_vec();
    let mut it = rest.into_iter();
    for (s, h) in hands.iter_mut().enumerate() {
        if s as u8 != me {
            for _ in 0..size {
                h.push(it.next().ok_or("not enough cards")?);
            }
        }
    }
    let mut round = Round::from_hands(rules, size, dealer, &hands, turned);
    // Simultaneous: the earlier seats' bids are hidden from us, so any values will do.
    let placeholders = vec![0u8; position as usize - 1];
    let earlier = if simultaneous {
        &placeholders[..]
    } else {
        bids_before
    };
    for &b in earlier {
        if b > size {
            return Err(format!("a bid of {b} with {size} cards"));
        }
        round.apply(Action::Bid(b)).map_err(|e| e.to_string())?;
    }
    debug_assert_eq!(round.phase(), Phase::Bid { seat: me });
    Ok((round, me))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::card::parse;
    use crate::encode::{self, FEATURES};
    use crate::view::View;

    #[test]
    fn builds_the_situation_asked_for() {
        let hand: Vec<Card> = ["7h", "10h", "3h"]
            .iter()
            .map(|s| parse(s).unwrap())
            .collect();
        let mut rng = Rng::new(1);
        let (r, me) =
            bid_scenario(false, 4, &hand, Some(Suit::Hearts), 3, &[1, 0], &mut rng).unwrap();
        assert_eq!(r.trump(), Some(Suit::Hearts));
        assert_eq!(r.phase(), Phase::Bid { seat: me });
        assert_eq!(r.hand(me), hand.iter().fold(0, |m, &c| m | bit(c)));
        assert_eq!(r.bids().iter().filter(|b| b.is_some()).count(), 2);
        let mut o = vec![0.0; FEATURES];
        encode::observe(&View::new(&r, me, &[]), &mut o);
        assert_eq!(o[encode::PHASE + 1], 1.0);
        // No trump: a Jester is turned up. Last round: nothing turned.
        let (r, _) = bid_scenario(false, 4, &hand, None, 1, &[], &mut rng).unwrap();
        assert_eq!(r.trump(), None);
        let big: Vec<Card> = (0..15).collect();
        assert!(bid_scenario(false, 4, &big, Some(Suit::Spades), 1, &[], &mut rng).is_err());
        assert!(bid_scenario(false, 4, &big, None, 4, &[0, 0, 0], &mut rng).is_ok());
        // The dealer bids last.
        let (r, me) =
            bid_scenario(false, 4, &hand, Some(Suit::Clubs), 4, &[0, 1, 0], &mut rng).unwrap();
        assert_eq!(me, r.dealer());
        assert!(bid_scenario(false, 4, &hand, Some(Suit::Clubs), 2, &[], &mut rng).is_err());
        // Simultaneous: no earlier bids to give, and none visible to the bidder.
        let (r, me) = bid_scenario(true, 4, &hand, Some(Suit::Clubs), 3, &[], &mut rng).unwrap();
        let v = View::new(&r, me, &[]);
        assert_eq!(v.bids().iter().filter(|b| b.is_some()).count(), 0);
        assert_eq!(
            r.bids().iter().filter(|b| b.is_some()).count(),
            2,
            "the engine has them"
        );
        assert!(bid_scenario(true, 4, &hand, Some(Suit::Clubs), 3, &[1, 0], &mut rng).is_err());
    }
}
