#/bin/sh
set -eux
cd backend
cargo-release release --tag-prefix="" "$@"
