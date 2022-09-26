import * as React from "react";
import {
  Trump,
  Bid,
  Deck,
  TractorRequirements,
  FoundViablePlay,
  FindValidBidsRequest,
  SortAndGroupCardsRequest,
  SuitGroup,
  DecomposeTrickFormatRequest,
  DecomposedTrickFormat,
  CanPlayCardsRequest,
  ExplainScoringRequest,
  ExplainScoringResponse,
  NextThresholdReachableRequest,
  ComputeScoreRequest,
  ComputeScoreResponse,
  GameMessage,
} from "./gen-types";

interface Context {
  findViablePlays: (
    trump: Trump,
    tractorRequirements: TractorRequirements,
    cards: string[]
  ) => FoundViablePlay[];
  findValidBids: (req: FindValidBidsRequest) => Bid[];
  sortAndGroupCards: (req: SortAndGroupCardsRequest) => SuitGroup[];
  decomposeTrickFormat: (
    req: DecomposeTrickFormatRequest
  ) => DecomposedTrickFormat[];
  canPlayCards: (req: CanPlayCardsRequest) => boolean;
  explainScoring: (req: ExplainScoringRequest) => ExplainScoringResponse;
  nextThresholdReachable: (req: NextThresholdReachableRequest) => boolean;
  computeScore: (req: ComputeScoreRequest) => ComputeScoreResponse;
  computeDeckLen: (req: Deck[]) => number;
  decodeWireFormat: (req: Uint8Array) => GameMessage;
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
  decodeWireFormat: (_) => {
    throw new Error("cannot decode wire format");
  },
});

export default WasmContext;
