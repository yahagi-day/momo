fn main() {
    #[cfg(feature = "decklink")]
    {
        let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
        if target_os == "linux" {
            let sdk_include = std::path::Path::new("sdk/include");

            cxx_build::bridge("src/ffi.rs")
                .file("cpp/decklink_bridge.cpp")
                .file(sdk_include.join("DeckLinkAPIDispatch.cpp"))
                .include(sdk_include)
                .include("cpp")
                .flag_if_supported("-std=c++14")
                .compile("momo_decklink_bridge");

            println!("cargo:rustc-link-lib=dl");
            println!("cargo:rerun-if-changed=cpp/decklink_bridge.h");
            println!("cargo:rerun-if-changed=cpp/decklink_bridge.cpp");
            println!("cargo:rerun-if-changed=src/ffi.rs");
        }
    }
}
