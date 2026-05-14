use crate::println;

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
    b"TAMPERED: Journalist report from Jharkhand"
}

/// Constant-time byte comparison — never short-circuits
/// Prevents timing side-channel attacks on hash comparison
pub fn constant_time_eq(a: &[u8; 32], b: &[u8; 32]) -> bool {
    let mut result = 0u8;
    for i in 0..32 {
        result |= a[i] ^ b[i];
    }
    result == 0
}
