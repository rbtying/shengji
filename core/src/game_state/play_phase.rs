use std::collections::HashMap;

use anyhow::{anyhow, bail, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shengji_mechanics::deck::Deck;
use shengji_mechanics::hands::Hands;
use shengji_mechanics::player::Player;
use shengji_mechanics::scoring::{compute_level_deltas, next_threshold_reachable, GameScoreResult};
use shengji_mechanics::trick::{PlayCards, PlayCardsMessage, Trick, TrickEnded, TrickUnit};
use shengji_mechanics::types::{Card, PlayerID, Rank, Trump};

use crate::message::MessageVariant;
use crate::settings::{
    AdvancementPolicy, GameMode, KittyPenalty, MultipleJoinPolicy, PlayTakebackPolicy,
    PropagatedState, ThrowPenalty,
};

use crate::game_state::initialize_phase::InitializePhase;

macro_rules! bail_unwrap {
    ($opt:expr) => {
        match $opt {
            Some(v) => v,
            None => return Err(anyhow!("option was none")),
        }
    };
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, JsonSchema, Eq, PartialEq)]
pub struct PlayerGameFinishedResult {
    pub won_game: bool,
    pub is_defending: bool,
    pub is_landlord: bool,
    pub ranks_up: usize,
    pub confetti: bool,
    pub rank: Rank,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PlayPhase {
    num_decks: usize,
    game_mode: GameMode,
    propagated: PropagatedState,
    hands: Hands,
    points: HashMap<PlayerID, Vec<Card>>,
    penalties: HashMap<PlayerID, usize>,
    kitty: Vec<Card>,
    landlord: PlayerID,
    landlords_team: Vec<PlayerID>,
    exchanger: PlayerID,
    trump: Trump,
    trick: Trick,
    last_trick: Option<Trick>,
    game_ended_early: bool,
    #[serde(default)]
    removed_cards: Vec<Card>,
    #[serde(default)]
    decks: Vec<Deck>,
    player_requested_reset: Option<PlayerID>,
}

impl PlayPhase {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        propagated: PropagatedState,
        num_decks: usize,
        game_mode: GameMode,
        hands: Hands,
        kitty: Vec<Card>,
        trump: Trump,
        landlord: PlayerID,
        exchanger: PlayerID,
        landlords_team: Vec<PlayerID>,
        removed_cards: Vec<Card>,
        decks: Vec<Deck>,
    ) -> Result<Self, Error> {
        let landlord_idx = bail_unwrap!(propagated.players.iter().position(|p| p.id == landlord));
        Ok(PlayPhase {
            trick: Trick::new(
                trump,
                (0..propagated.players.len()).map(|offset| {
                    let idx = (landlord_idx + offset) % propagated.players.len();
                    propagated.players[idx].id
                }),
            ),
            points: propagated
                .players
                .iter()
                .map(|p| (p.id, Vec::new()))
                .collect(),
            penalties: propagated.players.iter().map(|p| (p.id, 0)).collect(),
            num_decks,
            game_mode,
            hands,
            kitty,
            landlord,
            exchanger,
            landlords_team,
            trump,
            propagated,
            removed_cards,
            decks,
            game_ended_early: false,
            last_trick: None,
            player_requested_reset: None,
        })
    }

    pub fn add_observer(&mut self, name: String) -> Result<PlayerID, Error> {
        self.propagated.add_observer(name)
    }

    pub fn remove_observer(&mut self, id: PlayerID) -> Result<(), Error> {
        self.propagated.remove_observer(id)
    }

    pub fn next_player(&self) -> Result<PlayerID, Error> {
        Ok(bail_unwrap!(self.trick.next_player()))
    }

    pub fn game_mode(&self) -> &GameMode {
        &self.game_mode
    }

    pub fn landlords_team(&self) -> &[PlayerID] {
        &self.landlords_team
    }

    pub fn trick(&self) -> &Trick {
        &self.trick
    }

