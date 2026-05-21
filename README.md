This repository reflects the v0.2.0-alpha research prototype described in the submitted manuscript.

# Axiom OS

A bare-metal OS kernel in Rust enforcing BLAKE3 cryptographic 
provenance on every file read — not just at load time.

## Core Contribution

Every `read()` call unconditionally recomputes the BLAKE3 hash 
and compares it against the stored provenance record before 
returning data. There is no API to bypass this check. The 
verification IS the read path.

This differs architecturally from Linux IMA, which verifies 
at file open/exec time only. A file modified in memory after 
load is invisible to IMA but blocked by Axiom OS on the next 
read.

## Architecture

- **Target:** x86_64 (primary) + ARM64 (QEMU virt)
- **Language:** Rust (no_std, no libc)
- **Lines:** ~3,200
- **Release:** v0.2.0-alpha

**Kernel subsystems:**
GDT/IDT, 8MB linked-list heap, priority scheduler, 
per-process page tables (CR3 isolation), IPC message queues, 
ATA PIO driver, syscall interface (INT 0x80), 28-command shell

**Provenance layer:**
- VFS: per-read BLAKE3 verify, constant_time_eq comparison
- FAT32: 32-byte hash store at sector 1, full 256-bit comparison
- Mitra DSL: `trusted_data` type routes through kernel read path

## Quick Start

```bash
# Prerequisites
rustup component add rust-src llvm-tools-preview
rustup target add aarch64-unknown-none
cargo install bootimage
sudo apt install qemu-system-x86 qemu-system-arm nasm

# Boot x86_64
cargo run --bin axiom_os

# Boot ARM64
./run_arm.sh
```

## Tamper Detection Demo

trust secret hello world

cat secret          # returns: hello world

tamper secret       # flips byte in memory

cat secret          # READ BLOCKED: provenance violation

## Benchmarks

RDTSC bare-metal x86_64, 5 independent cold-boot runs:

| Operation | Mean cycles/op | CV | Latency @3GHz |
|---|---|---|---|
| BLAKE3 hash | 424,013 | 2.9% | 0.141ms |
| VFS read+verify | 2,153,973 | 3.0% | 0.718ms |

BLAKE3 constitutes 19.7% of total read+verify overhead.

## Mitra DSL

Domain-specific language with kernel-enforced provenance:

trusted_data secret = classified report

verify secret     → KERNEL VERIFIED

[tamper]

verify secret     → KERNEL BLOCKED

## Known Limitations

- QEMU only — UEFI bootloader pending (v0.3.0)
- Single-core — SMP planned
- In-memory VFS — persistence via ATA only
- Pure Rust BLAKE3 — SIMD (NEON/AVX-512) pending
- ARM64 cycle benchmarks invalid on QEMU (CNTVCT_EL0)

## Threat Model

Defends against post-write in-memory tampering by ring-3 
processes within a single-core, non-DMA execution model. 
Does not defend against ring-0 compromise, DMA attacks, 
multicore races, or speculative execution side channels.

## Paper

Under review. Data and full methodology available upon 
acceptance.

## License

MIT, Apache-2.0