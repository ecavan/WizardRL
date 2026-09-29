//! `wizard sim`  — bots play each other; prints scores, bid accuracy and speed.
//! `wizard play` — you play a game against bots in the terminal.

use std::io::{self, BufRead, Write};
use std::time::Instant;
use wizard::bots::{self, Bot};
use wizard::card::{self, cards, Card, Suit};
use wizard::game::play_game;
use wizard::rng::Rng;
use wizard::round::{Action, Phase, Round, TrumpSource};
use wizard::rules::Rules;
use wizard::view::View;

const USAGE: &str = "usage:
  wizard sim  [--players N] [--games G] [--seed S] [--bots counting,random,...]
  wizard play [--players N] [--seed S] [--bots counting,...]

  --players  3 to 6                                        default 4
  --bots     one name per seat (sim) or per opponent (play); random | counting";

struct Args {
    cmd: String,
    players: u8,
    games: u64,
    seed: Option<u64>,
    bots: Vec<String>,
}

fn parse_args() -> Result<Args, String> {
    let mut it = std::env::args().skip(1);
    let cmd = it.next().ok_or("missing command")?;
    let mut a = Args {
        cmd,
        players: 4,
        games: 1000,
        seed: None,
        bots: Vec::new(),
    };
    while let Some(k) = it.next() {
        let mut val = || it.next().ok_or(format!("{k} needs a value"));
        match k.as_str() {
            "--players" => a.players = val()?.parse().map_err(|_| "bad --players")?,
            "--games" => a.games = val()?.parse().map_err(|_| "bad --games")?,
            "--seed" => a.seed = Some(val()?.parse().map_err(|_| "bad --seed")?),
            "--bots" => a.bots = val()?.split(',').map(|s| s.trim().to_string()).collect(),
            "-h" | "--help" => return Err(String::new()),
            _ => return Err(format!("unknown option {k}")),
        }
    }
    Ok(a)
}

fn rules_for(a: &Args) -> Result<Rules, String> {
    let r = Rules::official(a.players);
    r.validate()?;
    Ok(r)
}

fn make_bots(names: &[String]) -> Result<Vec<Box<dyn Bot>>, String> {
    names
        .iter()
        .map(|n| bots::by_name(n).ok_or(format!("unknown bot '{n}'")))
        .collect()
}

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            if !e.is_empty() {
                eprintln!("{e}\n");
            }
            eprintln!("{USAGE}");
            std::process::exit(2);
        }
    };
    let res = match args.cmd.as_str() {
        "sim" => sim(&args),
        "play" => play(&args),
        _ => Err(format!("unknown command '{}'", args.cmd)),
    };
    if let Err(e) = res {
        eprintln!("{e}\n\n{USAGE}");
        std::process::exit(2);
    }
}

// ------------------------------------------------------------------------------------------- sim

fn sim(a: &Args) -> Result<(), String> {
    let rules = rules_for(a)?;
    let n = rules.players as usize;
    let names: Vec<String> = if a.bots.is_empty() {
        vec!["counting".into(); n]
    } else {
        a.bots.clone()
    };
    if names.len() != n {
        return Err(format!("--bots needs {n} names, got {}", names.len()));
    }
    let mut pool = make_bots(&names)?;
    let seed = a.seed.unwrap_or(1);
    let mut rng = Rng::new(seed);
    // Per bot (not per seat): bots rotate through the seats game by game.
    let mut total = vec![0i64; n];
    let mut made = vec![0u64; n];
    let mut rounds = vec![0u64; n];
    let mut wins = vec![0f64; n];
    let mut decisions = 0u64;
    let t0 = Instant::now();
    for g in 0..a.games {
        let shift = (g % n as u64) as usize;
        // seat s is played by bot (s + shift) % n
        let order: Vec<usize> = (0..n).map(|s| (s + shift) % n).collect();
        let mut seats: Vec<&mut dyn Bot> = Vec::with_capacity(n);
        {
            // Borrow the bots in seat order.
            let mut refs: Vec<Option<&mut Box<dyn Bot>>> = pool.iter_mut().map(Some).collect();
            for &b in &order {
                seats.push(refs[b].take().unwrap().as_mut());
            }
        }
        let res = play_game(rules, &mut seats, &mut rng);
        decisions += res.decisions;
        let best = *res.totals.iter().max().unwrap();
        let tied = res.totals.iter().filter(|&&t| t == best).count() as f64;
        for (seat, &b) in order.iter().enumerate() {
            total[b] += res.totals[seat] as i64;
            if res.totals[seat] == best {
                wins[b] += 1.0 / tied;
            }
            for r in &res.rounds {
                rounds[b] += 1;
                made[b] += (r.bids[seat] == r.won[seat]) as u64;
            }
        }
    }
    let secs = t0.elapsed().as_secs_f64();
    println!("{} games, {} players, seed {}", a.games, n, seed);
    println!(
        "{:<4} {:<10} {:>10} {:>9} {:>10}",
        "bot", "name", "avg score", "win %", "bid made"
    );
    for b in 0..n {
        println!(
            "{:<4} {:<10} {:>10.1} {:>8.1}% {:>9.1}%",
            b,
            names[b],
            total[b] as f64 / a.games as f64,
            100.0 * wins[b] / a.games as f64,
            100.0 * made[b] as f64 / rounds[b] as f64
        );
    }
    println!(
        "{:.2}s, {:.0} games/s, {:.2}M decisions/s (single thread)",
        secs,
        a.games as f64 / secs,
        decisions as f64 / secs / 1e6
    );
    Ok(())
}

