#!/bin/bash
cargo build --bin axiom_os_arm --target aarch64-axiom.json -Z build-std=core,alloc --release 2>&1 | tail -2
aarch64-linux-gnu-objcopy -O binary \
  target/aarch64-axiom/release/axiom_os_arm \
  target/aarch64-axiom/release/axiom_os_arm.bin
qemu-system-aarch64 \
  -machine virt \
  -cpu cortex-a57 \
  -nographic \
  -device loader,file=target/aarch64-axiom/release/axiom_os_arm.bin,addr=0x41000000,cpu-num=0 \
  -device loader,addr=0x41000000,cpu-num=0 \
  -m 256M
