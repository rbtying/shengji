#![deny(warnings)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]

#[macro_use]
pub mod settings;

pub mod bidding;
pub mod deck;
pub mod game_state;
pub mod hands;
pub mod interactive;
pub mod message;
pub mod ordered_card;
pub mod player;
pub mod scoring;
pub mod trick;
pub mod types;
