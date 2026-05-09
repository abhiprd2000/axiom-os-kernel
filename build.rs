use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    let host = env::var("HOST").unwrap_or_default();
    
    eprintln!("build.rs: TARGET={} HOST={}", target, host);
    
    if !target.starts_with("x86_64") {
        eprintln!("build.rs: skipping nasm for non-x86_64 target");
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    println!("cargo:rerun-if-changed=src/task/switch.s");
    
    let status = std::process::Command::new("nasm")
        .args(&["-f", "elf64", "src/task/switch.s", "-o"])
        .arg(&format!("{}/switch.o", out_dir.display()))
        .status()
        .expect("Failed to assemble switch.s");
        
    if !status.success() {
        panic!("nasm failed");
    }
        
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=switch");
    
    ar::Builder::new(std::fs::File::create(format!("{}/libswitch.a", out_dir.display())).unwrap())
        .append_path(format!("{}/switch.o", out_dir.display()))
        .unwrap();
}
