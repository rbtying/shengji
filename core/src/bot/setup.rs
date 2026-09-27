//! Drawing, bidding, burying the kitty and calling friends.

use rand::rngs::StdRng;

use shengji_mechanics::bidding::Bid;
use shengji_mechanics::types::{Card, EffectiveSuit, Number, PlayerID, Rank, Trump, ALL_SUITS};

use super::heuristics::{counts, hand_cards, points, sort_by_rank, subtract};
use super::Decision;
use crate::game_state::draw_phase::DrawPhase;
use crate::game_state::exchange_phase::ExchangePhase;
use crate::interactive::Action;
use crate::settings::{FriendSelection, GameMode, KittyTheftPolicy};

fn seat(players: &[shengji_mechanics::player::Player], id: PlayerID) -> Option<usize> {
    players.iter().position(|p| p.id == id)
}

pub(crate) fn draw_decision(d: &DrawPhase, me: PlayerID, _rng: &mut StdRng) -> Option<Decision> {
    if let Some(bid) = bid_choice(d, me) {
        return Some(Decision::new(
            Action::Bid(bid.card, bid.count),
            500,
            format!("bid {} x{}", bid.card.as_char(), bid.count),
        ));
    }

    if !d.deck.is_empty() {
        if d.propagated.players.get(d.position).map(|p| p.id) == Some(me) {
            return Some(Decision::new(Action::DrawCard, 200, "draw"));
        }
        return None;
    }

    // Done drawing. The landlord (or winning bidder) moves things along once others have had a
    // moment to bid.
    let landlord_rank = d
        .propagated
        .landlord
        .and_then(|l| d.propagated.players.iter().find(|p| p.id == l))
        .map(|p| p.rank());
    if d.bids.is_empty() && d.autobid.is_none() {
        if d.propagated.landlord == Some(me) {
            if landlord_rank == Some(Rank::NoTrump) {
                return Some(Decision::new(Action::PickUpKitty, 3000, "pick up kitty"));
            }
            if d.revealed_cards < d.kitty.len() {
                return Some(Decision::new(
                    Action::RevealCard,
                    4000,
                    "no bids; reveal from kitty",
                ));
            }
        }
        return None;
    }
    if d.next_player().ok() == Some(me) {
        return Some(Decision::new(Action::PickUpKitty, 4000, "pick up kitty"));
    }
    None
}

fn bid_choice(d: &DrawPhase, me: PlayerID) -> Option<Bid> {
    if d.revealed_cards > 0 || d.autobid.is_some() {
        return None;
    }
    let players = &d.propagated.players;
    let valid = Bid::valid_bids(
        me,
        &d.bids,
        &d.hands,
        players,
        d.propagated.landlord,
        0,
        d.propagated.bid_policy,
        d.propagated.bid_reinforcement_policy,
        d.propagated.joker_bid_policy,
        d.num_decks,
    )
    .ok()?;
    if valid.is_empty() {
        return None;
    }

    let winner = Bid::first_and_winner(&d.bids, d.autobid)
        .ok()
        .map(|(_, w)| w);
    if let Some(w) = winner {
        if w.id == me {
            return None;
        }
        // In tractor, don't outbid your partner across the table.
        if matches!(d.game_mode, GameMode::Tractor) {
            if let (Some(a), Some(b)) = (seat(players, me), seat(players, w.id)) {
                if a % 2 == b % 2 {
                    return None;
                }
            }
        }
    }

    let level = match d
        .propagated
        .landlord
        .or(Some(me))
        .and_then(|id| players.iter().find(|p| p.id == id))
        .map(|p| p.rank())?
    {
        Rank::Number(n) => n,
        Rank::NoTrump => return None,
    };

    let hand = hand_cards(&d.hands, me);
    let held: usize = hand.len();
    let total_cards = d.deck.len()
        + players
            .iter()
            .map(|p| {
                d.hands
                    .get(p.id)
                    .map(|h| h.values().sum::<usize>())
                    .unwrap_or(0)
            })
            .sum::<usize>();
    let final_hand = (total_cards / players.len().max(1)).max(1);
    let progress = held as f64 / final_hand as f64;

    let strength = |b: &Bid| -> f64 {
        let trump = match b.card {
            Card::SmallJoker | Card::BigJoker => Trump::NoTrump {
                number: Some(level),
            },
            Card::Suited { suit, .. } => Trump::Standard {
                suit,
                number: level,
            },
            Card::Unknown => return 0.0,
        };
        let trumps: Vec<&Card> = hand
            .iter()
            .filter(|c| trump.effective_suit(**c) == EffectiveSuit::Trump)
            .collect();
        let big = trumps
            .iter()
            .filter(|c| {
                c.is_joker() || c.number() == Some(level) || c.number() == Some(Number::Ace)
            })
            .count();
        (trumps.len() as f64 + 0.5 * big as f64) / held.max(1) as f64
    };

    let mut best: Option<(f64, Bid)> = None;
    for b in valid {
        let s = strength(&b);
        let better = match &best {
            None => true,
            Some((bs, bb)) => s > *bs + 1e-9 || ((s - *bs).abs() < 1e-9 && b.count < bb.count),
        };
        if better {
            best = Some((s, b));
        }
    }
    let (score, bid) = best?;

    // Nobody has bid, the deck is gone and there's no landlord yet: somebody has to.
    let forced = d.deck.is_empty() && winner.is_none() && d.propagated.landlord.is_none();
    let threshold = if winner.is_none() { 0.36 } else { 0.45 };
    if forced || (progress >= 0.4 && score >= threshold) {
        Some(bid)
    } else {
        None
    }
}

