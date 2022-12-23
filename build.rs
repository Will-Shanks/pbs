#[cfg(feature="bindgen")]
extern crate bindgen;

use std::env;
#[cfg(feature="bindgen")]
use std::path::PathBuf;

fn main() {
    // add libclang to link path only needed to generate bindings
    #[cfg(feature="bindgen")]
    let libclang_path = env::var("LIBCLANG_PATH").unwrap();
    let pbs_path = env::var("PBS_PATH").unwrap();
    #[cfg(feature="bindgen")]
    println!("cargo-rustc-link-search={}",libclang_path);
    println!("cargo-rustc-link-search={}/lib", pbs_path);

    // Tell cargo to tell rustc to link the system pbs
    println!("cargo:rustc-link-lib=pbs");

    // Tell cargo to invalidate the built crate whenever the wrapper changes
    println!("cargo:rerun-if-changed=wrapper.h");

    // The bindgen::Builder is the main entry point
    // to bindgen, and lets you build up options for
    // the resulting bindings.
    #[cfg(feature="bindgen")]
    let bindings = bindgen::Builder::default()
        .rustfmt_bindings(true)
        .generate_comments(true)
        // The input header we would like to generate
        // bindings for.
        .header("wrapper.h")
        .clang_arg(format!("-I{}/include",pbs_path))
        .clang_arg(format!("-I{}/clang/16.0.0/include", libclang_path))
        .clang_arg("-fparse-all-comments")
        .clang_arg("-fretain-comments-from-system-headers")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .derive_copy(false)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    #[cfg(feature="bindgen")]
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    #[cfg(feature="bindgen")]
    bindings
        .write_to_file(out_path.join("pbsffi.rs"))
        .expect("Couldn't write bindings!");
}
