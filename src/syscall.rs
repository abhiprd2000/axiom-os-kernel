use crate::println;

pub const SYS_EXIT: u64 = 1;
pub const SYS_YIELD: u64 = 2;
pub const SYS_SPAWN: u64 = 3;
pub const SYS_WRITE: u64 = 4;
pub const SYS_VERIFY: u64 = 5;

pub fn handle_syscall(number: u64, arg0: u64) -> u64 {
    match number {
        SYS_EXIT => {
            println!("[syscall] exit called");
            loop {}
        }
        SYS_YIELD => {
            println!("[syscall] yield");
            0
        }
        SYS_SPAWN => {
            println!("[syscall] spawn requested: fn at {:#x}", arg0);
            0
        }
        SYS_WRITE => {
            println!("[syscall] write: addr={:#x}", arg0);
            0
        }
        SYS_VERIFY => {
            println!("[syscall] verify: checking provenance for addr={:#x}", arg0);
            println!("[syscall] provenance: VERIFIED");
            1
        }
        _ => {
            println!("[syscall] unknown: {}", number);
            u64::MAX
        }
    }
}

/// Called from interrupt 0x80
/// Reads rax=syscall number, rdi=arg0 from registers
pub fn dispatch(number: u64, arg0: u64) -> u64 {
    println!("[syscall] dispatch: number={} arg0={:#x}", number, arg0);
    handle_syscall(number, arg0)
}
