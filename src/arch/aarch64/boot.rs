use core::arch::global_asm;

global_asm!(
    ".section .text._start",
    "_start:",
    "   mrs x0, mpidr_el1",
    "   and x0, x0, #0xFF",
    "   cbnz x0, halt",

    // Enable NEON/FP at EL1
    // CPACR_EL1 bits [21:20] = 0b11 enables FP/SIMD access
    "   mrs x0, cpacr_el1",
    "   orr x0, x0, #(0x3 << 20)",
    "   msr cpacr_el1, x0",
    "   isb",

    // Set stack pointer
    "   mov x0, #0x4800",
    "   lsl x0, x0, #16",
    "   mov sp, x0",

    "   bl zero_bss",
    "   bl kernel_main_arm",

    "halt:",
    "   wfe",
    "   b halt",
);

#[unsafe(no_mangle)]
pub unsafe extern "C" fn zero_bss() {
    unsafe extern "C" {
        static mut __bss_start: u64;
        static mut __bss_end: u64;
    }
    unsafe {
        let mut ptr = &raw mut __bss_start as *mut u64;
        let end = &raw mut __bss_end as *mut u64;
        while ptr < end {
            ptr.write_volatile(0);
            ptr = ptr.add(1);
        }
    }
}
