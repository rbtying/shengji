import * as React from "react";
import WasmContext from "./WasmContext";
import { isWasmAvailable } from "./detectWasm";
import {
  Trump,
  TractorRequirements,
  FoundViablePlay,
  FindValidBidsRequest,
  Bid,
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
  Deck,
  BatchCardInfoRequest,
  BatchCardInfoResponse,
  FindValidBidsResult,
  SortAndGroupCardsResponse,
  DecomposeTrickFormatResponse,
  CanPlayCardsResponse,
  FindViablePlaysRequest,
  FindViablePlaysResult,
} from "./gen-types";

import type { JSX } from "react";

interface IProps {
  children: React.ReactNode;
}

// Type for the dynamically imported WASM module
type ShengjiModule = typeof import("../shengji-wasm/pkg/shengji-core.js");

// Define the RPC request types
type WasmRpcRequest =
  | ({ type: "FindViablePlays" } & FindViablePlaysRequest)
  | ({ type: "FindValidBids" } & FindValidBidsRequest)
  | ({ type: "SortAndGroupCards" } & SortAndGroupCardsRequest)
  | ({ type: "DecomposeTrickFormat" } & DecomposeTrickFormatRequest)
  | ({ type: "CanPlayCards" } & CanPlayCardsRequest)
  | ({ type: "ExplainScoring" } & ExplainScoringRequest)
  | ({ type: "ComputeScore" } & ComputeScoreRequest)
  | ({ type: "NextThresholdReachable" } & NextThresholdReachableRequest)
  | { type: "ComputeDeckLen"; decks: Deck[] }
  | ({ type: "BatchGetCardInfo" } & BatchCardInfoRequest);

// Helper to make RPC calls to the server
async function callRpc<T>(request: WasmRpcRequest): Promise<T> {
  const response = await fetch("/api/rpc", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    throw new Error(`RPC call failed: ${response.statusText}`);
  }

  const result = await response.json();

  // Check if it's an error response
  if (result.type === "Error") {
    throw new Error(result.Error || "Unknown error");
  }

  // Extract the inner response based on the type
  const responseType = Object.keys(result)[0];
  return result[responseType];
}

// Create async versions of each function that can fallback to RPC
const createAsyncFunctions = (
  useWasm: boolean,
  wasmModule: ShengjiModule | null,
) => {
  if (useWasm && wasmModule) {
    // WASM is available and loaded, use synchronous WASM functions wrapped in promises
    return {
      findViablePlays: async (
        trump: Trump,
        tractorRequirements: TractorRequirements,
        cards: string[],
      ): Promise<FoundViablePlay[]> => {
        return wasmModule.find_viable_plays({
          trump,
          cards,
          tractor_requirements: tractorRequirements,
        }).results;
      },
      findValidBids: async (req: FindValidBidsRequest): Promise<Bid[]> => {
        return wasmModule.find_valid_bids(req).results;
      },
      sortAndGroupCards: async (
        req: SortAndGroupCardsRequest,
      ): Promise<SuitGroup[]> => {
        return wasmModule.sort_and_group_cards(req).results;
      },
      decomposeTrickFormat: async (
        req: DecomposeTrickFormatRequest,
      ): Promise<DecomposedTrickFormat[]> => {
        return wasmModule.decompose_trick_format(req).results;
      },
      canPlayCards: async (req: CanPlayCardsRequest): Promise<boolean> => {
        return wasmModule.can_play_cards(req).playable;
      },
      explainScoring: async (
        req: ExplainScoringRequest,
      ): Promise<ExplainScoringResponse> => {
        return wasmModule.explain_scoring(req);
      },
      nextThresholdReachable: async (
        req: NextThresholdReachableRequest,
      ): Promise<boolean> => {
        return wasmModule.next_threshold_reachable(req);
      },
      computeScore: async (
        req: ComputeScoreRequest,
      ): Promise<ComputeScoreResponse> => {
        return wasmModule.compute_score(req);
      },
      computeDeckLen: async (decks: Deck[]): Promise<number> => {
        return wasmModule.compute_deck_len({ decks });
      },
      batchGetCardInfo: async (
        req: BatchCardInfoRequest,
      ): Promise<BatchCardInfoResponse> => {
        // WASM doesn't have batch API, so call individually
        const results = req.requests.map((r) => wasmModule.get_card_info(r));
        return { results };
      },
    };
  } else {
    // WASM not available, use RPC calls
    return {
      findViablePlays: async (
        trump: Trump,
        tractorRequirements: TractorRequirements,
        cards: string[],
      ): Promise<FoundViablePlay[]> => {
        const response = await callRpc<FindViablePlaysResult>({
          type: "FindViablePlays",
          trump,
          tractor_requirements: tractorRequirements,
          cards,
        });
        return response.results;
      },
      findValidBids: async (req: FindValidBidsRequest): Promise<Bid[]> => {
        const response = await callRpc<FindValidBidsResult>({
          type: "FindValidBids",
          ...req,
        });
        return response.results;
      },
      sortAndGroupCards: async (
        req: SortAndGroupCardsRequest,
      ): Promise<SuitGroup[]> => {
        const response = await callRpc<SortAndGroupCardsResponse>({
          type: "SortAndGroupCards",
          ...req,
        });
        return response.results;
      },
      decomposeTrickFormat: async (
        req: DecomposeTrickFormatRequest,
      ): Promise<DecomposedTrickFormat[]> => {
        const response = await callRpc<DecomposeTrickFormatResponse>({
          type: "DecomposeTrickFormat",
          ...req,
        });
        return response.results;
      },
      canPlayCards: async (req: CanPlayCardsRequest): Promise<boolean> => {
        const response = await callRpc<CanPlayCardsResponse>({
          type: "CanPlayCards",
          ...req,
        });
        return response.playable;
      },
      explainScoring: async (
        req: ExplainScoringRequest,
      ): Promise<ExplainScoringResponse> => {
        return await callRpc<ExplainScoringResponse>({
          type: "ExplainScoring",
          ...req,
        });
      },
      nextThresholdReachable: async (
        req: NextThresholdReachableRequest,
      ): Promise<boolean> => {
        return await callRpc<boolean>({
          type: "NextThresholdReachable",
          ...req,
        });
      },
      computeScore: async (
        req: ComputeScoreRequest,
      ): Promise<ComputeScoreResponse> => {
        return await callRpc<ComputeScoreResponse>({
          type: "ComputeScore",
          ...req,
        });
      },
      computeDeckLen: async (decks: Deck[]): Promise<number> => {
        const response = await callRpc<{ length: number }>({
          type: "ComputeDeckLen",
          decks,
        });
        return response.length;
      },
      batchGetCardInfo: async (
        req: BatchCardInfoRequest,
      ): Promise<BatchCardInfoResponse> => {
        return await callRpc<BatchCardInfoResponse>({
          type: "BatchGetCardInfo",
          ...req,
        });
      },
    };
  }
};

