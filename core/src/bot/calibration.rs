//! Corrects the coach's simulated win chances for how strongly the other players really play.
//!
//! The simulations play the rest of the hand with a fast rollout policy for every seat. Strong
//! bots play better than that, so raw win rates drift (too hopeful when attacking, too gloomy on
//! the landlord's side). Self-play measured the drift per opponent level, game and side, and a
//! logistic fit maps the raw rate back: logit(q) = a + b * logit(p).
//!
//! Regenerate the table with the `selfplay` tool (see selfplay/README.md).

use shengji_mechanics::types::PlayerID;

use super::Difficulty;
use crate::game_state::play_phase::PlayPhase;
use crate::settings::GameMode;

/// (opponents, Finding Friends?, on the landlord's side?, a, b)
const FITS: &[(Difficulty, bool, bool, f64, f64)] = &[];

/// Whether `me` is on the landlord's side, as far as `me` can know: the landlord's team, or
/// holding a called friend card that hasn't been played yet.
pub fn on_landlord_side(p: &PlayPhase, me: PlayerID) -> bool {
    if p.landlords_team.contains(&me) {
        return true;
    }
    if let GameMode::FindingFriends { friends, .. } = &p.game_mode {
        let hand = p.hands.get(me).ok();
        return friends.iter().any(|f| {
            f.player_id.is_none()
                && hand
                    .map(|h| h.get(&f.card).copied().unwrap_or(0) > 0)
                    .unwrap_or(false)
        });
    }
    false
}

/// The corrected win chance, or `p` unchanged when there's no fit for this table.
pub(crate) fn adjust(p: f64, opponents: Difficulty, friends: bool, landlord_side: bool) -> f64 {
    let Some(&(_, _, _, a, b)) = FITS
        .iter()
        .find(|f| f.0 == opponents && f.1 == friends && f.2 == landlord_side)
    else {
        return p;
    };
    let x = p.clamp(0.005, 0.995);
    let z = a + b * (x / (1.0 - x)).ln();
    1.0 / (1.0 + (-z).exp())
}
