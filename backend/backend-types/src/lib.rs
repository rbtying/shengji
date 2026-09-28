use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shengji_core::{game_state, interactive};

pub mod wasm_rpc;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum GameMessage {
    State {
        state: game_state::GameState,
    },
    Message {
        from: String,
        message: String,
    },
    Broadcast {
        data: interactive::BroadcastMessage,
        message: String,
    },
    Beep {
        target: String,
    },
    ReadyCheck {
        from: String,
    },
    Error(String),
    Header {
        messages: Vec<String>,
    },
    Kicked {
        target: String,
    },
    /// Advice for the player who asked; `None` when there's nothing to suggest right now.
    Hint {
        hint: Option<shengji_core::bot::Hint>,
    },
    /// The coach's review of the hand just played (offline play).
    Review {
        review: Option<shengji_core::bot::HandReview>,
    },
}

/// zstd dictionary, compressed with zstd.
pub const ZSTD_ZSTD_DICT: &[u8] = include_bytes!("../dict.zstd");
