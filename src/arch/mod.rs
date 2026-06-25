// Architecture module
// Selects the correct arch at compile time

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "x86_64")]
// x86_64 is the primary architecture, configured at the crate root
pub mod x86_64 {}
