import * as React from "react";
import WasmContext from "./WasmContext";
import { isWasmAvailable } from "./detectWasm";
import { prefillCardInfoCache } from "./util/cachePrefill";
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
  console.log("RPC Request object:", request);

  const bodyString = JSON.stringify(request);
  console.log("RPC Request JSON string:", bodyString);

  const response = await fetch("/api/rpc", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: bodyString,
  });

  if (!response.ok) {
    const errorText = await response.text();
    console.error(`RPC call failed with status ${response.status}:`, errorText);
    console.error("Failed request was:", bodyString);
    throw new Error(`RPC call failed: ${response.statusText}`);
  }

  const responseText = await response.text();
  let result;
  try {
    result = JSON.parse(responseText);
  } catch (e) {
    console.error("Failed to parse JSON response:", responseText);
    throw new Error(
      `Invalid JSON response from server: ${responseText.substring(0, 100)}`,
    );
  }

  // Check if it's an error response
  if (result.type === "Error") {
    throw new Error(result.Error || "Unknown error");
  }

  // Since the response uses serde tag="type", the structure is { type: "ResponseType", ...data }
  // We need to return the whole result minus the type field for most responses
  // or extract based on the actual response structure

  if (!result.type) {
    console.error("Invalid RPC response - missing type field:", result);
    throw new Error("Invalid RPC response structure");
  }

  // For tagged enums, the data is directly in the result object
  // Remove the type field and return the rest
  const { type, ...responseData } = result;

  // Some responses might be wrapped, others might have the data directly
  // BatchGetCardInfo should have results directly in responseData
  return responseData as T;
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
        const response = wasmModule.next_threshold_reachable(req);
        return response.reachable;
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
        console.log("FindValidBids input request:", req);

        // The issue is that JavaScript objects with numeric-looking keys
        // get serialized as strings ("0", "1", etc.) in JSON.
        // But Rust's serde_json expects actual numbers for HashMap<PlayerID, _>
        // where PlayerID wraps usize.
        //
        // WASM works because serde_wasm_bindgen handles this automatically,
        // but serde_json does not. This is a known limitation.
        //
        // We need to send the request in a format that serde_json can handle.
        // The backend would need to be updated to handle this properly,
        // or we need a workaround.

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
        const response = await callRpc<{ reachable: boolean }>({
          type: "NextThresholdReachable",
          ...req,
        });
        return response.reachable;
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
        const response = await callRpc<BatchCardInfoResponse>({
          type: "BatchGetCardInfo",
          ...req,
        });
        // Log the response for debugging
        console.log("BatchGetCardInfo RPC response:", response);
        return response;
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
      console.log("Loading WASM module...");
      import("../shengji-wasm/pkg/shengji-core.js")
        .then((module) => {
          setWasmModule(module);
          // Set module on window for debugging
          (window as Window & { shengji?: ShengjiModule }).shengji = module;
          console.log("✅ WASM module loaded successfully");
          setIsLoading(false);
        })
        .catch((error) => {
          console.error("Failed to load WASM module:", error);
          setIsLoading(false);
        });
    } else {
      console.log("🔄 Using server-side RPC fallback (no-WASM mode)");
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

  // Track if initial prefill is complete
  const [isPrefillComplete, setIsPrefillComplete] = React.useState(false);

  // Eagerly prefill cache for common trump configurations when engine is ready
  React.useEffect(() => {
    if (!isLoading && engineContextValue && !isPrefillComplete) {
      console.log(
        "Engine ready, eagerly prefilling card cache for common trumps...",
      );

      // Create an array of prefill promises
      const prefillPromises: Promise<void>[] = [];

      // Prefill for NoTrump (used in JoinRoom for the joker cards display)
      const noTrumpBasic: Trump = { NoTrump: {} };
      prefillPromises.push(
        prefillCardInfoCache(engineContextValue, noTrumpBasic)
          .then(() => console.log("✅ Prefilled cache for NoTrump (no rank)"))
          .catch((error) =>
            console.error("Failed to prefill NoTrump cache:", error),
          ),
      );

      // Also prefill for NoTrump with rank 2 (most common starting rank)
      const noTrump2: Trump = { NoTrump: { number: "2" } };
      prefillPromises.push(
        prefillCardInfoCache(engineContextValue, noTrump2)
          .then(() => console.log("✅ Prefilled cache for NoTrump rank 2"))
          .catch((error) =>
            console.error("Failed to prefill NoTrump rank 2 cache:", error),
          ),
      );

      // Wait for all prefills to complete before marking as done
      Promise.all(prefillPromises).then(() => {
        setIsPrefillComplete(true);
        console.log("✅ All initial prefills complete");
      });
    }
  }, [isLoading, engineContextValue, isPrefillComplete]);

  // Show loading indicator while WASM is being loaded or initial cache is being prefilled
  if (isLoading) {
    return <div>Loading game engine...</div>;
  }

  // Optionally wait for prefill to complete before rendering children
  // This prevents the initial cards from making individual requests
  if (!isPrefillComplete) {
    return <div>Initializing game data...</div>;
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
