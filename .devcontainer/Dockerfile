FROM menci/archlinuxarm:base-devel

RUN pacman -Sy --noconfirm \
    curl \
    wget \
    git \
    inetutils \
    npm \
    python-pip \
    openssh \
    fish \
    rustup

ENV RUSTUP_HOME=/usr/local/rustup \
    CARGO_HOME=/usr/local/cargo \
    CARGO_TARGET_DIR=/cargo_target \
    PATH=/usr/local/cargo/bin:$PATH

RUN rustup toolchain install stable --profile minimal
RUN rustup target add wasm32-unknown-unknown
RUN rustup component add clippy rustfmt
#RUN --mount=type=cache,target=/cargo_target,sharing=private \
#    cargo install -f wasm-bindgen-cli basic-http-server