// Create a new context for game engine functions
export interface EngineContext {
  findViablePlays: (
    trump: Trump,
    tractorRequirements: TractorRequirements,
    cards: string[],
  ) => Promise<FoundViablePlay[]>;
  findValidBids: (req: FindValidBidsRequest) => Promise<Bid[]>;
  sortAndGroupCards: (req: SortAndGroupCardsRequest) => Promise<SuitGroup[]>;
  decomposeTrickFormat: (
    req: DecomposeTrickFormatRequest,
  ) => Promise<DecomposedTrickFormat[]>;
  canPlayCards: (req: CanPlayCardsRequest) => Promise<boolean>;
  explainScoring: (
    req: ExplainScoringRequest,
  ) => Promise<ExplainScoringResponse>;
  nextThresholdReachable: (
    req: NextThresholdReachableRequest,
  ) => Promise<boolean>;
  computeScore: (req: ComputeScoreRequest) => Promise<ComputeScoreResponse>;
  computeDeckLen: (req: Deck[]) => Promise<number>;
  batchGetCardInfo: (
    req: BatchCardInfoRequest,
  ) => Promise<BatchCardInfoResponse>;
  decodeWireFormat: (req: Uint8Array) => unknown;
  isUsingWasm: boolean;
}

export const EngineContext = React.createContext<EngineContext | null>(null);

const WasmOrRpcProvider = (props: IProps): JSX.Element => {
  const useWasm = isWasmAvailable();
  const [wasmModule, setWasmModule] = React.useState<ShengjiModule | null>(
    null,
  );
  const [isLoading, setIsLoading] = React.useState(useWasm);

  // Load WASM module dynamically if available
  React.useEffect(() => {
    if (useWasm) {
      import("../shengji-wasm/pkg/shengji-core.js")
        .then((module) => {
          setWasmModule(module);
          // Set module on window for debugging
          (window as Window & { shengji?: ShengjiModule }).shengji = module;
          setIsLoading(false);
        })
        .catch((error) => {
          console.error("Failed to load WASM module:", error);
          setIsLoading(false);
        });
    } else {
      setIsLoading(false);
    }
  }, [useWasm]);

  const engineFuncs = React.useMemo(
    () => createAsyncFunctions(useWasm, wasmModule),
    [useWasm, wasmModule],
  );

  // Only provide decodeWireFormat in the synchronous context
  const syncContextValue = React.useMemo(
    () => ({
      decodeWireFormat: (req: Uint8Array) => {
        if (useWasm && wasmModule) {
          return JSON.parse(wasmModule.zstd_decompress(req));
        } else {
          // When WASM is not available, messages should already be decompressed
          // by the server, so we can just parse them directly
          const text = new TextDecoder().decode(req);
          return JSON.parse(text);
        }
      },
    }),
    [useWasm, wasmModule],
  );

  const engineContextValue: EngineContext = React.useMemo(
    () => ({
      ...engineFuncs,
      decodeWireFormat: syncContextValue.decodeWireFormat,
      isUsingWasm: useWasm && wasmModule !== null,
    }),
    [engineFuncs, syncContextValue, useWasm, wasmModule],
  );

  // Show loading indicator while WASM is being loaded
  if (isLoading) {
    return <div>Loading game engine...</div>;
  }

  return (
    <EngineContext.Provider value={engineContextValue}>
      <WasmContext.Provider value={syncContextValue}>
        {props.children}
      </WasmContext.Provider>
    </EngineContext.Provider>
  );
};

export default WasmOrRpcProvider;