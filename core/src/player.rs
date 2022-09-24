use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::{Number, PlayerID, Rank};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Player {
    pub(crate) id: PlayerID,
    pub(crate) name: String,
    pub(crate) level: Rank,
    pub(crate) metalevel: usize,
}

impl Player {
    pub fn new(id: PlayerID, name: String) -> Player {
        Player {
            id,
            name,
            level: Rank::Number(Number::Two),
            metalevel: 1,
        }
    }

    pub fn rank(&self) -> Rank {
        self.level
    }

    pub fn set_rank(&mut self, level: Rank) {
        self.level = level;
    }

    pub fn set_meta_rank(&mut self, metalevel: usize) {
        self.metalevel = metalevel;
    }

    pub fn advance(&mut self, max_rank: Rank) {
        match self.level.successor() {
            Some(next_level) if self.level != max_rank => {
                self.level = next_level;
            }
            None | Some(_) => {
                self.metalevel += 1;
                self.level = Rank::Number(Number::Two);
            }
        }
    }
}
