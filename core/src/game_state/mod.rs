use std::ops::Deref;

use anyhow::{bail, Error};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use shengji_mechanics::types::PlayerID;

use crate::message::MessageVariant;
use crate::settings::PropagatedState;

pub mod draw_phase;
pub mod exchange_phase;
pub mod initialize_phase;
pub mod play_phase;

use draw_phase::DrawPhase;
use exchange_phase::ExchangePhase;
use initialize_phase::InitializePhase;
use play_phase::PlayPhase;

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum GameState {
    Initialize(InitializePhase),
    Draw(DrawPhase),
    Exchange(ExchangePhase),
    Play(PlayPhase),
}

impl GameState {
    pub fn next_player(&self) -> Result<PlayerID, Error> {
        match self {
            GameState::Play(p) => Ok(p.next_player()?),
            GameState::Draw(p) => Ok(p.next_player()?),
            GameState::Exchange(p) => Ok(p.next_player()?),
            _ => bail!("Not valid in this phase!"),
        }
    }

    pub fn propagated(&self) -> &'_ PropagatedState {
        match self {
            GameState::Initialize(p) => p.propagated(),
            GameState::Draw(p) => p.propagated(),
            GameState::Exchange(p) => p.propagated(),
            GameState::Play(p) => p.propagated(),
        }
    }

    pub fn is_player(&self, id: PlayerID) -> bool {
        self.propagated().players.iter().any(|p| p.id == id)
    }

    pub fn player_name(&self, id: PlayerID) -> Result<&'_ str, Error> {
        for p in &self.propagated().players {
            if p.id == id {
                return Ok(&p.name);
            }
        }
        for p in &self.propagated().observers {
            if p.id == id {
                return Ok(&p.name);
            }
        }
        bail!("Couldn't find player name")
    }

    pub fn player_id(&self, name: &str) -> Result<PlayerID, Error> {
        for p in &self.propagated().players {
            if p.name == name {
                return Ok(p.id);
            }
        }
        for p in &self.propagated().observers {
            if p.name == name {
                return Ok(p.id);
            }
        }
        bail!("Couldn't find player id")
    }

    pub fn register(&mut self, name: String) -> Result<(PlayerID, Vec<MessageVariant>), Error> {
        if let Ok(pid) = self.player_id(&name) {
            return Ok((
                pid,
                vec![MessageVariant::JoinedGameAgain {
                    player: pid,
                    game_shadowing_policy: self.game_shadowing_policy,
                }],
            ));
        }
        match self {
            GameState::Initialize(ref mut p) => p.add_player(name),
            GameState::Draw(ref mut p) => p.add_observer(name).map(|id| (id, vec![])),
            GameState::Exchange(ref mut p) => p.add_observer(name).map(|id| (id, vec![])),
            GameState::Play(ref mut p) => p.add_observer(name).map(|id| (id, vec![])),
        }
    }

    pub fn kick(&mut self, id: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        match self {
            GameState::Initialize(ref mut p) => p.remove_player(id),
            GameState::Draw(ref mut p) => p.remove_observer(id).map(|()| vec![]),
            GameState::Exchange(ref mut p) => p.remove_observer(id).map(|()| vec![]),
            GameState::Play(ref mut p) => p.remove_observer(id).map(|()| vec![]),
        }
    }

    pub fn set_chat_link(&mut self, chat_link: Option<String>) -> Result<(), Error> {
        match self {
            GameState::Initialize(ref mut p) => p.propagated_mut().set_chat_link(chat_link),
            GameState::Draw(ref mut p) => p.propagated_mut().set_chat_link(chat_link),
            GameState::Exchange(ref mut p) => p.propagated_mut().set_chat_link(chat_link),
            GameState::Play(ref mut p) => p.propagated_mut().set_chat_link(chat_link),
        }
    }

    pub fn request_reset(&mut self, player: PlayerID) -> Result<Vec<MessageVariant>, Error> {
        match self {
            GameState::Initialize(_) => bail!("Game has not started yet!"),
            GameState::Draw(ref mut p) => {
                let (s, m) = p.request_reset(player)?;
                if let Some(s) = s {
                    *self = GameState::Initialize(s);
                }
                Ok(m)
            }
            GameState::Exchange(ref mut p) => {
                let (s, m) = p.request_reset(player)?;
                if let Some(s) = s {
                    *self = GameState::Initialize(s);
                }
                Ok(m)
            }
            GameState::Play(ref mut p) => {
                let (s, m) = p.request_reset(player)?;
                if let Some(s) = s {
                    *self = GameState::Initialize(s);
                }
                Ok(m)
            }
        }
    }

    pub fn cancel_reset(&mut self) -> Result<Vec<MessageVariant>, Error> {
        match self {
            GameState::Initialize(_) => bail!("Game has not started yet!"),
            GameState::Draw(ref mut p) => {
                if let Some(m) = p.cancel_reset() {
                    return Ok(vec![m]);
                }
            }
            GameState::Exchange(ref mut p) => {
                if let Some(m) = p.cancel_reset() {
                    return Ok(vec![m]);
                }
            }
            GameState::Play(ref mut p) => {
                if let Some(m) = p.cancel_reset() {
                    return Ok(vec![m]);
                }
            }
        }
        Ok(vec![])
    }

    pub fn for_player(&self, id: PlayerID) -> GameState {
        let mut s = self.clone();
        match s {
            GameState::Initialize { .. } => (),
            GameState::Draw(ref mut p) => {
                p.destructively_redact_for_player(id);
            }
            GameState::Exchange(ref mut p) => {
                p.destructively_redact_for_player(id);
            }
            GameState::Play(ref mut p) => {
                p.destructively_redact_for_player(id);
            }
        }
        s
    }
}

impl Deref for GameState {
    type Target = PropagatedState;

    fn deref(&self) -> &PropagatedState {
        self.propagated()
    }
}

#[cfg(test)]
mod tests {
    use crate::settings::{
        AdvancementPolicy, BackToTwoSetting, FriendSelection, FriendSelectionPolicy, GameMode,
        GameModeSettings, KittyTheftPolicy,
    };

    use shengji_mechanics::player::Player;
    use shengji_mechanics::types::{cards, Card, Number, PlayerID, Rank, Suit, Trump, FULL_DECK};

    use crate::game_state::{initialize_phase::InitializePhase, play_phase::PlayPhase};
    use crate::message::MessageVariant;

