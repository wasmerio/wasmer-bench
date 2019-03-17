#[macro_use]
extern crate criterion;

extern crate wasmer_runtime_core;

extern crate wasmer_clif_backend;
extern crate wasmer_llvm_backend;

use std::str;

use wasmer_runtime_core::{import::ImportObject, Func};

static WASM: &'static [u8] = include_bytes!(
    "../benchmarks/target/wasm32-unknown-unknown/release/wasm_bench_benchmarks.wasm"
);

static SMALL_WASM: &'static [u8] = include_bytes!("../benchmarks/src/printf.wasm");

static LARGE_WASM: &'static [u8] = include_bytes!("../benchmarks/src/lua.wasm");

use criterion::*;
use wasm_bench_benchmarks;
use wasmer_clif_backend::CraneliftCompiler;
use wasmer_llvm_backend::LLVMCompiler;

fn compile_benchmark(c: &mut Criterion) {
    c.bench(
        "small compile benchmark",
        Benchmark::new("clif", |b| {
            let compiler = &CraneliftCompiler::new();
            b.iter(|| {
                black_box(
                    wasmer_runtime_core::compile_with(SMALL_WASM, compiler)
                        .expect("should compile"),
                )
            })
        })
        .sample_size(10)
        .throughput(Throughput::Bytes(SMALL_WASM.len() as u32))
        .with_function("llvm", |b| {
            let compiler = &LLVMCompiler::new();
            b.iter(|| {
                black_box(
                    wasmer_runtime_core::compile_with(SMALL_WASM, compiler)
                        .expect("should compile"),
                )
            })
        }),
    );

    c.bench(
        "large compile benchmark",
        Benchmark::new("clif", |b| {
            let compiler = &CraneliftCompiler::new();
            b.iter(|| {
                black_box(
                    wasmer_runtime_core::compile_with(LARGE_WASM, compiler)
                        .expect("should compile"),
                )
            })
        })
        .sample_size(2)
        .throughput(Throughput::Bytes(LARGE_WASM.len() as u32))
        .with_function("llvm", |b| {
            let compiler = &LLVMCompiler::new();
            b.iter(|| {
                black_box(
                    wasmer_runtime_core::compile_with(LARGE_WASM, compiler)
                        .expect("should compile"),
                )
            })
        }),
    );
}

fn sum_benchmark(c: &mut Criterion) {
    c.bench(
        "sum 1, 2",
        Benchmark::new("native", |b| {
            b.iter(|| black_box(wasm_bench_benchmarks::sum(1, 2)))
        })
        .with_function("clif", |b| {
            let module = wasmer_runtime_core::compile_with(WASM, &CraneliftCompiler::new())
                .expect("should compile");
            let instance = module
                .instantiate(&ImportObject::new())
                .expect("should instantiate");
            let sum: Func<(i32, i32), i32> = instance.func("sum").unwrap();
            b.iter(|| black_box(sum.call(1, 2)))
        })
        .with_function("llvm", |b| {
            let module = wasmer_runtime_core::compile_with(WASM, &LLVMCompiler::new())
                .expect("should compile");
            let instance = module
                .instantiate(&ImportObject::new())
                .expect("should instantiate");
            let sum: Func<(i32, i32), i32> = instance.func("sum").unwrap();
            b.iter(|| black_box(sum.call(1, 2)))
        }),
    );
}

fn fib_benchmark(c: &mut Criterion) {
    c.bench(
        "fib 30",
        Benchmark::new("native", |b| {
            b.iter(|| black_box(wasm_bench_benchmarks::fib(30)))
        })
        .with_function("clif", |b| {
            let module = wasmer_runtime_core::compile_with(WASM, &CraneliftCompiler::new())
                .expect("should compile");
            let instance = module
                .instantiate(&ImportObject::new())
                .expect("should instantiate");
            let fib: Func<(i64), i64> = instance.func("fib").unwrap();
            b.iter(|| black_box(fib.call(30)))
        })
        .with_function("llvm", |b| {
            let module = wasmer_runtime_core::compile_with(WASM, &LLVMCompiler::new())
                .expect("should compile");
            let instance = module
                .instantiate(&ImportObject::new())
                .expect("should instantiate");
            let fib: Func<(i64), i64> = instance.func("fib").unwrap();
            b.iter(|| black_box(fib.call(30)))
        }),
    );
}

fn nbody_benchmark(c: &mut Criterion) {
    c.bench(
        "nbody",
        Benchmark::new("native", |b| {
            b.iter(|| black_box(unsafe { wasm_bench_benchmarks::nbody::bench(5000) }))
        })
        .with_function("clif", |b| {
            let module = wasmer_runtime_core::compile_with(WASM, &CraneliftCompiler::new())
                .expect("should compile");
            let instance = module
                .instantiate(&ImportObject::new())
                .expect("should instantiate");
            let func: Func<(i32)> = instance.func("bench").unwrap();
            b.iter(|| black_box(func.call(5000)))
        })
        .with_function("llvm", |b| {
            let module = wasmer_runtime_core::compile_with(WASM, &LLVMCompiler::new())
                .expect("should compile");
            let instance = module
                .instantiate(&ImportObject::new())
                .expect("should instantiate");
            let func: Func<(i32)> = instance.func("bench").unwrap();
            b.iter(|| black_box(func.call(5000)))
        }),
    );
}

fn sha1_benchmark(c: &mut Criterion) {
    c.bench(
        "sha1",
        Benchmark::new("native", |b| {
            b.iter(|| black_box(unsafe { wasm_bench_benchmarks::sha1(1000) }))
        })
        .with_function("clif", |b| {
            let module = wasmer_runtime_core::compile_with(WASM, &CraneliftCompiler::new())
                .expect("should compile");
            let instance = module
                .instantiate(&ImportObject::new())
                .expect("should instantiate");
            let func: Func<(i32)> = instance.func("sha1").unwrap();
            b.iter(|| black_box(func.call(1000)))
        })
        .with_function("llvm", |b| {
            let module = wasmer_runtime_core::compile_with(WASM, &LLVMCompiler::new())
                .expect("should compile");
            let instance = module
                .instantiate(&ImportObject::new())
                .expect("should instantiate");
            let func: Func<(i32)> = instance.func("sha1").unwrap();
            b.iter(|| black_box(func.call(1000)))
        }),
    );
}

// criterion_group!(benches, compile_benchmark);

criterion_group!(
    benches,
    fib_benchmark,
    sha1_benchmark,
    sum_benchmark,
    nbody_benchmark,
    compile_benchmark
);
criterion_main!(benches);

#[cfg(test)]
mod tests {

    #[test]
    fn test_sum() {
        assert_eq!(3, wasm_bench_benchmarks::sum(1, 2));
    }

}
