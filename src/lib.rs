#![no_std]
#![cfg_attr(target_arch = "x86_64", feature(abi_x86_interrupt))]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![allow(unused_unsafe)]

extern crate alloc;
use core::panic::PanicInfo;

// Shared modules - work on all architectures
pub mod print;
pub mod vfs;
pub mod provenance;
#[cfg(target_arch = "x86_64")]
pub mod benchmark;
pub mod ramdisk;
pub mod fat32;
#[cfg(target_arch = "x86_64")]
pub mod mitra;
#[cfg(target_arch = "x86_64")]
pub mod scheduler;
#[cfg(target_arch = "x86_64")]
pub mod ipc;
#[cfg(target_arch = "x86_64")]
pub mod syscall;
#[cfg(target_arch = "x86_64")]
pub mod shell;
#[cfg(target_arch = "x86_64")]
pub mod editor;
#[cfg(target_arch = "x86_64")]
pub mod calc;

// x86_64 only modules
#[cfg(target_arch = "x86_64")]
pub mod allocator;
#[cfg(target_arch = "x86_64")]
pub mod ata;
#[cfg(target_arch = "x86_64")]
pub mod process;
#[cfg(target_arch = "x86_64")]
pub mod task;
#[cfg(target_arch = "x86_64")]
pub mod gdt;
#[cfg(target_arch = "x86_64")]
pub mod interrupts;
#[cfg(target_arch = "x86_64")]
pub mod memory;
#[cfg(target_arch = "x86_64")]
pub mod serial;
#[cfg(target_arch = "x86_64")]
pub mod vga_buffer;

// x86_64 init
#[cfg(target_arch = "x86_64")]
pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

// x86_64 halt loop
#[cfg(target_arch = "x86_64")]
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

// ARM64 halt loop
#[cfg(target_arch = "aarch64")]
pub fn hlt_loop() -> ! {
    loop {
        unsafe { core::arch::asm!("wfe") };
    }
}

// x86_64 QEMU exit
#[cfg(target_arch = "x86_64")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed  = 0x11,
}

#[cfg(target_arch = "x86_64")]
pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

// Test framework (x86_64 only)
pub trait Testable {
    fn run(&self) -> ();
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        #[cfg(target_arch = "x86_64")]
        serial_print!("{}...\t", core::any::type_name::<T>());
        self();
        #[cfg(target_arch = "x86_64")]
        serial_println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    #[cfg(target_arch = "x86_64")]
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    #[cfg(target_arch = "x86_64")]
    exit_qemu(QemuExitCode::Success);
}

pub fn test_panic_handler(info: &PanicInfo) -> ! {
    #[cfg(target_arch = "x86_64")]
    {
        serial_println!("[failed]\n");
        serial_println!("Error: {}\n", info);
        exit_qemu(QemuExitCode::Failed);
    }
    hlt_loop();
}

// x86_64 globals
#[cfg(target_arch = "x86_64")]
use spin::Mutex;
#[cfg(target_arch = "x86_64")]
use lazy_static::lazy_static;

#[cfg(target_arch = "x86_64")]
lazy_static! {
    pub static ref PROCESS_MANAGER: Mutex<process::ProcessManager> =
        Mutex::new(process::ProcessManager::new());
}

#[cfg(target_arch = "x86_64")]
use memory::BootInfoFrameAllocator;

#[cfg(target_arch = "x86_64")]
pub static PHYS_MEM_OFFSET: spin::Mutex<u64> = spin::Mutex::new(0);

#[cfg(target_arch = "x86_64")]
lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<Option<BootInfoFrameAllocator>> =
        Mutex::new(None);
}

// x86_64 test entry point
#[cfg(all(test, target_arch = "x86_64"))]
use bootloader::{BootInfo, entry_point};

#[cfg(all(test, target_arch = "x86_64"))]
entry_point!(test_kernel_main);

#[cfg(all(test, target_arch = "x86_64"))]
fn test_kernel_main(_boot_info: &'static BootInfo) -> ! {
    init();
    test_main();
    hlt_loop();
}

#[cfg(all(test, target_arch = "x86_64"))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
