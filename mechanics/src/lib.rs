#![deny(warnings)]
#![allow(clippy::implicit_hasher)]
#![allow(clippy::should_implement_trait)]
#![allow(clippy::new_without_default)]
#![allow(clippy::upper_case_acronyms)]

pub mod bidding;
pub mod deck;
pub mod format_match;
pub mod hands;
pub mod multiset_iter;
pub mod ordered_card;
pub mod player;
pub mod scoring;
pub mod trick;
pub mod types;

#[macro_export]
macro_rules! impl_slog_value {
    ($x: ident) => {
        impl slog::Value for $x {
            fn serialize(
                &self,
                _: &slog::Record,
                key: slog::Key,
                serializer: &mut dyn slog::Serializer,
            ) -> slog::Result {
                serializer.emit_str(key, &format!("{:?}", self))
            }
        }
    };
}
