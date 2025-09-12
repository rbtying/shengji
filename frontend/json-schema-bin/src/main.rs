use std::env;

use schemars::{schema_for, JsonSchema};
use shengji_core::interactive::Action;
use shengji_types::GameMessage;
use shengji_types::wasm_rpc::{
    CanPlayCardsRequest, CanPlayCardsResponse, CardInfo, CardInfoRequest, ComputeScoreRequest,
    ComputeScoreResponse, DecomposeTrickFormatRequest, DecomposeTrickFormatResponse,
    DecomposedTrickFormat, ExplainScoringRequest, ExplainScoringResponse, FindValidBidsRequest,
    FindValidBidsResult, FindViablePlaysRequest, FindViablePlaysResult, FoundViablePlay,
    NextThresholdReachableRequest, ScoreSegment, SortAndGroupCardsRequest,
    SortAndGroupCardsResponse, SuitGroup,
};
use tempdir::TempDir;

#[derive(JsonSchema)]
pub struct _Combined {
    pub action: Action,
    pub game_message: GameMessage,
    pub find_viable_plays_request: FindViablePlaysRequest,
    pub find_viable_plays_response: FindViablePlaysResult,
    pub found_viable_play: FoundViablePlay,
    pub decompose_trick_format_request: DecomposeTrickFormatRequest,
    pub decompose_trick_format_response: DecomposeTrickFormatResponse,
    pub decomposed_trick_format: DecomposedTrickFormat,
    pub can_play_cards_request: CanPlayCardsRequest,
    pub can_play_cards_response: CanPlayCardsResponse,
    pub find_valid_bids_request: FindValidBidsRequest,
    pub find_valid_bids_response: FindValidBidsResult,
    pub sort_and_group_cards_request: SortAndGroupCardsRequest,
    pub sort_and_group_cards_response: SortAndGroupCardsResponse,
    pub suit_group: SuitGroup,
    pub next_threshold_reachable_request: NextThresholdReachableRequest,
    pub explain_scoring_request: ExplainScoringRequest,
    pub explain_scoring_response: ExplainScoringResponse,
    pub score_segment: ScoreSegment,
    pub compute_score_request: ComputeScoreRequest,
    pub compute_score_response: ComputeScoreResponse,
    pub card_info_request: CardInfoRequest,
    pub card_info: CardInfo,
}

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let path = &args[1];
    let contents = serde_json::to_string_pretty(&schema_for!(_Combined)).unwrap();

    let tmp = TempDir::new("jsonschema").unwrap();
    let tmp_path = tmp.path().join("tmp.json");
    std::fs::write(&tmp_path, &contents).unwrap();

    let existing = std::fs::read(path);
    if let Ok(existing) = existing {
        if String::from_utf8(existing).unwrap() == contents {
            return;
        }
    }

    std::fs::copy(&tmp_path, path).unwrap();
}
