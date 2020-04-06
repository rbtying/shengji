export interface IPlayer {
  id: number;
  name: string;
  level: string;
}

export interface IGameState {
  Initialize: IInitializePhase | null;
  Draw: IDrawPhase | null;
  Exchange: IExchangePhase | null;
  Play: IPlayPhase | null;
  Done: string | null;
}

export interface IInitializePhase {
  max_player_id: number;
  players: IPlayer[];
  num_decks: number | null;
  kitty_size: number | null;
  game_mode: IGameMode;
  landlord: number | null;
  hide_landlord_points: boolean | null;
}

export interface IBid {
  id: number;
  card: string;
  count: number;
}

export interface IDrawPhase {
  max_player_id: number;
  num_decks: number;
  game_mode: IGameMode;
  deck: string[];
  players: IPlayer[];
  observers: IPlayer[];
  hands: IHands;
  bids: IBid[];
  position: number;
  landlord: number | null;
  kitty: string[];
  level: number;
  hide_landlord_points: boolean | null;
}

export interface IExchangePhase {
  max_player_id: number;
  num_decks: number;
  game_mode: IGameMode;
  hands: IHands;
  kitty: string[];
  kitty_size: number;
  landlord: number;
  players: IPlayer[];
  observers: IPlayer[];
  trump: ITrump;
  hide_landlord_points: boolean | null;
}

export interface IPlayPhase {
  max_player_id: number;
  num_decks: number;
  game_mode: IGameMode;
  hands: IHands;
  points: {[id: number]: string[]};
  kitty: string[];
  landlord: number;
  landlords_team: number[];
  players: IPlayer[];
  observers: IPlayer[];
  trump: ITrump;
  trick: ITrick;
  last_trick: ITrick | null;
  hide_landlord_points: boolean | null;
}

export interface IHands {
  hands: {[playerId: number]: {[card: string]: number}};
  level: number;
  trump: ITrump | null;
}

export type IGameMode = 'Tractor' | {FindingFriends: IFindingFriends};

export interface IFindingFriends {
  num_friends: number;
  friends: [IFriend];
}

export interface IFriend {
  card: string;
  skip: number;
  player_id: number | null;
}

export interface ICardInfo {
  value: string;
  display_value: string;
  typ: string;
  number: string | null;
  points: number;
}

export interface IPlayer {
  id: number;
  name: string;
  level: string;
}

export interface ITrickUnit {
  Tractor: {count: number; members: string[]} | null;
  Repeated: {count: number; card: string} | null;
}

export interface ITrickFormat {
  suit: string;
  trump: ITrump;
  units: [ITrickUnit];
}

export interface ITrick {
  player_queue: number[];
  played_cards: {id: number; cards: string[]}[];
  current_winner: number | null;
  trick_format: ITrickFormat | null;
  trump: ITrump;
}

export type ITrump =
  | {
      Standard: {suit: string; number: string};
      NoTrump: null;
    }
  | {
      Standard: null;
      NoTrump: {number: string};
    };
