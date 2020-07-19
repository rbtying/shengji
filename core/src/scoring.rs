use anyhow::{anyhow, bail, Error};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum BonusLevelPolicy {
    NoBonusLevel,
    BonusLevelForSmallerLandlordTeam,
}

impl Default for BonusLevelPolicy {
    fn default() -> Self {
        BonusLevelPolicy::BonusLevelForSmallerLandlordTeam
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameScoringParameters {
    landlord_wins: SmallVec<[LandlordWinningScoreSegment; 3]>,
    landlord_loses: SmallVec<[LandlordLosingScoreSegment; 1]>,
}

impl GameScoringParameters {
    pub fn new(
        landlord_wins: impl IntoIterator<Item = LandlordWinningScoreSegment>,
        landlord_loses: impl IntoIterator<Item = LandlordLosingScoreSegment>,
    ) -> Result<Self, Error> {
        let mut gsp = GameScoringParameters {
            landlord_wins: landlord_wins.into_iter().collect(),
            landlord_loses: landlord_loses.into_iter().collect(),
        };
        gsp.landlord_wins.sort_by_key(|s| s.start);
        gsp.landlord_loses.sort_by_key(|s| s.start);

        // Check that landlord wins and loses share a value
        match (gsp.landlord_wins.last(), gsp.landlord_loses.first()) {
            (None, _) | (_, None) => {
                bail!("Both the landlord and the non-landlord have to be able to win!")
            }
            (Some(w), Some(l)) if w.end != l.start => {
                bail!("The landlord winning and not-winning must share an endpoint")
            }
            (Some(_), Some(_)) => (),
        }

        let windows = gsp
            .landlord_wins
            .iter()
            .map(|s| (s.start, s.end))
            .chain(gsp.landlord_loses.iter().map(|s| (s.start, s.end)));
        let mut last_end = None;
        for (start, end) in windows {
            if start >= end {
                bail!("Start must be strictly less than end")
            }
            if start % 5 != 0 || end % 5 != 0 {
                bail!("Start and end must be multiples of 5")
            }
            if let Some(e) = last_end {
                if start < e {
                    bail!("Cannot use scoring parameters with overlapping windows!");
                } else if start > e {
                    bail!("Missing a window between {} and {}", e, start);
                }
            }
            last_end = Some(end);
        }

        Ok(gsp)
    }

    /// Generates the default scoring parameters for the provided number of
    /// decks.
    pub fn default_for_num_decks(num_decks: usize) -> Self {
        let s = num_decks as isize * 20;
        Self::new(
            vec![
                LandlordWinningScoreSegment {
                    start: 5 - s,
                    end: 5,
                    landlord_delta: 3,
                },
                // Note: it's kind of strange here that the intervals are not
                // all exactly 40 points; in particular, the window including
                // zero is "special" and results in 3 levels.
                LandlordWinningScoreSegment {
                    start: 5,
                    end: s,
                    landlord_delta: 2,
                },
                LandlordWinningScoreSegment {
                    start: s,
                    end: 2 * s,
                    landlord_delta: 1,
                },
            ],
            vec![LandlordLosingScoreSegment {
                start: 2 * s,
                end: 3 * s,
                non_landlord_delta: 0,
            }],
        )
        .unwrap()
    }

    pub fn score(&self, non_landlords_points: isize) -> Result<(usize, usize, bool), Error> {
        let landlord_won = non_landlords_points
            < self
                .landlord_wins
                .last()
                .ok_or_else(|| anyhow!("Landlord must be able to win"))?
                .end;

        if landlord_won {
            for s in PropagateMore::new(self.landlord_wins.iter().rev().copied()).take(10) {
                if s.start <= non_landlords_points && non_landlords_points < s.end {
                    return Ok((0, s.landlord_delta, true));
                }
            }
        } else {
            for s in PropagateMore::new(self.landlord_loses.iter().copied()).take(10) {
                if s.start <= non_landlords_points && non_landlords_points < s.end {
                    return Ok((s.non_landlord_delta, 0, false));
                }
            }
        }
        bail!("Failed to score game!")
    }
}

struct PropagateMore<I: Iterator<Item = P>, P: Propagatable> {
    initial: Option<I>,
    propagatable: Option<P>,
}

impl<I: Iterator<Item = P>, P: Propagatable + Clone> PropagateMore<I, P> {
    pub fn new(iter: I) -> Self {
        Self {
            initial: Some(iter),
            propagatable: None,
        }
    }
}

impl<I: Iterator<Item = P>, P: Propagatable + Clone> Iterator for PropagateMore<I, P> {
    type Item = P;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(n) = self.initial.as_mut().and_then(|i| i.next()) {
                self.propagatable = Some(n.clone());
                break Some(n);
            }
            match self.propagatable.take() {
                Some(p) => {
                    self.propagatable = Some(p.propagate());
                    break self.propagatable.clone();
                }
                None => break None,
            }
        }
    }
}

