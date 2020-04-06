#![deny(warnings)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::new_without_default)]

#![feature(async_closure)]
#![feature(const_fn)]
#![feature(const_if_match)]

pub mod game_state;
pub mod hands;
pub mod interactive;
pub mod trick;
pub mod types;
