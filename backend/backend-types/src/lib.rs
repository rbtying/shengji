use serde::{Deserialize, Serialize};
use shengji_core::{game_state, interactive};

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug, Serialize, Deserialize)]
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
    Beep,
    ReadyCheck,
    Error(String),
    Header {
        messages: Vec<String>,
    },
    Kicked,
}

/// zstd dictionary, compressed with zstd.
pub const ZSTD_ZSTD_DICT: &[u8] = include_bytes!("../dict.zstd");
