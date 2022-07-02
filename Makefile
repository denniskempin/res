PROFILE=debug

web: target/web/$(PROFILE)

serve: target/web/$(PROFILE)
	basic-http-server -a 127.0.0.1:8080  $<

target/web/$(PROFILE): web/* target/web/$(PROFILE)/chip8emu.js
	mkdir -p $@
	cp -rf web/* $@

target/web/$(PROFILE)/chip8emu.js: target/wasm32-unknown-unknown/$(PROFILE)/chip8emu.wasm
	wasm-bindgen $< --out-dir target/web/$(PROFILE) --target web

target/wasm32-unknown-unknown/debug/chip8emu.wasm: src/*
	cargo build --lib --target wasm32-unknown-unknown

target/wasm32-unknown-unknown/release/chip8emu.wasm: src/*
	cargo build --release --lib --target wasm32-unknown-unknown
