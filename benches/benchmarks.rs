#[macro_use]
extern crate criterion;

#[cfg(feature = "v8")]
extern crate rust_wasm_c_api;
#[cfg(feature = "v8")]
use rust_wasm_c_api::*;

extern crate wasmer_clif_backend;
extern crate wasmer_dynasm_backend;
extern crate wasmer_llvm_backend;
extern crate wasmer_runtime_core;

#[cfg(feature = "bench-wasmi")]
extern crate wasmi;

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
use wasmer_dynasm_backend::SinglePassCompiler;
use wasmer_llvm_backend::LLVMCompiler;

#[cfg(feature = "bench-wasmi")]
use wasmi::{ImportsBuilder, ModuleInstance, NopExternals, RuntimeValue};

fn compile_benchmark(c: &mut Criterion) {
    let mut small_benchmark = Benchmark::new("wasmer-clif", |b| {
        let compiler = &CraneliftCompiler::new();
        b.iter(|| {
            black_box(
                wasmer_runtime_core::compile_with(SMALL_WASM, compiler).expect("should compile"),
            )
        })
    })
    .sample_size(10)
    .throughput(Throughput::Bytes(SMALL_WASM.len() as u32))
    .with_function("wasmer-llvm", |b| {
        let compiler = &LLVMCompiler::new();
        b.iter(|| {
            black_box(
                wasmer_runtime_core::compile_with(SMALL_WASM, compiler).expect("should compile"),
            )
        })
    })
    .with_function("wasmer-dynasm", |b| {
        let compiler = &SinglePassCompiler::new();
        b.iter(|| {
            black_box(
                wasmer_runtime_core::compile_with(SMALL_WASM, compiler).expect("should compile"),
            )
        })
    });

    c.bench("small_compile", small_benchmark);

    let mut large_benchmark = Benchmark::new("wasmer-clif", |b| {
        let compiler = &CraneliftCompiler::new();
        b.iter(|| {
            black_box(
                wasmer_runtime_core::compile_with(LARGE_WASM, compiler).expect("should compile"),
            )
        })
    })
    .sample_size(2)
    .throughput(Throughput::Bytes(LARGE_WASM.len() as u32))
    .with_function("wasmer-llvm", |b| {
        let compiler = &LLVMCompiler::new();
        b.iter(|| {
            black_box(
                wasmer_runtime_core::compile_with(LARGE_WASM, compiler).expect("should compile"),
            )
        })
    })
    .with_function("wasmer-dynasm", |b| {
        let compiler = &SinglePassCompiler::new();
        b.iter(|| {
            black_box(
                wasmer_runtime_core::compile_with(LARGE_WASM, compiler).expect("should compile"),
            )
        })
    });

    c.bench("large_compile", large_benchmark);
}

#[cfg(feature = "v8")]
mod wasm_c_api_support {
    use rust_wasm_c_api::*;
    pub struct WasmCApiEnv {
        pub engine: *mut wasm_engine_t,
        pub store: *mut wasm_store_t,
        pub module: *mut wasm_module_t,
        pub instance: *mut wasm_instance_t,
        pub exports: *mut wasm_extern_vec_t,
    }
    impl Drop for WasmCApiEnv {
        fn drop(&mut self) {
            unsafe {
                wasm_extern_vec_delete(self.exports);
                wasm_module_delete(self.module);
                wasm_instance_delete(self.instance);
                wasm_store_delete(self.store);
                wasm_engine_delete(self.engine);
            }
        }
    }
}

