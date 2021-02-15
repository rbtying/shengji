use std::env;
use std::io::prelude::*;

use rand::distributions::Alphanumeric;
use rand::prelude::*;
use rand_distr::WeightedIndex;
use shengji_core::{
    game_state::{GameState, InitializePhase},
    settings::{FriendSelection, GameModeSettings},
    trick::{TrickUnit, UnitLike},
    types::{Card, EffectiveSuit, Number, Suit},
};
use std::collections::HashMap;

/// This simulates a (very dumb) shengji AI-driven game and writes the resulting game-states out to
/// the provided file. Initially developed to get a corpus of "valid" games to seed a zstd
/// dictionary.
fn main() {
    let args = env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("Usage: {} state-file-name", args[0]);
        std::process::exit(1);
    }
    println!("Simulating play! Outputting state to {}", args[1]);
    let mut f = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&args[1])
        .unwrap();

    let mut rng = rand::thread_rng();

    // Test 4-8 players, ony tractor
    let weights = [4, 0, 2, 0, 1];
    let player_dist = WeightedIndex::new(&weights).unwrap();

    // Test up to two extra decks, usually zero
    let deck_weights = [4, 2, 1];
    let deck_dist = WeightedIndex::new(&deck_weights).unwrap();

    loop {
        let mut initialize = InitializePhase::new();
        let num_players = 4 + player_dist.sample(&mut rng);
        let num_decks = num_players / 2 + deck_dist.sample(&mut rng);

        let mut players_ids = Vec::with_capacity(num_players);

        for _ in 0..num_players {
            let player_name = std::iter::repeat(())
                .map(|()| Alphanumeric.sample(&mut rng))
                .take(6)
                .collect();
            players_ids.push(initialize.add_player(player_name).unwrap().0);
        }

        initialize.set_num_decks(Some(num_decks)).unwrap();

        let is_finding_friends = num_players % 2 == 1 || rng.gen_range(0, 2) == 0;

        if is_finding_friends {
            initialize
                .set_game_mode(GameModeSettings::FindingFriends { num_friends: None })
                .unwrap();
            println!(
                "init: {} players, {} decks, finding friends",
                num_players, num_decks
            );
        } else {
            println!(
                "init: {} players, {} decks, tractor",
                num_players, num_decks
            );
        }

        // TODO: set other settings
        let mut game_state = GameState::Initialize(initialize);

        for num in 0..10 {
            println!("game {}", num + 1);

            loop {
                let game_finished = if let GameState::Play(ref s) = game_state {
                    s.game_finished()
                } else {
                    false
                };
                match game_state {
                    GameState::Initialize(ref mut s) => match s.landlord() {
                        None => {
                            s.set_landlord(players_ids.choose(&mut rng).copied())
                                .unwrap();
                        }
                        Some(landlord) => {
                            game_state = GameState::Draw(s.start(landlord).unwrap());
                        }
                    },
                    GameState::Draw(ref mut s) if !s.done_drawing() => {
                        s.draw_card(s.next_player().unwrap()).unwrap();
                    }
                    GameState::Draw(ref mut s) => {
                        // Always bid by revealing from the bottom
                        s.reveal_card().unwrap();
                        game_state =
                            GameState::Exchange(s.advance(s.next_player().unwrap()).unwrap());
                    }
                    GameState::Exchange(ref mut s) => {
                        // Don't exchange anything
                        if is_finding_friends {
                            // Viable friend cards
                            let mut viable_friends = vec![];
                            for suit in &[Suit::Clubs, Suit::Diamonds, Suit::Hearts, Suit::Spades] {
                                let c = Card::Suited {
                                    number: Number::Ace,
                                    suit: *suit,
                                };
                                if s.trump().effective_suit(c) != EffectiveSuit::Trump {
                                    for skip in 0..num_decks {
                                        viable_friends.push(FriendSelection {
                                            card: c,
                                            initial_skip: skip,
                                        });
                                    }
                                }
                            }
                            viable_friends.shuffle(&mut rng);
                            s.set_friends(
                                s.landlord(),
                                viable_friends[0..s.num_friends()].iter().copied(),
                            )
                            .unwrap();
                        }
                        game_state = GameState::Play(s.advance(s.next_player().unwrap()).unwrap());
                    }
                    GameState::Play(ref mut s)
                        if !game_finished && s.trick().played_cards().is_empty() =>
                    {
                        // Start the trick

                        // TODO: maybe some strategy here?
                        let p = s.next_player().unwrap();
                        let hand = s.hands().get(p).unwrap();
                        let cards = Card::cards(hand.iter()).copied().collect::<Vec<Card>>();

                        // Group cards by effective suit
                        let mut cards_by_suit = HashMap::new();
                        for card in cards {
                            cards_by_suit
                                .entry(s.trick().trump().effective_suit(card))
                                .or_insert_with(Vec::new)
                                .push(card);
                        }

                        let mut best_play = None;
                        for (_, cards) in cards_by_suit.into_iter() {
                            let results = TrickUnit::find_plays(s.trick().trump(), cards.clone());
                            let play = results
                                .into_iter()
                                // Never throw, so only pick one unit
                                .map(|play| play.into_iter().max_by_key(|u| u.size()).unwrap())
                                .max_by_key(|u| u.size())
                                .unwrap();
                            let play_cards = play.cards();
                            match best_play {
                                None => {
                                    best_play = Some(play_cards);
                                }
                                Some(b) if play_cards.len() > b.len() => {
                                    best_play = Some(play_cards);
                                }
                                Some(_) => (),
                            }
                        }

                        s.play_cards(p, &best_play.unwrap()).unwrap();
                    }
                    GameState::Play(ref mut s)
                        if !game_finished && s.trick().played_cards().len() < num_players =>
                    {
                        // Follow the required plays
                        let p = s.next_player().unwrap();
                        let hand = s.hands().get(p).unwrap().clone();
                        let trick_format = s.trick().trick_format().unwrap().clone();
                        let available_cards = Card::cards(hand.iter().filter(|(c, _)| {
                            trick_format.trump().effective_suit(**c) == trick_format.suit()
                        }))
                        .copied()
                        .collect::<Vec<_>>();

                        let matching_play = trick_format
                            .decomposition(Default::default())
                            .filter_map(|format| {
                                let (playable, units) = UnitLike::check_play(
                                    trick_format.trump(),
                                    available_cards.iter().copied(),
                                    format.iter().cloned(),
                                    s.propagated().trick_draw_policy(),
                                );
                                if playable {
                                    Some(
                                        units
                                            .into_iter()
                                            .flat_map(|x| {
                                                x.into_iter().flat_map(|(card, count)| {
                                                    std::iter::repeat(card.card).take(count)
                                                })
                                            })
                                            .collect::<Vec<_>>(),
                                    )
                                } else {
                                    None
                                }
                            })
                            .next();

                        let num_required = trick_format.size();
                        let mut play = match matching_play {
                            Some(matching) if matching.len() == num_required => matching,
                            Some(_) if num_required >= available_cards.len() => available_cards,
                            Some(mut matching) => {
                                // There are more available cards than required; we must at least
                                let mut available_cards = available_cards;
                                // pick the matching. Do this inefficiently!
                                for m in &matching {
                                    available_cards.remove(
                                        available_cards.iter().position(|c| *c == *m).unwrap(),
                                    );
                                }
                                available_cards.shuffle(&mut rng);
                                matching.extend(
                                    available_cards[0..num_required - matching.len()]
                                        .iter()
                                        .copied(),
                                );

                                matching
                            }
                            None => available_cards,
                        };
                        let required_other_cards = num_required - play.len();
                        if required_other_cards > 0 {
                            let mut other_cards = Card::cards(hand.iter().filter(|(c, _)| {
                                trick_format.trump().effective_suit(**c) != trick_format.suit()
                            }))
                            .copied()
                            .collect::<Vec<_>>();
                            other_cards.shuffle(&mut rng);
                            play.extend(other_cards[0..required_other_cards].iter().copied());
                        }
                        s.play_cards(p, &play).unwrap();
                    }
                    GameState::Play(ref mut s)
                        if !game_finished && s.trick().played_cards().len() == num_players =>
                    {
                        // Finish the trick
                        s.finish_trick().unwrap();
                    }
                    GameState::Play(ref mut s) => {
                        let (init, _, _) = s.finish_game().unwrap();
                        game_state = GameState::Initialize(init);
                        break;
                    }
                }
                let serialized = serde_json::to_vec(&game_state).unwrap();
                f.write_all(&serialized).unwrap();
                f.write_all(b"\n").unwrap();
            }
        }
    }
}
