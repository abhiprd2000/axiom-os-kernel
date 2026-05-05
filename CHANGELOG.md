# CHANGELOG

All notable changes to this project will be documented in this file.

## [Unreleased] - 2026-05
### Added
- Initialized a clean repository structure following the WSL filesystem crash.
- Linked the Zenodo publication (April 2026) as the formal baseline for the architectural whitepaper.

## [Phase 1 / Local Storage Era] - March 2026
*Note: This section documents the core architectural milestones achieved during local development prior to formal Git initialization.*

### Architected
- Bootstrapped the bare-metal x86_64 environment utilizing `blog_os` foundational crates for GDT, IDT, and PIC8259 hardware interrupts.
- Initialized physical memory mapping and heap allocation via `linked_list_allocator`.
- Scaffolded basic process isolation, passing the L4 page table to spawned tasks.

### Security & File System
- Engineered `VirtualFS`, a custom in-memory file system with an integrated provenance intercept.
- Implemented a mandatory read-blocking mechanism: the kernel automatically calculates and verifies cryptographic hashes (`provenance_hash`) before returning file slices.
- Added a `tamper()` simulation routine to validate the kernel's ability to block compromised data at the VFS layer.
- Integrated ATA persistent disk initialization.

### User Space
- Built an async keyboard polling task integrated with a rudimentary shell and text editor interface.