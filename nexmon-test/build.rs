use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=wrapper.h");
    println!("cargo:rerun-if-changed=unpack.c");

    println!("cargo:rustc-link-lib=c++");
    // println!("cargo:rustc-link-lib=cstdio");

    // Configure and generate bindings.
    let bindings = bindgen::builder()
        .header("wrapper.h")
        .allowlist_function("unpack_float_acphy")
        .generate()
        .expect("unable to generate bindings");

    // Write the generated bindings to an output file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new().file("unpack.c").compile("unpack");
}