trait Propagatable {
    fn propagate(self) -> Self;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LandlordWinningScoreSegment {
    /// The beginning of the scoring segment, inclusive
    start: isize,
    /// The end of the scoring segment, exclusive.
    end: isize,
    landlord_delta: usize,
}

impl Propagatable for LandlordWinningScoreSegment {
    /// Compute the next scoring window, downwards from the present one
    fn propagate(self) -> Self {
        Self {
            start: self.start - (self.end - self.start),
            end: self.start,
            landlord_delta: self.landlord_delta + 1,
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct LandlordLosingScoreSegment {
    start: isize,
    end: isize,
    non_landlord_delta: usize,
}

impl Propagatable for LandlordLosingScoreSegment {
    /// Compute the next scoring window, upwards from the present one
    fn propagate(self) -> Self {
        Self {
            start: self.end,
            end: self.end + (self.end - self.start),
            non_landlord_delta: self.non_landlord_delta + 1,
        }
    }
}

pub fn compute_level_deltas(
    num_decks: usize,
    non_landlords_points: isize,
    bonus_level_policy: BonusLevelPolicy,
    smaller_landlord_team_size: bool,
) -> (usize, usize, bool, bool) {
    let (non_landlord, landlord, landlord_won) =
        GameScoringParameters::default_for_num_decks(num_decks)
            .score(non_landlords_points)
            .expect("Should always resolve");

    if landlord_won
        && bonus_level_policy == BonusLevelPolicy::BonusLevelForSmallerLandlordTeam
        && smaller_landlord_team_size
    {
        (non_landlord, landlord + 1, landlord_won, true)
    } else {
        (non_landlord, landlord, landlord_won, false)
    }
}

#[cfg(test)]
mod tests {
    use super::{compute_level_deltas, BonusLevelPolicy};

    #[test]
    fn test_level_deltas() {
        assert_eq!(
            compute_level_deltas(2, -80, BonusLevelPolicy::NoBonusLevel, false),
            (0, 5, true, false)
        );
        assert_eq!(
            compute_level_deltas(2, -40, BonusLevelPolicy::NoBonusLevel, false),
            (0, 4, true, false)
        );
        assert_eq!(
            compute_level_deltas(2, -35, BonusLevelPolicy::NoBonusLevel, false),
            (0, 3, true, false)
        );
        assert_eq!(
            compute_level_deltas(2, 0, BonusLevelPolicy::NoBonusLevel, false),
            (0, 3, true, false)
        );
        assert_eq!(
            compute_level_deltas(2, 5, BonusLevelPolicy::NoBonusLevel, false),
            (0, 2, true, false)
        );
        assert_eq!(
            compute_level_deltas(2, 35, BonusLevelPolicy::NoBonusLevel, false),
            (0, 2, true, false)
        );
        assert_eq!(
            compute_level_deltas(2, 40, BonusLevelPolicy::NoBonusLevel, false),
            (0, 1, true, false)
        );
        assert_eq!(
            compute_level_deltas(2, 75, BonusLevelPolicy::NoBonusLevel, false),
            (0, 1, true, false)
        );
        assert_eq!(
            compute_level_deltas(2, 80, BonusLevelPolicy::NoBonusLevel, false),
            (0, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(2, 115, BonusLevelPolicy::NoBonusLevel, false),
            (0, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(2, 120, BonusLevelPolicy::NoBonusLevel, false),
            (1, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(2, 155, BonusLevelPolicy::NoBonusLevel, false),
            (1, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(2, 160, BonusLevelPolicy::NoBonusLevel, false),
            (2, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(2, 195, BonusLevelPolicy::NoBonusLevel, false),
            (2, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(2, 200, BonusLevelPolicy::NoBonusLevel, false),
            (3, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(2, 235, BonusLevelPolicy::NoBonusLevel, false),
            (3, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(2, 240, BonusLevelPolicy::NoBonusLevel, false),
            (4, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(2, 280, BonusLevelPolicy::NoBonusLevel, false),
            (5, 0, false, false)
        );
        assert_eq!(
            compute_level_deltas(
                2,
                0,
                BonusLevelPolicy::BonusLevelForSmallerLandlordTeam,
                true
            ),
            (0, 4, true, true)
        );
        assert_eq!(
            compute_level_deltas(
                3,
                0,
                BonusLevelPolicy::BonusLevelForSmallerLandlordTeam,
                true
            ),
            (0, 4, true, true)
        );
        assert_eq!(
            compute_level_deltas(
                3,
                50,
                BonusLevelPolicy::BonusLevelForSmallerLandlordTeam,
                true
            ),
            (0, 3, true, true)
        );
    }
}
