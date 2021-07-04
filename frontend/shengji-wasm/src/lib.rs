use std::io::{Cursor, Read};
use std::sync::Mutex;

use ruzstd::frame_decoder::FrameDecoder;
use ruzstd::streaming_decoder::StreamingDecoder;
use serde::{Deserialize, Serialize};
use shengji_core::{
    bidding::{Bid, BidPolicy, BidReinforcementPolicy, JokerBidPolicy},
    deck::Deck,
    hands::Hands,
    player::Player,
    scoring::{
        self, compute_level_deltas, explain_level_deltas, GameScoreResult, GameScoringParameters,
    },
    trick::{TractorRequirements, Trick, TrickDrawPolicy, TrickFormat, TrickUnit, UnitLike},
    types::{Card, EffectiveSuit, PlayerID, Trump},
};
use shengji_types::ZSTD_ZSTD_DICT;
use wasm_bindgen::prelude::*;
// use web_sys::console;

lazy_static::lazy_static! {
    static ref ZSTD_DICT: Vec<u8> = {
        let mut reader = Cursor::new(ZSTD_ZSTD_DICT);
        let mut decoder =
            StreamingDecoder::new(&mut reader).map_err(|_| "Failed to construct decoder").unwrap();
        let mut v = Vec::new();
        decoder
            .read_to_end(&mut v)
            .map_err(|e| format!("Failed to decode data {:?}", e)).unwrap();
        v
    };
    static ref ZSTD_DECODER: Mutex<Option<FrameDecoder>> = {
        let mut fd = FrameDecoder::new();
        fd.add_dict(&ZSTD_DICT).unwrap();
        Mutex::new(Some(fd))
    };
}

#[derive(Deserialize)]
struct FindViablePlaysRequest {
    trump: Trump,
    tractor_requirements: TractorRequirements,
    cards: Vec<Card>,
}

#[derive(Serialize)]
struct FindViablePlaysResult {
    results: Vec<FoundViablePlay>,
}

#[derive(Serialize)]
struct FoundViablePlay {
    grouping: Vec<TrickUnit>,
    description: String,
}

#[wasm_bindgen]
pub fn find_viable_plays(req: JsValue) -> Result<JsValue, JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let FindViablePlaysRequest {
        trump,
        cards,
        tractor_requirements,
    } = req.into_serde().map_err(|e| e.to_string())?;
    let results = TrickUnit::find_plays(trump, tractor_requirements, cards)
        .into_iter()
        .map(|p| {
            let description = UnitLike::multi_description(p.iter().map(UnitLike::from));
            FoundViablePlay {
                grouping: p,
                description,
            }
        })
        .collect::<Vec<_>>();
    Ok(JsValue::from_serde(&FindViablePlaysResult { results }).map_err(|e| e.to_string())?)
}

#[derive(Deserialize)]
struct DecomposeTrickFormatRequest {
    trick_format: TrickFormat,
    hands: Hands,
    player_id: PlayerID,
    trick_draw_policy: TrickDrawPolicy,
}

#[derive(Serialize)]
struct DecomposeTrickFormatResponse {
    results: Vec<DecomposedTrickFormat>,
}

#[derive(Serialize)]
struct DecomposedTrickFormat {
    format: Vec<UnitLike>,
    description: String,
    playable: Vec<Card>,
}

#[wasm_bindgen]
pub fn decompose_trick_format(req: JsValue) -> Result<JsValue, JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let DecomposeTrickFormatRequest {
        trick_format,
        hands,
        player_id,
        trick_draw_policy,
    } = req.into_serde().map_err(|e| e.to_string())?;

    let hand = hands.get(player_id).map_err(|e| e.to_string())?;
    let available_cards = Card::cards(
        hand.iter()
            .filter(|(c, _)| trick_format.trump().effective_suit(**c) == trick_format.suit()),
    )
    .copied()
    .collect::<Vec<_>>();

    let results = trick_format
        .decomposition(trick_draw_policy)
        .map(|format| {
            let description = UnitLike::multi_description(format.iter().cloned());
            let (playable, units) = UnitLike::check_play(
                trick_format.trump(),
                available_cards.iter().copied(),
                format.iter().cloned(),
                trick_draw_policy,
            );
            DecomposedTrickFormat {
                format,
                description,
                playable: if playable {
                    units
                        .into_iter()
                        .flat_map(|u| {
                            u.into_iter()
                                .flat_map(|(card, count)| std::iter::repeat(card.card).take(count))
                                .collect::<Vec<_>>()
                        })
                        .collect()
                } else {
                    vec![]
                },
            }
        })
        .collect();
    Ok(
        JsValue::from_serde(&DecomposeTrickFormatResponse { results })
            .map_err(|e| e.to_string())?,
    )
}

#[derive(Deserialize)]
struct CanPlayCardsRequest {
    trick: Trick,
    id: PlayerID,
    hands: Hands,
    cards: Vec<Card>,
    trick_draw_policy: TrickDrawPolicy,
}

#[derive(Serialize)]
struct CanPlayCardsResponse {
    playable: bool,
}

#[wasm_bindgen]
pub fn can_play_cards(req: JsValue) -> Result<JsValue, JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let CanPlayCardsRequest {
        trick,
        id,
        hands,
        cards,
        trick_draw_policy,
    } = req.into_serde().map_err(|e| e.to_string())?;
    Ok(JsValue::from_serde(&CanPlayCardsResponse {
        playable: trick
            .can_play_cards(id, &hands, &cards, trick_draw_policy)
            .is_ok(),
    })
    .map_err(|e| e.to_string())?)
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
    bid_reinforcement_policy: BidReinforcementPolicy,
    joker_bid_policy: JokerBidPolicy,
    num_decks: usize,
}

