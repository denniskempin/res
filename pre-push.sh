set -ex

cargo nextest run
cargo clippy
cargo +nightly fmt --check
yarn build 