    use shengji_mechanics::hands::Hands;
    use shengji_mechanics::trick::{
        PlayCards, ThrowEvaluationPolicy, TractorRequirements, Trick, TrickDrawPolicy,
    };

    const R2: Rank = Rank::Number(Number::Two);
    const R3: Rank = Rank::Number(Number::Three);
    const R4: Rank = Rank::Number(Number::Four);
    const R5: Rank = Rank::Number(Number::Five);
    const R6: Rank = Rank::Number(Number::Six);
    const R7: Rank = Rank::Number(Number::Seven);
    const R8: Rank = Rank::Number(Number::Eight);
    const R9: Rank = Rank::Number(Number::Nine);
    const R10: Rank = Rank::Number(Number::Ten);
    const RJ: Rank = Rank::Number(Number::Jack);
    const RQ: Rank = Rank::Number(Number::Queen);
    const RK: Rank = Rank::Number(Number::King);
    const RA: Rank = Rank::Number(Number::Ace);
    const RNT: Rank = Rank::NoTrump;

    const P1: PlayerID = PlayerID(0);
    const P2: PlayerID = PlayerID(1);
    const P3: PlayerID = PlayerID(2);
    const P4: PlayerID = PlayerID(3);

    macro_rules! pc {
        ($id:expr, $hands:expr, $cards:expr) => {
            PlayCards {
                id: $id,
                hands: $hands,
                cards: $cards,
                trick_draw_policy: TrickDrawPolicy::NoProtections,
                throw_eval_policy: ThrowEvaluationPolicy::All,
                format_hint: None,
                hide_throw_halting_player: false,
                tractor_requirements: TractorRequirements::default(),
            }
        };
    }

    const JACK_TRUMP: Trump = Trump::Standard {
        number: Number::Jack,
        suit: Suit::Spades,
    };

    fn init_players() -> Vec<Player> {
        vec![
            Player {
                id: PlayerID(0),
                name: "p1".into(),
                level: R2,
                metalevel: 0,
            },
            Player {
                id: PlayerID(1),
                name: "p2".into(),
                level: R2,
                metalevel: 0,
            },
            Player {
                id: PlayerID(2),
                name: "p3".into(),
                level: R2,
                metalevel: 0,
            },
            Player {
                id: PlayerID(3),
                name: "p4".into(),
                level: R2,
                metalevel: 0,
            },
        ]
    }

    #[test]
    fn test_must_defend_sequence_landlord_advancing() {
        let initial_players = init_players();

        let base_sequence = vec![
            vec![R3, R2, R3, R2],
            vec![R4, R2, R4, R2],
            vec![R5, R2, R5, R2],
            vec![R6, R2, R6, R2],
            vec![R7, R2, R7, R2],
            vec![R8, R2, R8, R2],
            vec![R9, R2, R9, R2],
            vec![R10, R2, R10, R2],
            vec![RJ, R2, RJ, R2],
            vec![RQ, R2, RQ, R2],
            vec![RK, R2, RK, R2],
            vec![RA, R2, RA, R2],
            vec![RNT, R2, RNT, R2],
            vec![R2, R2, R2, R2],
        ];

        let tbl = [
            (AdvancementPolicy::Unrestricted, &base_sequence),
            (AdvancementPolicy::FullyUnrestricted, &base_sequence),
            (AdvancementPolicy::DefendPoints, &base_sequence),
        ];

        for (advance_policy, expected_seq) in tbl {
            let mut p = initial_players.clone();

            for v in expected_seq {
                let starting_rank = p[0].rank();
                let _ = PlayPhase::compute_player_level_deltas(
                    p.iter_mut(),
                    // Level up one at a time, landlord is always winning
                    0,
                    1,
                    &[PlayerID(0), PlayerID(2)],
                    true,
                    (PlayerID(0), starting_rank),
                    advance_policy,
                    RNT,
                    None,
                    BackToTwoSetting::Disabled,
                );
                let ranks = p.iter().map(|pp| pp.rank()).collect::<Vec<Rank>>();
                assert_eq!(
                    &ranks, v,
                    "Starting rank: {advance_policy:?} / {starting_rank:?}"
                );
            }
        }
    }

    #[test]
    fn test_must_defend_sequence_landlord_advancing_no_nt() {
        let initial_players = init_players();

        let base_sequence = vec![
            vec![R3, R2, R3, R2],
            vec![R4, R2, R4, R2],
            vec![R5, R2, R5, R2],
            vec![R6, R2, R6, R2],
            vec![R7, R2, R7, R2],
            vec![R8, R2, R8, R2],
            vec![R9, R2, R9, R2],
            vec![R10, R2, R10, R2],
            vec![RJ, R2, RJ, R2],
            vec![RQ, R2, RQ, R2],
            vec![RK, R2, RK, R2],
            vec![RA, R2, RA, R2],
            vec![R2, R2, R2, R2],
        ];

        let tbl = [
            (AdvancementPolicy::Unrestricted, &base_sequence),
            (AdvancementPolicy::FullyUnrestricted, &base_sequence),
            (AdvancementPolicy::DefendPoints, &base_sequence),
        ];

        for (advance_policy, expected_seq) in tbl {
            let mut p = initial_players.clone();

            for v in expected_seq {
                let starting_rank = p[0].rank();
                let _ = PlayPhase::compute_player_level_deltas(
                    p.iter_mut(),
                    // Level up one at a time, landlord is always winning
                    0,
                    1,
                    &[PlayerID(0), PlayerID(2)],
                    true,
                    (PlayerID(0), starting_rank),
                    advance_policy,
                    RA,
                    None,
                    BackToTwoSetting::Disabled,
                );
                let ranks = p.iter().map(|pp| pp.rank()).collect::<Vec<Rank>>();
                assert_eq!(
                    &ranks, v,
                    "Starting rank: {advance_policy:?} / {starting_rank:?}"
                );
            }
        }
    }

