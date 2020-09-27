use serde::{Deserialize, Serialize};

use crate::types::{Number, PlayerID};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Player {
    pub(crate) id: PlayerID,
    pub(crate) name: String,
    pub(crate) level: Number,
    pub(crate) metalevel: usize,
}

impl Player {
    pub fn new(id: PlayerID, name: String) -> Player {
        Player {
            id,
            name,
            level: Number::Two,
            metalevel: 1,
        }
    }

    pub fn rank(&self) -> Number {
        self.level
    }

    pub fn set_rank(&mut self, level: Number) {
        self.level = level;
    }

    pub fn advance(&mut self) {
        if let Some(next_level) = self.level.successor() {
            self.level = next_level;
        } else {
            self.metalevel += 1;
            self.level = Number::Two;
        }
    }
}
