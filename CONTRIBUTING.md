# Contributing to Axiom OS

Axiom OS is a research prototype implementing kernel-level cryptographic provenance for high-integrity computing on low-end hardware. We welcome contributors who are passionate about systems security, Rust, and bare-metal engineering.

## 🚀 Project Roadmap (v2.0 Goals)
Based on our current whitepaper, we are seeking help with:
* [cite_start]**TCP/IP Network Stack:** Required for remote attestation and remote provenance logging[cite: 527].
* [cite_start]**Multi-core SMP Scheduler:** Moving from single-core to modern multi-core support[cite: 528].
* [cite_start]**Formal Verification:** Using Coq or Lean to verify the provenance enforcement path[cite: 528].
* [cite_start]**Mitra DSL Expansion:** Extending the type system for advanced information flow tracking[cite: 528].

## 🛠 Getting Started
1. **Environment:** Ensure you have a Rust nightly toolchain and `qemu-system-x86_64` installed.
2. **Build:** Use `cargo build` to compile the kernel.
3. **Run:** Execute using our provided scripts to boot the ISO in QEMU.

## 📜 Pull Request Guidelines
* **No Standard Library:** Axiom is `no_std`. [cite_start]Avoid any crates that require `std`.
* [cite_start]**Performance First:** All new features must be benchmarked using RDTSC to ensure they fit within our low-end device resource envelope[cite: 383, 446].
* [cite_start]**Documentation:** Updates to kernel subsystems must be reflected in the subsystem inventory[cite: 462, 464].

## 🧑‍🔬 Research Background
[cite_start]Please review our (https://zenodo.org/records/19387932) for the full architectural comparison between Axiom and Linux IMA[cite: 491, 492].
