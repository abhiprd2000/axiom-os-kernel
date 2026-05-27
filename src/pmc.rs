cat > src/pmc.rs << 'EOF'
// Hardware Performance Monitoring Counter (PMC) support
// Replaces RDTSC-only benchmarking with cache/memory visibility
// Implemented following profiling guidance from Prof. Subodh Kumar, IIT Delhi

use x86_64::instructions::port::Port;

// x86_64 MSR addresses for performance monitoring
const IA32_PERFEVTSEL0: u32 = 0x186;  // Event selector register
const IA32_PMC0: u32        = 0xC1;   // Counter register
const IA32_PERF_GLOBAL_CTRL: u32 = 0x38F;

// Event codes (Intel Vol. 3B Table 19-1)
// LLC (Last Level Cache) misses — best proxy for memory pressure
const LLC_MISS_EVENT: u64 = 0x412E; // event=0x2E, umask=0x41

// Enable: OS + User mode, enable bit
const PMC_ENABLE: u64 = (1 << 22) | (1 << 17) | (1 << 16);

/// Write to a Model-Specific Register
unsafe fn wrmsr(msr: u32, value: u64) {
    let lo = value as u32;
    let hi = (value >> 32) as u32;
    core::arch::asm!(
        "wrmsr",
        in("ecx") msr,
        in("eax") lo,
        in("edx") hi,
    );
}

/// Read from a Model-Specific Register
unsafe fn rdmsr(msr: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!(
        "rdmsr",
        in("ecx") msr,
        out("eax") lo,
        out("edx") hi,
    );
    ((hi as u64) << 32) | lo as u64
}

/// Read the hardware performance counter directly
/// More accurate than RDTSC for memory bottleneck analysis
unsafe fn rdpmc(counter: u32) -> u64 {
    let lo: u32;
    let hi: u32;
    core::arch::asm!(
        "rdpmc",
        in("ecx") counter,
        out("eax") lo,
        out("edx") hi,
    );
    ((hi as u64) << 32) | lo as u64
}

/// Initialize PMC0 to count LLC cache misses safely via Read-Modify-Write
pub fn init_cache_miss_counter() {
    unsafe {
        // 1. Read current global control bits to preserve existing configuration
        let current_ctrl = rdmsr(IA32_PERF_GLOBAL_CTRL);

        // 2. Program PMC0 safely without wiping out other hypervisor/CPU flags
        wrmsr(IA32_PERFEVTSEL0, LLC_MISS_EVENT | PMC_ENABLE);
        wrmsr(IA32_PMC0, 0);
        wrmsr(IA32_PERF_GLOBAL_CTRL, current_ctrl | 1); // Enable PMC0 (bit 0)
    }
}

/// Read current LLC miss count
pub fn read_cache_misses() -> u64 {
    unsafe { rdpmc(0) }
}

pub fn measure<F, R>(f: F) -> (R, u64, u64)
where
    F: FnOnce() -> R,
{
    // Serialize instructions before capturing start telemetry
    unsafe { core::arch::asm!("lfence", options(nomem, nostack, preserves_flags)); }

    let cycles_start = crate::benchmark::read_tsc();
    let misses_start = read_cache_misses();

    // Compiler barrier: prevent optimization reordering across this boundary
    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);

    let result = f();

    core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    // Serialize instructions before capturing end telemetry
    unsafe { core::arch::asm!("lfence", options(nomem, nostack, preserves_flags)); }

    let misses_end = read_cache_misses();
    let cycles_end = crate::benchmark::read_tsc();

    (
        result,
        misses_end.wrapping_sub(misses_start),
        cycles_end.wrapping_sub(cycles_start),
    )
}
EOF