fn sum_benchmark(c: &mut Criterion) {
    let mut benchmark = Benchmark::new("rust-native", |b| {
        b.iter(|| black_box(wasm_bench_benchmarks::sum(1, 2)))
    })
    .with_function("wasmer-clif", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &CraneliftCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let sum: Func<(i32, i32), i32> = instance.func("sum").unwrap();
        b.iter(|| black_box(sum.call(1, 2)))
    })
    .with_function("wasmer-llvm", |b| {
        let module =
            wasmer_runtime_core::compile_with(WASM, &LLVMCompiler::new()).expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let sum: Func<(i32, i32), i32> = instance.func("sum").unwrap();
        b.iter(|| black_box(sum.call(1, 2)))
    })
    .with_function("wasmer-dynasm", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &SinglePassCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let sum: Func<(i32, i32), i32> = instance.func("sum").unwrap();
        b.iter(|| black_box(sum.call(1, 2)))
    });

    #[cfg(feature = "bench-wasmi")]
    {
        benchmark = benchmark.with_function("wasmi", |b| {
            let module = wasmi::Module::from_buffer(WASM).expect("error loading wasm");
            let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
                .expect("error instantiating module")
                .assert_no_start();
            b.iter(|| {
                black_box(instance.invoke_export(
                    "sum",
                    &[RuntimeValue::I32(1), RuntimeValue::I32(2)],
                    &mut NopExternals,
                ))
            })
        });
    }

    #[cfg(feature = "v8")]
    {
        use wasm_c_api_support::WasmCApiEnv;
        unsafe {
            use std::mem;
            use std::slice;
            // Instantiate wasm file
            let engine = wasm_engine_new();
            let store = wasm_store_new(engine);
            let mut byte_vec = WASM.to_vec();
            let bytes_slice: &mut [u8] = byte_vec.as_mut_slice();
            let len = bytes_slice.len();
            let bytes = wasm_byte_vec_t {
                size: len,
                data: bytes_slice.as_mut_ptr() as _,
            };
            let module = wasm_module_new(store, &bytes as *const wasm_byte_vec_t);
            let imports = &[];
            let instance = wasm_instance_new(store, module, imports.as_ptr());
            let mut exports: wasm_extern_vec_t = mem::uninitialized();
            wasm_instance_exports(instance, &mut exports as *mut wasm_extern_vec_t);
            let data = exports.data;
            let size = exports.size;
            let exports_slice = slice::from_raw_parts(data, size);
            let v8_func = wasm_extern_as_func(exports_slice[4]);
            let env = WasmCApiEnv {
                engine,
                store,
                module,
                instance,
                exports: &mut exports as *mut wasm_extern_vec_t,
            };

            benchmark = benchmark.with_function("wasm-c-api-v8", move |b| {
                let _env = &env;
                let val1 = wasm_val_t__bindgen_ty_1 { i32: 3 };
                let arg1 = wasm_val_t {
                    kind: wasm_valkind_t_WASM_I32,
                    of: val1,
                };
                let val2 = wasm_val_t__bindgen_ty_1 { i32: 4 };
                let arg2 = wasm_val_t {
                    kind: wasm_valkind_t_WASM_I32,
                    of: val2,
                };
                let args = [arg1, arg2];
                let resval1 = wasm_val_t__bindgen_ty_1 { i32: 0 };
                let res1 = wasm_val_t {
                    kind: wasm_valkind_t_WASM_I32,
                    of: resval1,
                };
                let results: &mut [wasm_val_t] = &mut [res1];
                b.iter(|| black_box(wasm_func_call(v8_func, args.as_ptr(), results.as_mut_ptr())))
            });
        }
    }

    c.bench("sum", benchmark);
}

