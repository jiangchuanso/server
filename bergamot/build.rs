use cmake::Config;
use std::env;
use std::str::FromStr;

fn main() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dst = Config::new("translation")
        .env(
            "BERGAMOT_SUBMODULE_PATH",
            env::var("BERGAMOT_SUBMODULE_PATH")
                .unwrap_or_else(|_| format!("{}/bergamot-translator", crate_dir)),
        )
        .profile(if let "release" = env::var("PROFILE").unwrap().as_str() {
            "Release"
        } else {
            "Debug"
        })
        .build();

    println!("cargo:rustc-link-search=native={}/build", dst.display());
    println!("cargo:rustc-link-search=native={}/build/lib", dst.display());
    println!("cargo:rustc-link-lib=static=translation");
    println!("cargo:rustc-link-lib=static=ssplit");
    println!("cargo:rustc-link-lib=static=marian");
    println!("cargo:rustc-link-lib=static=bergamot-translator");
    println!("cargo:rustc-link-lib=static=intgemm");
    println!("cargo:rustc-link-lib=static=sentencepiece_train");
    println!("cargo:rustc-link-lib=static=sentencepiece");
    println!("cargo:rustc-link-lib=pcre2-8");
    println!("cargo:rustc-link-lib=blas");
    println!("cargo:rustc-link-lib=lapack");
    println!("cargo:rustc-link-lib=stdc++");

    let cfg = intel_mkl_tool::Config::from_str("mkl-static-ilp64-iomp").unwrap();
    intel_mkl_tool::Library::new(cfg)
        .unwrap()
        .print_cargo_metadata()
        .unwrap();

    println!("cargo:rerun-if-changed=translation/translation.h");
    println!("cargo:rerun-if-changed=translation/translation.cpp");
    println!("cargo:rerun-if-changed=translation/CMakeLists.txt");
    println!("cargo:rerun-if-changed=src/lib.rs");
}