    pub fn hands(&self) -> &Hands {
        &self.hands
    }

    pub fn propagated(&self) -> &PropagatedState {
        &self.propagated
    }

    pub fn propagated_mut(&mut self) -> &mut PropagatedState {
        &mut self.propagated
    }

    pub fn can_play_cards(&self, id: PlayerID, cards: &[Card]) -> Result<(), Error> {
        if self.game_ended_early {
            bail!("Game has already ended; cards can't be played");
        }
        Ok(self
            .trick
            .can_play_cards(id, &self.hands, cards, self.propagated.trick_draw_policy)?)
    }

    pub fn play_cards(
        &mut self,
        id: PlayerID,
        cards: &[Card],
    ) -> Result<Vec<MessageVariant>, Error> {
        self.play_cards_with_hint(id, cards, None)
    }

    pub fn play_cards_with_hint(
        &mut self,
        id: PlayerID,
        cards: &[Card],
        format_hint: Option<&'_ [TrickUnit]>,
    ) -> Result<Vec<MessageVariant>, Error> {
        if self.game_ended_early {
            bail!("Game has already ended; cards can't be played");
        }

        let mut msgs = self.trick.play_cards(PlayCards {
            id,
            hands: &mut self.hands,
            cards,
            trick_draw_policy: self.propagated.trick_draw_policy,
            throw_eval_policy: self.propagated.throw_evaluation_policy,
            format_hint,
            hide_throw_halting_player: self.propagated.hide_throw_halting_player,
            tractor_requirements: self.propagated.tractor_requirements,
        })?;
        if self.propagated.hide_played_cards {
            for msg in &mut msgs {
                match msg {
                    PlayCardsMessage::PlayedCards { ref mut cards, .. } => {
                        for card in cards {
                            *card = Card::Unknown;
                        }
                    }
                    PlayCardsMessage::ThrowFailed {
                        ref mut original_cards,
                        ..
                    } => {
                        for card in original_cards {
                            *card = Card::Unknown;
                        }
                    }
                }
            }
        }
        Ok(msgs
            .into_iter()
            .map(|p| match p {
                PlayCardsMessage::ThrowFailed {
                    original_cards,
                    better_player,
                } => MessageVariant::ThrowFailed {
                    original_cards,
                    better_player,
                },
                PlayCardsMessage::PlayedCards { cards } => MessageVariant::PlayedCards { cards },
            })
            .collect())
    }

    pub fn take_back_cards(&mut self, id: PlayerID) -> Result<(), Error> {
        if self.game_ended_early {
            bail!("Game has already ended; cards can't be taken back");
        }
        if self.propagated.play_takeback_policy == PlayTakebackPolicy::NoPlayTakeback {
            bail!("Taking back played cards is not allowed")
        }
        Ok(self
            .trick
            .take_back(id, &mut self.hands, self.propagated.throw_evaluation_policy)?)
    }

