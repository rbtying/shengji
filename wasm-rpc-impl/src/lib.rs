use shengji_mechanics::{
    bidding::Bid,
    ordered_card::OrderedCard,
    scoring::{self, compute_level_deltas, explain_level_deltas},
    trick::{TrickUnit, UnitLike},
    types::Card,
};

use shengji_types::wasm_rpc::{
    CanPlayCardsRequest, CanPlayCardsResponse, CardInfo, CardInfoRequest, ComputeDeckLenRequest,
    ComputeDeckLenResponse, ComputeScoreRequest, ComputeScoreResponse,
    DecomposeTrickFormatRequest, DecomposeTrickFormatResponse, DecomposedTrickFormat,
    ExplainScoringRequest, ExplainScoringResponse, FindValidBidsRequest, FindValidBidsResult,
    FindViablePlaysRequest, FindViablePlaysResult, FoundViablePlay, NextThresholdReachableRequest,
    ScoreSegment, SortAndGroupCardsRequest, SortAndGroupCardsResponse, SuitGroup,
};

pub fn find_viable_plays(req: FindViablePlaysRequest) -> FindViablePlaysResult {
    let results = TrickUnit::find_plays(req.trump, req.tractor_requirements, req.cards)
        .into_iter()
        .map(|p| {
            let description = UnitLike::multi_description(p.iter().map(UnitLike::from));
            FoundViablePlay {
                grouping: p,
                description,
            }
        })
        .collect::<Vec<_>>();
    FindViablePlaysResult { results }
}

pub fn decompose_trick_format(
    req: DecomposeTrickFormatRequest,
) -> Result<DecomposeTrickFormatResponse, String> {
    let hand = req.hands.get(req.player_id).map_err(|e| e.to_string())?;
    let available_cards = Card::cards(
        hand.iter()
            .filter(|(c, _)| req.trick_format.trump().effective_suit(**c) == req.trick_format.suit()),
    )
    .copied()
    .collect::<Vec<_>>();

    let mut results: Vec<_> = req
        .trick_format
        .decomposition(req.trick_draw_policy)
        .map(|format| {
            let description = UnitLike::multi_description(format.iter().cloned());
            DecomposedTrickFormat {
                format,
                description,
                playable: vec![],
                more_than_one: false,
            }
        })
        .collect();

    for res in results.iter_mut() {
        let mut iter = UnitLike::check_play(
            OrderedCard::make_map(available_cards.iter().copied(), req.trick_format.trump()),
            res.format.iter().cloned(),
            req.trick_draw_policy,
        );

        let playable = if let Some(units) = iter.next() {
            units
                .into_iter()
                .flat_map(|u| {
                    u.into_iter()
                        .flat_map(|(card, count)| std::iter::repeat_n(card.card, count))
                        .collect::<Vec<_>>()
                })
                .collect()
        } else {
            vec![]
        };

        if !playable.is_empty() {
            res.playable = playable;
            res.more_than_one = iter.next().is_some();
            break;
        }
    }

    Ok(DecomposeTrickFormatResponse { results })
}

pub fn can_play_cards(req: CanPlayCardsRequest) -> CanPlayCardsResponse {
    let playable = req
        .trick
        .can_play_cards(req.id, &req.hands, &req.cards, req.trick_draw_policy)
        .is_ok();
    CanPlayCardsResponse { playable }
}

pub fn find_valid_bids(req: FindValidBidsRequest) -> FindValidBidsResult {
    let results = Bid::valid_bids(
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
    .unwrap_or_default();
    FindValidBidsResult { results }
}

pub fn sort_and_group_cards(mut req: SortAndGroupCardsRequest) -> SortAndGroupCardsResponse {
    let trump = req.trump;
    req.cards.sort_by(|a, b| trump.compare(*a, *b));

    let mut results: Vec<SuitGroup> = vec![];
    for card in req.cards {
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

    SortAndGroupCardsResponse { results }
}

pub fn next_threshold_reachable(
    req: NextThresholdReachableRequest,
) -> Result<bool, String> {
    scoring::next_threshold_reachable(
        &req.params,
        &req.decks,
        req.non_landlord_points,
        req.observed_points,
    )
    .map_err(|_| "Failed to determine if next threshold is reachable".to_string())
}

pub fn explain_scoring(
    req: ExplainScoringRequest,
) -> Result<ExplainScoringResponse, String> {
    let deltas = explain_level_deltas(&req.params, &req.decks, req.smaller_landlord_team_size)
        .map_err(|e| format!("Failed to explain scores: {:?}", e))?;

    Ok(ExplainScoringResponse {
        results: deltas
            .into_iter()
            .map(|(pts, res)| ScoreSegment {
                point_threshold: pts,
                results: res,
            })
            .collect(),
        step_size: req
            .params
            .step_size(&req.decks)
            .map_err(|e| format!("Failed to compute step size: {:?}", e))?,
        total_points: req.decks.iter().map(|d| d.points() as isize).sum::<isize>(),
    })
}

pub fn compute_score(req: ComputeScoreRequest) -> Result<ComputeScoreResponse, String> {
    let score = compute_level_deltas(
        &req.params,
        &req.decks,
        req.non_landlord_points,
        req.smaller_landlord_team_size,
    )
    .map_err(|_| "Failed to compute score".to_string())?;

    let next_threshold = req
        .params
        .materialize(&req.decks)
        .and_then(|n| n.next_relevant_score(req.non_landlord_points))
        .map_err(|_| "Couldn't find next valid score".to_string())?
        .0;

    Ok(ComputeScoreResponse {
        score,
        next_threshold,
    })
}

pub fn compute_deck_len(req: ComputeDeckLenRequest) -> ComputeDeckLenResponse {
    let length = req.decks.iter().map(|d| d.len()).sum::<usize>();
    ComputeDeckLenResponse { length }
}

pub fn get_card_info(req: CardInfoRequest) -> CardInfo {
    let info = req.card.as_info();
    let effective_suit = req.trump.effective_suit(req.card);

    CardInfo {
        suit: req.card.suit(),
        value: info.value,
        display_value: info.display_value,
        typ: info.typ,
        number: info.number.map(|s| s.to_string()),
        points: info.points,
        effective_suit,
    }
}