export type IGameMessage = "Beep" | "ReadyCheck" | IGameMessageUnion;
export interface IGameMessageUnion {
  Broadcast?: IGameMessageBroadcast;
  Error?: string;
  Message?: IGameMessageMessage;
  State?: IGameMessageState;
  Header?: {
    messages: string[];
  };
}

export interface IGameMessageMessage {
  from: string;
  message: string;
}
export interface IGameMessageBroadcast {
  data: IBroadcastMessage;
  message: string;
}
export interface IGameMessageState {
  state: IGameState;
}
export interface IBroadcastMessage {
  actor: number;
  actor_name: string;
  variant: MessageVariant;
}
export type MessageVariant =
  | { type: "GameModeSet"; game_mode: IGameMode }
  | { type: "JoinedGame"; player: number }
  | { type: "JoinedTeam"; player: number }
  | { type: "KittySizeSet"; size: number | null }
  | { type: "LeftGame"; name: string }
  | { type: "MadeBid"; card: string; count: number }
  | { type: "NewLandlordForNextGame"; landlord: number }
  | { type: "NumDecksSet"; num_decks: number | null }
  | { type: "NumFriendsSet"; num_friends: number | null }
  | { type: "PlayedCards"; cards: string[] }
  | { type: "EndOfGameKittyReveal"; cards: string[] }
  | { type: "PointsInKitty"; points: number; multiplier: number }
  | { type: "RankAdvanced"; player: number; new_rank: number }
  | { type: "ResettingGame" }
  | { type: "SetDefendingPointVisibility"; visible: boolean }
  | { type: "SetLandlord"; landlord: number | null }
  | { type: "SetRank"; rank: string }
  | { type: "StartingGame" }
  | { type: "TookBackPlay" }
  | { type: "TrickWon"; winner: number; points: number }
  | {
      type: "GameFinished";
      result: {
        [player_name: string]: {
          won_game: boolean;
          is_defending: boolean;
          is_landlord: boolean;
          ranks_up: number;
          confetti: boolean;
        };
      };
    }
  | {
      type: "GameScoringParametersChanged";
      parameters: IGameScoringParameters;
      old_parameters: IGameScoringParameters;
    };

export interface IPlayer {
  id: number;
  name: string;
  level: string;
  metalevel: number;
}

export interface IGameState {
  Initialize: IInitializePhase | null;
  Draw: IDrawPhase | null;
  Exchange: IExchangePhase | null;
  Play: IPlayPhase | null;
}

export interface IInitializePhase {
  propagated: IPropagatedState;
}

export interface IBid {
  id: number;
  card: string;
  count: number;
  epoch: number;
}

export interface IDrawPhase {
  num_decks: number;
  game_mode: IGameMode;
  deck: string[];
  propagated: IPropagatedState;
  hands: IHands;
  autobid: IBid | null;
  bids: IBid[];
  position: number;
  kitty: string[];
  revealed_cards: number;
  level: number;
  removed_cards: string[];
  decks: IDeck[];
}

export interface IExchangePhase {
  propagated: IPropagatedState;
  num_decks: number;
  game_mode: IGameMode;
  hands: IHands;
  kitty: string[];
  kitty_size: number;
  landlord: number;
  trump: ITrump;
  autobid: IBid | null;
  bids: IBid[];
  epoch: number;
  finalized: boolean;
  exchanger: number | null;
  removed_cards: string[];
  decks: IDeck[];
}

export interface IPlayPhase {
  num_decks: number;
  game_mode: IGameMode;
  propagated: IPropagatedState;
  hands: IHands;
  points: { [id: number]: string[] };
  penalties: { [id: number]: number };
  kitty: string[];
  landlord: number;
  landlords_team: number[];
  trump: ITrump;
  trick: ITrick;
  last_trick: ITrick | null;
  game_ended_early: boolean;
  removed_cards: string[];
  decks: IDeck[];
}

export type BidPolicy = "JokerOrGreaterLength" | "GreaterLength";
export type BidReinforcementPolicy =
  | "ReinforceWhileWinning"
  | "ReinforceWhileEquivalent"
  | "OverturnOrReinforceWhileWinning";
export type JokerBidPolicy =
  | "BothTwoOrMore"
  | "BothNumDecks"
  | "LJNumDecksHJNumDecksLessOne";
