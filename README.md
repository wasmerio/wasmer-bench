
# Wasm Bench
The purpose of this project is to measure and compare performance of WebAssembly runtimes.

## Install dependencies

First, you will need to install Rust in your system.
You can follow the instructions here: https://rustup.rs/

Then, you will need to install Rust Nightly and the `wasm32-unknown-unknown` target.

```bash
# Install nightly
rustup install nightly

# Install wasm32-unknown-unknown target
rustup target add wasm32-unknown-unknown --toolchain nightly
```

## Build & Run

```bash
# Build the benchmarks in wasm
cd benchmarks && cargo +nightly build --release --target wasm32-unknown-unknown && cd ..

# Run the benchmarks
cargo +nightly bench
```
