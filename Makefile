PROFILE=debug

web: target/web/$(PROFILE)

serve: target/web/$(PROFILE)
	basic-http-server -a 127.0.0.1:8080  $<

target/web/$(PROFILE): web/* target/web/$(PROFILE)/res.js
	mkdir -p $@
	cp -rf web/* $@

target/web/$(PROFILE)/res.js: target/wasm32-unknown-unknown/$(PROFILE)/res.wasm
	wasm-bindgen $< --out-dir target/web/$(PROFILE) --target web

target/wasm32-unknown-unknown/debug/res.wasm: src/*
	cargo build --lib --target wasm32-unknown-unknown

target/wasm32-unknown-unknown/release/res.wasm: src/*
	cargo build --release --lib --target wasm32-unknown-unknown
