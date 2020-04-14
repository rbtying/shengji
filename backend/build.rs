use vergen::{generate_cargo_keys, ConstantsFlags};
fn main() {
    generate_cargo_keys(
        ConstantsFlags::SHA | ConstantsFlags::SHA_SHORT | ConstantsFlags::REBUILD_ON_HEAD_CHANGE,
    )
    .expect("Unable to generate the cargo keys!")
}
