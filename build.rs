use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=tessdata/");

    if target_os == "macos" {
        create_macos_plist();
    }

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

fn create_macos_plist() {
    let plist_content = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>Circle to Search</string>
    <key>CFBundleDisplayName</key>
    <string>Circle to Search</string>
    <key>CFBundleIdentifier</key>
    <string>com.circle-to-search.app</string>
    <key>CFBundleVersion</key>
    <string>0.1.0</string>
    <key>CFBundleShortVersionString</key>
    <string>0.1.0</string>
    <key>LSUIElement</key>
    <true/>
</dict>
</plist>"#;

    let out_dir = env::var("OUT_DIR").unwrap();
    let plist_path = PathBuf::from(&out_dir).join("Info.plist");

    std::fs::write(&plist_path, plist_content).unwrap();

    println!("cargo:warning=Created Info.plist for macOS with LSUIElement=true");
}
