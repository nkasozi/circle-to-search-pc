use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tessdata/");

    if target_os == "windows" {
        println!("cargo:rustc-link-search=native=C:/Program Files/Tesseract-OCR");
    }

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    let tessdata_src = manifest_dir.join("tessdata");
    if tessdata_src.exists() {
        println!("cargo:warning=Found tessdata directory for bundling");
    } else {
        println!("cargo:warning=No tessdata directory found. Tesseract will use system data.");
    }
}
