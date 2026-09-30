//! Build a bidding situation to ask a trained network about: "4 players, hearts trump, I hold
//! 7♥ 10♥ 3♥ and bid first — what should I bid?"
//!
//! Only what the bidder can see matters (their hand, trump, their seat, and the bids before them
//! when bidding in turn); the other hands are dealt at random from the rest of the deck.

use crate::card::{self, bit, cards, is_jester, suit_of, Card, CardSet, Suit, ALL_CARDS};
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

/// A first-trick card-play situation: "4 players, hearts trump, bids 1/0/2/1, the player on my
/// right led the 5 of clubs; I hold A♥ J♥ 9♠, which card?". `bids` are everyone's bids in seat
/// order from the left of the dealer (position 1) to the dealer; `trick` is the cards already
/// played to this first trick, by positions 1, 2, ... (the leader is position 1). Returns the
/// round waiting for `me` (at `position`) to play.
///
/// The hidden hands are random, but consistent with what was seen: a player who didn't follow
/// the suit led holds none of it.
#[allow(clippy::too_many_arguments)]
pub fn play_scenario(
    simultaneous: bool,
    players: u8,
    hand: &[Card],
    trump: Option<Suit>,
    position: u8,
    bids: &[u8],
    trick: &[Card],
    rng: &mut Rng,
) -> Result<(Round, u8), String> {
    let rules = if simultaneous {
        Rules::simultaneous(players)
    } else {
        Rules::official(players)
    };
    rules.validate()?;
    let n = players as usize;
    let size = hand.len() as u8;
    if size == 0 || size > rules.rounds() {
        return Err(format!(
            "{size} cards: a {players}-player round has 1 to {}",
            rules.rounds()
        ));
    }
    if bids.len() != n {
        return Err(format!(
            "give all {n} bids, from the left of the dealer to the dealer"
        ));
    }
    if !(1..=players).contains(&position) || trick.len() != position as usize - 1 {
        return Err(format!(
            "position {position} plays after {} card(s) in the trick",
            position.max(1) - 1
        ));
    }
    let seat_of = |pos: u8| pos % players; // position 1 = seat 1 (left of dealer 0) ... n = seat 0
    let me = seat_of(position);
    let mut used: CardSet = 0;
    for &c in hand.iter().chain(trick) {
        if used & bit(c) != 0 {
            return Err("a card appears twice".into());
        }
        used |= bit(c);
    }
    // The suit that must be followed, from the first standard card played.
    let led = match trick.first() {
        Some(&c) if crate::card::is_wizard(c) => None, // a Wizard led: no suit to follow
        _ => trick.iter().find_map(|&c| suit_of(c)),
    };
    let mut rest: Vec<Card> = cards(ALL_CARDS & !used).collect();
    rng.shuffle(&mut rest);
    let last_round = size == rules.rounds();
    let turned = match trump {
        Some(s) if !last_round => {
            let i = rest
                .iter()
                .position(|&c| suit_of(c) == Some(s))
                .ok_or("no card of that suit left to turn up")?;
            Some(rest.remove(i))
        }
        Some(_) => return Err("the last round has no trump".into()),
        None if last_round => None,
        None => {
            let i = rest
                .iter()
                .position(|&c| is_jester(c))
                .ok_or("no Jester left")?;
            Some(rest.remove(i))
        }
    };
    let mut hands = vec![Vec::new(); n];
    hands[me as usize] = hand.to_vec();
    // Earlier players in the trick: their played card, plus random cards (none of the led suit
    // if they didn't follow it).
    for (k, &c) in trick.iter().enumerate() {
        let seat = seat_of(k as u8 + 1) as usize;
        hands[seat].push(c);
        let followed = led.is_none() || suit_of(c) == led || suit_of(c).is_none() && k == 0;
        let void = led.is_some() && !followed && suit_of(c).is_some();
        while hands[seat].len() < size as usize {
            let i = rest
                .iter()
                .position(|&x| !(void && suit_of(x) == led))
                .ok_or("not enough cards")?;
            hands[seat].push(rest.remove(i));
        }
    }
    for (s, h) in hands.iter_mut().enumerate() {
        while h.len() < size as usize && s as u8 != me {
            h.push(rest.pop().ok_or("not enough cards")?);
        }
    }
    let mut round = Round::from_hands(rules, size, 0, &hands, turned);
    if let Phase::PickTrump { .. } = round.phase() {
        round
            .apply(Action::PickTrump(
                trump.ok_or("a Wizard was turned: name the trump")?,
            ))
            .map_err(|e| e.to_string())?;
    }
    for &b in bids {
        if b > size {
            return Err(format!("a bid of {b} with {size} cards"));
        }
        round.apply(Action::Bid(b)).map_err(|e| e.to_string())?;
    }
    for &c in trick {
        round
            .apply(Action::Play(c))
            .map_err(|e| format!("{}: {e}", card::name(c)))?;
    }
    debug_assert_eq!(round.to_act(), Some(me));
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

    #[test]
    fn builds_a_first_trick_play_situation() {
        let c = |s: &str| parse(s).unwrap();
        let hand = vec![c("Ah"), c("Jh"), c("9s")];
        let mut rng = Rng::new(3);
        for _ in 0..50 {
            // hearts trump, the leader (position 1) played 5 of clubs, I'm next
            let (r, me) = play_scenario(
                true,
                4,
                &hand,
                Some(Suit::Hearts),
                2,
                &[1, 0, 2, 1],
                &[c("5c")],
                &mut rng,
            )
            .unwrap();
            assert_eq!(r.to_act(), Some(me));
            assert_eq!(r.current_trick().len(), 1);
            assert_eq!(r.bid(me), Some(0));
            // someone who didn't follow clubs holds no clubs
            let (r, _) = play_scenario(
                true,
                4,
                &hand,
                Some(Suit::Hearts),
                3,
                &[1, 0, 2, 1],
                &[c("5c"), c("Kd")],
                &mut rng,
            )
            .unwrap();
            assert_eq!(r.hand(2) & Suit::Clubs.mask(), 0);
        }
        assert!(play_scenario(true, 4, &hand, None, 2, &[1, 0, 2], &[c("5c")], &mut rng).is_err());
    }
}
