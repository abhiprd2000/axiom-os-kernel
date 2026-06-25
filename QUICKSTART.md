# Axiom OS — Quick Start Guide

## Prerequisites

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# The repo pins nightly via its rust-toolchain file; just add the components.
rustup component add rust-src llvm-tools-preview

# Bootimage (x86_64 bootable image)
cargo install bootimage

# QEMU + ARM cross-binutils (objcopy) + assembler
sudo apt update
sudo apt install -y qemu-system-x86 qemu-system-arm binutils-aarch64-linux-gnu nasm
```

> The ARM build uses the in-repo custom target `aarch64-axiom.json` with `-Z build-std`
> (which is why `rust-src` is required) and `aarch64-linux-gnu-objcopy` from
> `binutils-aarch64-linux-gnu`. No extra `rustup target add` step is needed.

## Boot x86_64

```bash
cd axiom-os-kernel
cargo run --bin axiom_os
```

Expected: the AXIOM OS v0.3.0-alpha banner and an interactive shell prompt.

## Boot ARM64 (QEMU virt)

```bash
./run_arm.sh
```

Expected: the AXIOM OS v0.3.0-alpha aarch64 banner, a BLAKE3 hash, and the benchmark output.

## Demo: tamper detection (x86_64)

Inside the booted shell:

```text
trust secret hello world
cat secret
tamper secret
cat secret
```

Expected on the final `cat`: `READ BLOCKED — provenance violation`.

## In-kernel benchmark (QEMU)

```text
bench
```

Reports the proportional-access ratio (whole-file vs. block-level verification) measured
in-kernel via RDTSC. Cycle counts under QEMU are used only to confirm the ratio, not as
absolute timings.

## Real-hardware benchmark (`bench-native`)

`bench-native` is a `std` crate nested in this `no_std` repo, so it inherits the kernel's
bare-metal cargo config. Build and run it **outside** the kernel tree:

```bash
cp -r bench-native /tmp/axiom-bench && cd /tmp/axiom-bench
cargo run --release
```

Reports BLAKE3 throughput, whole-file vs. block-level verification time, and the
inline-vs-periodic comparison. See `BENCHMARKS.md` for current numbers and hardware.