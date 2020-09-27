use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::bidding::{BidPolicy, BidTakebackPolicy, JokerBidPolicy};
use crate::game_state::PlayerGameFinishedResult;
use crate::scoring::GameScoringParameters;
use crate::settings::{
    AdvancementPolicy, FirstLandlordSelectionPolicy, FriendSelectionPolicy, GameModeSettings,
    GameShadowingPolicy, GameStartPolicy, KittyBidPolicy, KittyPenalty, KittyTheftPolicy,
    PlayTakebackPolicy, ThrowPenalty,
};
use crate::trick::{ThrowEvaluationPolicy, TrickDrawPolicy};
use crate::types::{Card, Number, PlayerID};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum MessageVariant {
    ResettingGame,
    StartingGame,
    TrickWon {
        winner: PlayerID,
        points: usize,
    },
    RankAdvanced {
        player: PlayerID,
        new_rank: Number,
    },
    AdvancementBlocked {
        player: PlayerID,
        rank: Number,
    },
    NewLandlordForNextGame {
        landlord: PlayerID,
    },
    PointsInKitty {
        points: usize,
        multiplier: usize,
    },
    EndOfGameKittyReveal {
        cards: Vec<Card>,
    },
    JoinedGame {
        player: PlayerID,
    },
    JoinedGameAgain {
        player: PlayerID,
        game_shadowing_policy: GameShadowingPolicy,
    },
    JoinedTeam {
        player: PlayerID,
    },
    LeftGame {
        name: String,
    },
    AdvancementPolicySet {
        policy: AdvancementPolicy,
    },
    KittySizeSet {
        size: Option<usize>,
    },
    FriendSelectionPolicySet {
        policy: FriendSelectionPolicy,
    },
    FirstLandlordSelectionPolicySet {
        policy: FirstLandlordSelectionPolicy,
    },
    BidPolicySet {
        policy: BidPolicy,
    },
    JokerBidPolicySet {
        policy: JokerBidPolicy,
    },
    ShouldRevealKittyAtEndOfGameSet {
        should_reveal: bool,
    },
    NumDecksSet {
        num_decks: Option<usize>,
    },
    NumFriendsSet {
        num_friends: Option<usize>,
    },
    GameModeSet {
        game_mode: GameModeSettings,
    },
    KittyTheftPolicySet {
        policy: KittyTheftPolicy,
    },
    TookBackPlay,
    TookBackBid,
    PlayedCards {
        cards: Vec<Card>,
    },
    ThrowFailed {
        original_cards: Vec<Card>,
        better_player: PlayerID,
    },
    SetDefendingPointVisibility {
        visible: bool,
    },
    SetCardVisibility {
        visible: bool,
    },
    SetLandlord {
        landlord: Option<PlayerID>,
    },
    SetLandlordEmoji {
        emoji: String,
    },
    SetRank {
        rank: Number,
    },
    MadeBid {
        card: Card,
        count: usize,
    },
    KittyPenaltySet {
        kitty_penalty: KittyPenalty,
    },
    ThrowPenaltySet {
        throw_penalty: ThrowPenalty,
    },
    KittyBidPolicySet {
        policy: KittyBidPolicy,
    },
    TrickDrawPolicySet {
        policy: TrickDrawPolicy,
    },
    ThrowEvaluationPolicySet {
        policy: ThrowEvaluationPolicy,
    },
    PlayTakebackPolicySet {
        policy: PlayTakebackPolicy,
    },
    BidTakebackPolicySet {
        policy: BidTakebackPolicy,
    },
    GameShadowingPolicySet {
        policy: GameShadowingPolicy,
    },
    GameStartPolicySet {
        policy: GameStartPolicy,
    },
    GameScoringParametersChanged {
        parameters: GameScoringParameters,
        old_parameters: GameScoringParameters,
    },
    PickedUpCards,
    PutDownCards,
    RevealedCardFromKitty,
    GameFinished {
        result: HashMap<String, PlayerGameFinishedResult>,
    },
    BonusLevelEarned,
    EndOfGameSummary {
        landlord_won: bool,
        non_landlords_points: isize,
    },
}
