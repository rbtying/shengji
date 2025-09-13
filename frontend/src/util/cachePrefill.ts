import {
  Trump,
  CardInfo,
  GameScoringParameters,
  Deck,
  ExplainScoringResponse,
} from "../gen-types";
import type { EngineContext } from "../WasmOrRpcProvider";
import preloadedCards from "../preloadedCards";

// Cache for card info to avoid repeated async calls
export const cardInfoCache: { [key: string]: CardInfo } = {};

// Cache for explainScoring results
export const explainScoringCache: { [key: string]: ExplainScoringResponse } =
  {};

// Track which trumps are currently being prefilled to avoid duplicate requests
const prefillInProgressMap: { [trumpKey: string]: Promise<void> | null } = {};

export function isPrefillInProgress(trump: Trump): boolean {
  const trumpKey = getTrumpKey(trump);
  return prefillInProgressMap[trumpKey] !== undefined && prefillInProgressMap[trumpKey] !== null;
}

export function markPrefillStarted(trump: Trump, promise: Promise<void>): void {
  const trumpKey = getTrumpKey(trump);
  prefillInProgressMap[trumpKey] = promise;
}

function markPrefillCompleted(trump: Trump): void {
  const trumpKey = getTrumpKey(trump);
  delete prefillInProgressMap[trumpKey];
}

export function getPrefillPromise(trump: Trump): Promise<void> | null {
  const trumpKey = getTrumpKey(trump);
  return prefillInProgressMap[trumpKey] || null;
}

// Helper to create a stable cache key from trump
export const getTrumpKey = (trump: Trump): string => {
  if ("Standard" in trump) {
    return `std_${trump.Standard.suit}_${trump.Standard.number}`;
  } else if ("NoTrump" in trump) {
    return `nt_${trump.NoTrump.number || "none"}`;
  }
  return "unknown";
};

// Prefill card info cache for all cards with a given trump using batch API
export const prefillCardInfoCache = async (
  engine: EngineContext,
  trump: Trump,
): Promise<void> => {
  const trumpKey = getTrumpKey(trump);

  // Check if a prefill is already in progress for this trump
  const existingPromise = getPrefillPromise(trump);
  if (existingPromise) {
    console.log(`Prefill already in progress for trump ${trumpKey}, waiting...`);
    return existingPromise;
  }

  // Create the prefill promise
  const prefillPromise = (async () => {
    const requestsToMake: { card: string; cacheKey: string }[] = [];

    // Get all unique card values from preloadedCards
    for (const cardInfo of preloadedCards) {
      const cacheKey = `${cardInfo.value}_${trumpKey}`;

      // Skip if already cached
      if (cacheKey in cardInfoCache) {
        continue;
      }

      requestsToMake.push({ card: cardInfo.value, cacheKey });
    }

    // Also add the unknown card if not cached
    const unknownCacheKey = `🂠_${trumpKey}`;
    if (!(unknownCacheKey in cardInfoCache)) {
      requestsToMake.push({ card: "🂠", cacheKey: unknownCacheKey });
    }

    // If nothing to fetch, return early
    if (requestsToMake.length === 0) {
      console.log(`Card info cache already filled for trump ${trumpKey}`);
      return;
    }

    try {
      console.log(`Attempting to batch fetch ${requestsToMake.length} cards for trump ${trumpKey}`);

      // Use batch API to fetch all card info at once
      const batchResponse = await engine.batchGetCardInfo({
        requests: requestsToMake.map((r) => ({
          card: r.card,
          trump: trump,
        })),
      });

      console.log("Batch response received:", batchResponse);

      // Validate response structure
      if (!batchResponse || !batchResponse.results || !Array.isArray(batchResponse.results)) {
        throw new Error(`Invalid batch response structure: ${JSON.stringify(batchResponse)}`);
      }

      if (batchResponse.results.length !== requestsToMake.length) {
        console.warn(`Response length mismatch: expected ${requestsToMake.length}, got ${batchResponse.results.length}`);
      }

      // Store results in cache
      batchResponse.results.forEach((info, index) => {
        if (index >= requestsToMake.length) {
          console.warn(`Skipping extra response at index ${index}`);
          return;
        }
        const { cacheKey } = requestsToMake[index];
        cardInfoCache[cacheKey] = info;
      });

      console.log(
        `✅ Successfully prefilled card info cache for ${requestsToMake.length} cards with trump ${trumpKey} (single batch request)`,
      );
    } catch (error) {
      console.error("❌ Error batch fetching card info:", error);
      console.error("Error details:", error instanceof Error ? error.stack : error);

      // Fallback to individual requests or static data
      for (const { card, cacheKey } of requestsToMake) {
        const cardData = preloadedCards.find((c) => c.value === card);
        cardInfoCache[cacheKey] = {
          suit: null,
          effective_suit: "Unknown" as any,
          value: card,
          display_value: cardData?.display_value || card,
          typ: cardData?.typ || "unknown",
          number: cardData?.number || null,
          points: cardData?.points || 0,
        };
      }
    }
  })();

  // Store the promise and set up cleanup
  markPrefillStarted(trump, prefillPromise);

  // Clear the in-progress flag when done
  prefillPromise.finally(() => {
    markPrefillCompleted(trump);
  });

  return prefillPromise;
};

// Create a cache key for explainScoring requests
export const getExplainScoringKey = (
  params: GameScoringParameters,
  smallerLandlordTeamSize: boolean,
  decks: Deck[],
): string => {
  // Create a stable key based on the request parameters
  return JSON.stringify({
    params,
    smallerLandlordTeamSize,
    deckCount: decks.length,
    // We assume deck configuration is the same for a given count
  });
};

// Prefill explainScoring cache
export const prefillExplainScoringCache = async (
  engine: EngineContext,
  params: GameScoringParameters,
  decks: Deck[],
): Promise<void> => {
  const promises: Promise<void>[] = [];

  // Prefill both regular and bonus scoring
  for (const smallerTeamSize of [false, true]) {
    const cacheKey = getExplainScoringKey(params, smallerTeamSize, decks);

    if (cacheKey in explainScoringCache) {
      continue;
    }

    const promise = engine
      .explainScoring({
        params,
        smaller_landlord_team_size: smallerTeamSize,
        decks,
      })
      .then((result: ExplainScoringResponse) => {
        explainScoringCache[cacheKey] = result;
      })
      .catch((error: unknown) => {
        console.error(`Error prefilling explainScoring cache:`, error);
        // Fallback to empty result
        explainScoringCache[cacheKey] = {
          results: [],
          step_size: 10,
          total_points: 100,
        };
      });

    promises.push(promise);
  }

  await Promise.all(promises);
  console.log(`Prefilled explainScoring cache with ${promises.length} entries`);
};