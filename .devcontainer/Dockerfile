FROM mcr.microsoft.com/devcontainers/rust:latest

RUN rustup target add wasm32-unknown-unknown
RUN cargo install -f wasm-bindgen-cli basic-http-server