    pub fn finish_trick(&mut self) -> Result<Vec<MessageVariant>, Error> {
        if self.game_ended_early {
            bail!("Game has already ended; trick can't be finished");
        }
        let TrickEnded {
            winner,
            points: mut new_points,
            largest_trick_unit_size,
            failed_throw_size,
        } = self.trick.complete()?;

        let kitty_multipler = match self.propagated.kitty_penalty {
            KittyPenalty::Times => 2 * largest_trick_unit_size,
            KittyPenalty::Power => 2usize.pow(largest_trick_unit_size as u32),
        };

        if failed_throw_size > 0 {
            match self.propagated.throw_penalty {
                ThrowPenalty::None => (),
                ThrowPenalty::TenPointsPerAttempt => {
                    if let Some(id) = self.trick.played_cards().first().map(|pc| pc.id) {
                        *self.penalties.entry(id).or_insert(0) += 10;
                    }
                }
            }
        }

        let mut msgs = vec![];
        if let GameMode::FindingFriends {
            ref mut friends, ..
        } = self.game_mode
        {
            for played in self.trick.played_cards() {
                for card in played.cards.iter() {
                    for friend in friends.iter_mut() {
                        if friend.card == *card {
                            if friend.skip == 0 {
                                if friend.player_id.is_none() {
                                    let already_on_the_team =
                                        self.landlords_team.contains(&played.id);

                                    match self.propagated.multiple_join_policy {
                                        MultipleJoinPolicy::Unrestricted if already_on_the_team => {
                                            // double-join!
                                            friend.player_id = Some(played.id);
                                            msgs.push(MessageVariant::JoinedTeam {
                                                player: played.id,
                                                already_joined: true,
                                            });
                                        }
                                        MultipleJoinPolicy::NoDoubleJoin if already_on_the_team => {
                                        }
                                        MultipleJoinPolicy::Unrestricted
                                        | MultipleJoinPolicy::NoDoubleJoin => {
                                            friend.player_id = Some(played.id);
                                            self.landlords_team.push(played.id);
                                            msgs.push(MessageVariant::JoinedTeam {
                                                player: played.id,
                                                already_joined: false,
                                            });
                                        }
                                    }
                                }
                            } else {
                                friend.skip -= 1;
                            }
                        }
                    }
                }
            }
        }
        let points = bail_unwrap!(self.points.get_mut(&winner));
        let kitty_points = self
            .kitty
            .iter()
            .filter(|c| c.points().is_some())
            .copied()
            .collect::<Vec<_>>();

        if self.hands.is_empty() {
            if self.propagated.should_reveal_kitty_at_end_of_game {
                msgs.push(MessageVariant::EndOfGameKittyReveal {
                    cards: self.kitty.clone(),
                });
            }
            for _ in 0..kitty_multipler {
                new_points.extend(kitty_points.iter().copied());
            }
            if !kitty_points.is_empty() && kitty_multipler > 0 {
                msgs.push(MessageVariant::PointsInKitty {
                    points: kitty_points.iter().flat_map(|c| c.points()).sum::<usize>(),
                    multiplier: kitty_multipler,
                });
            }
        }
        let winner_idx = bail_unwrap!(self.propagated.players.iter().position(|p| p.id == winner));
        if !new_points.is_empty() {
            let trump = self.trump;
            let num_points = new_points.iter().flat_map(|c| c.points()).sum::<usize>();
            points.extend(new_points);
            points.sort_by(|a, b| trump.compare(*a, *b));
            msgs.push(MessageVariant::TrickWon {
                winner: self.propagated.players[winner_idx].id,
                points: num_points,
            });
        } else {
            msgs.push(MessageVariant::TrickWon {
                winner: self.propagated.players[winner_idx].id,
                points: 0,
            });
        }
        let new_trick = Trick::new(
            self.trump,
            (0..self.propagated.players.len()).map(|offset| {
                let idx = (winner_idx + offset) % self.propagated.players.len();
                self.propagated.players[idx].id
            }),
        );
        self.last_trick = Some(std::mem::replace(&mut self.trick, new_trick));

        Ok(msgs)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compute_player_level_deltas<'a, 'b: 'a>(
        players: impl Iterator<Item = &'b mut Player>,
        non_landlord_level_bump: usize,
        landlord_level_bump: usize,
        landlords_team: &'a [PlayerID],
        landlord_won: bool,
        landlord: (PlayerID, Rank),
        advancement_policy: AdvancementPolicy,
        max_rank: Rank,
    ) -> Vec<MessageVariant> {
        let mut msgs = vec![];

        let result = players
            .map(|player| {
                let is_defending = landlords_team.contains(&player.id);
                let bump = if is_defending {
                    landlord_level_bump
                } else {
                    non_landlord_level_bump
                };
                let mut num_advances = 0;
                let mut was_blocked = false;
                let initial_rank = player.rank();

                for bump_idx in 0..bump {
                    let must_defend = match (advancement_policy, player.rank()) {
                        (AdvancementPolicy::Unrestricted, r)
                        | (AdvancementPolicy::Unrestricted, r)
                        | (AdvancementPolicy::DefendPoints, r)
                        | (AdvancementPolicy::DefendPoints, r)
                            if r == max_rank
                                || (r.successor() == Some(max_rank)
                                    && max_rank == Rank::NoTrump) =>
                        {
                            true
                        }
                        (AdvancementPolicy::DefendPoints, Rank::Number(n))
                            if n.points().is_some() =>
                        {
                            true
                        }
                        (AdvancementPolicy::FullyUnrestricted, _)
                        | (AdvancementPolicy::Unrestricted, _)
                        | (AdvancementPolicy::DefendPoints, _) => false,
                    };
                    // In order to advance past NoTrump, the landlord must also be defending
                    // NoTrump.
                    let landlord_must_defend = must_defend && player.rank() == Rank::NoTrump;

                    if must_defend
                        && (!is_defending
                            || bump_idx > 0
                            || (landlord_must_defend && landlord.1 != Rank::NoTrump))
                    {
                        was_blocked = true;
                        break;
                    }

                    player.advance(max_rank);
                    num_advances += 1;
                }
                if num_advances > 0 {
                    msgs.push(MessageVariant::RankAdvanced {
                        player: player.id,
                        new_rank: player.rank(),
                    });
                }
                if was_blocked {
                    msgs.push(MessageVariant::AdvancementBlocked {
                        player: player.id,
                        rank: player.rank(),
                    });
                }

                (
                    player.name.to_string(),
                    PlayerGameFinishedResult {
                        won_game: landlord_won == is_defending,
                        is_defending,
                        is_landlord: landlord.0 == player.id,
                        ranks_up: num_advances,
                        confetti: num_advances > 0
                            && landlord_won
                            && is_defending
                            && initial_rank == max_rank,
                        rank: initial_rank,
                    },
                )
            })
            .collect();

        msgs.push(MessageVariant::GameFinished { result });
        msgs
    }

