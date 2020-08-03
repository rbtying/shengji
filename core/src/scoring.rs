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

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameScoreResult {
    landlord_won: bool,
    landlord_delta: usize,
    non_landlord_delta: usize,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameScoringParameters {
    /// Number of points per "step" in the deck.
    step_size_per_deck: usize,
    /// Number of steps (as a fraction of the overall number in the deck)
    /// necessary to give the attacking team landlord.
    num_steps_to_non_landlord_turnover: usize,
    /// Number of steps (as a fraction of the overall number in the deck) that
    /// control is turned over, but neither side goes up a level.
    deadzone_size: usize,
    truncate_zero_crossing_window: bool,
    // TODO: move `BonusLevelPolicy` in here.
}

impl Default for GameScoringParameters {
    fn default() -> Self {
        Self {
            step_size_per_deck: 20,
            num_steps_to_non_landlord_turnover: 2,
            deadzone_size: 1,
            truncate_zero_crossing_window: true,
        }
    }
}

impl GameScoringParameters {
    pub fn materialize(
        &self,
        num_decks: usize,
        num_points_per_deck: usize,
    ) -> Result<MaterializedScoringParameters, Error> {
        if num_points_per_deck % self.step_size_per_deck != 0 {
            bail!(
                "Step size must evenly divide the number of points available ({} mod {} != 0)",
                num_points_per_deck,
                self.step_size_per_deck
            );
        }
        if self.step_size_per_deck <= 0 || self.step_size_per_deck >= num_points_per_deck {
            bail!("Step size must be between 5 and {}", num_points_per_deck);
        }
        if self.num_steps_to_non_landlord_turnover <= 1 {
            bail!("Landlord team must be able to win")
        }
        let s = num_decks as isize * self.step_size_per_deck as isize;
        let landlord_wins = if self.truncate_zero_crossing_window {
            let mut landlord_wins = vec![];

            let mut delta = 1;
            for offset in (2..=self.num_steps_to_non_landlord_turnover).rev() {
                landlord_wins.push(LandlordWinningScoreSegment {
                    start: (offset as isize - 1) * s,
                    end: offset as isize * s,
                    landlord_delta: delta,
                });
                delta += 1;
            }
            // Note: it's kind of strange here that the intervals are not
            // all exactly 40 points; in particular, the window including
            // zero is "special" and results in 3 levels.
            landlord_wins.push(LandlordWinningScoreSegment {
                start: 5,
                end: s,
                landlord_delta: delta,
            });
            landlord_wins.push(LandlordWinningScoreSegment {
                start: 5 - s,
                end: 5,
                landlord_delta: delta + 1,
            });
            landlord_wins
        } else {
            vec![LandlordWinningScoreSegment {
                start: (self.num_steps_to_non_landlord_turnover - 1) as isize * s,
                end: self.num_steps_to_non_landlord_turnover as isize * s,
                landlord_delta: 1,
            }]
        };

        let mut landlord_loses = if self.deadzone_size == 0 {
            vec![]
        } else {
            vec![LandlordLosingScoreSegment {
                start: self.num_steps_to_non_landlord_turnover as isize * s,
                end: (self.num_steps_to_non_landlord_turnover + self.deadzone_size) as isize * s,
                non_landlord_delta: 0,
            }]
        };
        landlord_loses.push(LandlordLosingScoreSegment {
            start: (self.num_steps_to_non_landlord_turnover + self.deadzone_size) as isize * s,
            end: (self.num_steps_to_non_landlord_turnover + self.deadzone_size + 1) as isize * s,
            non_landlord_delta: 1,
        });

        Ok(MaterializedScoringParameters::new(
            landlord_wins.into_iter().rev(),
            landlord_loses,
        )?)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct MaterializedScoringParameters {
    landlord_wins: SmallVec<[LandlordWinningScoreSegment; 3]>,
    landlord_loses: SmallVec<[LandlordLosingScoreSegment; 1]>,
}

impl MaterializedScoringParameters {
    pub fn new(
        landlord_wins: impl IntoIterator<Item = LandlordWinningScoreSegment>,
        landlord_loses: impl IntoIterator<Item = LandlordLosingScoreSegment>,
    ) -> Result<Self, Error> {
        let mut gsp = Self {
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
                    bail!(
                        "Cannot use scoring parameters with overlapping windows! {} < {}",
                        start,
                        e
                    );
                } else if start > e {
                    bail!("Missing a window between {} and {}", e, start);
                }
            }
            last_end = Some(end);
        }

        Ok(gsp)
    }

    pub fn score(&self, non_landlords_points: isize) -> Result<GameScoreResult, Error> {
        let landlord_won = non_landlords_points
            < self
                .landlord_wins
                .last()
                .ok_or_else(|| anyhow!("Landlord must be able to win"))?
                .end;

        if landlord_won {
            for s in PropagateMore::new(self.landlord_wins.iter().rev().copied()).take(10) {
                if s.start <= non_landlords_points && non_landlords_points < s.end {
                    return Ok(GameScoreResult {
                        non_landlord_delta: 0,
                        landlord_delta: s.landlord_delta,
                        landlord_won: true,
                    });
                }
            }
        } else {
            for s in PropagateMore::new(self.landlord_loses.iter().copied()).take(10) {
                if s.start <= non_landlords_points && non_landlords_points < s.end {
                    return Ok(GameScoreResult {
                        non_landlord_delta: s.non_landlord_delta,
                        landlord_delta: 0,
                        landlord_won: false,
                    });
                }
            }
        }
        bail!("Failed to score game!")
    }

    pub fn next_relevant_score(
        &self,
        current_score: isize,
    ) -> Result<(isize, GameScoreResult), Error> {
        let gsr = self.score(current_score)?;
        for offset in 1..1000 {
            let offset_gsr = self.score(current_score + offset * 5)?;
            if gsr != offset_gsr {
                return Ok((current_score + offset * 5, offset_gsr));
            }
        }
        bail!("Failed to find next relevant score")
    }

    pub fn explain(
        &self,
        num_decks: usize,
        num_points_per_deck: usize,
    ) -> Result<Vec<(isize, GameScoreResult)>, Error> {
        let mut current_score = 0;
        let total_points = (num_decks * num_points_per_deck) as isize;
        let mut explanatory = vec![(0, self.score(current_score)?)];
        loop {
            let (next, score) = self.next_relevant_score(current_score)?;
            explanatory.push((next, score));
            current_score = next;
            if current_score >= total_points {
                break Ok(explanatory);
            }
        }
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
    let GameScoreResult {
        non_landlord_delta,
        landlord_delta,
        landlord_won,
    } = GameScoringParameters::default()
        .materialize(num_decks, 100)
        .unwrap()
        .score(non_landlords_points)
        .expect("Should always resolve");

    eprintln!(
        "{:?}",
        GameScoringParameters::default()
            .materialize(num_decks, 100)
            .unwrap()
            .explain(num_decks, 100)
    );

    if landlord_won
        && bonus_level_policy == BonusLevelPolicy::BonusLevelForSmallerLandlordTeam
        && smaller_landlord_team_size
    {
        (non_landlord_delta, landlord_delta + 1, landlord_won, true)
    } else {
        (non_landlord_delta, landlord_delta, landlord_won, false)
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
