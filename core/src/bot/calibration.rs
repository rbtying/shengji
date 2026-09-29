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
///
/// From selfplay/results/analysis.txt: Expert coach and bots both with basic rollouts, 80 hands
/// per opponent level and game (33-51 per side). Beginner is not calibrated.
const FITS: &[(Difficulty, bool, bool, f64, f64)] = &[
    (Difficulty::Easy, false, false, 0.0314, 0.8287),
    (Difficulty::Easy, false, true, 1.8670, 0.9451),
    (Difficulty::Easy, true, false, -0.8141, 1.0496),
    (Difficulty::Easy, true, true, 2.4637, 0.7743),
    (Difficulty::Expert, false, false, -2.0871, 1.7151),
    (Difficulty::Expert, false, true, 0.0337, 0.6369),
    (Difficulty::Expert, true, false, -0.7681, 0.6854),
    (Difficulty::Expert, true, true, -0.1496, 0.9651),
    (Difficulty::Hard, false, false, -1.9172, 0.7106),
    (Difficulty::Hard, false, true, 0.6631, 0.7284),
    (Difficulty::Hard, true, false, -1.4029, 0.8198),
    (Difficulty::Hard, true, true, 0.1323, 1.0153),
    (Difficulty::Intermediate, false, false, -0.8630, 0.8229),
    (Difficulty::Intermediate, false, true, 0.4035, 0.6330),
    (Difficulty::Intermediate, true, false, -1.8255, 1.1579),
    (Difficulty::Intermediate, true, true, 0.2880, 0.5669),
];

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fits_are_unique_and_increasing() {
        for (i, f) in FITS.iter().enumerate() {
            assert!(
                f.4 > 0.0,
                "{:?}: a corrected chance must rise with the raw one",
                f
            );
            assert!(
                !FITS[i + 1..]
                    .iter()
                    .any(|g| (g.0, g.1, g.2) == (f.0, f.1, f.2)),
                "{:?} appears twice",
                f
            );
        }
    }

    #[test]
    fn adjust_applies_the_fit() {
        // Hard, Tractor, attacking: a=-1.9172, b=0.7106; expected values computed separately.
        assert!((adjust(0.5, Difficulty::Hard, false, false) - 0.12817).abs() < 1e-4);
        assert!((adjust(0.93, Difficulty::Hard, false, false) - 0.48024).abs() < 1e-4);
        // No fit for Beginner: unchanged.
        assert_eq!(adjust(0.7, Difficulty::Beginner, false, false), 0.7);
    }
}
