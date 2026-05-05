# Axiom OS

A bare-metal x86_64 operating system kernel written in Rust with hardware-enforced
BLAKE3 cryptographic provenance on every file read.

## Built on blog_os

Commits 1–30 follow Philipp Oppermann's "Writing an OS in Rust" tutorial
(os.phil-opp.com, posts 1–10) as the foundation. All original work begins
from Day 31 — FAT32, BLAKE3 provenance, ATA driver, process isolation,
priority scheduler, IPC, Mitra DSL, calculator, and the full 28-command shell.

## Run
```bash
qemu-system-x86_64 \
  -drive if=ide,format=raw,file=target/x86_64-blog_os/debug/bootimage-blog_os.bin,index=0 \
  -drive if=ide,format=raw,file=axiom-disk.img,index=2 \
  -no-reboot
```

## Key Features

- Per-read BLAKE3 provenance on VFS + FAT32
- Process isolation via per-process page tables
- Priority scheduler, IPC, syscall interface (int 0x80)
- Mitra DSL with trusted_data type
- 28-command shell with tab completion and history
- Text editor, calculator, persistent ATA disk storage

## Benchmarks (bare metal RDTSC)

- BLAKE3: 574,302 avg cycles/op (1000 iterations)
- VFS read+verify: 2,389,648 avg cycles/op (100 iterations)
- Boot: ~4 seconds | RAM: <1% | CPU overhead: 0.038%

## Research Paper

See: (https://zenodo.org/records/19387932)

## License

MIT — built on phil-opp/blog_os (MIT)
