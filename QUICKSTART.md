# Axiom OS — Quick Start Guide

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# The repo sets nightly automatically via rust-toolchain file
# Just add required components
rustup component add rust-src llvm-tools-preview
rustup target add aarch64-unknown-none

# Install bootimage
cargo install bootimage

# Install QEMU and ARM tools
sudo apt update
sudo apt install -y qemu-system-x86 qemu-system-arm binutils-aarch64-linux-gnu nasm
```

## Boot x86_64

```bash
git clone https://github.com/abhiprd2000/axiom-os-kernel.git
cd axiom-os-kernel
cargo run --bin axiom_os
```

Expected: AXIOM OS v0.2.0-alpha banner + shell prompt.

## Boot ARM64

```bash
./run_arm.sh
```

Expected: AXIOM OS v0.2.0-alpha - aarch64 + BLAKE3 hash + benchmark.

## Demo: Tamper Detection (x86_64)

Type these commands inside the booted OS:
trust secret hello world
cat secret
tamper secret
cat secret

Expected on last command: READ BLOCKED — provenance violation

## Run Benchmarks
bench

Reports BLAKE3 and VFS read+verify cycles with mean and CV.
