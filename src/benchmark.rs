pub struct Benchmark {
    pub name: &'static str,
    pub iterations: u64,
    pub total_cycles: u64,
}

impl Benchmark {
    pub fn new(name: &'static str) -> Self {
        Benchmark { name, iterations: 0, total_cycles: 0 }
    }

    pub fn run(&mut self, iterations: u64, f: fn()) {
        self.iterations = iterations;
        let start = read_tsc();
        for _ in 0..iterations {
            f();
        }
        let end = read_tsc();
        self.total_cycles = end.wrapping_sub(start);
    }

    pub fn report(&self) {
        let avg = if self.iterations > 0 {
            self.total_cycles / self.iterations
        } else { 0 };
        crate::println!("[bench] {}: {} iterations, {} total cycles, {} avg cycles/op",
            self.name, self.iterations, self.total_cycles, avg);
    }
}

/// x86_64: read RDTSC hardware cycle counter
#[cfg(target_arch = "x86_64")]
pub fn read_tsc() -> u64 {
    let lo: u32;
    let hi: u32;
    unsafe {
        core::arch::asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
        );
    }
    ((hi as u64) << 32) | (lo as u64)
}

/// ARM64: read CNTVCT_EL0 virtual counter register
/// Equivalent to RDTSC on x86 - counts CPU cycles
#[cfg(target_arch = "aarch64")]
pub fn read_tsc() -> u64 {
    let val: u64;
    unsafe {
        core::arch::asm!(
            "mrs {}, cntvct_el0",
            out(reg) val,
        );
    }
    val
}
