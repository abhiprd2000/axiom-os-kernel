// src/pmc.rs
// Hardware Performance Monitoring Counter (PMC) support
// Replaces RDTSC-only benchmarking with cache/memory visibility
// Implemented following profiling guidance from Prof. Subodh Kumar, IIT Delhi

#[cfg(target_arch = "x86_64")]
pub use x86_64_impl::*;

#[cfg(target_arch = "x86_64")]
mod x86_64_impl {
    // x86_64 MSR addresses for performance monitoring
    const IA32_PERFEVTSEL0: u32 = 0x186;
    const IA32_PMC0: u32        = 0xC1;
    const IA32_PERF_GLOBAL_CTRL: u32 = 0x38F;

    const LLC_MISS_EVENT: u64 = 0x412E;
    const PMC_ENABLE: u64 = (1 << 22) | (1 << 17) | (1 << 16);

    unsafe fn wrmsr(msr: u32, value: u64) {
        let lo = value as u32;
        let hi = (value >> 32) as u32;
        core::arch::asm!("wrmsr", in("ecx") msr, in("eax") lo, in("edx") hi);
    }

    unsafe fn rdmsr(msr: u32) -> u64 {
        let lo: u32;
        let hi: u32;
        core::arch::asm!("rdmsr", in("ecx") msr, out("eax") lo, out("edx") hi);
        ((hi as u64) << 32) | lo as u64
    }

    unsafe fn rdpmc(counter: u32) -> u64 {
        let lo: u32;
        let hi: u32;
        core::arch::asm!("rdpmc", in("ecx") counter, out("eax") lo, out("edx") hi);
        ((hi as u64) << 32) | lo as u64
    }

    pub fn init_cache_miss_counter() {
        unsafe {
            let current_ctrl = rdmsr(IA32_PERF_GLOBAL_CTRL);
            wrmsr(IA32_PERFEVTSEL0, LLC_MISS_EVENT | PMC_ENABLE);
            wrmsr(IA32_PMC0, 0);
            wrmsr(IA32_PERF_GLOBAL_CTRL, current_ctrl | 1);
        }
    }

    pub fn read_cache_misses() -> u64 {
        unsafe { rdpmc(0) }
    }

    pub fn measure<F, R>(f: F) -> (R, u64, u64)
    where
        F: FnOnce() -> R,
    {
        unsafe { core::arch::asm!("lfence", options(nomem, nostack, preserves_flags)); }
        let cycles_start = crate::benchmark::read_tsc();
        let misses_start = read_cache_misses();

        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
        let result = f();
        core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

        unsafe { core::arch::asm!("lfence", options(nomem, nostack, preserves_flags)); }
        let misses_end = read_cache_misses();
        let cycles_end = crate::benchmark::read_tsc();

        (result, misses_end.wrapping_sub(misses_start), cycles_end.wrapping_sub(cycles_start))
    }
}

// Fallback implementation for running unit tests on host machines
#[cfg(not(target_arch = "x86_64"))]
pub fn init_cache_miss_counter() {}

#[cfg(not(target_arch = "x86_64"))]
pub fn measure<F, R>(f: F) -> (R, u64, u64) 
where F: FnOnce() -> R 
{
    (f(), 0, 0)
}