set -e

printf '\e[1m\e[32m---------------------------- Tests -------------------------------------\e[0m\n'
cargo nextest run --workspace
echo
printf '\e[1m\e[32m-------------------------- Benchmarks ----------------------------------\e[0m\n'
cargo bench --bench nestest --bench alter_ego -- --sample-size 10 --measurement-time 1 --warm-up-time 1
echo
printf '\e[1m\e[32m---------------------------- Clippy ------------------------------------\e[0m\n'
cargo clippy --workspace
echo
printf '\e[1m\e[32m---------------------------- Format ------------------------------------\e[0m\n'
cargo +nightly fmt --check && echo "ok"
echo
printf '\e[1m\e[32m-------------------------- Wasm Build ----------------------------------\e[0m\n'
cd res_egui && yarn build
