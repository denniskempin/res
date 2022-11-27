set -ex

cargo nextest run
cargo criterion
cargo clippy
cargo +nightly fmt --check
yarn build
