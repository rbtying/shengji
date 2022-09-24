use std::collections::HashMap;

use anyhow::Error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::bidding::{BidPolicy, BidReinforcementPolicy, BidTakebackPolicy, JokerBidPolicy};
use crate::deck::Deck;
use crate::game_state::play_phase::PlayerGameFinishedResult;
use crate::scoring::GameScoringParameters;
use crate::settings::{
    AdvancementPolicy, FirstLandlordSelectionPolicy, FriendSelectionPolicy, GameModeSettings,
    GameShadowingPolicy, GameStartPolicy, KittyBidPolicy, KittyPenalty, KittyTheftPolicy,
    MultipleJoinPolicy, PlayTakebackPolicy, ThrowPenalty,
};
use crate::trick::{ThrowEvaluationPolicy, TractorRequirements, TrickDrawPolicy};
use crate::types::{Card, PlayerID, Rank};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
        new_rank: Rank,
    },
    AdvancementBlocked {
        player: PlayerID,
        rank: Rank,
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
        rank: Rank,
    },
    SetMetaRank {
        metarank: usize,
    },
    SetMaxRank {
        rank: Rank,
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

impl MessageVariant {
    pub fn to_string<'a>(
        &'a self,
        actor: PlayerID,
        player_name: impl Fn(PlayerID) -> Result<&'a str, Error>,
    ) -> Result<String, Error> {
        let n = player_name(actor);

        use MessageVariant::*;
        Ok(match self {
            ResettingGame => format!("{} reset the game", n?),
            StartingGame => format!("{} started the game", n?),
            TrickWon { winner, points: 0 } =>
                format!("{} wins the trick, but gets no points :(", player_name(*winner)?),
            TrickWon { winner, points } =>
                format!("{} wins the trick and gets {} points", player_name(*winner)?, points),
            RankAdvanced { player, new_rank } =>
                format!("{} has advanced to rank {}", player_name(*player)?, new_rank.as_str()),
            AdvancementBlocked { player, rank } =>
                format!("{} must defend on rank {}", player_name(*player)?, rank.as_str()),
            NewLandlordForNextGame { landlord } =>
                format!("{} will start the next game", player_name(*landlord)?),
            PointsInKitty { points, multiplier } =>
                format!("{} points were buried and are attached to the last trick, with a multiplier of {}", points, multiplier),
            JoinedGame { player } =>
                format!("{} has joined the game", player_name(*player)?),
            JoinedGameAgain { player, game_shadowing_policy: GameShadowingPolicy::SingleSessionOnly } =>
                format!("{} has joined the game again, prior connection removed", player_name(*player)?),
            JoinedGameAgain { player, game_shadowing_policy: GameShadowingPolicy::AllowMultipleSessions } =>
                format!("{} is being shadowed", player_name(*player)?),
            JoinedTeam { player, already_joined: false } =>
                format!("{} has joined the team", player_name(*player)?),
            JoinedTeam { player, already_joined: true } =>
                format!("{} tried to join the team, but was already a member", player_name(*player)?),
            LeftGame { ref name } => format!("{} has left the game", name),
            AdvancementPolicySet { policy: AdvancementPolicy::FullyUnrestricted } =>
                format!("{} removed all advancement restrictions", n?),
            AdvancementPolicySet { policy: AdvancementPolicy::Unrestricted } =>
                format!("{} required players to defend on A", n?),
            AdvancementPolicySet { policy: AdvancementPolicy::DefendPoints } =>
                format!("{} required players to defend on points and A", n?),
            GameScoringParametersChanged { .. } => format!("{} changed the game's scoring parameters", n?),
            KittySizeSet { size: Some(size) } => format!("{} set the number of cards in the bottom to {}", n?, size),
            KittySizeSet { size: None } => format!("{} set the number of cards in the bottom to default", n?),
            FriendSelectionPolicySet { policy: FriendSelectionPolicy::Unrestricted } =>
                format!("{} allowed any non-trump card to be selected as a friend", n?),
            FriendSelectionPolicySet { policy: FriendSelectionPolicy::TrumpsIncluded } =>
                format!("{} allowed any card to be selected as a friend", n?),
            FriendSelectionPolicySet { policy: FriendSelectionPolicy::HighestCardNotAllowed } =>
                format!("{} disallowed the highest non-trump card, as well as trump cards, from being selected as a friend", n?),
            FriendSelectionPolicySet { policy: FriendSelectionPolicy::PointCardNotAllowed } =>
                format!("{} disallowed point cards, as well as trump cards, from being selected as a friend", n?),
            MultipleJoinPolicySet { policy: MultipleJoinPolicy::Unrestricted } =>
                format!("{} allowed players to join the team multiple times", n?),
            MultipleJoinPolicySet { policy: MultipleJoinPolicy::NoDoubleJoin } =>
                format!("{} prevented players from joining the team multiple times", n?),
            FirstLandlordSelectionPolicySet { policy: FirstLandlordSelectionPolicy::ByWinningBid } =>
                format!("{} set winning bid to decide both landlord and trump", n?),
            FirstLandlordSelectionPolicySet { policy: FirstLandlordSelectionPolicy::ByFirstBid } =>
                format!("{} set first bid to decide landlord, winning bid to decide trump", n?),
            BidPolicySet { policy: BidPolicy::JokerOrHigherSuit } =>
                format!("{} allowed joker or higher suit bids to outbid non-joker bids with the same number of cards", n?),
            BidPolicySet { policy: BidPolicy::JokerOrGreaterLength } =>
                format!("{} allowed joker bids to outbid non-joker bids with the same number of cards", n?),
            BidPolicySet { policy: BidPolicy::GreaterLength } =>
                format!("{} required all bids to have more cards than the previous bids", n?),
            BidReinforcementPolicySet { policy: BidReinforcementPolicy::ReinforceWhileWinning } =>
                format!("{} allowed reinforcing the winning bid", n?),
            BidReinforcementPolicySet { policy: BidReinforcementPolicy::ReinforceWhileEquivalent } =>
                format!("{} allowed reinforcing bids after they have been overturned", n?),
            BidReinforcementPolicySet { policy: BidReinforcementPolicy::OverturnOrReinforceWhileWinning } =>
                format!("{} allowed overturning your own bids", n?),
            JokerBidPolicySet { policy: JokerBidPolicy::BothNumDecks } =>
                format!("{} required no-trump bids to have every low or high joker", n?),
            JokerBidPolicySet { policy: JokerBidPolicy::LJNumDecksHJNumDecksLessOne } =>
                format!("{} required low no-trump bids to have every low joker (one less required for high joker)", n?),
            JokerBidPolicySet { policy: JokerBidPolicy::BothTwoOrMore } =>
                format!("{} required no-trump bids to have at least two low or high jokers", n?),
            ShouldRevealKittyAtEndOfGameSet { should_reveal: true } =>
                format!("{} enabled the kitty to be revealed at the end of each game", n?),
            ShouldRevealKittyAtEndOfGameSet { should_reveal: false } =>
                format!("{} disabled the kitty from being revealed at the end of each game", n?),
            NumDecksSet { num_decks: Some(num_decks) } =>
                format!("{} set the number of decks to {}", n?, num_decks),
            NumDecksSet { num_decks: None } => format!("{} set the number of decks to default", n?),
            SpecialDecksSet { ref special_decks } if special_decks.is_empty() =>
                format!("{} set the decks to standard 54-card decks", n?),
            SpecialDecksSet { .. } => format!("{} changed the special deck settings", n?),
            NumFriendsSet { num_friends: Some(num_friends) } =>
                format!("{} set the number of friends to {}", n?, num_friends),
            NumFriendsSet { num_friends: None } =>
                format!("{} set the number of friends to default", n?),
            GameModeSet { game_mode: GameModeSettings::Tractor } =>
                format!("{} set the game mode to Tractor", n?),
            GameModeSet { game_mode: GameModeSettings::FindingFriends { num_friends: None }} =>
                format!("{} set the game mode to Finding Friends", n?),
            GameModeSet { game_mode: GameModeSettings::FindingFriends { num_friends: Some(1) }} =>
                format!("{} set the game mode to Finding Friends with 1 friend", n?),
            GameModeSet { game_mode: GameModeSettings::FindingFriends { num_friends: Some(friends) }} =>
                format!("{} set the game mode to Finding Friends with {} friends", n?, friends),
            TookBackBid => format!("{} took back their last bid", n?),
            TookBackPlay => format!("{} took back their last play", n?),
            PlayedCards { ref cards } =>
                format!("{} played {}", n?, cards.iter().map(|c| c.as_char()).collect::<String>()),
            EndOfGameKittyReveal { ref cards } =>
                format!("{} in kitty", cards.iter().map(|c| c.as_char()).collect::<String>()),
            ThrowFailed { ref original_cards, better_player: Some(better_player) } =>
                format!("{} tried to throw {}, but {} can beat it", n?, original_cards.iter().map(|c| c.as_char()).collect::<String>(), player_name(*better_player)?),
            ThrowFailed { ref original_cards, better_player: None } =>
                format!("{} tried to throw {}, but someone can beat it", n?, original_cards.iter().map(|c| c.as_char()).collect::<String>()),
            SetDefendingPointVisibility { visible: true } => format!("{} made the defending team's points visible", n?),
            SetDefendingPointVisibility { visible: false } => format!("{} hid the defending team's points", n?),
            SetCardVisibility { visible: true } => format!("{} made the played cards visible in the chat", n?),
            SetCardVisibility { visible: false } => format!("{} hid the played cards from the chat", n?),
            SetLandlord { landlord: None } => format!("{} set the leader to the winner of the bid", n?),
            SetLandlord { landlord: Some(landlord) } => format!("{} set the leader to {}", n?, player_name(*landlord)?),
            SetLandlordEmoji { ref emoji } => format!("{} set landlord emoji to {}", n?, *emoji),
            SetRank { rank } => format!("{} set their rank to {}", n?, rank.as_str()),
            SetMetaRank { metarank } => format!("{} set their meta-rank to {}", n?, metarank),
            SetMaxRank { rank} => format!("{} set the max rank to {}", n?, rank.as_str()),
            MadeBid { card, count } => format!("{} bid {} {:?}", n?, count, card),
            KittyPenaltySet { kitty_penalty: KittyPenalty::Times } =>
                format!("{} set the penalty for points in the bottom to twice the size of the last trick", n?),
            KittyPenaltySet { kitty_penalty: KittyPenalty::Power } =>
                format!("{} set the penalty for points in the bottom to two to the power of the size of the last trick", n?),
            ThrowPenaltySet { throw_penalty: ThrowPenalty::None } =>
                format!("{} removed the throw penalty", n?),
            ThrowPenaltySet { throw_penalty: ThrowPenalty::TenPointsPerAttempt } =>
                format!("{} set the throw penalty to 10 points per throw", n?),
            KittyBidPolicySet { policy: KittyBidPolicy::FirstCard } =>
                format!("{} set the bid-from-bottom policy to be the first card revealed", n?),
            KittyBidPolicySet { policy: KittyBidPolicy::FirstCardOfLevelOrHighest } =>
                format!("{} set the bid-from-bottom policy to be the first card of the appropriate level, or the highest if none are found", n?),
            TrickDrawPolicySet { policy: TrickDrawPolicy::NoProtections } =>
                format!("{} removed all protections (pair can draw triple)", n?),
            TrickDrawPolicySet { policy: TrickDrawPolicy::NoFormatBasedDraw } =>
                format!("{} removed format-based forced-plays (pairs do not draw pairs)", n?),
            TrickDrawPolicySet { policy: TrickDrawPolicy::LongerTuplesProtected } =>
                format!("{} protected longer tuples from being drawn out by shorter ones (pair does not draw triple)", n?),
            TrickDrawPolicySet { policy: TrickDrawPolicy::OnlyDrawTractorOnTractor } =>
                format!("{} protected tractors from being drawn out by non-tractors", n?),
            ThrowEvaluationPolicySet { policy: ThrowEvaluationPolicy::All } =>
                format!("{} set throws to be evaluated based on all of the cards", n?),
            ThrowEvaluationPolicySet { policy: ThrowEvaluationPolicy::Highest } =>
                format!("{} set throws to be evaluated based on the highest card", n?),
            ThrowEvaluationPolicySet { policy: ThrowEvaluationPolicy::TrickUnitLength } =>
                format!("{} set throws to be evaluated based on the longest component", n?),
            PlayTakebackPolicySet { policy: PlayTakebackPolicy::AllowPlayTakeback } =>
                format!("{} allowed taking back plays", n?),
            PlayTakebackPolicySet { policy: PlayTakebackPolicy::NoPlayTakeback } =>
                format!("{} disallowed taking back plays", n?),
            BidTakebackPolicySet { policy: BidTakebackPolicy::AllowBidTakeback } =>
                format!("{} allowed taking back bids", n?),
            BidTakebackPolicySet { policy: BidTakebackPolicy::NoBidTakeback } =>
                format!("{} disallowed taking back bids", n?),
            KittyTheftPolicySet { policy: KittyTheftPolicy::AllowKittyTheft } =>
                format!("{} allowed stealing the bottom cards after the leader", n?),
            KittyTheftPolicySet { policy: KittyTheftPolicy::NoKittyTheft } =>
                format!("{} disabled stealing the bottom cards after the leader", n?),
            GameShadowingPolicySet { policy: GameShadowingPolicy::AllowMultipleSessions } =>
                format!("{} allowed players to be shadowed by joining with the same name", n?),
            GameShadowingPolicySet { policy: GameShadowingPolicy::SingleSessionOnly } =>
                format!("{} prohibited players from being shadowed", n?),
            GameStartPolicySet { policy: GameStartPolicy::AllowAnyPlayer } =>
                format!("{} allowed any player to start a game", n?),
            GameStartPolicySet { policy: GameStartPolicy::AllowLandlordOnly } =>
                format!("{} allowed only landlord to start a game", n?),
            RevealedCardFromKitty => format!("{} revealed a card from the bottom of the deck", n?),
            PickedUpCards => format!("{} picked up the bottom cards", n?),
            PutDownCards => format!("{} put down the bottom cards", n?),
            GameFinished { result: _ } => "The game has finished".to_string(),
            GameEndedEarly => format!("{} ended the game early", n?),
            BonusLevelEarned => "Landlord team earned a bonus level for defending with a smaller team".to_string(),
            EndOfGameSummary { landlord_won : true, non_landlords_points } =>
                format!("Landlord team won, opposing team only collected {} points", non_landlords_points),
            EndOfGameSummary { landlord_won: false, non_landlords_points } =>
                format!("Landlord team lost, opposing team collected {} points", non_landlords_points),
            HideThrowHaltingPlayer { set: true } => format!("{} hid the player who prevents throws", n?),
            HideThrowHaltingPlayer { set: false } => format!("{} un-hid the player who prevents throws", n?),
            TractorRequirementsChanged { tractor_requirements } =>
                format!("{} required tractors to be at least {} cards wide by {} tuples long", n?, tractor_requirements.min_count, tractor_requirements.min_length),
        })
    }
}
