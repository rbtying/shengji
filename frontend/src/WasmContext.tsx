import * as React from "react";
import { ITrump, ITrickUnit, IBid, IHands, IPlayer, BidPolicy } from "./types";

interface Context {
  findViablePlays: (trump: ITrump, cards: string[]) => ITrickUnit[][];
  findValidBids: (req: IFindValidBidsRequest) => IBid[];
  sortAndGroupCards: (
    req: ISortAndGroupCardsRequest
  ) => ISortedAndGroupedCards[];
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

export const WasmContext = React.createContext<Context>({
  findViablePlays: (trump, cards) => [],
  findValidBids: (req) => [],
  sortAndGroupCards: (req) => [],
});

export default WasmContext;
