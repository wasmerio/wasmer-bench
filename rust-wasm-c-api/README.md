# Rust Wasm C API
This library builds rust bindings to call the wasm-c-api (v8) using rust.

## Requirements
The [wasm-c-api](https://github.com/WebAssembly/wasm-c-api) project should be checked out and built at this locatioon relative to this project: `../../wasm-c-api`.

Set the environment variable `WASM_C_API` to the path to this project.

Modify the `Makefile` to remove the `-fsanitize=address` option and run `make c`.

After builing `wasm-c-api` create a library in the `wasm-c-api/out` directory:
`ar rcs libwasmc.a wasm-bin.o wasm-c.o`

## Building
`cargo +nightly build`

## Testing
`RAYON_NUM_THREADS=1 cargo +nightly test test_call_wasm`

## Bench with V8
To bench with V8 feature, build the dependencies described in `rust-wasm-c-api/README.md` and then run:
`RAYON_NUM_THREADS=1 cargo +nightly bench --features v8`