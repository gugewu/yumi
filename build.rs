// 简化后的 build.rs
use std::process::Command;
use std::env;
use std::path::PathBuf;

fn build_ebpf() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ebpf_dir = manifest_dir.join("yumi-ebpf");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let target_dir = out_dir.join("ebpf_target");

    println!("cargo:rerun-if-changed={}", ebpf_dir.join("Cargo.toml").display());
    println!("cargo:rerun-if-changed={}", ebpf_dir.join("src").display());

    let mut ebpf_args = vec![
        "--target", "bpfel-unknown-none",
        "-Z", "build-std=core",
        "--target-dir", target_dir.to_str().unwrap(),
    ];

    #[cfg(not(debug_assertions))]
    ebpf_args.push("--release");

    let status = Command::new("cargo")
        .arg("build")
        .args(&ebpf_args)
        .current_dir(&ebpf_dir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .env("BPF_LINKER_OPT_LEVEL", "2")
        .env("RUSTFLAGS", "-C opt-level=2")
        .status()?;

    if !status.success() {
        panic!("yumi-ebpf 编译失败");
    }

    #[cfg(debug_assertions)]
    let profile = "debug";
    #[cfg(not(debug_assertions))]
    let profile = "release";

    let built_obj = target_dir
        .join("bpfel-unknown-none")
        .join(profile)
        .join("yumi-ebpf");

    Ok(built_obj)
}

fn main() {
    match build_ebpf() {
        Ok(bpf_obj) => {
            println!("cargo:warning=✅ yumi-ebpf 编译成功: {}", bpf_obj.display());
        }
        Err(e) => {
            panic!("yumi-ebpf 编译失败: {e}");
        }
    }
}
