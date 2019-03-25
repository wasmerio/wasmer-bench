extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() {
    let path = if let Some(path) = env::var_os("WASM_C_API") {
        path.to_str().unwrap().to_owned()
    } else {
        panic!("environment var WASM_C_API not set")
    };
    println!("cargo:rustc-link-lib=static=v8_monolith");
    
    let v8_library_path = format!("{}/v8/v8/out.gn/x64.release/obj", path);
    let v8_link = format!("cargo:rustc-link-search=native={}", v8_library_path);
    println!("{}", v8_link);
    
    let wasm_c_api_library_path = format!("{}/out", path);
    let wasm_c_api_link = format!("cargo:rustc-link-search={}", wasm_c_api_library_path);
    println!("{}", wasm_c_api_link);

    let wasm_c_api_include_path = format!("{}/include", path);
    let wasm_c_api_include = format!("cargo:include={}", wasm_c_api_include_path);
    println!("{}", wasm_c_api_include);

    println!("cargo:rustc-link-lib=wasmc");
    println!("cargo:rustc-link-lib=dl");
    println!("cargo:rustc-link-lib=dylib=c++");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
