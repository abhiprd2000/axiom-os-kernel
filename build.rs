use std::env;
use std::path::PathBuf;

fn main() {
    let target = env::var("TARGET").unwrap_or_default();
    
    if !target.starts_with("x86_64") {
        return;
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    
    println!("cargo:rerun-if-changed=src/task/switch.s");
    
    std::process::Command::new("nasm")
        .args(&["-f", "elf64", "src/task/switch.s", "-o"])
        .arg(&format!("{}/switch.o", out_dir.display()))
        .status()
        .expect("Failed to assemble switch.s");
        
    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=static=switch");
    
    ar::Builder::new(std::fs::File::create(format!("{}/libswitch.a", out_dir.display())).unwrap())
        .append_path(format!("{}/switch.o", out_dir.display()))
        .unwrap();
}
