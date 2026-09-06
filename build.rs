use std::process::Command;
use std::env;
use std::path::PathBuf;

/// 构建 yumi-ebpf BPF 程序，参照 frame-analyzer 的 build_ebpf()
fn build_ebpf() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let ebpf_dir = manifest_dir.join("yumi-ebpf");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let target_dir = out_dir.join("ebpf_target");
    let tools_dir = out_dir.join("ebpf_tools");
    let tools_bin = tools_dir.join("bin");

    println!("cargo:rerun-if-changed={}", ebpf_dir.join("Cargo.toml").display());
    println!("cargo:rerun-if-changed={}", ebpf_dir.join("src").display());

    // 1. 安装 bpf-linker
    Command::new("cargo")
        .args([
            "install", "bpf-linker", "--force",
            "--root", tools_dir.to_str().unwrap(),
            "--target-dir", tools_dir.to_str().unwrap(),
        ])
        .env_remove("RUSTUP_TOOLCHAIN")
        .status()?;

    // 2. 编译 BPF 程序
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
        .env("PATH", add_path(&tools_bin)?)
        // 关键修复：强制 bpf-linker 使用 O2 而不是 Oz
        .env("BPF_LINKER_OPT_LEVEL", "2")
        // RUSTFLAGS 保留，但实际起作用的是上面的环境变量
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

fn add_path(add: &std::path::Path) -> Result<String, std::env::VarError> {
    let path = env::var("PATH")?;
    Ok(format!("{}:{}", add.display(), path))
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