    pub fn calculate_points(&self) -> (isize, isize) {
        let mut non_landlords_points: isize = self
            .points
            .iter()
            .filter(|(id, _)| !self.landlords_team.contains(id))
            .flat_map(|(_, cards)| cards)
            .flat_map(|c| c.points())
            .sum::<usize>() as isize;

        let observed_points = self
            .points
            .iter()
            .filter(|(id, _)| {
                !self.propagated.hide_landlord_points || !self.landlords_team.contains(id)
            })
            .flat_map(|(_, cards)| cards)
            .flat_map(|c| c.points())
            .sum::<usize>() as isize;

        for (id, penalty) in &self.penalties {
            if *penalty > 0 {
                if self.landlords_team.contains(id) {
                    non_landlords_points += *penalty as isize;
                } else {
                    non_landlords_points -= *penalty as isize;
                }
            }
        }
        (non_landlords_points, observed_points)
    }

    pub fn game_finished(&self) -> bool {
        self.game_ended_early || self.hands.is_empty() && self.trick.played_cards().is_empty()
    }

    pub fn finish_game_early(&mut self) -> Result<MessageVariant, Error> {
        if self.game_finished() {
            bail!("Game has already ended");
        }
        let (non_landlords_points, observed_points) = self.calculate_points();
        let can_end_early = !next_threshold_reachable(
            &self.propagated.game_scoring_parameters,
            &self.decks,
            non_landlords_points,
            observed_points,
        )?;

        if can_end_early {
            self.game_ended_early = true;
            Ok(MessageVariant::GameEndedEarly)
        } else {
            bail!("Game can't be ended early; there are still points in play")
        }
    }

