# Testing No-WASM Mode and Auto-Prefill

## Setup Complete
- Frontend built successfully with `yarn dev`
- Backend is running with `cargo run --features dynamic`

## Key Features Implemented

### 1. URL Parameter for No-WASM Mode
- Access the application with `?no-wasm=true` to force RPC mode
- Example: `http://localhost:8080/?no-wasm=true`

### 2. Fixed RPC Serialization Issues
- Added custom Deserialize for PlayerID to handle string keys from JSON
- Added custom Deserialize for Hands struct to parse HashMap<PlayerID, _> correctly
- This fixes the "422 Unprocessable Entity" error with FindValidBids

### 3. Automatic Cache Prefilling
- WasmOrRpcProvider eagerly prefills cache for NoTrump configurations on startup
- Card.tsx detects uncached trumps and triggers automatic prefill
- Prevents O(n) duplicate requests with promise tracking mechanism
- Exchange phase includes prefill on mount

## Testing Steps

1. **Test No-WASM Mode:**
   - Open browser to `http://localhost:8080/?no-wasm=true`
   - Check console for "No-WASM mode enabled via URL parameter"
   - Verify "🔄 Using server-side RPC fallback (no-WASM mode)" appears

2. **Test Card Prefilling:**
   - Create/join a room
   - Check console for prefill messages:
     - "✅ Prefilled cache for NoTrump (no rank)"
     - "✅ Prefilled cache for NoTrump rank 2"
   - Verify no individual batchGetCardInfo requests for joker cards

3. **Test Auto-Prefill on New Trump:**
   - Start a game and bid with a new trump
   - Check console for "Detected uncached trump X, triggering full prefill..."
   - Verify single batch prefill completes
   - Confirm no duplicate prefill requests

4. **Test Exchange Phase:**
   - Progress to exchange phase
   - Verify "Exchange: Prefilling cache for trump" message
   - Confirm no excessive individual card requests

## Implementation Details

### Files Modified:
- `frontend/src/detectWasm.ts` - URL parameter detection
- `frontend/src/WasmOrRpcProvider.tsx` - Eager prefilling and loading states
- `frontend/src/Card.tsx` - Auto-prefill detection logic
- `frontend/src/Exchange.tsx` - Exchange phase prefilling
- `frontend/src/util/cachePrefill.ts` - Promise-based tracking mechanism
- `mechanics/src/types.rs` - PlayerID Display/FromStr/Deserialize
- `mechanics/src/hands.rs` - Custom Deserialize for Hands

### Key Mechanisms:
1. **Promise Tracking:** Prevents duplicate prefills with `prefillInProgressMap`
2. **Cache Detection:** Counts cached entries to determine if prefill needed
3. **Threshold:** Triggers prefill if <5 cards cached for a trump
4. **Cleanup:** Removes promise from tracking map after completion