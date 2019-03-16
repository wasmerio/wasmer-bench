
# Wasm Bench
The purpose of this project is to measure and compare performance of WebAssembly runtimes.

## Build & Run
```
# Build the benchmarks in wasm
cd benchmarks && cargo +nightly build --release --target wasm32-unknown-unknown && cd ..

# Run the benchmarks
cargo +nightly bench

```