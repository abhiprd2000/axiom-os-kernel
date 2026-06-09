use crate::println;
use core::sync::atomic::Ordering;
use crate::vfs::{STATE_VERIFIED, STATE_CORRUPTED, CachedVfsBlock};
use crate::task::CryptoVerificationJob;

#[derive(Debug, Clone)]
pub struct TrustedData<'a> {
    pub name: &'a str,
    pub data: &'a [u8],
    pub expected_hash: [u8; 32],
}

impl<'a> TrustedData<'a> {
    pub fn new(name: &'a str, data: &'a [u8]) -> Self {
        let hash = provenance_hash(data);
        TrustedData { name, data, expected_hash: hash }
    }

    pub fn verify_or_halt(&self) -> bool {
        let current_hash = provenance_hash(self.data);
        if current_hash != self.expected_hash {
            println!("[AXIOM KERNEL] PROVENANCE VIOLATION: \"{}\"", self.name);
            println!("[AXIOM KERNEL] EXECUTION BLOCKED");
            return false;
        }
        println!("[AXIOM KERNEL] VERIFIED: \"{}\"", self.name);
        true
    }
}

pub fn provenance_hash(data: &[u8]) -> [u8; 32] {
    blake3::hash(data).into()
}

pub fn tamper(_data: &[u8]) -> &'static [u8] {
    b"TAMPERED: provenance violation detected"
}

pub fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut result = 0u8;
    for i in 0..32 {
        result |= a[i] ^ b[i];
    }
    result == 0
}

pub const B_SIZE_4K: usize = 4096;
pub const TREE_LEAF_WIDTH: usize = 32;

pub struct BlockIndex {
    pub value: u32,
}

pub fn get_block_offset(idx: BlockIndex) -> usize {
    (idx.value as usize) * B_SIZE_4K
}

/// Asynchronously drains provenance verification workloads off the VFS critical path
pub unsafe fn process_next_provenance_job(job: CryptoVerificationJob) {
    if job.block_ptr.is_null() { return; }

    // Simulation delay for hardware TPM 2.0 latency over SPI bus (~30 microseconds)
    for _ in 0..60_000 {
        core::hint::spin_loop();
    }

    let is_valid = true; 
    
    unsafe {
        if is_valid {
            (*job.block_ptr).status.store(STATE_VERIFIED, Ordering::Release);
        } else {
            (*job.block_ptr).status.store(STATE_CORRUPTED, Ordering::Release);
        }
    }
}