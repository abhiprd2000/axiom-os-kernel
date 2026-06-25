# Axiom OS — Benchmark Log

A running record of Axiom OS provenance benchmarks across versions. Older sections are
kept for history; the most recent version reflects the current design.

---

## v0.3.0-alpha — Block-level provenance, real hardware

**What changed from v0.2.0.** v0.2.0 measured the *whole-file* provenance design in RDTSC
cycle counts under QEMU. v0.3.0 introduces *block-level* provenance — a read verifies only
the 4 KiB blocks it actually touches — and is measured as wall-clock time on **real
hardware** via the `bench-native` harness. The two versions are not directly comparable
cycle-for-cycle; the improvement is a change in the **cost model**, not a constant-factor
speedup: per-read cost moves from O(file size) to O(bytes read).

### Read+verify: whole-file vs. block-level

Median wall-clock time, 7 trials with warm-up:

| File size | Blocks | x86-64 whole (ms) | x86-64 block (ms) | ARM64 whole (ms) | ARM64 block (ms) |
|-----------|-------:|------------------:|------------------:|-----------------:|-----------------:|
| 4 KiB     | 1      | 0.0035            | 0.0027            | 0.0036           | 0.0037           |
| 16 KiB    | 4      | 0.0094            | 0.0033            | 0.0143           | 0.0037           |
| 64 KiB    | 16     | 0.0339            | 0.0030            | 0.0570           | 0.0037           |
| 256 KiB   | 64     | 0.108             | 0.0027            | 0.228            | 0.0037           |
| 1 MiB     | 256    | 0.555             | 0.0027            | 0.918            | 0.0037           |
| 4 MiB     | 1024   | 2.13              | 0.0026            | 3.72             | 0.0037           |

Block-level verification is constant regardless of file size — **~2.7 µs (x86-64)** and
**~3.65 µs (ARM64)** — while whole-file verification grows with size. At 4 MiB, block-level
is **~810× cheaper** (x86-64) and **~1020× cheaper** (ARM64).

### Inline vs. out-of-band detection

Over a 200,000-read trace on a 1 MiB file with one block tampered midway:

| Strategy | Cost | Corrupted reads served |
|----------|------|------------------------|
| Inline per-read | 2.66 µs/op (x86-64) | **0** |
| Periodic scan, 1,000-op interval | 0.80 µs/op amortized | 2 |
| Periodic scan, 10,000-op interval | 0.080 µs/op | 32 |
| Periodic scan, 100,000-op interval | 0.008 µs/op | 396 |

Inline enforcement pays a constant per-read cost to guarantee no tampered byte is ever
returned; periodic scanning is cheaper per operation only by tolerating a proportionally
larger corruption window. This is a safety/cost trade-off, not a speed result.

### Setup

- **x86-64:** Intel Core i5-5200U, Linux, `rustc` 1.96-nightly
- **ARM64:** Qualcomm Snapdragon 7 Gen 4 (Cortex-A720 / A520), Termux on Android 16, `rustc` 1.95
- **Hash:** `blake3` v1.8.5, runtime SIMD (AVX2 / NEON)
- **Method:** median ns/op over 7 trials with warm-up; identical block-vs-whole hashing
  logic to the kernel VFS path. The kernel itself is run under QEMU only to confirm the
  proportional-access ratio; all millisecond figures above are from physical hardware.

> Reproduce: build `bench-native` **outside** the kernel tree (it is a `std` crate nested
> in a `no_std` repo and otherwise inherits the bare-metal target):
> ```bash
> cp -r bench-native /tmp/axiom-bench && cd /tmp/axiom-bench && cargo run --release
> ```

---

## v0.2.0-alpha — Whole-file provenance (historical, QEMU cycle counts)

> Historical entry. These numbers measure the earlier *whole-file* design in RDTSC cycles
> under QEMU emulation, and are superseded by the block-level design and real-hardware
> measurements in v0.3.0 above. Cycle counts under emulation do not reflect real silicon.

### Platform
- Architecture: x86_64 bare metal
- Environment: QEMU system emulation
- Measurement: RDTSC hardware cycle counter
- Runs: 5 independent cold boots

### x86_64 Results

| Operation | Mean cycles/op | Std Dev | CV | Latency @3 GHz |
|---|---|---|---|---|
| BLAKE3 hash (1000 iters) | 424,013 | ±12,421 | 2.9% | 0.141 ms |
| VFS read+verify (100 iters) | 2,153,973 | ±64,739 | 3.0% | 0.718 ms |

### Methodology
- Each run is a fresh QEMU boot with no prior state
- RDTSC instruction used for cycle-accurate measurement
- VFS read+verify includes hash recomputation, comparison, and memory lookup

### ARM64
BLAKE3 executes correctly on ARM64 under QEMU (Cortex-A57 model). `CNTVCT_EL0` under QEMU is
a virtual timer, not a cycle counter, so values do not reflect real silicon; hardware ARM64
measurement was deferred (and is now provided natively in v0.3.0 above).

---

## Comparison: Axiom OS vs. Linux IMA

| Property | Axiom OS | Linux IMA |
|---|---|---|
| Verification trigger | Every read (in-memory) | Load / exec time only |
| Hash algorithm | BLAKE3 | SHA-256 |
| Trust boundary | Kernel read path | LSM hook |
| Re-verifies cached data | Yes | No |
| In threat model | No ring-0 / DMA adversary | — |

> Note: Axiom's guarantee holds within its stated threat model (single-core, no DMA, no
> ring-0 adversary). It does not claim to resist a privileged (ring-0) attacker; like most
> in-kernel mechanisms, it can be bypassed by an adversary already executing in the kernel.