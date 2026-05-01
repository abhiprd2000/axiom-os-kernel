const UART_BASE: *mut u8 = 0x09000000 as *mut u8;
const UARTDR: usize = 0x000;
const UARTFR: usize = 0x018;
const UARTFR_TXFF: u8 = 1 << 5;

pub fn uart_putc(c: u8) {
    unsafe {
        while UART_BASE.add(UARTFR).read_volatile() & UARTFR_TXFF != 0 {}
        UART_BASE.add(UARTDR).write_volatile(c);
    }
}

pub fn uart_puts(s: &str) {
    for byte in s.bytes() {
        if byte == b'\n' { uart_putc(b'\r'); }
        uart_putc(byte);
    }
}

// Print u64 as hex directly - no local buffer needed
#[inline(never)]
pub fn uart_put_u64(n: u64) {
    let hex = b"0123456789abcdef";
    uart_putc(b'0');
    uart_putc(b'x');
    // Print all 16 hex digits from most significant
    for i in (0..16).rev() {
        let nibble = ((n >> (i * 4)) & 0xF) as usize;
        uart_putc(hex[nibble]);
    }
}
