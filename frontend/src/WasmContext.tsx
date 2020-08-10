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
  IGameScoringParameters,
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
  explainScoring: (req: IExplainScoringRequest) => IScoreSegment[];
  computeScore: (req: IComputeScoreRequest) => IComputeScoreResponse;
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

interface IExplainScoringRequest {
  num_decks: number;
  params: IGameScoringParameters;
  smaller_landlord_team_size: boolean;
}

export interface IScoreSegment {
  point_threshold: number;
  results: IGameScoreResult;
}

interface IGameScoreResult {
  landlord_won: boolean;
  landlord_bonus: boolean;
  landlord_delta: number;
  non_landlord_delta: number;
}

interface IComputeScoreRequest {
  num_decks: number;
  params: IGameScoringParameters;
  smaller_landlord_team_size: boolean;
  non_landlord_points: number;
}

interface IComputeScoreResponse {
  score: IGameScoreResult;
  next_threshold: number;
}

export const WasmContext = React.createContext<Context>({
  findViablePlays: (trump, cards) => [],
  findValidBids: (req) => [],
  sortAndGroupCards: (req) => [],
  decomposeTrickFormat: (req) => [],
  canPlayCards: (req) => false,
  explainScoring: (req) => [],
  computeScore: (req) => ({
    score: {
      landlord_won: true,
      landlord_bonus: false,
      landlord_delta: 0,
      non_landlord_delta: 0,
    },
    next_threshold: 0,
  }),
});

export default WasmContext;