fn fib_benchmark(c: &mut Criterion) {
    let mut benchmark = Benchmark::new("rust-native", |b| {
        b.iter(|| black_box(wasm_bench_benchmarks::fib(30)))
    })
    .with_function("wasmer-clif", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &CraneliftCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let fib: Func<(i64), i64> = instance.func("fib").unwrap();
        b.iter(|| black_box(fib.call(30)))
    })
    .with_function("wasmer-llvm", |b| {
        let module =
            wasmer_runtime_core::compile_with(WASM, &LLVMCompiler::new()).expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let fib: Func<(i64), i64> = instance.func("fib").unwrap();
        b.iter(|| black_box(fib.call(30)))
    })
    .with_function("wasmer-dynasm", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &SinglePassCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let fib: Func<(i64), i64> = instance.func("fib").unwrap();
        b.iter(|| black_box(fib.call(30)))
    });

    #[cfg(feature = "bench-wasmi")]
    {
        benchmark = benchmark.sample_size(25).with_function("wasmi", |b| {
            let module = wasmi::Module::from_buffer(WASM).expect("error loading wasm");
            let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
                .expect("error instantiating module")
                .assert_no_start();
            b.iter(|| {
                black_box(instance.invoke_export(
                    "fib",
                    &[RuntimeValue::I64(30)],
                    &mut NopExternals,
                ))
            })
        });
    }

    #[cfg(feature = "v8")]
    {
        use wasm_c_api_support::WasmCApiEnv;
        unsafe {
            use std::mem;
            use std::slice;
            // Instantiate wasm file
            let engine = wasm_engine_new();
            let store = wasm_store_new(engine);
            let mut byte_vec = WASM.to_vec();
            let bytes_slice: &mut [u8] = byte_vec.as_mut_slice();
            let len = bytes_slice.len();
            let bytes = wasm_byte_vec_t {
                size: len,
                data: bytes_slice.as_mut_ptr() as _,
            };
            let module = wasm_module_new(store, &bytes as *const wasm_byte_vec_t);
            let imports = &[];
            let instance = wasm_instance_new(store, module, imports.as_ptr());
            let mut exports: wasm_extern_vec_t = mem::uninitialized();
            wasm_instance_exports(instance, &mut exports as *mut wasm_extern_vec_t);
            let data = exports.data;
            let size = exports.size;
            let exports_slice = slice::from_raw_parts(data, size);
            let v8_func = wasm_extern_as_func(exports_slice[6]);
            let env = WasmCApiEnv {
                engine,
                store,
                module,
                instance,
                exports: &mut exports as *mut wasm_extern_vec_t,
            };

            benchmark = benchmark.with_function("wasm-c-api-v8", move |b| {
                let _env = &env;
                let val1 = wasm_val_t__bindgen_ty_1 { i64: 30 };
                let arg1 = wasm_val_t {
                    kind: wasm_valkind_t_WASM_I64,
                    of: val1,
                };
                let args = [arg1];
                let resval1 = wasm_val_t__bindgen_ty_1 { i64: 0 };
                let res1 = wasm_val_t {
                    kind: wasm_valkind_t_WASM_I64,
                    of: resval1,
                };
                let results: &mut [wasm_val_t] = &mut [res1];
                b.iter(|| black_box(wasm_func_call(v8_func, args.as_ptr(), results.as_mut_ptr())))
            });
        }
    }

    c.bench("fibonacci", benchmark);
}

fn nbody_benchmark(c: &mut Criterion) {
    let mut benchmark = Benchmark::new("rust-native", |b| {
        b.iter(|| black_box(unsafe { wasm_bench_benchmarks::nbody::nbody_bench(5000) }))
    })
    .with_function("wasmer-clif", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &CraneliftCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let func: Func<(i32)> = instance.func("nbody_bench").unwrap();
        b.iter(|| black_box(func.call(5000)))
    })
    .with_function("wasmer-llvm", |b| {
        let module =
            wasmer_runtime_core::compile_with(WASM, &LLVMCompiler::new()).expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let func: Func<(i32)> = instance.func("nbody_bench").unwrap();
        b.iter(|| black_box(func.call(5000)))
    })
    .with_function("wasmer-dynasm", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &SinglePassCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let func: Func<(i32)> = instance.func("nbody_bench").unwrap();
        b.iter(|| black_box(func.call(5000)))
    });

    #[cfg(feature = "bench-wasmi")]
    {
        benchmark = benchmark.sample_size(25).with_function("wasmi", |b| {
            let module = wasmi::Module::from_buffer(WASM).expect("error loading wasm");
            let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
                .expect("error instantiating module")
                .assert_no_start();
            b.iter(|| {
                black_box(instance.invoke_export(
                    "nbody_bench",
                    &[RuntimeValue::I32(5000)],
                    &mut NopExternals,
                ))
            })
        });
    }

    #[cfg(feature = "v8")]
    {
        use wasm_c_api_support::WasmCApiEnv;
        unsafe {
            use std::mem;
            use std::slice;
            // Instantiate wasm file
            let engine = wasm_engine_new();
            let store = wasm_store_new(engine);
            let mut byte_vec = WASM.to_vec();
            let bytes_slice: &mut [u8] = byte_vec.as_mut_slice();
            let len = bytes_slice.len();
            let bytes = wasm_byte_vec_t {
                size: len,
                data: bytes_slice.as_mut_ptr() as _,
            };
            let module = wasm_module_new(store, &bytes as *const wasm_byte_vec_t);
            let imports = &[];
            let instance = wasm_instance_new(store, module, imports.as_ptr());
            let mut exports: wasm_extern_vec_t = mem::uninitialized();
            wasm_instance_exports(instance, &mut exports as *mut wasm_extern_vec_t);
            let data = exports.data;
            let size = exports.size;
            let exports_slice = slice::from_raw_parts(data, size);
            let v8_func = wasm_extern_as_func(exports_slice[5]);
            let env = WasmCApiEnv {
                engine,
                store,
                module,
                instance,
                exports: &mut exports as *mut wasm_extern_vec_t,
            };

            benchmark = benchmark.with_function("wasm-c-api-v8", move |b| {
                let _env = &env;
                let val1 = wasm_val_t__bindgen_ty_1 { i32: 5000 };
                let arg1 = wasm_val_t {
                    kind: wasm_valkind_t_WASM_I32,
                    of: val1,
                };
                let args = [arg1];
                let results: &mut [wasm_val_t] = &mut [];
                b.iter(|| black_box(wasm_func_call(v8_func, args.as_ptr(), results.as_mut_ptr())))
            });
        }
    }

    c.bench("nbody", benchmark);
}

