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
        } else if target_os == "windows" {
            let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
            let sdk_win = std::path::Path::new("sdk/include/win");

            // Run midl.exe to generate C++ headers from IDL files
            let midl_status = std::process::Command::new("midl.exe")
                .arg("/h")
                .arg("DeckLinkAPI_h.h")
                .arg("/iid")
                .arg("DeckLinkAPI_i.c")
                .arg("/proxy")
                .arg("nul")
                .arg("/dlldata")
                .arg("nul")
                .arg("/tlb")
                .arg("nul")
                .arg("/out")
                .arg(&out_dir)
                .arg("/I")
                .arg(sdk_win)
                .arg(sdk_win.join("DeckLinkAPI.idl"))
                .status()
                .expect("failed to run midl.exe — is MSVC installed?");
            assert!(midl_status.success(), "midl.exe failed to compile IDL");

            cxx_build::bridge("src/ffi.rs")
                .file("cpp/decklink_bridge.cpp")
                .file(out_dir.join("DeckLinkAPI_i.c"))
                .include(&out_dir)
                .include("cpp")
                .include(sdk_win)
                .compile("momo_decklink_bridge");

            println!("cargo:rustc-link-lib=ole32");
            println!("cargo:rustc-link-lib=oleaut32");
        }

        println!("cargo:rerun-if-changed=cpp/decklink_bridge.h");
        println!("cargo:rerun-if-changed=cpp/decklink_bridge.cpp");
        println!("cargo:rerun-if-changed=src/ffi.rs");
    }
}