    #[test]
    fn test_must_defend_sequence_landlord_advancing_skip_by_2() {
        let initial_players = init_players();

        let tbl = [
            (
                AdvancementPolicy::Unrestricted,
                vec![
                    vec![R4, R2, R4, R2],
                    vec![R6, R2, R6, R2],
                    vec![R8, R2, R8, R2],
                    vec![R10, R2, R10, R2],
                    vec![RQ, R2, RQ, R2],
                    vec![RA, R2, RA, R2],
                    vec![RNT, R2, RNT, R2],
                    vec![R3, R2, R3, R2],
                ],
            ),
            (
                AdvancementPolicy::FullyUnrestricted,
                vec![
                    vec![R4, R2, R4, R2],
                    vec![R6, R2, R6, R2],
                    vec![R8, R2, R8, R2],
                    vec![R10, R2, R10, R2],
                    vec![RQ, R2, RQ, R2],
                    vec![RA, R2, RA, R2],
                    vec![R2, R2, R2, R2],
                ],
            ),
            (
                AdvancementPolicy::DefendPoints,
                vec![
                    vec![R4, R2, R4, R2],
                    vec![R5, R2, R5, R2],
                    vec![R7, R2, R7, R2],
                    vec![R9, R2, R9, R2],
                    vec![R10, R2, R10, R2],
                    vec![RQ, R2, RQ, R2],
                    vec![RK, R2, RK, R2],
                    vec![RA, R2, RA, R2],
                    vec![RNT, R2, RNT, R2],
                    vec![R3, R2, R3, R2],
                ],
            ),
        ];

        for (advance_policy, expected_seq) in tbl {
            let mut p = initial_players.clone();

            for v in expected_seq {
                let starting_rank = p[0].rank();
                let _ = PlayPhase::compute_player_level_deltas(
                    p.iter_mut(),
                    // Level up two at a time, landlord is always winning
                    0,
                    2,
                    &[PlayerID(0), PlayerID(2)],
                    true,
                    (PlayerID(0), starting_rank),
                    advance_policy,
                    RNT,
                    None,
                    BackToTwoSetting::Disabled,
                );
                let ranks = p.iter().map(|pp| pp.rank()).collect::<Vec<Rank>>();
                assert_eq!(
                    ranks, v,
                    "Starting rank: {advance_policy:?} / {starting_rank:?}"
                );
            }
        }
    }

    #[test]
    fn test_must_defend_sequence_non_landlord_advancing() {
        let initial_players = init_players();

        let tbl = [
            (
                AdvancementPolicy::Unrestricted,
                vec![
                    vec![R2, R4, R2, R4],
                    vec![R2, R6, R2, R6],
                    vec![R2, R8, R2, R8],
                    vec![R2, R10, R2, R10],
                    vec![R2, RQ, R2, RQ],
                    vec![R2, RA, R2, RA],
                    // Get stuck at A b/c not landlord
                    vec![R2, RA, R2, RA],
                ],
            ),
            (
                AdvancementPolicy::FullyUnrestricted,
                vec![
                    vec![R2, R4, R2, R4],
                    vec![R2, R6, R2, R6],
                    vec![R2, R8, R2, R8],
                    vec![R2, R10, R2, R10],
                    vec![R2, RQ, R2, RQ],
                    vec![R2, RA, R2, RA],
                    vec![R2, R2, R2, R2],
                ],
            ),
            (
                AdvancementPolicy::DefendPoints,
                vec![
                    vec![R2, R4, R2, R4],
                    vec![R2, R5, R2, R5],
                    // Get stuck at 5 because not landlord
                    vec![R2, R5, R2, R5],
                ],
            ),
        ];

        for (advance_policy, expected_seq) in tbl {
            let mut p = initial_players.clone();

            for v in expected_seq {
                let starting_rank = p[1].rank();
                let p0_rank = p[0].rank();
                let _ = PlayPhase::compute_player_level_deltas(
                    p.iter_mut(),
                    // Level up two at a time, landlord is always losing
                    2,
                    0,
                    &[PlayerID(0), PlayerID(2)],
                    true,
                    (PlayerID(0), p0_rank),
                    advance_policy,
                    RNT,
                    None,
                    BackToTwoSetting::Disabled,
                );
                let ranks = p.iter().map(|pp| pp.rank()).collect::<Vec<Rank>>();
                assert_eq!(
                    ranks, v,
                    "Starting rank: {advance_policy:?} / {starting_rank:?}"
                );
            }
        }
    }

    #[test]
    fn test_must_defend_sequence_landlord_must_defend_to_win() {
        let mut p = init_players();
        p[2].level = RNT;

        let p0_rank = p[0].rank();
        let _ = PlayPhase::compute_player_level_deltas(
            p.iter_mut(),
            0,
            // Level up 2, but the landlord is on a lower rank.
            2,
            &[PlayerID(0), PlayerID(2)],
            true,
            (PlayerID(0), p0_rank),
            AdvancementPolicy::Unrestricted,
            RNT,
            None,
            BackToTwoSetting::Disabled,
        );
        let ranks = p.iter().map(|pp| pp.rank()).collect::<Vec<Rank>>();
        assert_eq!(ranks, vec![R4, R2, RNT, R2],);

        p[0].level = RNT;

        let p0_rank = p[0].rank();
        let _ = PlayPhase::compute_player_level_deltas(
            p.iter_mut(),
            0,
            // Level up 2, but this time the landlord is also NT.
            2,
            &[PlayerID(0), PlayerID(2)],
            true,
            (PlayerID(0), p0_rank),
            AdvancementPolicy::Unrestricted,
            RNT,
            None,
            BackToTwoSetting::Disabled,
        );
        let ranks = p.iter().map(|pp| pp.rank()).collect::<Vec<Rank>>();
        assert_eq!(ranks, vec![R3, R2, R3, R2],);
    }

    #[test]
    fn test_player_level_deltas() {
        let mut players = init_players();

        let _ = PlayPhase::compute_player_level_deltas(
            players.iter_mut(),
            // Pretend both sides are leveling up somehow.
            2,
            2,
            &[PlayerID(0), PlayerID(2)],
            true,
            (PlayerID(0), R5),
            AdvancementPolicy::Unrestricted,
            RNT,
            None,
            BackToTwoSetting::Disabled,
        );
        for p in &players {
            assert_eq!(p.rank(), Rank::Number(Number::Four));
        }

        let _ = PlayPhase::compute_player_level_deltas(
            players.iter_mut(),
            // Pretend both sides are leveling up somehow.
            2,
            2,
            &[PlayerID(0), PlayerID(2)],
            true,
            (PlayerID(0), Rank::Number(Number::Ace)),
            AdvancementPolicy::DefendPoints,
            RNT,
            None,
            BackToTwoSetting::Disabled,
        );
        for p in &players {
            assert_eq!(p.rank(), R5);
        }

        // Advance again!
        let _ = PlayPhase::compute_player_level_deltas(
            players.iter_mut(),
            // Pretend both sides are leveling up somehow.
            2,
            2,
            &[PlayerID(0), PlayerID(2)],
            true,
            (PlayerID(0), RA),
            AdvancementPolicy::DefendPoints,
            RNT,
            None,
            BackToTwoSetting::Disabled,
        );
        for p in &players {
            if p.id == PlayerID(0) || p.id == PlayerID(2) {
                assert_eq!(p.rank(), R7);
            } else {
                assert_eq!(p.rank(), R5);
            }
        }

        // Advance again!
        let _ = PlayPhase::compute_player_level_deltas(
            players.iter_mut(),
            // Pretend both sides are leveling up somehow.
            2,
            2,
            &[PlayerID(0), PlayerID(2)],
            true,
            (PlayerID(0), Rank::Number(Number::Ace)),
            AdvancementPolicy::DefendPoints,
            RNT,
            None,
            BackToTwoSetting::Disabled,
        );

        for p in &players {
            if p.id == PlayerID(0) || p.id == PlayerID(2) {
                assert_eq!(p.rank(), R9);
            } else {
                assert_eq!(p.rank(), R5);
            }
        }
    }

