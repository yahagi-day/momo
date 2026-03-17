fn main() {
    #[cfg(feature = "gpu")]
    {
        use std::path::PathBuf;
        use std::process::Command;

        let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let kernels_dir = PathBuf::from("kernels");

        for kernel in &["crop", "scale", "flip"] {
            let cu_path = kernels_dir.join(format!("{kernel}.cu"));
            let ptx_path = out_dir.join(format!("{kernel}.ptx"));

            println!("cargo:rerun-if-changed={}", cu_path.display());

            let status = Command::new("nvcc")
                .args([
                    "--ptx",
                    "-o",
                    ptx_path.to_str().unwrap(),
                    cu_path.to_str().unwrap(),
                ])
                .status()
                .expect("Failed to run nvcc — is CUDA Toolkit installed?");

            assert!(
                status.success(),
                "nvcc failed to compile {}.cu",
                kernel
            );
        }
    }
}
