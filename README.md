This repository reflects the v0.3.0-alpha research prototype described in the submitted manuscript
*"Proportional-Cost Per-Read Provenance: Making Continuous File Integrity Verification Affordable on Edge Hardware."*

# Axiom OS

A bare-metal `no_std` Rust kernel that enforces BLAKE3 file-integrity provenance on **every read** —
not just at load time — and makes that affordable by verifying only the file blocks a read actually touches.

## Core Contribution

Most integrity mechanisms verify once: at boot, at open, or when a page is first read from storage
(e.g. fs-verity, dm-verity, IMA). After that, the cached in-memory copy is trusted. Axiom re-verifies
the in-memory data on **every** read, so tampering that happens *after* the initial check (a memory bug,
DMA, a row-hammer flip) is caught on the next read.

The naive way to do this — re-hash the whole file on every read — costs O(file size) per read and is
impractical on small devices. Axiom instead uses **block-level provenance**:

- A file is split into fixed 4 KiB blocks; each block has its own stored BLAKE3 leaf hash.
- A ranged read `read_range(offset, len)` maps to the blocks it overlaps and re-hashes **only those blocks**,
  comparing each against its stored leaf in constant time. Per-read cost becomes O(bytes read), not O(file size).
- A **lazy Merkle root** over the block leaves gives a single 32-byte commitment to the file; a single-block
  write updates one leaf and defers the root recomputation, so writes cost one block hash instead of a full rehash.

This is the affordability result: per-read verification cost stays flat regardless of file size (see Results).

## How it differs from fs-verity / IMA

fs-verity and IMA bind verification to the storage-access path — a page is checked when read from the device,
then trusted in the page cache. They do not re-verify the cached copy on subsequent reads. Axiom's check *is*
the read path, on every read, against per-block hashes. See Table II of the paper for the full comparison.

## Architecture

- **Target:** x86_64 (primary) + ARM64 (QEMU `virt`)
- **Language:** Rust (`no_std`, no libc)
- **Source:** ~3,600 lines across 36 files
- **Release:** v0.3.0-alpha (research prototype)

**Kernel subsystems:** GDT/IDT, 8 MB linked-list heap, priority scheduler, per-process page tables
(CR3 isolation), IPC message queues, ATA PIO driver, syscall interface (INT 0x80), interactive shell.

**Provenance layer:**
- `vfs.rs` — per-block BLAKE3 leaves, `read_range`/`verify_range` (verify only touched blocks),
  `write_block` + lazy `merkle_root`, `constant_time_eq` comparison.
- `provenance.rs` — `provenance_hash` (BLAKE3) and constant-time comparison primitive.
- `fat32.rs` — on-disk hash store for the persistent path.
- `mitra/` — small DSL whose `trusted_data` type routes through the verified read path.

## Quick Start

```bash
# Prerequisites
rustup component add rust-src llvm-tools-preview
rustup target add aarch64-unknown-none
cargo install bootimage
sudo apt install qemu-system-x86 qemu-system-arm nasm

# Boot x86_64 (use the bin target; plain `cargo build` also tries the ARM bin)
cargo run --bin axiom_os

# Boot ARM64
./run_arm.sh
```

## Tamper-Detection Demo

Inside the booted shell:

```
trust secret hello world     # store a file + its provenance
cat secret                   # -> hello world
tamper secret                # flip a byte in memory
cat secret                   # -> READ BLOCKED: provenance violation
```

## Results

The headline result is **proportional access**: block-level verification cost is constant (~3.6 µs)
across file sizes, while whole-file verification grows with size. Numbers below are **median wall-clock
times on real hardware** (a commodity x86-64 laptop and a mobile-class ARM64 core), produced by the
standalone `bench-native` harness:

| File size | Blocks | x86-64 whole (ms) | x86-64 block (ms) | ARM64 whole (ms) | ARM64 block (ms) |
|---|---|---|---|---|---|
| 4 KiB   | 1    | 0.0040 | 0.0037 | 0.0037 | 0.0037 |
| 16 KiB  | 4    | 0.0097 | 0.0033 | 0.0143 | 0.0037 |
| 64 KiB  | 16   | 0.046  | 0.0035 | 0.057  | 0.0037 |
| 256 KiB | 64   | 0.159  | 0.0037 | 0.230  | 0.0037 |
| 1 MiB   | 256  | 0.580  | 0.0035 | 0.923  | 0.0037 |
| 4 MiB   | 1024 | 2.51   | 0.0039 | 3.74   | 0.0037 |

Block-level verification is >640× cheaper (x86-64) and >1000× cheaper (ARM64) than whole-file at 4 MiB.

Reproduce the real-hardware numbers:

```bash
cd bench-native
cargo run --release        # prints the whole-file vs block table for this CPU
```

Reproduce the in-kernel proportional ratio under emulation:

```
bench                       # in the booted shell; RDTSC, proportional-access size sweep
```

**Note on QEMU:** in-kernel cycle counts are emulated and are used only to confirm the *ratio*
(block-flat vs whole-file-growing). All absolute millisecond figures above come from real hardware,
not emulation.

## Limitations

- Boots under QEMU; UEFI bootloader for bare-metal boot is pending.
- Single-core; SMP and multi-core verification are future work.
- BLAKE3 runs in portable mode in-kernel (no in-kernel SIMD yet); the native harness uses runtime SIMD.
- Real-hardware results use a consumer laptop and phone; a dedicated embedded SoC (e.g. Raspberry Pi)
  measurement is the immediate next step.

## Threat Model

Assumes an adversary who can modify a file's in-memory bytes after they were validated, but cannot forge
the stored provenance record (per-block leaves + root); single-core, no DMA, no ring-0. Within this model,
per-read verification **reduces** the TOCTOU window: bytes returned by `read()` are verified at the moment
of return. It does **not** eliminate TOCTOU (corruption after a verified read is out of scope), nor address
replay/splicing on persistent storage or hardware memory-integrity attacks.

## Paper

- Earlier formulation (whole-file per-read, equity framing): Zenodo, Apr. 2026,
  [doi:10.5281/zenodo.19387932](https://doi.org/10.5281/zenodo.19387932).

## License

MIT and Apache-2.0