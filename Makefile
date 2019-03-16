.PHONY: build bench

bench:
	cargo +nightly bench

build:
	cd benchmarks && cargo +nightly build --release --target wasm32-unknown-unknown && cd ..