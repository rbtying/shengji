import * as React from "react";
import { ITrump, ITrickUnit, IBid, IHands, IPlayer, BidPolicy } from "./types";

interface Context {
  findViablePlays: (trump: ITrump, cards: string[]) => ITrickUnit[][];
  findValidBids: (req: IFindValidBidsRequest) => IBid[];
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

export const WasmContext = React.createContext<Context>({
  findViablePlays: (trump, cards) => [],
  findValidBids: (req) => [],
});

export default WasmContext;
