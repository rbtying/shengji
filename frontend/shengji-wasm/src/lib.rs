use serde::{Deserialize, Serialize};
use shengji_core::{
    bidding::{Bid, BidPolicy},
    game_state::Player,
    hands::Hands,
    trick::TrickUnit,
    types::{Card, PlayerID, Trump},
};
use smallvec::SmallVec;
use wasm_bindgen::prelude::*;
// use web_sys::console;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[derive(Deserialize)]
struct FindViablePlaysRequest {
    trump: Trump,
    cards: Vec<Card>,
}

#[derive(Serialize)]
struct FindViablePlaysResult {
    results: Vec<SmallVec<[TrickUnit; 4]>>,
}

#[wasm_bindgen]
pub fn find_viable_plays(req: JsValue) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();

    let FindViablePlaysRequest { trump, cards } = req
        .into_serde()
        .map_err(|_| "Failed to deserialize request")?;
    let results = TrickUnit::find_plays(trump, cards)
        .into_iter()
        .collect::<Vec<_>>();
    Ok(JsValue::from_serde(&FindViablePlaysResult { results })
        .map_err(|_| "failed to serialize response")?)
}

#[derive(Deserialize)]
struct FindValidBidsRequest {
    id: PlayerID,
    bids: Vec<Bid>,
    hands: Hands,
    players: Vec<Player>,
    landlord: Option<PlayerID>,
    epoch: usize,
    bid_policy: BidPolicy,
}

#[derive(Serialize)]
struct FindValidBidsResult {
    results: Vec<Bid>,
}

#[wasm_bindgen]
pub fn find_valid_bids(req: JsValue) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();

    let req: FindValidBidsRequest = req
        .into_serde()
        .map_err(|_| "Failed to deserialize phase")?;
    Ok(JsValue::from_serde(&FindValidBidsResult {
        results: Bid::valid_bids(
            req.id,
            &req.bids,
            &req.hands,
            &req.players,
            req.landlord,
            req.epoch,
            req.bid_policy,
        )
        .unwrap_or_default(),
    })
    .map_err(|_| "failed to serialize response")?)
}