fn fannkuck_benchmark(c: &mut Criterion) {
    let mut benchmark = Benchmark::new("rust-native", |b| {
        b.iter(|| black_box(unsafe { wasm_bench_benchmarks::fannkuch_steps(5) }))
    })
    .with_function("wasmer-clif", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &CraneliftCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let func: Func<(i32)> = instance.func("fannkuch_steps").unwrap();
        b.iter(|| black_box(func.call(5)))
    })
    .with_function("wasmer-llvm", |b| {
        let module =
            wasmer_runtime_core::compile_with(WASM, &LLVMCompiler::new()).expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let func: Func<(i32)> = instance.func("fannkuch_steps").unwrap();
        b.iter(|| black_box(func.call(5)))
    })
    .with_function("wasmer-dynasm", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &SinglePassCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let func: Func<(i32)> = instance.func("fannkuch_steps").unwrap();
        b.iter(|| black_box(func.call(5)))
    });

    #[cfg(feature = "bench-wasmi")]
    {
        benchmark = benchmark.sample_size(25).with_function("wasmi", |b| {
            let module = wasmi::Module::from_buffer(WASM).expect("error loading wasm");
            let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
                .expect("error instantiating module")
                .assert_no_start();
            b.iter(|| {
                black_box(instance.invoke_export(
                    "fannkuch_steps",
                    &[RuntimeValue::I32(5)],
                    &mut NopExternals,
                ))
            })
        });
    }

    #[cfg(feature = "v8")]
    {
        use wasm_c_api_support::WasmCApiEnv;
        unsafe {
            use std::mem;
            use std::slice;
            // Instantiate wasm file
            let engine = wasm_engine_new();
            let store = wasm_store_new(engine);
            let mut byte_vec = WASM.to_vec();
            let bytes_slice: &mut [u8] = byte_vec.as_mut_slice();
            let len = bytes_slice.len();
            let bytes = wasm_byte_vec_t {
                size: len,
                data: bytes_slice.as_mut_ptr() as _,
            };
            let module = wasm_module_new(store, &bytes as *const wasm_byte_vec_t);
            let imports = &[];
            let instance = wasm_instance_new(store, module, imports.as_ptr());
            let mut exports: wasm_extern_vec_t = mem::uninitialized();
            wasm_instance_exports(instance, &mut exports as *mut wasm_extern_vec_t);
            let data = exports.data;
            let size = exports.size;
            let exports_slice = slice::from_raw_parts(data, size);
            let v8_func = wasm_extern_as_func(exports_slice[8]);
            let env = WasmCApiEnv {
                engine,
                store,
                module,
                instance,
                exports: &mut exports as *mut wasm_extern_vec_t,
            };

            benchmark = benchmark.with_function("wasm-c-api-v8", move |b| {
                let _env = &env;
                let val1 = wasm_val_t__bindgen_ty_1 { i32: 5 };
                let arg1 = wasm_val_t {
                    kind: wasm_valkind_t_WASM_I32,
                    of: val1,
                };
                let args = [arg1];
                let results: &mut [wasm_val_t] = &mut [];
                b.iter(|| black_box(wasm_func_call(v8_func, args.as_ptr(), results.as_mut_ptr())))
            });
        }
    }

    c.bench("fannkuch", benchmark);
}

