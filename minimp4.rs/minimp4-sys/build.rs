extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .define("NDEBUG", Some("1"))
        .file("minimp4.c")
        .compile("minimp4");
    let bindings = bindgen::Builder::default()
        .header("minimp4.h")
        .generate()
        .expect("Unable to generate bindings");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
    println!("cargo:rustc-link-lib=static=minimp4");
}