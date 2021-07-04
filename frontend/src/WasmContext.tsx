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
  BidReinforcementPolicy,
  IDeck,
  ITrick,
  TrickDrawPolicy,
  IGameScoringParameters,
  JokerBidPolicy,
  ITractorRequirements,
} from "./types";

interface Context {
  findViablePlays: (
    trump: ITrump,
    tractorRequirements: ITractorRequirements,
    cards: string[]
  ) => IFoundViablePlay[];
  findValidBids: (req: IFindValidBidsRequest) => IBid[];
  sortAndGroupCards: (
    req: ISortAndGroupCardsRequest
  ) => ISortedAndGroupedCards[];
  decomposeTrickFormat: (
    req: IDecomposeTrickFormatRequest
  ) => IDecomposedTrickFormat[];
  canPlayCards: (req: ICanPlayCardsRequest) => boolean;
  explainScoring: (req: IExplainScoringRequest) => IExplainScoringResponse;
  nextThresholdReachable: (req: INextThresholdReachableRequest) => boolean;
  computeScore: (req: IComputeScoreRequest) => IComputeScoreResponse;
  computeDeckLen: (req: IDeck[]) => number;
  decodeWireFormat: (req: any) => any;
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
  bid_reinforcement_policy: BidReinforcementPolicy;
  joker_bid_policy: JokerBidPolicy;
  num_decks: number;
}

interface ISortAndGroupCardsRequest {
  trump: ITrump | null;
  cards: string[];
}

export interface ISortedAndGroupedCards {
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
  decks: IDeck[];
  params: IGameScoringParameters;
  smaller_landlord_team_size: boolean;
}

interface INextThresholdReachableRequest {
  decks: IDeck[];
  params: IGameScoringParameters;
  non_landlord_points: number;
  observed_points: number;
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
  decks: IDeck[];
  params: IGameScoringParameters;
  smaller_landlord_team_size: boolean;
  non_landlord_points: number;
}

interface IComputeScoreResponse {
  score: IGameScoreResult;
  next_threshold: number;
}

interface IExplainScoringResponse {
  results: IScoreSegment[];
  step_size: number;
  total_points: number;
}

export const WasmContext = React.createContext<Context>({
  findViablePlays: (_, __) => [],
  findValidBids: (_) => [],
  sortAndGroupCards: (_) => [],
  decomposeTrickFormat: (_) => [],
  canPlayCards: (_) => false,
  explainScoring: (_) => ({ results: [], step_size: 0, total_points: 0 }),
  nextThresholdReachable: (_) => true,
  computeScore: (_) => ({
    score: {
      landlord_won: true,
      landlord_bonus: false,
      landlord_delta: 0,
      non_landlord_delta: 0,
    },
    next_threshold: 0,
  }),
  computeDeckLen: (_) => 0,
  decodeWireFormat: (_) => {},
});

export default WasmContext;