pub(crate) fn exchange_decision(e: &ExchangePhase, me: PlayerID) -> Option<Decision> {
    let theft =
        e.propagated.kitty_theft_policy == KittyTheftPolicy::AllowKittyTheft && e.autobid.is_none();
    if e.exchanger == me && !e.finalized {
        let hand = hand_cards(&e.hands, me);
        let mut all = hand.clone();
        all.extend(e.kitty.iter().copied().filter(|c| *c != Card::Unknown));
        // Hand order comes from a hash map and changes as cards move; sort so the choice of
        // cards to bury is stable while we move them one at a time.
        sort_by_rank(e.trump, &mut all);
        let desired = choose_bury(e.trump, &all, e.kitty_size);
        let extra_in_kitty = subtract(&e.kitty, &desired);
        if let Some(c) = extra_in_kitty.first() {
            return Some(Decision::new(Action::MoveCardToHand(*c), 60, "exchange"));
        }
        let missing = subtract(&desired, &e.kitty);
        if let Some(c) = missing.first() {
            return Some(Decision::new(Action::MoveCardToKitty(*c), 60, "exchange"));
        }
        if theft {
            return Some(Decision::new(Action::PutDownKitty, 800, "put down kitty"));
        }
    }
    if e.landlord == me {
        if let GameMode::FindingFriends {
            num_friends,
            friends,
        } = &e.game_mode
        {
            if friends.len() != *num_friends {
                if let Some(sel) = choose_friends(e, me) {
                    let note = sel
                        .iter()
                        .map(|f| format!("{} (skip {})", f.card.as_char(), f.initial_skip))
                        .collect::<Vec<_>>()
                        .join(", ");
                    return Some(Decision::new(
                        Action::SetFriends(sel),
                        600,
                        format!("call friends: {note}"),
                    ));
                }
                return None;
            }
        }
        let ready = e.kitty.len() == e.kitty_size && (!theft || e.finalized);
        if ready {
            return Some(Decision::new(Action::BeginPlay, 1500, "begin play"));
        }
    }
    None
}