    #[test]
    fn test_jack_variation_landlord_loses() {
        use cards::*;

        let mut players = init_players();
        let mut hands = Hands::new(vec![P1, P2, P3, P4]);
        hands.add(P1, vec![S_2]).unwrap();
        hands.add(P2, vec![S_J]).unwrap();
        hands.add(P3, vec![S_2]).unwrap();
        hands.add(P4, vec![S_3]).unwrap();
        let mut trick = Trick::new(JACK_TRUMP, vec![P1, P2, P3, P4]);
        trick.play_cards(pc!(P1, &mut hands, &[S_2])).unwrap();
        trick.play_cards(pc!(P2, &mut hands, &[S_J])).unwrap();
        trick.play_cards(pc!(P3, &mut hands, &[S_2])).unwrap();
        trick.play_cards(pc!(P4, &mut hands, &[S_3])).unwrap();

        // Neither side levels up, but the non-landlord team wins the final trick with
        // a single jack
        let _ = PlayPhase::compute_player_level_deltas(
            players.iter_mut(),
            0,
            0,
            &[PlayerID(0), PlayerID(2)],
            false, // landlord team does not defend
            (PlayerID(0), Rank::Number(Number::Jack)),
            AdvancementPolicy::DefendPoints,
            RNT,
            Some(trick),
            BackToTwoSetting::SingleJack,
        );

        for p in &players {
            assert_eq!(p.rank(), R2);
        }
    }

    #[test]
    fn test_jack_variation_landlord_advances_multiple() {
        use cards::*;

        let mut players = init_players();
        let mut hands = Hands::new(vec![P1, P2, P3, P4]);
        hands.add(P1, vec![S_2]).unwrap();
        hands.add(P2, vec![S_J]).unwrap();
        hands.add(P3, vec![S_2]).unwrap();
        hands.add(P4, vec![S_3]).unwrap();
        let mut trick = Trick::new(JACK_TRUMP, vec![P1, P2, P3, P4]);
        trick.play_cards(pc!(P1, &mut hands, &[S_2])).unwrap();
        trick.play_cards(pc!(P2, &mut hands, &[S_J])).unwrap();
        trick.play_cards(pc!(P3, &mut hands, &[S_2])).unwrap();
        trick.play_cards(pc!(P4, &mut hands, &[S_3])).unwrap();

        // The landlord team defends, but the non-landlord team wins the final trick with
        // a single jack
        let _ = PlayPhase::compute_player_level_deltas(
            players.iter_mut(),
            0,
            2,
            &[PlayerID(0), PlayerID(2)],
            true, // landlord team defends
            (PlayerID(0), Rank::Number(Number::Jack)),
            AdvancementPolicy::DefendPoints,
            RNT,
            Some(trick),
            BackToTwoSetting::SingleJack,
        );

        for p in &players {
            if p.id == PlayerID(0) || p.id == PlayerID(2) {
                assert_eq!(p.rank(), R4);
            } else {
                assert_eq!(p.rank(), R2);
            }
        }
    }

    #[test]
    fn test_jack_variation_non_landlord_advances() {
        use cards::*;

        let mut players = init_players();
        let mut hands = Hands::new(vec![P1, P2, P3, P4]);
        hands.add(P1, vec![S_2]).unwrap();
        hands.add(P2, vec![S_J]).unwrap();
        hands.add(P3, vec![S_2]).unwrap();
        hands.add(P4, vec![S_3]).unwrap();
        let mut trick = Trick::new(JACK_TRUMP, vec![P1, P2, P3, P4]);
        trick.play_cards(pc!(P1, &mut hands, &[S_2])).unwrap();
        trick.play_cards(pc!(P2, &mut hands, &[S_J])).unwrap();
        trick.play_cards(pc!(P3, &mut hands, &[S_2])).unwrap();
        trick.play_cards(pc!(P4, &mut hands, &[S_3])).unwrap();

        // The non-landlord team advances and they win the final trick with
        // a single jack
        let _ = PlayPhase::compute_player_level_deltas(
            players.iter_mut(),
            2,
            0,
            &[PlayerID(0), PlayerID(2)],
            false, // landlord team does not defend
            (PlayerID(0), Rank::Number(Number::Jack)),
            AdvancementPolicy::DefendPoints,
            RNT,
            Some(trick),
            BackToTwoSetting::SingleJack,
        );

        for p in &players {
            if p.id == PlayerID(0) || p.id == PlayerID(2) {
                assert_eq!(p.rank(), R2);
            } else {
                assert_eq!(p.rank(), R4);
            }
        }
    }