// ------------------------------------------------------------------------------------------ play

/// The human player at the terminal.
struct Human {
    input: io::StdinLock<'static>,
}

fn sorted_hand(v: &View) -> Vec<Card> {
    let trump = v.trump();
    let mut hand: Vec<Card> = cards(v.hand()).collect();
    // Wizards first, then trump, then the other suits, high to low, Jesters last.
    let key = |c: &Card| -> (u8, u8, i16) {
        match card::kind(*c) {
            card::Kind::Wizard => (0, 0, 0),
            card::Kind::Jester => (3, 0, 0),
            card::Kind::Normal { suit, rank } => (
                if Some(suit) == trump { 1 } else { 2 },
                suit.index(),
                -(rank as i16),
            ),
        }
    };
    hand.sort_by_key(key);
    hand
}

impl Human {
    fn ask(&mut self, prompt: &str) -> String {
        print!("{prompt}");
        io::stdout().flush().ok();
        let mut line = String::new();
        if self.input.read_line(&mut line).unwrap_or(0) == 0 {
            println!("\nbye");
            std::process::exit(0);
        }
        line.trim().to_string()
    }
}

impl Bot for Human {
    fn name(&self) -> String {
        "you".into()
    }
    fn act(&mut self, v: &View, _rng: &mut Rng) -> Action {
        let hand = sorted_hand(v);
        let names: Vec<String> = hand
            .iter()
            .enumerate()
            .map(|(i, &c)| format!("{}:{}", i + 1, card::name(c)))
            .collect();
        match v.phase() {
            Phase::PickTrump { .. } => {
                println!("  your hand: {}", names.join("  "));
                loop {
                    let s = self.ask("  name trump (c/d/h/s): ");
                    if let Some(suit) = s.chars().next().and_then(Suit::from_char) {
                        return Action::PickTrump(suit);
                    }
                }
            }
            Phase::Bid { .. } => {
                let trump = v
                    .trump()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "none".into());
                let so_far: Vec<String> = (0..v.players())
                    .filter_map(|s| v.bids()[s as usize].map(|b| format!("P{s} {b}")))
                    .collect();
                println!(
                    "  trump: {trump} | bids so far: {}",
                    if so_far.is_empty() {
                        "none (you bid first)".into()
                    } else {
                        so_far.join(", ")
                    }
                );
                println!("  your hand: {}", names.join("  "));
                loop {
                    let s = self.ask(&format!("  your bid (0-{}): ", v.size()));
                    if let Ok(b) = s.parse::<u8>() {
                        if b <= v.size() {
                            return Action::Bid(b);
                        }
                    }
                }
            }
            Phase::Play { .. } => {
                let legal = v.legal_plays();
                let marked: Vec<String> = hand
                    .iter()
                    .enumerate()
                    .map(|(i, &c)| {
                        if legal & card::bit(c) != 0 {
                            format!("{}:{}", i + 1, card::name(c))
                        } else {
                            format!("({})", card::name(c))
                        }
                    })
                    .collect();
                println!(
                    "  you bid {}, won {} | your hand: {}",
                    v.my_bid().unwrap_or(0),
                    v.tricks_won(v.seat()),
                    marked.join("  ")
                );
                loop {
                    let s = self.ask("  play (number or card, e.g. 2 or As): ");
                    let c = s
                        .parse::<usize>()
                        .ok()
                        .and_then(|i| hand.get(i.wrapping_sub(1)).copied())
                        .or_else(|| card::parse(&s));
                    match c {
                        Some(c) if legal & card::bit(c) != 0 => return Action::Play(c),
                        Some(c) if v.hand() & card::bit(c) != 0 => {
                            println!("  you must follow suit")
                        }
                        _ => println!("  not a card in your hand"),
                    }
                }
            }
            Phase::Done => unreachable!(),
        }
    }
}

/// Wraps a bot so each of its moves is printed.
struct Loud {
    inner: Box<dyn Bot>,
    label: String,
}

impl Bot for Loud {
    fn name(&self) -> String {
        self.inner.name()
    }
    fn act(&mut self, v: &View, rng: &mut Rng) -> Action {
        let a = self.inner.act(v, rng);
        println!("  {} {}", self.label, a);
        a
    }
}

