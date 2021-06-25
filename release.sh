#/bin/sh
set -eux
cargo-release release --tag-prefix="" "$@"