fn sha1_benchmark(c: &mut Criterion) {
    let mut benchmark = Benchmark::new("rust-native", |b| {
        b.iter(|| black_box(unsafe { wasm_bench_benchmarks::sha1(1000) }))
    })
    .with_function("wasmer-clif", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &CraneliftCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let func: Func<(i32)> = instance.func("sha1").unwrap();
        b.iter(|| black_box(func.call(1000)))
    })
    .with_function("wasmer-llvm", |b| {
        let module =
            wasmer_runtime_core::compile_with(WASM, &LLVMCompiler::new()).expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let func: Func<(i32)> = instance.func("sha1").unwrap();
        b.iter(|| black_box(func.call(1000)))
    })
    .with_function("wasmer-dynasm", |b| {
        let module = wasmer_runtime_core::compile_with(WASM, &SinglePassCompiler::new())
            .expect("should compile");
        let instance = module
            .instantiate(&ImportObject::new())
            .expect("should instantiate");
        let func: Func<(i32)> = instance.func("sha1").unwrap();
        b.iter(|| black_box(func.call(1000)))
    });

    #[cfg(feature = "bench-wasmi")]
    {
        benchmark = benchmark.sample_size(20).with_function("wasmi", |b| {
            let module = wasmi::Module::from_buffer(WASM).expect("error loading wasm");
            let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
                .expect("error instantiating module")
                .assert_no_start();
            b.iter(|| {
                black_box(instance.invoke_export(
                    "sha1",
                    &[RuntimeValue::I32(1000)],
                    &mut NopExternals,
                ))
            })
        });
    }

    #[cfg(feature = "v8")]
    {
        use wasm_c_api_support::WasmCApiEnv;
        unsafe {
            use std::mem;
            use std::slice;
            // Instantiate wasm file
            let engine = wasm_engine_new();
            let store = wasm_store_new(engine);
            let mut byte_vec = WASM.to_vec();
            let bytes_slice: &mut [u8] = byte_vec.as_mut_slice();
            let len = bytes_slice.len();
            let bytes = wasm_byte_vec_t {
                size: len,
                data: bytes_slice.as_mut_ptr() as _,
            };
            let module = wasm_module_new(store, &bytes as *const wasm_byte_vec_t);
            let imports = &[];
            let instance = wasm_instance_new(store, module, imports.as_ptr());
            let mut exports: wasm_extern_vec_t = mem::uninitialized();
            wasm_instance_exports(instance, &mut exports as *mut wasm_extern_vec_t);
            let data = exports.data;
            let size = exports.size;
            let exports_slice = slice::from_raw_parts(data, size);
            let v8_func = wasm_extern_as_func(exports_slice[9]);
            let env = WasmCApiEnv {
                engine,
                store,
                module,
                instance,
                exports: &mut exports as *mut wasm_extern_vec_t,
            };

            benchmark = benchmark.with_function("wasm-c-api-v8", move |b| {
                let _env = &env;
                let val1 = wasm_val_t__bindgen_ty_1 { i32: 1000 };
                let arg1 = wasm_val_t {
                    kind: wasm_valkind_t_WASM_I32,
                    of: val1,
                };
                let args = [arg1];
                let results: &mut [wasm_val_t] = &mut [];
                b.iter(|| black_box(wasm_func_call(v8_func, args.as_ptr(), results.as_mut_ptr())))
            });
        }
    }

    c.bench("sha1", benchmark);
}

//criterion_group!(benches, compile_benchmark);

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
