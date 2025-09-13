use std::cell::RefCell;
use std::io::{Cursor, Read};

use gloo_utils::format::JsValueSerdeExt;
use ruzstd::decoding::dictionary::Dictionary;
use ruzstd::frame_decoder::FrameDecoder;
use ruzstd::streaming_decoder::StreamingDecoder;
use shengji_types::wasm_rpc::{
    CanPlayCardsRequest, CardInfoRequest, ComputeDeckLenRequest, ComputeScoreRequest,
    DecomposeTrickFormatRequest, ExplainScoringRequest, FindValidBidsRequest,
    FindViablePlaysRequest, NextThresholdReachableRequest, NextThresholdReachableResponse,
    SortAndGroupCardsRequest,
};
use shengji_types::ZSTD_ZSTD_DICT;
use wasm_bindgen::prelude::*;

thread_local! {
    static ZSTD_DECODER: RefCell<Option<FrameDecoder>> = {
        let mut reader = Cursor::new(ZSTD_ZSTD_DICT);
        let mut decoder =
            StreamingDecoder::new(&mut reader).map_err(|_| "Failed to construct decoder").unwrap();
        let mut dict = Vec::new();
        decoder
            .read_to_end(&mut dict)
            .map_err(|e| format!("Failed to decode data {e:?}")).unwrap();

        let mut fd = FrameDecoder::new();
        fd.add_dict(Dictionary::decode_dict(&dict).unwrap()).unwrap();
        RefCell::new(Some(fd))
    };
}

#[wasm_bindgen]
pub fn find_viable_plays(req: JsValue) -> Result<JsValue, JsValue> {
    let request: FindViablePlaysRequest = req.into_serde().map_err(|e| e.to_string())?;
    let result = wasm_rpc_impl::find_viable_plays(request);
    Ok(JsValue::from_serde(&result).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn decompose_trick_format(req: JsValue) -> Result<JsValue, JsValue> {
    let request: DecomposeTrickFormatRequest = req.into_serde().map_err(|e| e.to_string())?;
    let result = wasm_rpc_impl::decompose_trick_format(request).map_err(|e| e.to_string())?;
    Ok(JsValue::from_serde(&result).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn can_play_cards(req: JsValue) -> Result<JsValue, JsValue> {
    let request: CanPlayCardsRequest = req.into_serde().map_err(|e| e.to_string())?;
    let result = wasm_rpc_impl::can_play_cards(request);
    Ok(JsValue::from_serde(&result).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn find_valid_bids(req: JsValue) -> Result<JsValue, JsValue> {
    let request: FindValidBidsRequest = req.into_serde().map_err(|e| e.to_string())?;
    let result = wasm_rpc_impl::find_valid_bids(request);
    Ok(JsValue::from_serde(&result).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn sort_and_group_cards(req: JsValue) -> Result<JsValue, JsValue> {
    let request: SortAndGroupCardsRequest = req.into_serde().map_err(|e| e.to_string())?;
    let result = wasm_rpc_impl::sort_and_group_cards(request);
    Ok(JsValue::from_serde(&result).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn next_threshold_reachable(req: JsValue) -> Result<JsValue, JsValue> {
    let request: NextThresholdReachableRequest = req.into_serde().map_err(|e| e.to_string())?;
    let reachable = wasm_rpc_impl::next_threshold_reachable(request).map_err(|e| JsValue::from_str(&e))?;
    // Return the same structure as the RPC version: { reachable: bool }
    Ok(JsValue::from_serde(&NextThresholdReachableResponse { reachable }).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn explain_scoring(req: JsValue) -> Result<JsValue, JsValue> {
    let request: ExplainScoringRequest = req.into_serde().map_err(|e| e.to_string())?;
    let result = wasm_rpc_impl::explain_scoring(request).map_err(|e| e.to_string())?;
    Ok(JsValue::from_serde(&result).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn compute_deck_len(req: JsValue) -> Result<usize, JsValue> {
    let request: ComputeDeckLenRequest = req.into_serde().map_err(|e| e.to_string())?;
    let result = wasm_rpc_impl::compute_deck_len(request);
    Ok(result.length)
}

#[wasm_bindgen]
pub fn compute_score(req: JsValue) -> Result<JsValue, JsValue> {
    let request: ComputeScoreRequest = req.into_serde().map_err(|e| e.to_string())?;
    let result = wasm_rpc_impl::compute_score(request).map_err(|e| e.to_string())?;
    Ok(JsValue::from_serde(&result).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn get_card_info(req: JsValue) -> Result<JsValue, JsValue> {
    let request: CardInfoRequest = req.into_serde().map_err(|e| e.to_string())?;
    let result = wasm_rpc_impl::get_card_info(request);
    Ok(JsValue::from_serde(&result).map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn zstd_decompress(req: &[u8]) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();

    let mut reader = Cursor::new(req);
    ZSTD_DECODER.with(|frame_decoder| {
        let mut decoder =
            StreamingDecoder::new_with_decoder(&mut reader, frame_decoder.take().unwrap())
                .map_err(|_| "Failed to construct decoder")?;
        let mut v = Vec::new();
        decoder
            .read_to_end(&mut v)
            .map_err(|e| format!("Failed to decode data {e:?}"))?;
        *(frame_decoder.borrow_mut()) = Some(decoder.inner());

        Ok(String::from_utf8(v).map_err(|_| "Failed to parse utf-8")?)
    })
}