    pub fn finish_game(&self) -> Result<(InitializePhase, bool, Vec<MessageVariant>), Error> {
        let mut msgs = vec![];
        if !self.game_finished() {
            bail!("not done playing yet!")
        }

        let (non_landlords_points, _) = self.calculate_points();

        let mut smaller_landlord_team = false;

        if let GameMode::FindingFriends {
            num_friends,
            friends: _,
        } = &self.game_mode
        {
            let setting_team_size = *num_friends + 1;

            let actual_team_size = self.landlords_team.len();
            smaller_landlord_team = actual_team_size < setting_team_size;
        }

        let mut propagated = self.propagated.clone();

        let GameScoreResult {
            non_landlord_delta: non_landlord_level_bump,
            landlord_delta: landlord_level_bump,
            landlord_won,
            landlord_bonus: bonus_level_earned,
        } = compute_level_deltas(
            &propagated.game_scoring_parameters,
            &self.decks,
            non_landlords_points,
            smaller_landlord_team,
        )?;

        msgs.push(MessageVariant::EndOfGameSummary {
            landlord_won,
            non_landlords_points,
        });

        if bonus_level_earned {
            msgs.push(MessageVariant::BonusLevelEarned);
        };

        let landlord_idx = bail_unwrap!(propagated
            .players
            .iter()
            .position(|p| p.id == self.landlord));

        msgs.extend(Self::compute_player_level_deltas(
            propagated.players.iter_mut(),
            non_landlord_level_bump,
            landlord_level_bump,
            &self.landlords_team[..],
            landlord_won,
            (self.landlord, self.propagated.players[landlord_idx].level),
            propagated.advancement_policy,
            *propagated.max_rank,
        ));

        let mut idx = (landlord_idx + 1) % propagated.players.len();
        let (next_landlord, next_landlord_idx) = loop {
            if landlord_won == self.landlords_team.contains(&propagated.players[idx].id) {
                break (propagated.players[idx].id, idx);
            }
            idx = (idx + 1) % propagated.players.len()
        };

        msgs.push(MessageVariant::NewLandlordForNextGame {
            landlord: propagated.players[next_landlord_idx].id,
        });
        propagated.set_landlord(Some(next_landlord))?;
        propagated.num_games_finished += 1;
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((
            InitializePhase::from_propagated(propagated),
            landlord_won,
            msgs,
        ))
    }

    pub fn request_reset(
        &mut self,
        player: PlayerID,
    ) -> Result<(Option<InitializePhase>, Vec<MessageVariant>), Error> {
        match self.player_requested_reset {
            Some(p) => {
                // ignore duplicate reset requests from same player
                if p == player {
                    return Ok((None, vec![]));
                }

                let (s, m) = self.return_to_initialize()?;
                Ok((Some(s), m))
            }
            None => {
                self.player_requested_reset = Some(player);
                Ok((None, vec![MessageVariant::ResetRequested]))
            }
        }
    }

    pub fn cancel_reset(&mut self) -> Option<MessageVariant> {
        if self.player_requested_reset.is_some() {
            self.player_requested_reset = None;
            return Some(MessageVariant::ResetCanceled);
        }
        None
    }

    fn return_to_initialize(&self) -> Result<(InitializePhase, Vec<MessageVariant>), Error> {
        let mut msgs = vec![MessageVariant::ResettingGame];

        let mut propagated = self.propagated.clone();
        msgs.extend(propagated.make_all_observers_into_players()?);

        Ok((InitializePhase::from_propagated(propagated), msgs))
    }

    pub fn destructively_redact_for_player(&mut self, player: PlayerID) {
        if self.propagated.hide_landlord_points {
            for (k, v) in self.points.iter_mut() {
                if self.landlords_team.contains(k) {
                    v.clear();
                }
            }
        }
        // Don't redact at the end of the game.
        let game_ongoing = !self.game_ended_early
            && (!self.hands.is_empty() || !self.trick.played_cards().is_empty());
        if game_ongoing {
            self.hands.destructively_redact_except_for_player(player);
        }
        if game_ongoing && player != self.exchanger {
            for card in &mut self.kitty {
                *card = Card::Unknown;
            }
        }
    }
}