    #[test]
    fn test_unusual_kitty_sizes() {
        let mut init = InitializePhase::new();
        let p1 = init.add_player("p1".into()).unwrap().0;
        init.add_player("p2".into()).unwrap();
        init.add_player("p3".into()).unwrap();
        init.set_game_mode(GameModeSettings::FindingFriends { num_friends: None })
            .unwrap();
        for n_players in 4..10 {
            init.add_player(format!("p{n_players}")).unwrap();
            for n_decks in 1..n_players {
                for kitty_size in 1..30 {
                    let mut init_ = init.clone();
                    init_.set_num_decks(Some(n_decks)).unwrap();
                    if init_.set_kitty_size(Some(kitty_size)).is_ok() {
                        let draw = init_.start(p1).unwrap();
                        assert_eq!(draw.deck().len() % n_players, 0);
                        assert_eq!(draw.kitty().len(), kitty_size);
                        assert_eq!(
                            draw.removed_cards().len() + draw.deck().len() + draw.kitty().len(),
                            n_decks * FULL_DECK.len()
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn test_bid_sequence() {
        let mut init = InitializePhase::new();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        let mut draw = init.start(PlayerID(0)).unwrap();
        // Hackily ensure that everyone can bid.
        *draw.deck_mut() = vec![
            cards::S_2,
            Card::SmallJoker,
            Card::BigJoker,
            cards::H_2,
            cards::S_2,
            Card::SmallJoker,
            Card::BigJoker,
            cards::H_2,
        ];
        *draw.position_mut() = 0;

        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();
        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();

        assert!(draw.bid(p1, cards::H_2, 1));
        assert!(draw.bid(p1, cards::H_2, 2));
        assert!(draw.bid(p3, Card::SmallJoker, 2));
        assert!(draw.bid(p2, Card::BigJoker, 2));
        assert!(!draw.bid(p1, cards::H_2, 2));
    }

    #[test]
    fn test_kitty_stealing_bid_sequence() {
        let mut init = InitializePhase::new();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        init.set_kitty_theft_policy(KittyTheftPolicy::AllowKittyTheft)
            .unwrap();
        let mut draw = init.start(PlayerID(0)).unwrap();
        // Hackily ensure that everyone can bid.
        *draw.deck_mut() = vec![
            cards::S_2,
            Card::SmallJoker,
            Card::BigJoker,
            cards::H_2,
            cards::S_2,
            Card::SmallJoker,
            Card::BigJoker,
            cards::H_2,
        ];
        *draw.position_mut() = 0;

        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();
        draw.draw_card(p1).unwrap();
        draw.draw_card(p2).unwrap();
        draw.draw_card(p3).unwrap();
        draw.draw_card(p4).unwrap();

        assert!(draw.bid(p1, cards::H_2, 1));
        let mut exchange = draw.advance(p1).unwrap();
        exchange.finalize(p1).unwrap();
        assert!(exchange.bid(p1, cards::H_2, 2));
        assert!(exchange.bid(p3, Card::SmallJoker, 2));
        exchange.pick_up_cards(p3).unwrap();
        exchange.advance(p1).unwrap_err();
        exchange.finalize(p3).unwrap();
        assert!(exchange.bid(p2, Card::BigJoker, 2));
        exchange.pick_up_cards(p2).unwrap();
        exchange.finalize(p2).unwrap();
        assert!(!exchange.bid(p1, cards::H_2, 2));
        exchange.advance(p1).unwrap();
    }

    #[test]
    fn test_tuple_protection_case() {
        use cards::*;

        let mut init = InitializePhase::new();
        init.set_trick_draw_policy(
            shengji_mechanics::trick::TrickDrawPolicy::LongerTuplesProtected,
        )
        .unwrap();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        let mut draw = init.start(PlayerID(0)).unwrap();

        let p1_hand = [S_9, S_9, S_10, S_10, S_K, S_3, S_4, S_5, S_7, S_7, H_2];
        let p2_hand = [S_3, S_3, S_5, S_5, S_7, S_8, S_J, S_Q, C_3, C_4, C_5];
        let p3_hand = [S_3, S_5, S_10, S_J, S_Q, S_6, S_8, S_8, S_8, C_6, C_7];
        let p4_hand = [S_6, S_6, S_6, C_8, C_9, C_10, C_J, C_Q, C_K, C_A, C_A];

        let mut deck = vec![];
        for i in 0..11 {
            deck.push(p1_hand[i]);
            deck.push(p2_hand[i]);
            deck.push(p3_hand[i]);
            deck.push(p4_hand[i]);
        }
        deck.reverse();
        *draw.deck_mut() = deck;
        *draw.position_mut() = 0;

        for _ in 0..11 {
            draw.draw_card(p1).unwrap();
            draw.draw_card(p2).unwrap();
            draw.draw_card(p3).unwrap();
            draw.draw_card(p4).unwrap();
        }

        assert!(draw.bid(p1, cards::H_2, 1));

        let exchange = draw.advance(p1).unwrap();
        let mut play = exchange.advance(p1).unwrap();
        play.play_cards(p1, &[S_9, S_9, S_10, S_10, S_K]).unwrap();
        play.play_cards(p2, &[S_3, S_3, S_5, S_5, S_7]).unwrap();
        play.play_cards(p3, &[S_3, S_5, S_10, S_J, S_Q]).unwrap();
        play.play_cards(p4, &[S_6, S_6, S_6, C_8, C_9]).unwrap();
    }

    #[test]
    fn test_set_friends() {
        use cards::*;

        let setup_exchange = |friend_selection_policy, bid: Card| {
            let mut init = InitializePhase::new();
            init.set_game_mode(GameModeSettings::FindingFriends { num_friends: None })
                .unwrap();
            init.set_friend_selection_policy(friend_selection_policy)
                .unwrap();
            let p1 = init.add_player("p1".into()).unwrap().0;
            let p2 = init.add_player("p2".into()).unwrap().0;
            let p3 = init.add_player("p3".into()).unwrap().0;
            let p4 = init.add_player("p4".into()).unwrap().0;
            init.set_landlord(Some(p2)).unwrap();
            init.set_rank(p2, Rank::Number(bid.number().unwrap()))
                .unwrap();

            let mut draw = init.start(PlayerID(1)).unwrap();
            *draw.deck_mut() = vec![bid, bid, bid, bid];
            draw.draw_card(p2).unwrap();
            draw.draw_card(p3).unwrap();
            draw.draw_card(p4).unwrap();
            draw.draw_card(p1).unwrap();

            assert!(draw.bid(p1, bid, 1));

            (p2, draw.advance(p2).unwrap())
        };

        let test_cases = vec![
            (
                FriendSelectionPolicy::Unrestricted,
                S_7,
                vec![(C_K, true), (S_3, false), (C_3, true), (C_A, true)],
            ),
            (
                FriendSelectionPolicy::PointCardNotAllowed,
                S_7,
                vec![(C_K, false), (S_3, false), (C_3, true), (C_A, true)],
            ),
            (
                FriendSelectionPolicy::PointCardNotAllowed,
                S_A,
                vec![(C_K, true), (S_3, false), (C_3, true), (C_A, false)],
            ),
            (
                FriendSelectionPolicy::HighestCardNotAllowed,
                S_7,
                vec![(C_K, true), (S_3, false), (C_3, true), (C_A, false)],
            ),
            (
                FriendSelectionPolicy::TrumpsIncluded,
                S_7,
                vec![(C_K, true), (S_3, true), (C_3, true), (C_A, true)],
            ),
        ];

        for (friend_selection_policy, landlord_level, friends) in test_cases {
            for (friend, ok) in friends {
                let (p2, mut exchange) = setup_exchange(friend_selection_policy, landlord_level);

                assert_eq!(
                    exchange
                        .set_friends(
                            p2,
                            vec![FriendSelection {
                                card: friend,
                                initial_skip: 0,
                            }],
                        )
                        .is_ok(),
                    ok,
                    "Expected {:?} to be a {} friend for {:?}",
                    friend,
                    if ok { "legal" } else { "illegal" },
                    friend_selection_policy
                );
            }
        }
    }

    #[test]
    fn test_full_game_play() {
        use cards::*;

        let mut init = InitializePhase::new();

        init.set_game_mode(GameModeSettings::FindingFriends { num_friends: None })
            .unwrap();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        let p5 = init.add_player("p5".into()).unwrap().0;
        let p6 = init.add_player("p6".into()).unwrap().0;

        init.set_landlord(Some(p2)).unwrap();
        init.set_rank(p2, Rank::Number(Number::Seven)).unwrap();

        let mut draw = init.start(PlayerID(1)).unwrap();

        let p1_hand = [
            Card::SmallJoker,
            D_7,
            D_7,
            H_7,
            H_K,
            H_9,
            H_9,
            H_4,
            H_3,
            S_A,
            S_Q,
            S_Q,
            S_9,
            S_9,
            S_8,
            S_5,
            D_K,
            D_8,
            D_6,
            D_5,
            D_4,
            C_K,
            C_K,
            C_J,
            C_9,
            C_8,
        ];
        let p2_hand = [
            Card::BigJoker,
            Card::BigJoker,
            C_7,
            C_7,
            S_7,
            H_K,
            H_K,
            H_6,
            H_4,
            H_3,
            S_K,
            S_J,
            S_4,
            S_3,
            S_2,
            D_K,
            D_10,
            D_4,
            D_4,
            D_2,
            D_2,
            C_K,
            C_9,
            C_5,
            C_4,
            C_3,
        ];
        let p3_hand = [
            Card::SmallJoker,
            S_7,
            H_A,
            H_10,
            H_10,
            H_8,
            H_8,
            H_5,
            H_5,
            H_2,
            S_10,
            S_8,
            S_5,
            S_3,
            D_A,
            D_J,
            D_8,
            D_6,
            D_5,
            C_A,
            C_J,
            C_10,
            C_6,
            C_5,
            C_5,
            C_2,
        ];
        let p4_hand = [
            H_7, S_7, H_Q, H_Q, H_J, H_J, H_8, S_K, S_J, S_10, S_10, S_6, S_2, D_Q, D_8, D_5, D_3,
            D_2, C_A, C_Q, C_J, C_9, C_8, C_6, C_2, C_2,
        ];
        let p5_hand = [
            Card::SmallJoker,
            D_7,
            H_A,
            H_9,
            H_6,
            H_3,
            H_2,
            H_2,
            S_K,
            S_6,
            S_6,
            S_5,
            S_4,
            S_2,
            D_Q,
            D_J,
            D_J,
            D_10,
            D_9,
            D_9,
            D_3,
            D_3,
            C_Q,
            C_10,
            C_3,
            C_3,
        ];
        let p6_hand = [
            Card::BigJoker,
            H_7,
            H_A,
            H_Q,
            H_10,
            H_6,
            H_5,
            H_4,
            S_A,
            S_A,
            S_Q,
            S_J,
            S_8,
            S_4,
            S_3,
            D_A,
            D_A,
            D_K,
            D_Q,
            D_10,
            D_9,
            C_A,
            C_8,
            C_6,
            C_4,
            C_4,
        ];

        assert_eq!(p1_hand.len(), 26);
        assert_eq!(p2_hand.len(), 26);
        assert_eq!(p3_hand.len(), 26);
        assert_eq!(p4_hand.len(), 26);
        assert_eq!(p5_hand.len(), 26);
        assert_eq!(p6_hand.len(), 26);

        let mut deck = vec![];
        for i in 0..26 {
            deck.push(p1_hand[i]);
            deck.push(p2_hand[i]);
            deck.push(p3_hand[i]);
            deck.push(p4_hand[i]);
            deck.push(p5_hand[i]);
            deck.push(p6_hand[i]);
        }
        deck.reverse();
        *draw.deck_mut() = deck;
        *draw.position_mut() = 0;

        for _ in 0..26 {
            draw.draw_card(p1).unwrap();
            draw.draw_card(p2).unwrap();
            draw.draw_card(p3).unwrap();
            draw.draw_card(p4).unwrap();
            draw.draw_card(p5).unwrap();
            draw.draw_card(p6).unwrap();
        }

        *draw.kitty_mut() = vec![C_7, S_9, D_6, D_J, C_Q, C_10];

        assert!(draw.bid(p1, D_7, 2));

        let mut exchange = draw.advance(p2).unwrap();
        let friends = vec![
            FriendSelection {
                card: C_K,
                initial_skip: 0,
            },
            FriendSelection {
                card: H_K,
                initial_skip: 0,
            },
        ];
        exchange.set_friends(p2, friends).unwrap();
        let mut play = exchange.advance(p2).unwrap();

        assert_eq!(play.landlords_team().len(), 1);
        assert_eq!(play.game_mode().num_friends(), Some(2));

        play.play_cards(p2, &[H_K, H_K]).unwrap();
        play.play_cards(p3, &[H_8, H_8]).unwrap();
        play.play_cards(p4, &[H_J, H_J]).unwrap();
        play.play_cards(p5, &[H_2, H_2]).unwrap();
        play.play_cards(p6, &[H_4, H_5]).unwrap();
        play.play_cards(p1, &[H_9, H_9]).unwrap();
        play.finish_trick().unwrap();
        assert_eq!(play.landlords_team().len(), 1);
        assert_eq!(play.game_mode().num_friends(), Some(2));

        play.play_cards(p2, &[C_3]).unwrap();
        play.play_cards(p3, &[C_6]).unwrap();
        play.play_cards(p4, &[C_6]).unwrap();
        play.play_cards(p5, &[C_10]).unwrap();
        play.play_cards(p6, &[C_6]).unwrap();
        play.play_cards(p1, &[C_K]).unwrap();
        play.finish_trick().unwrap();

        assert_eq!(play.landlords_team().len(), 2);
        assert_eq!(play.game_mode().num_friends(), Some(2));

        play.play_cards(p1, &[S_A]).unwrap();
        play.play_cards(p2, &[S_2]).unwrap();
        play.play_cards(p3, &[S_3]).unwrap();
        play.play_cards(p4, &[S_2]).unwrap();
        play.play_cards(p5, &[S_2]).unwrap();
        play.play_cards(p6, &[S_3]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p1, &[S_Q, S_Q]).unwrap();
        play.play_cards(p2, &[S_3, S_4]).unwrap();
        play.play_cards(p3, &[S_5, S_8]).unwrap();
        play.play_cards(p4, &[S_10, S_10]).unwrap();
        play.play_cards(p5, &[S_6, S_6]).unwrap();
        play.play_cards(p6, &[S_A, S_A]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p6, &[Card::BigJoker]).unwrap();
        play.play_cards(p1, &[D_4]).unwrap();
        play.play_cards(p2, &[S_7]).unwrap();
        play.play_cards(p3, &[D_5]).unwrap();
        play.play_cards(p4, &[D_5]).unwrap();
        play.play_cards(p5, &[D_10]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p6, &[D_A, D_A]).unwrap();
        play.play_cards(p1, &[D_7, D_7]).unwrap();
        play.play_cards(p2, &[D_2, D_2]).unwrap();
        play.play_cards(p3, &[D_6, D_8]).unwrap();
        play.play_cards(p4, &[D_2, D_3]).unwrap();
        play.play_cards(p5, &[D_3, D_3]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p1, &[S_9, S_9]).unwrap();
        play.play_cards(p2, &[S_J, S_K]).unwrap();
        play.play_cards(p3, &[S_10, H_2]).unwrap();
        play.play_cards(p4, &[S_6, S_J]).unwrap();
        play.play_cards(p5, &[S_4, S_5]).unwrap();
        play.play_cards(p6, &[S_4, S_8]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p1, &[S_5]).unwrap();
        play.play_cards(p2, &[D_10]).unwrap();
        play.play_cards(p3, &[C_2]).unwrap();
        play.play_cards(p4, &[S_K]).unwrap();
        play.play_cards(p5, &[S_K]).unwrap();
        play.play_cards(p6, &[S_J]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p2, &[Card::BigJoker, Card::BigJoker])
            .unwrap();
        play.play_cards(p3, &[D_J, D_A]).unwrap();
        play.play_cards(p4, &[D_8, D_Q]).unwrap();
        play.play_cards(p5, &[D_9, D_9]).unwrap();
        play.play_cards(p6, &[D_9, D_10]).unwrap();
        play.play_cards(p1, &[D_5, D_K]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p2, &[C_7, C_7]).unwrap();
        play.play_cards(p3, &[S_7, Card::SmallJoker]).unwrap();
        play.play_cards(p4, &[S_7, H_7]).unwrap();
        play.play_cards(p5, &[D_J, D_J]).unwrap();
        play.play_cards(p6, &[D_Q, D_K]).unwrap();
        play.play_cards(p1, &[D_6, D_8]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p2, &[D_4, D_4]).unwrap();
        play.play_cards(p3, &[C_10, C_J]).unwrap();
        play.play_cards(p4, &[C_8, C_9]).unwrap();
        play.play_cards(p5, &[D_Q, D_7]).unwrap();
        play.play_cards(p6, &[C_8, H_7]).unwrap();
        play.play_cards(p1, &[H_7, Card::SmallJoker]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p2, &[H_3]).unwrap();
        play.play_cards(p3, &[H_A]).unwrap();
        play.play_cards(p4, &[H_8]).unwrap();
        play.play_cards(p5, &[H_3]).unwrap();
        play.play_cards(p6, &[H_6]).unwrap();
        play.play_cards(p1, &[H_3]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p3, &[H_10, H_10]).unwrap();
        play.play_cards(p4, &[H_Q, H_Q]).unwrap();
        play.play_cards(p5, &[H_6, H_9]).unwrap();
        play.play_cards(p6, &[H_10, H_Q]).unwrap();
        play.play_cards(p1, &[H_4, H_K]).unwrap();
        play.play_cards(p2, &[H_4, H_6]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p4, &[C_2]).unwrap();
        play.play_cards(p5, &[C_3]).unwrap();
        play.play_cards(p6, &[C_4]).unwrap();
        play.play_cards(p1, &[C_K]).unwrap();
        play.play_cards(p2, &[C_K]).unwrap();
        play.play_cards(p3, &[C_5]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p1, &[S_8]).unwrap();
        play.play_cards(p2, &[C_4]).unwrap();
        play.play_cards(p3, &[C_A]).unwrap();
        play.play_cards(p4, &[C_A]).unwrap();
        play.play_cards(p5, &[C_3]).unwrap();
        play.play_cards(p6, &[S_Q]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p6, &[C_4]).unwrap();
        play.play_cards(p1, &[C_8]).unwrap();
        play.play_cards(p2, &[C_9]).unwrap();
        play.play_cards(p3, &[C_5]).unwrap();
        play.play_cards(p4, &[C_2]).unwrap();
        play.play_cards(p5, &[C_Q]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p5, &[H_A]).unwrap();
        play.play_cards(p6, &[H_A]).unwrap();
        play.play_cards(p1, &[C_9]).unwrap();
        play.play_cards(p2, &[C_5]).unwrap();
        play.play_cards(p3, &[H_5]).unwrap();
        play.play_cards(p4, &[C_J]).unwrap();
        play.finish_trick().unwrap();

        play.play_cards(p5, &[Card::SmallJoker]).unwrap();
        play.play_cards(p6, &[C_A]).unwrap();
        play.play_cards(p1, &[C_J]).unwrap();
        play.play_cards(p2, &[D_K]).unwrap();
        play.play_cards(p3, &[H_5]).unwrap();
        play.play_cards(p4, &[C_Q]).unwrap();
        play.finish_trick().unwrap();

        if let Ok((phase, _, _msgs)) = play.finish_game() {
            assert_eq!(phase.propagated().landlord, Some(p3));
        };
    }

    #[test]
    fn test_landlord_small_team() {
        let mut init = InitializePhase::new();
        init.set_game_mode(GameModeSettings::FindingFriends {
            num_friends: Some(3),
        })
        .unwrap();
        let p1 = init.add_player("p1".into()).unwrap().0;
        let p2 = init.add_player("p2".into()).unwrap().0;
        let p3 = init.add_player("p3".into()).unwrap().0;
        let p4 = init.add_player("p4".into()).unwrap().0;
        let p5 = init.add_player("p5".into()).unwrap().0;
        let p6 = init.add_player("p6".into()).unwrap().0;
        let p7 = init.add_player("p7".into()).unwrap().0;
        let p8 = init.add_player("p8".into()).unwrap().0;

        init.set_landlord(Some(p1)).unwrap();
        init.set_rank(p1, Rank::Number(Number::Seven)).unwrap();

        let mut draw = init.start(PlayerID(0)).unwrap();
        let mut deck = vec![];

        // We need at least two cards per person, since the landlord needs to
        // bid, and the biddable card can't be the friend-selection card.
        let p1_hand = [cards::S_7, cards::D_3];
        let p2_hand = [cards::D_4, cards::D_5];
        let p3_hand = [cards::C_6, cards::C_8];
        let p4_hand = [cards::C_9, cards::C_10];
        let p5_hand = [cards::C_J, cards::C_Q];
        let p6_hand = [cards::C_K, cards::C_A];
        let p7_hand = [cards::H_2, cards::H_3];
        let p8_hand = [cards::H_4, cards::H_5];

        // Set up the deck to have the appropriate cards.
        for i in 0..2 {
            deck.push(p1_hand[i]);
            deck.push(p2_hand[i]);
            deck.push(p3_hand[i]);
            deck.push(p4_hand[i]);
            deck.push(p5_hand[i]);
            deck.push(p6_hand[i]);
            deck.push(p7_hand[i]);
            deck.push(p8_hand[i]);
        }
        deck.reverse();
        *draw.deck_mut() = deck;
        *draw.position_mut() = 0;

        // Draw the deck
        for _ in 0..2 {
            draw.draw_card(p1).unwrap();
            draw.draw_card(p2).unwrap();
            draw.draw_card(p3).unwrap();
            draw.draw_card(p4).unwrap();
            draw.draw_card(p5).unwrap();
            draw.draw_card(p6).unwrap();
            draw.draw_card(p7).unwrap();
            draw.draw_card(p8).unwrap();
        }

        // p1 bids and wins, trump is now Spades and 7s.
        assert!(draw.bid(p1, cards::S_7, 1));

        let mut exchange = draw.advance(p1).unwrap();
        let friends = vec![
            FriendSelection {
                card: cards::D_3,
                initial_skip: 0,
            },
            FriendSelection {
                card: cards::D_4,
                initial_skip: 0,
            },
            FriendSelection {
                card: cards::D_5,
                initial_skip: 0,
            },
        ];
        exchange.set_friends(p1, friends).unwrap();
        let mut play = exchange.advance(p1).unwrap();
        match play.game_mode() {
            GameMode::FindingFriends { num_friends: 3, .. } => (),
            _ => panic!("Didn't have 3 friends once game was started"),
        }

        assert_eq!(
            play.landlords_team(),
            vec![p1],
            "Nobody should have joined the team yet"
        );

        // Play the first hand. P2 will join the team.
        play.play_cards(p1, &p1_hand[..1]).unwrap();
        play.play_cards(p2, &p2_hand[..1]).unwrap();
        play.play_cards(p3, &p3_hand[..1]).unwrap();
        play.play_cards(p4, &p4_hand[..1]).unwrap();
        play.play_cards(p5, &p5_hand[..1]).unwrap();
        play.play_cards(p6, &p6_hand[..1]).unwrap();
        play.play_cards(p7, &p7_hand[..1]).unwrap();
        play.play_cards(p8, &p8_hand[..1]).unwrap();

        // Check that P2 actually joined the team.
        let msgs = play.finish_trick().unwrap();
        assert_eq!(
            msgs.into_iter()
                .filter(|m| matches!(m, MessageVariant::JoinedTeam { player, already_joined: false } if *player == p2))
                .count(),
            1
        );

        assert_eq!(play.landlords_team(), vec![p1, p2]);

        // Play the next trick, where the landlord will join the team, and then
        // p2 will join the team (again).
        play.play_cards(p1, &p1_hand[1..2]).unwrap();
        play.play_cards(p2, &p2_hand[1..2]).unwrap();
        play.play_cards(p3, &p3_hand[1..2]).unwrap();
        play.play_cards(p4, &p4_hand[1..2]).unwrap();
        play.play_cards(p5, &p5_hand[1..2]).unwrap();
        play.play_cards(p6, &p6_hand[1..2]).unwrap();
        play.play_cards(p7, &p7_hand[1..2]).unwrap();
        play.play_cards(p8, &p8_hand[1..2]).unwrap();

        // We get a re-joined team message, since p2 has already joined.
        let msgs = play.finish_trick().unwrap();
        assert_eq!(
            msgs.into_iter()
                .filter(|m| matches!(m, MessageVariant::JoinedTeam { player, already_joined: true } if *player == p2))
                .count(),
            1
        );

        // Assert that the team didn't get any bigger
        assert_eq!(play.landlords_team(), vec![p1, p2]);
        // But also that all of the friend cards have been played!
        match play.game_mode() {
            GameMode::FindingFriends { ref friends, .. } => assert!(
                friends.iter().all(|f| f.player_id.is_some()),
                "all friends lots taken"
            ),
            _ => unreachable!(),
        }

        // Finish the game; we should see the landlord go up 4 levels (3 for
        // keeping the opposing team at 0, and a bonus level)

        let (new_init_phase, _, msgs) = play.finish_game().unwrap();
        assert_eq!(
            msgs.into_iter()
                .filter(|m| match m {
                    MessageVariant::BonusLevelEarned => true,
                    MessageVariant::RankAdvanced { player, new_rank } if *player == p1 => {
                        assert_eq!(*new_rank, Rank::Number(Number::Jack));
                        false
                    }
                    MessageVariant::RankAdvanced { player, new_rank } if *player == p2 => {
                        assert_eq!(*new_rank, Rank::Number(Number::Six));
                        false
                    }
                    _ => false,
                })
                .count(),
            1
        );

        assert_eq!(
            new_init_phase
                .propagated()
                .players()
                .iter()
                .map(|p| p.level)
                .collect::<Vec<Rank>>(),
            vec![
                Rank::Number(Number::Jack),
                Rank::Number(Number::Six),
                Rank::Number(Number::Two),
                Rank::Number(Number::Two),
                Rank::Number(Number::Two),
                Rank::Number(Number::Two),
                Rank::Number(Number::Two),
                Rank::Number(Number::Two)
            ],
            "Check that propagated players have the right new levels"
        );
    }
}
