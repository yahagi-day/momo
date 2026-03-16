use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("index.html");

    // Look for the built frontend (relative to workspace root)
    let frontend_html = Path::new("frontend/dist/index.html");
    // Also check from crate directory (cargo may run from workspace root or crate dir)
    let frontend_html_alt = Path::new("../../frontend/dist/index.html");

    if frontend_html.exists() {
        std::fs::copy(frontend_html, &dest).expect("failed to copy frontend/dist/index.html");
        println!("cargo:warning=Using built frontend from frontend/dist/index.html");
    } else if frontend_html_alt.exists() {
        std::fs::copy(frontend_html_alt, &dest).expect("failed to copy frontend/dist/index.html");
        println!("cargo:warning=Using built frontend from ../../frontend/dist/index.html");
    } else {
        // Write the embedded fallback
        std::fs::write(&dest, FALLBACK_HTML).expect("failed to write fallback index.html");
        println!("cargo:warning=frontend/dist/index.html not found, using embedded fallback UI");
    }

    println!("cargo:rerun-if-changed=frontend/dist/index.html");
    println!("cargo:rerun-if-changed=../../frontend/dist/index.html");
    println!("cargo:rerun-if-changed=build.rs");
}

const FALLBACK_HTML: &str = include_str!("src/fallback.html");
