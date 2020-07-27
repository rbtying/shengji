import * as React from "react";
import {
  ITrump,
  ITrickUnit,
  IBid,
  IHands,
  IPlayer,
  IUnitLike,
  ITrickFormat,
  BidPolicy,
  ITrick,
  TrickDrawPolicy,
} from "./types";

interface Context {
  findViablePlays: (trump: ITrump, cards: string[]) => IFoundViablePlay[];
  findValidBids: (req: IFindValidBidsRequest) => IBid[];
  sortAndGroupCards: (
    req: ISortAndGroupCardsRequest
  ) => ISortedAndGroupedCards[];
  decomposeTrickFormat: (
    req: IDecomposeTrickFormatRequest
  ) => IDecomposedTrickFormat[];
  canPlayCards: (req: ICanPlayCardsRequest) => boolean;
}

export interface IFoundViablePlay {
  grouping: ITrickUnit[];
  description: string;
}

interface IFindValidBidsRequest {
  id: number;
  bids: IBid[];
  hands: IHands;
  players: IPlayer[];
  landlord: number | null;
  epoch: number;
  bid_policy: BidPolicy;
}

interface ISortAndGroupCardsRequest {
  trump: ITrump | null;
  cards: string[];
}

interface ISortedAndGroupedCards {
  suit: string;
  cards: string[];
}

interface IDecomposedTrickFormat {
  description: string;
  format: IUnitLike[];
  playable: string[];
}

interface IDecomposeTrickFormatRequest {
  trick_format: ITrickFormat;
  hands: IHands;
  player_id: number;
  trick_draw_policy: TrickDrawPolicy;
}

interface ICanPlayCardsRequest {
  trick: ITrick;
  id: number;
  hands: IHands;
  cards: string[];
  trick_draw_policy: TrickDrawPolicy;
}

export const WasmContext = React.createContext<Context>({
  findViablePlays: (trump, cards) => [],
  findValidBids: (req) => [],
  sortAndGroupCards: (req) => [],
  decomposeTrickFormat: (req) => [],
  canPlayCards: (req) => false,
});

export default WasmContext;