/// Pick the cards to bury: void short side suits where that's cheap, then the least useful
/// side cards, keeping trump, aces and pairs, and avoiding points where possible.
pub(crate) fn choose_bury(trump: Trump, cards: &[Card], k: usize) -> Vec<Card> {
    let c = counts(cards);
    let is_trump = |x: &Card| trump.effective_suit(*x) == EffectiveSuit::Trump;
    let keep_value = |x: &Card| -> i64 {
        let rank = cards
            .iter()
            .filter(|o| trump.effective_suit(**o) == trump.effective_suit(*x))
            .filter(|o| trump.compare(**o, *x) == std::cmp::Ordering::Less)
            .count() as i64;
        let mut v = rank;
        if x.number() == Some(Number::Ace) {
            v += 40;
        }
        if x.number() == Some(Number::King) {
            v += 15;
        }
        if c.get(x).copied().unwrap_or(0) >= 2 {
            v += 30;
        }
        v += points(*x) as i64 * 2;
        v
    };

    let mut side: Vec<(EffectiveSuit, Vec<Card>)> = vec![];
    for x in cards.iter().filter(|x| !is_trump(x)) {
        let s = trump.effective_suit(*x);
        match side.iter_mut().find(|(ss, _)| *ss == s) {
            Some((_, v)) => v.push(*x),
            None => side.push((s, vec![*x])),
        }
    }
    side.sort_by_key(|(_, v)| v.len());

    let mut bury: Vec<Card> = vec![];
    for (_, v) in &side {
        let has_ace = v.iter().any(|x| x.number() == Some(Number::Ace));
        let has_pair = v.iter().any(|x| c.get(x).copied().unwrap_or(0) >= 2);
        let pts: usize = v.iter().map(|x| points(*x)).sum();
        if bury.len() + v.len() <= k && v.len() <= 4 && !has_ace && !has_pair && pts <= 10 {
            bury.extend(v.iter().copied());
        }
    }
    let mut rest: Vec<Card> = subtract(cards, &bury)
        .into_iter()
        .filter(|x| !is_trump(x))
        .collect();
    rest.sort_by_key(keep_value);
    for x in rest {
        if bury.len() >= k {
            break;
        }
        bury.push(x);
    }
    if bury.len() < k {
        let mut trumps: Vec<Card> = cards.iter().copied().filter(|x| is_trump(x)).collect();
        sort_by_rank(trump, &mut trumps);
        for x in trumps {
            if bury.len() >= k {
                break;
            }
            bury.push(x);
        }
    }
    bury.truncate(k);
    bury
}

/// Call an ace (or king) in a side suit we're short in; skip over the copies we hold ourselves.
fn choose_friends(e: &ExchangePhase, me: PlayerID) -> Option<Vec<FriendSelection>> {
    let num = e.num_friends();
    let hand = hand_cards(&e.hands, me);
    let mut options: Vec<(f64, FriendSelection)> = vec![];
    for suit in ALL_SUITS {
        for number in [Number::Ace, Number::King, Number::Queen] {
            let card = Card::Suited { suit, number };
            if e.trump.effective_suit(card) == EffectiveSuit::Trump {
                continue;
            }
            let mine = hand.iter().filter(|c| **c == card).count();
            if mine >= e.num_decks {
                continue;
            }
            let suit_len = hand
                .iter()
                .filter(|c| e.trump.effective_suit(**c) == e.trump.effective_suit(card))
                .count();
            let rank_penalty = match number {
                Number::Ace => 0.0,
                Number::King => 3.0,
                _ => 6.0,
            };
            let score = -(suit_len as f64) - rank_penalty - 1.5 * mine as f64;
            options.push((
                score,
                FriendSelection {
                    card,
                    initial_skip: mine,
                },
            ));
        }
    }
    options.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
    let top: Vec<FriendSelection> = options.into_iter().take(10).map(|(_, f)| f).collect();

    let mut chosen: Vec<FriendSelection> = vec![];
    fn search(
        e: &ExchangePhase,
        me: PlayerID,
        top: &[FriendSelection],
        start: usize,
        num: usize,
        chosen: &mut Vec<FriendSelection>,
    ) -> bool {
        if chosen.len() == num {
            let mut probe = e.clone();
            return probe.set_friends(me, chosen.iter().copied()).is_ok();
        }
        for i in start..top.len() {
            if chosen.iter().any(|c| c.card == top[i].card) {
                continue;
            }
            chosen.push(top[i]);
            if search(e, me, top, i + 1, num, chosen) {
                return true;
            }
            chosen.pop();
        }
        false
    }
    if search(e, me, &top, 0, num, &mut chosen) {
        Some(chosen)
    } else {
        None
    }
}
