use anyhow::{Result, anyhow};
use clap::Parser;
/// Task builder
/// make the plugin
use std::path::PathBuf;
use std::process::Command;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    action: Option<String>,
}

const PLUGINS: [&str; 3] = ["plugin", "qrcoder", "base_service"];

fn main() -> Result<()> {
    let args = Args::parse();
    println!("{:#?}", args);
    let r = run();
    println!("{:#?}", r);
    // let target_arch = "wasm32-wasip2";
    // for plugin_name in PLUGINS.iter() {
    //     build(plugin_name, target_arch)?;
    //     copy_into_place(plugin_name, target_arch);
    // }
    // From "target\wasm32-unknown-unknown\release\plugin.wasm"
    // 2. Perform your post-build tasks here
    Ok(())
}

fn copy_into_place(plugin_name: &str, target_arch: &str) {
    println!("Doing post-build work (e.g., copying binaries)...");
    let binding = format!("{}.wasm", &plugin_name);
    let wasm_name = binding.as_str();
    let sections = vec!["target", target_arch, "release", wasm_name];
    let source: PathBuf = sections.iter().collect();

    // make the dir
    let _ = std::fs::create_dir("dist");
    let dsections = vec!["dist", wasm_name];
    let dest: PathBuf = dsections.iter().collect();

    println!("{:?} => {:?}", source, dest);

    std::fs::copy(source, dest).unwrap();
}

fn build(plugin_name: &str, target_arch: &str) -> Result<()> {
    let status = Command::new("cargo")
        .args([
            "build",
            "--package",
            plugin_name,
            "--target",
            target_arch,
            "--release",
        ])
        .status()
        .expect("Failed to run cargo build");

    if !status.success() {
        return Err(anyhow!("{} failed to build", plugin_name));
    }
    Ok(())
}

fn run() -> Result<()> {
    let status = Command::new("target/debug/runner.exe")
        .status()
        .expect("Failed to run cargo build");
    Ok(())
}
