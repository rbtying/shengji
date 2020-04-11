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
  propagated: IPropagatedState;
}

export interface IBid {
  id: number;
  card: string;
  count: number;
}

export interface IDrawPhase {
  num_decks: number;
  game_mode: IGameMode;
  deck: string[];
  propagated: IPropagatedState;
  hands: IHands;
  bids: IBid[];
  position: number;
  kitty: string[];
  level: number;
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
}

export interface IPlayPhase {
  num_decks: number;
  game_mode: IGameMode;
  propagated: IPropagatedState;
  hands: IHands;
  points: {[id: number]: string[]};
  kitty: string[];
  landlord: number;
  landlords_team: number[];
  trump: ITrump;
  trick: ITrick;
  last_trick: ITrick | null;
}

export interface IPropagatedState {
  game_mode: IGameMode;
  hide_landlord_points: boolean | null;
  kitty_size: number | null;
  num_decks: number | null;
  max_player_id: number;
  players: IPlayer[];
  observers: IPlayer[];
  landlord: number | null;
  chat_link: string | null;
}

export interface IHands {
  hands: {[playerId: number]: {[card: string]: number}};
  level: number;
  trump: ITrump | null;
}

export type IGameMode = 'Tractor' | {FindingFriends: IFindingFriends};

export interface IFindingFriends {
  num_friends: number;
  friends?: [IFriend];
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
