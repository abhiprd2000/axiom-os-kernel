use alloc::vec::Vec;
use alloc::string::String;
use crate::provenance::provenance_hash;
use crate::println;
use crate::task::CryptoVerificationJob;

#[derive(Debug, Clone)]
pub enum FileType {
    Regular,
    Directory,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub file_type: FileType,
    pub data: Vec<u8>,
    pub provenance_hash: [u8; 32],
}

impl FileNode {
    pub fn new_file(name: &str, data: &[u8]) -> Self {
        let hash = provenance_hash(data);
        FileNode {
            name: String::from(name),
            file_type: FileType::Regular,
            data: Vec::from(data),
            provenance_hash: hash,
        }
    }

    pub fn new_dir(name: &str) -> Self {
        FileNode {
            name: String::from(name),
            file_type: FileType::Directory,
            data: Vec::new(),
            provenance_hash: [0u8; 32],
        }
    }

    pub fn verify(&self) -> bool {
        crate::provenance::constant_time_eq(
            &provenance_hash(&self.data),
            &self.provenance_hash
        )
    }
}

pub struct VirtualFS {
    pub files: Vec<FileNode>,
}

impl VirtualFS {
    pub fn new() -> Self {
        VirtualFS { files: Vec::new() }
    }

    pub fn create(&mut self, name: &str, data: &[u8]) {
        self.files.push(FileNode::new_file(name, data));
    }

    pub fn read(&self, name: &str) -> Option<&[u8]> {
        let file = self.files.iter().find(|f| f.name == name)?;
        if !file.verify() {
            println!("[AXIOM KERNEL] READ BLOCKED: \"{}\" provenance violation", name);
            return None;
        }
        Some(file.data.as_slice())
    }

    pub fn verify(&self, name: &str) -> Option<bool> {
        self.files.iter()
            .find(|f| f.name == name)
            .map(|f| f.verify())
    }

    pub fn list(&self) {
        for f in &self.files {
            let status = if f.verify() { "OK" } else { "TAMPERED" };
            println!("  [{:?}] {} [{}]", f.file_type, f.name, status);
        }
    }

    pub fn tamper(&mut self, name: &str) {
        if let Some(f) = self.files.iter_mut().find(|f| f.name == name) {
            if !f.data.is_empty() {
                f.data[0] ^= 0xff;
                println!("[ATTACK] \"{}\" tampered with", name);
            }
        }
    }
}

// Atomic State Machine Definitions for Asynchronous Verification
pub const STATE_PENDING: u8 = 0;
pub const STATE_VERIFIED: u8 = 1;
pub const STATE_CORRUPTED: u8 = 2;

#[derive(Clone)]
pub struct CachedVfsBlock {
    pub block_id: u64,
    pub data: [u8; 4096],
    pub lineage_token: u64,
    pub status: core::sync::atomic::AtomicU8, 
}

pub fn read_block_async(block: &mut CachedVfsBlock) {
    block.status.store(STATE_PENDING, core::sync::atomic::Ordering::Relaxed);

    let job = CryptoVerificationJob {
        block_ptr: block as *mut CachedVfsBlock,
        expected_hash: [0u8; 32],
    };

    if let Some(queue) = crate::task::VERIFICATION_QUEUE.get() {
        let _ = queue.push(job);
    }
    
    crate::task::yield_current_thread(); 
}