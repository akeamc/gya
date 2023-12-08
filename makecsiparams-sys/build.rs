use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=makecsiparams");

    let bindings = bindgen::Builder::default()
        .header("makecsiparams/makecsiparams.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings");

    cc::Build::new()
        .file("makecsiparams/makecsiparams.c")
        .file("makecsiparams/bcmwifi_channels.c")
        .include("makecsiparams")
        .compile("makecsiparams");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
