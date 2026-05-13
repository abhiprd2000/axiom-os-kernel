# Axiom OS — Quick Start Guide

## Prerequisites

```bash
# Install Rust nightly
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup toolchain install nightly
rustup override set nightly

# Install bootimage
cargo install bootimage

# Install QEMU
sudo apt install qemu-system-x86 qemu-system-arm

# Install ARM64 cross tools (for ARM build only)
sudo apt install binutils-aarch64-linux-gnu
```

## Boot x86_64

```bash
git clone https://github.com/abhiprd2000/axiom-os-kernel.git
cd axiom-os-kernel
cargo run --bin axiom_os
```

Expected output: AXIOM OS v0.2.0-alpha banner with shell prompt.

## Boot ARM64

```bash
./run_arm.sh
```

Expected output: AXIOM OS v0.2.0-alpha - aarch64, BLAKE3 hash, benchmark results.

## Run Benchmarks

Inside the x86_64 shell:
bench

## Demo: Tamper Detection
trust secret hello world
cat secret
tamper secret
cat secret

Last command shows: READ BLOCKED — provenance violation
