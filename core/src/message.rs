use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::bidding::{BidPolicy, BidReinforcementPolicy, BidTakebackPolicy, JokerBidPolicy};
use crate::deck::Deck;
use crate::game_state::PlayerGameFinishedResult;
use crate::scoring::GameScoringParameters;
use crate::settings::{
    AdvancementPolicy, FirstLandlordSelectionPolicy, FriendSelectionPolicy, GameModeSettings,
    GameShadowingPolicy, GameStartPolicy, KittyBidPolicy, KittyPenalty, KittyTheftPolicy,
    MultipleJoinPolicy, PlayTakebackPolicy, ThrowPenalty,
};
use crate::trick::{ThrowEvaluationPolicy, TractorRequirements, TrickDrawPolicy};
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
        already_joined: bool,
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
    MultipleJoinPolicySet {
        policy: MultipleJoinPolicy,
    },
    FirstLandlordSelectionPolicySet {
        policy: FirstLandlordSelectionPolicy,
    },
    BidPolicySet {
        policy: BidPolicy,
    },
    BidReinforcementPolicySet {
        policy: BidReinforcementPolicy,
    },
    JokerBidPolicySet {
        policy: JokerBidPolicy,
    },
    ShouldRevealKittyAtEndOfGameSet {
        should_reveal: bool,
    },
    SpecialDecksSet {
        special_decks: Vec<Deck>,
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
        better_player: Option<PlayerID>,
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
    GameEndedEarly,
    GameFinished {
        result: HashMap<String, PlayerGameFinishedResult>,
    },
    BonusLevelEarned,
    EndOfGameSummary {
        landlord_won: bool,
        non_landlords_points: isize,
    },
    HideThrowHaltingPlayer {
        set: bool,
    },
    TractorRequirementsChanged {
        tractor_requirements: TractorRequirements,
    },
}
