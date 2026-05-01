// Architecture-agnostic print macro
// Routes to UART on ARM, VGA on x86

#[cfg(target_arch = "aarch64")]
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::print::_aarch64_print(format_args!($($arg)*))
    };
}

#[cfg(target_arch = "aarch64")]
#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => {
        $crate::print::_aarch64_print(format_args!("{}\n", format_args!($($arg)*)))
    };
}

#[cfg(target_arch = "aarch64")]
pub fn _aarch64_print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    UartWriter.write_fmt(args).ok();
}

#[cfg(target_arch = "aarch64")]
struct UartWriter;

#[cfg(target_arch = "aarch64")]
impl core::fmt::Write for UartWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        for byte in s.bytes() {
            if byte == b'\n' {
                uart_putc(b'\r');
            }
            uart_putc(byte);
        }
        Ok(())
    }
}

#[cfg(target_arch = "aarch64")]
fn uart_putc(c: u8) {
    const UART_BASE: *mut u8 = 0x09000000 as *mut u8;
    const UARTFR: usize = 0x018;
    const UARTDR: usize = 0x000;
    const TXFF: u8 = 1 << 5;
    unsafe {
        while UART_BASE.add(UARTFR).read_volatile() & TXFF != 0 {}
        UART_BASE.add(UARTDR).write_volatile(c);
    }
}

// x86 versions are already defined in vga_buffer.rs
// These are just re-exports to satisfy the compiler on x86
#[cfg(target_arch = "x86_64")]
pub use crate::{serial_print as _, serial_println as _};