#[derive(Serialize)]
struct FindValidBidsResult {
    results: Vec<Bid>,
}

#[wasm_bindgen]
pub fn find_valid_bids(req: JsValue) -> Result<JsValue, JsValue> {
    #[cfg(debug_assertions)]
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
            req.bid_reinforcement_policy,
            req.joker_bid_policy,
            req.num_decks,
        )
        .unwrap_or_default(),
    })
    .map_err(|e| e.to_string())?)
}

#[derive(Deserialize)]
struct SortAndGroupCardsRequest {
    trump: Trump,
    cards: Vec<Card>,
}

#[derive(Serialize)]
struct SortAndGroupCardsResponse {
    results: Vec<SuitGroup>,
}

#[derive(Serialize)]
struct SuitGroup {
    suit: EffectiveSuit,
    cards: Vec<Card>,
}

#[wasm_bindgen]
pub fn sort_and_group_cards(req: JsValue) -> Result<JsValue, JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let SortAndGroupCardsRequest { trump, mut cards } =
        req.into_serde().map_err(|e| e.to_string())?;

    cards.sort_by(|a, b| trump.compare(*a, *b));

    let mut results: Vec<SuitGroup> = vec![];
    for card in cards {
        let suit = trump.effective_suit(card);
        if let Some(group) = results.last_mut() {
            if group.suit == suit {
                group.cards.push(card);
                continue;
            }
        }
        results.push(SuitGroup {
            suit,
            cards: vec![card],
        })
    }

    Ok(JsValue::from_serde(&SortAndGroupCardsResponse { results }).map_err(|e| e.to_string())?)
}

#[derive(Deserialize)]
struct NextThresholdReachableRequest {
    decks: Vec<Deck>,
    params: GameScoringParameters,
    non_landlord_points: isize,
    observed_points: isize,
}

#[wasm_bindgen]
pub fn next_threshold_reachable(req: JsValue) -> Result<bool, JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let NextThresholdReachableRequest {
        decks,
        params,
        non_landlord_points,
        observed_points,
    } = req.into_serde().map_err(|e| e.to_string())?;
    Ok(
        scoring::next_threshold_reachable(&params, &decks, non_landlord_points, observed_points)
            .map_err(|_| "Failed to determine if next threshold is reachable")?,
    )
}

#[derive(Deserialize)]
struct ExplainScoringRequest {
    decks: Vec<Deck>,
    params: GameScoringParameters,
    smaller_landlord_team_size: bool,
}

#[derive(Serialize)]
struct ExplainScoringResponse {
    results: Vec<ScoreSegment>,
    total_points: isize,
    step_size: usize,
}

#[derive(Serialize)]
struct ScoreSegment {
    point_threshold: isize,
    results: GameScoreResult,
}

#[wasm_bindgen]
pub fn explain_scoring(req: JsValue) -> Result<JsValue, JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let ExplainScoringRequest {
        decks,
        params,
        smaller_landlord_team_size,
    } = req.into_serde().map_err(|e| e.to_string())?;
    let deltas = explain_level_deltas(&params, &decks, smaller_landlord_team_size)
        .map_err(|e| format!("Failed to explain scores: {:?}", e))?;

    Ok(JsValue::from_serde(&ExplainScoringResponse {
        results: deltas
            .into_iter()
            .map(|(pts, res)| ScoreSegment {
                point_threshold: pts,
                results: res,
            })
            .collect(),
        step_size: params
            .step_size(&decks)
            .map_err(|e| format!("Failed to compute step size: {:?}", e))?,
        total_points: decks.iter().map(|d| d.points() as isize).sum::<isize>(),
    })
    .map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn compute_deck_len(req: JsValue) -> Result<usize, JsValue> {
    let decks: Vec<Deck> = req.into_serde().map_err(|e| e.to_string())?;

    Ok(decks.iter().map(|d| d.len() as usize).sum::<usize>())
}

#[derive(Deserialize)]
struct ComputeScoreRequest {
    decks: Vec<Deck>,
    params: GameScoringParameters,
    smaller_landlord_team_size: bool,
    non_landlord_points: isize,
}

#[derive(Serialize)]
struct ComputeScoreResponse {
    score: GameScoreResult,
    next_threshold: isize,
}

#[wasm_bindgen]
pub fn compute_score(req: JsValue) -> Result<JsValue, JsValue> {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();

    let ComputeScoreRequest {
        decks,
        params,
        smaller_landlord_team_size,
        non_landlord_points,
    } = req.into_serde().map_err(|e| e.to_string())?;
    let score = compute_level_deltas(
        &params,
        &decks,
        non_landlord_points,
        smaller_landlord_team_size,
    )
    .map_err(|_| "Failed to compute score")?;
    let next_threshold = params
        .materialize(&decks)
        .and_then(|n| n.next_relevant_score(non_landlord_points))
        .map_err(|_| "Couldn't find next valid score")?
        .0;

    Ok(JsValue::from_serde(&ComputeScoreResponse {
        score,
        next_threshold,
    })
    .map_err(|e| e.to_string())?)
}

#[wasm_bindgen]
pub fn zstd_decompress(req: &[u8]) -> Result<String, JsValue> {
    let mut reader = Cursor::new(req);
    let mut frame_decoder = ZSTD_DECODER.lock().unwrap();
    let mut decoder =
        StreamingDecoder::new_with_decoder(&mut reader, frame_decoder.take().unwrap())
            .map_err(|_| "Failed to construct decoder")?;
    let mut v = Vec::new();
    decoder
        .read_to_end(&mut v)
        .map_err(|e| format!("Failed to decode data {:?}", e))?;
    *frame_decoder = Some(decoder.inner());
    drop(frame_decoder);
    Ok(String::from_utf8(v).map_err(|_| "Failed to parse utf-8")?)
}
