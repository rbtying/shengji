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
