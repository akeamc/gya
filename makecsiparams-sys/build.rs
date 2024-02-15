use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=vendor");
    println!("cargo:rerun-if-changed=makecsiparams.h");

    // Configure and generate bindings.
    let bindings = bindgen::builder()
        .header("makecsiparams.h")
        .header("vendor/bcmwifi_channels.h")
        .generate()
        .expect("unable to generate bindings");

    // Write the generated bindings to an output file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    cc::Build::new()
        .include("vendor")
        .file("vendor/makecsiparams.c")
        .file("vendor/bcmwifi_channels.c")
        .warnings(false)
        .flag("-Wno-pointer-to-int-cast")
        .compile("makecsiparams-sys-cc");
}