export type TrickDrawPolicy =
  | "NoProtections"
  | "LongerTuplesProtected"
  | "NoFormatBasedDraw"
  | "OnlyDrawTractorOnTractor";

export interface IPropagatedState {
  game_mode: IGameModeSettings;
  hide_landlord_points: boolean | null;
  kitty_size: number | null;
  friend_selection_policy:
    | "Unrestricted"
    | "TrumpsIncluded"
    | "HighestCardNotAllowed"
    | "PointCardNotAllowed";
  multiple_join_policy: "Unrestricted" | "NoDoubleJoin";
  first_landlord_selection_policy: "ByWinningBid" | "ByFirstBid";
  bid_policy: BidPolicy;
  bid_reinforcement_policy: BidReinforcementPolicy;
  joker_bid_policy: JokerBidPolicy;
  num_decks: number | null;
  special_decks: IDeck[];
  max_player_id: number;
  players: IPlayer[];
  observers: IPlayer[];
  landlord: number | null;
  chat_link: string | null;
  advancement_policy: "Unrestricted" | "DefendPoints";
  kitty_penalty: "Times" | "Power";
  kitty_bid_policy: "FirstCard" | "FirstCardOfLevelOrHighest";
  throw_penalty: "None" | "TenPointsPerAttempt";
  trick_draw_policy: TrickDrawPolicy;
  throw_evaluation_policy: "All" | "Highest" | "TrickUnitLength";
  hide_played_cards: boolean;
  landlord_emoji: string | null;
  play_takeback_policy: "AllowPlayTakeback" | "NoPlayTakeback";
  bid_takeback_policy: "AllowBidTakeback" | "NoBidTakeback";
  kitty_theft_policy: "AllowKittyTheft" | "NoKittyTheft";
  game_shadowing_policy: "AllowMultipleSessions" | "SingleSessionOnly";
  game_start_policy: "AllowAnyPlayer" | "AllowLandlordOnly";
  game_scoring_parameters: IGameScoringParameters;
  should_reveal_kitty_at_end_of_game: boolean;
  hide_throw_halting_player: boolean;
}

export interface IGameScoringParameters {
  step_size_per_deck: number;
  step_adjustments: { [num_decks: number]: number };
  num_steps_to_non_landlord_turnover: number;
  deadzone_size: number;
  truncate_zero_crossing_window: boolean;
  bonus_level_policy: "NoBonusLevel" | "BonusLevelForSmallerLandlordTeam";
}

export interface IHands {
  hands: { [playerId: number]: { [card: string]: number } };
  level: number;
  trump: ITrump | null;
}

export type IGameMode = "Tractor" | { FindingFriends: IFindingFriends };
export type IGameModeSettings =
  | "Tractor"
  | { FindingFriends: { num_friends: number } };

export interface IFindingFriends {
  num_friends: number;
  friends: [IFriend];
}

export interface IFriend {
  card: string;
  skip: number;
  initial_skip: number;
  player_id: number | null;
}

export interface ICardInfo {
  value: string;
  display_value: string;
  typ: string;
  number: string | null;
  points: number;
}

export interface ITrick {
  player_queue: number[];
  played_cards: IPlayedCards[];
  played_card_mappings: ICardMapping[];
  current_winner: number | null;
  trump: ITrump;
  trick_format: ITrickFormat | null;
}

export interface IPlayedCards {
  id: number;
  cards: string[];
  bad_throw_cards: string[];
  better_player: number | null;
}

export type ICardMapping = ITrickUnit[] | null;

export interface ITrickUnit {
  Repeated?: {
    card: IOrderedCard;
    count: number;
  };
  Tractor?: {
    members: IOrderedCard[];
    count: number;
  };
}

export interface ITrickFormat {
  suit: string;
  trump: ITrump;
  units: ITrickUnit[];
}

export interface IUnitLike {
  Repeated?: {
    count: number;
  };
  Tractor?: {
    length: number;
    count: number;
  };
}

export interface IOrderedCard {
  card: string;
  // elided fields
}

export type ITrump =
  | {
      Standard: { suit: string; number: string };
      NoTrump?: null;
    }
  | {
      Standard?: null;
      NoTrump: { number: string };
    };

export interface IDeck {
  exclude_small_joker: boolean;
  exclude_big_joker: boolean;
  min: string;
}
