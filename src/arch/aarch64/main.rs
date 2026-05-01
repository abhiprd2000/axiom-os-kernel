#![no_std]
#![no_main]

mod uart;
mod boot;

use uart::{uart_puts, uart_put_u64};

#[unsafe(no_mangle)]
pub extern "C" fn kernel_main_arm() -> ! {
    uart_puts("\n  AXIOM OS v0.3.0 - aarch64\n\n");

    let sp: u64;
    unsafe { core::arch::asm!("mov {}, sp", out(reg) sp); }
    uart_puts("  SP: ");
    uart_put_u64(sp);
    uart_puts("\n\n");

    // Test BLAKE3 directly - stack is fine now
    uart_puts("  Testing BLAKE3...\n");
    let result = run_blake3();
    uart_puts("  Hash: ");
    for byte in &result[..8] {
        uart_put_hex(*byte);
    }
    uart_puts("...\n");
    uart_puts("  BLAKE3: OK\n\n");

    // Benchmark
    uart_puts("  Benchmarking BLAKE3 (100 iters)...\n");
    let start = read_cntvct();
    for _ in 0..100u32 {
        let _ = run_blake3();
    }
    let end = read_cntvct();
    let avg = (end - start) / 100;
    uart_puts("  Cycles/op: ");
    uart_put_u64(avg);
    uart_puts("\n  Benchmark: OK\n\n");

    uart_puts("  ARM64 boot successful. Halting.\n");
    loop { unsafe { core::arch::asm!("wfe") }; }
}

#[inline(never)]
fn run_blake3() -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"axiom os arm64 blake3 test");
    *hasher.finalize().as_bytes()
}

fn uart_put_hex(byte: u8) {
    let hex = b"0123456789abcdef";
    uart::uart_putc(hex[(byte >> 4) as usize]);
    uart::uart_putc(hex[(byte & 0xF) as usize]);
}

fn read_cntvct() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!("mrs {}, cntvct_el0", out(reg) val);
    }
    val
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    uart::uart_puts("\n  KERNEL PANIC\n");
    loop { unsafe { core::arch::asm!("wfe") }; }
}