fn play(a: &Args) -> Result<(), String> {
    let rules = rules_for(a)?;
    let n = rules.players as usize;
    let names: Vec<String> = if a.bots.is_empty() {
        vec!["counting".into(); n - 1]
    } else {
        a.bots.clone()
    };
    if names.len() != n - 1 {
        return Err(format!(
            "--bots needs {} names for your opponents, got {}",
            n - 1,
            names.len()
        ));
    }
    let mut seats: Vec<Box<dyn Bot>> = Vec::new();
    seats.push(Box::new(Human {
        input: Box::leak(Box::new(io::stdin())).lock(),
    }));
    for (i, b) in make_bots(&names)?.into_iter().enumerate() {
        seats.push(Box::new(Loud {
            inner: b,
            label: format!("P{}", i + 1),
        }));
    }
    let seed = a.seed.unwrap_or_else(|| {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(7)
    });
    let mut rng = Rng::new(seed);
    let mut totals = vec![0i32; n];
    let mut dealer = rng.below(n as u64) as u8;
    let who = |s: u8| {
        if s == 0 {
            "you".to_string()
        } else {
            format!("P{s}")
        }
    };
    println!(
        "Wizard, {} players. You are seat 0; bots are P1..P{}.",
        n,
        n - 1
    );
    for size in 1..=rules.rounds() {
        let mut round = Round::deal(rules, size, dealer, &mut rng);
        println!(
            "\n=== Round {size}: {size} card{} each, {} deals ===",
            if size == 1 { "" } else { "s" },
            who(dealer)
        );
        match round.trump_source() {
            TrumpSource::TurnedCard(c) => println!(
                "turned up {}: {} are trump",
                card::name(c),
                round.trump().unwrap()
            ),
            TrumpSource::WizardTurned(_) => println!(
                "turned up a Wizard: the dealer ({}) names trump",
                who(dealer)
            ),
            TrumpSource::JesterTurned(_) => println!("turned up a Jester: no trump"),
            TrumpSource::NoCardTurned => println!("last round, no card to turn up: no trump"),
        }
        // Play it, printing trick results as they happen.
        {
            let mut refs: Vec<&mut dyn Bot> = seats.iter_mut().map(|b| b.as_mut()).collect();
            let mut done_tricks = 0;
            let mut shown_bids = false;
            while let Some(seat) = round.to_act() {
                if !shown_bids && matches!(round.phase(), Phase::Play { .. }) {
                    shown_bids = true;
                    let trump = round
                        .trump()
                        .map(|s| s.to_string())
                        .unwrap_or("none".into());
                    let bids: Vec<String> = (0..n as u8)
                        .map(|s| format!("{} {}", who(s), round.bid(s).unwrap()))
                        .collect();
                    println!(
                        "trump: {trump} | bids: {} (total {} of {size})",
                        bids.join(", "),
                        round.bids().iter().map(|b| b.unwrap() as u32).sum::<u32>()
                    );
                }
                if seat == 0 && matches!(round.phase(), Phase::Play { .. }) {
                    let t = round.current_trick();
                    let so_far: Vec<String> = t
                        .iter()
                        .map(|&(s, c)| format!("{} {}", who(s), card::name(c)))
                        .collect();
                    println!(
                        "  trick {}: {}",
                        done_tricks + 1,
                        if so_far.is_empty() {
                            "you lead".into()
                        } else {
                            so_far.join(", ")
                        }
                    );
                }
                let a = {
                    let v = View::new(&round, seat, &totals);
                    refs[seat as usize].act(&v, &mut rng)
                };
                round.apply(a).map_err(|e| e.to_string())?;
                if round.completed_tricks().len() > done_tricks {
                    let t = round.completed_tricks().last().unwrap();
                    done_tricks += 1;
                    let line: Vec<String> = t
                        .plays
                        .iter()
                        .map(|&(s, c)| format!("{} {}", who(s), card::name(c)))
                        .collect();
                    let taker = if t.winner == 0 {
                        "you take".to_string()
                    } else {
                        format!("{} takes", who(t.winner))
                    };
                    println!("  -> {taker} it ({})", line.join(", "));
                }
            }
        }
        let scores = round.scores().unwrap();
        for s in 0..n {
            totals[s] += scores[s];
        }
        let line: Vec<String> = (0..n as u8)
            .map(|s| {
                format!(
                    "{}: bid {} won {} {:+} = {}",
                    who(s),
                    round.bid(s).unwrap(),
                    round.tricks_won(s),
                    scores[s as usize],
                    totals[s as usize]
                )
            })
            .collect();
        println!("round {size} scores | {}", line.join(" | "));
        dealer = (dealer + 1) % n as u8;
    }
    let best = *totals.iter().max().unwrap();
    let winners: Vec<String> = (0..n as u8)
        .filter(|&s| totals[s as usize] == best)
        .map(who)
        .collect();
    println!("\nFinal: {:?}  winner: {}", totals, winners.join(" and "));
    println!("(seed {seed}: replay this deal sequence with --seed {seed})");
    Ok(())
}
