//! Build script for AI engine

use std::env;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    
    // Check for CUDA support
    if env::var("CUDA_VISIBLE_DEVICES").is_ok() {
        println!("cargo:rustc-cfg=feature=\"cuda\"");
        println!("cargo:rustc-env=CANDLE_CUDA=1");
    }
    
    // Check for MKL
    if env::var("MKL_ROOT").is_ok() {
        println!("cargo:rustc-cfg=feature=\"mkl\"");
    }
    
    // Create necessary directories
    let out_dir = env::var("OUT_DIR").unwrap();
    let cache_dir = Path::new(&out_dir).join("cache");
    std::fs::create_dir_all(&cache_dir).unwrap();
    
    println!("cargo:warning=AI Engine build complete");
}
