#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem;
    use std::slice;

    static WASM: &'static [u8] = include_bytes!(
        "../../benchmarks/target/wasm32-unknown-unknown/release/wasm_bench_benchmarks.wasm"
    );

    #[test]
    fn test_call_wasm() {
        unsafe {
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
            // call sum and assert result.
            let imports = &[];
            let instance = wasm_instance_new(store, module, imports.as_ptr());

            let mut exports: wasm_extern_vec_t = mem::uninitialized();
            wasm_instance_exports(instance, &mut exports as *mut wasm_extern_vec_t);
            let data = exports.data;
            let size = exports.size;
            let exports_slice = slice::from_raw_parts(data, size);

            let v8_func = wasm_extern_as_func(exports_slice[3]);

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
            wasm_func_call(v8_func, args.as_ptr(), results.as_mut_ptr());
            println!("result: {}", results[0].of.i32);
            assert_eq!(results[0].of.i32, 7);

            // Clean up
            wasm_extern_vec_delete(&mut exports as *mut wasm_extern_vec_t);
            wasm_module_delete(module);
            wasm_instance_delete(instance);
            wasm_store_delete(store);
            wasm_engine_delete(engine);
        }
    }

}
