//! Compiles the C++ shim and links `QuantLib`.
//!
//! `QuantLib` and Boost headers and the `QuantLib` library are found through the
//! compiler's default search paths, which `nix develop` and the Nix checks set
//! up. Elsewhere, point `QUANTLIB_INCLUDE_DIR`, `BOOST_INCLUDE_DIR` and
//! `QUANTLIB_LIB_DIR` at an installation.

use std::env;

fn main() {
    println!("cargo:rerun-if-changed=shim.cpp");
    for var in [
        "QUANTLIB_INCLUDE_DIR",
        "BOOST_INCLUDE_DIR",
        "QUANTLIB_LIB_DIR",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
    }

    let mut build = cc::Build::new();
    build.cpp(true).std("c++17").opt_level(2).file("shim.cpp");
    for var in ["QUANTLIB_INCLUDE_DIR", "BOOST_INCLUDE_DIR"] {
        if let Ok(dir) = env::var(var) {
            build.include(dir);
        }
    }
    build.compile("quantlib_shim");

    if let Ok(dir) = env::var("QUANTLIB_LIB_DIR") {
        println!("cargo:rustc-link-search=native={dir}");
    }
    println!("cargo:rustc-link-lib=dylib=QuantLib");
}
