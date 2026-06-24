use crate::println;
use crate::provenance::provenance_hash;
use alloc::string::String;
use alloc::vec::Vec;

/// Provenance block granularity (standard 4 KiB, matching fs-verity/dm-verity).
pub const BLOCK_SIZE: usize = 4096;

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
    /// Whole-file BLAKE3 (baseline path; goes stale after a partial write_block).
    pub provenance_hash: [u8; 32],
    /// Per-BLOCK_SIZE BLAKE3 leaves; a range read verifies only touched blocks.
    pub block_hashes: Vec<[u8; 32]>,
    /// BLAKE3 Merkle root over the leaves, recomputed lazily when dirty.
    merkle_root: [u8; 32],
    root_dirty: bool,
}

/// One BLAKE3 hash per BLOCK_SIZE chunk (last chunk may be short).
fn compute_block_hashes(data: &[u8]) -> Vec<[u8; 32]> {
    data.chunks(BLOCK_SIZE)
        .map(|c| provenance_hash(c))
        .collect()
}

/// BLAKE3 Merkle root over the leaves: pairs hashed bottom-up, a lone node is
/// promoted unchanged. Empty input -> zero root.
fn compute_merkle_root(leaves: &[[u8; 32]]) -> [u8; 32] {
    if leaves.is_empty() {
        return [0u8; 32];
    }
    let mut level: Vec<[u8; 32]> = leaves.to_vec();
    while level.len() > 1 {
        let mut next: Vec<[u8; 32]> = Vec::with_capacity((level.len() + 1) / 2);
        let mut i = 0;
        while i < level.len() {
            if i + 1 < level.len() {
                let mut buf = [0u8; 64];
                buf[..32].copy_from_slice(&level[i]);
                buf[32..].copy_from_slice(&level[i + 1]);
                next.push(provenance_hash(&buf));
            } else {
                next.push(level[i]);
            }
            i += 2;
        }
        level = next;
    }
    level[0]
}

impl FileNode {
    pub fn new_file(name: &str, data: &[u8]) -> Self {
        let hash = provenance_hash(data);
        let block_hashes = compute_block_hashes(data);
        let merkle_root = compute_merkle_root(&block_hashes);
        FileNode {
            name: String::from(name),
            file_type: FileType::Regular,
            data: Vec::from(data),
            provenance_hash: hash,
            block_hashes,
            merkle_root,
            root_dirty: false,
        }
    }

    pub fn new_file_untrusted(name: &str, data: &[u8]) -> Self {
        let nblocks = (data.len() + BLOCK_SIZE - 1) / BLOCK_SIZE;
        FileNode {
            name: String::from(name),
            file_type: FileType::Regular,
            data: Vec::from(data),
            provenance_hash: [0u8; 32], // deliberately invalid — always fails verify()
            block_hashes: alloc::vec![[0u8; 32]; nblocks], // also fails verify_range()
            merkle_root: [0u8; 32],
            root_dirty: false,
        }
    }

    pub fn new_dir(name: &str) -> Self {
        FileNode {
            name: String::from(name),
            file_type: FileType::Directory,
            data: Vec::new(),
            provenance_hash: [0u8; 32],
            block_hashes: Vec::new(),
            merkle_root: [0u8; 32],
            root_dirty: false,
        }
    }

    pub fn verify(&self) -> bool {
        crate::provenance::constant_time_eq(&provenance_hash(&self.data), &self.provenance_hash)
    }

    /// Block-level: verify only the leaves overlapping [offset, offset+len).
    pub fn verify_range(&self, offset: usize, len: usize) -> bool {
        if len == 0 {
            return true;
        }
        let end = core::cmp::min(offset + len, self.data.len());
        if offset >= end {
            return false;
        }
        let start_block = offset / BLOCK_SIZE;
        let end_block = (end - 1) / BLOCK_SIZE;
        for b in start_block..=end_block {
            let bstart = b * BLOCK_SIZE;
            let bend = core::cmp::min(bstart + BLOCK_SIZE, self.data.len());
            let live = provenance_hash(&self.data[bstart..bend]);
            match self.block_hashes.get(b) {
                Some(stored) if crate::provenance::constant_time_eq(&live, stored) => {}
                _ => return false,
            }
        }
        true
    }

    /// Lazily return the Merkle root, recomputing from leaves only if dirty.
    pub fn merkle_root(&mut self) -> [u8; 32] {
        if self.root_dirty {
            self.merkle_root = compute_merkle_root(&self.block_hashes);
            self.root_dirty = false;
        }
        self.merkle_root
    }

    /// Overwrite one block: updates that leaf and marks the root dirty, so a
    /// single-block write costs one block hash instead of a full-file rehash.
    /// The whole-file `provenance_hash` (baseline only) is left stale.
    pub fn write_block(&mut self, idx: usize, new_data: &[u8]) -> bool {
        let bstart = idx * BLOCK_SIZE;
        if bstart >= self.data.len() || idx >= self.block_hashes.len() {
            return false;
        }
        let bend = core::cmp::min(bstart + BLOCK_SIZE, self.data.len());
        let n = core::cmp::min(new_data.len(), bend - bstart);
        self.data[bstart..bstart + n].copy_from_slice(&new_data[..n]);
        self.block_hashes[idx] = provenance_hash(&self.data[bstart..bend]);
        self.root_dirty = true;
        true
    }
}

pub struct VirtualFS {
    pub files: Vec<FileNode>,
}

impl VirtualFS {
    pub fn new() -> Self {
        VirtualFS { files: Vec::new() }
    }

    /// Store a provenance-locked file. Hash computed and bound at write time.
    pub fn create(&mut self, name: &str, data: &[u8]) {
        self.files.push(FileNode::new_file(name, data));
    }

    /// Store an untrusted file with no valid provenance record. The kernel
    /// blocks all reads — the file exists in the VFS but is unreadable through
    /// the enforced path. Demonstrates the enforcement boundary.
    pub fn create_untrusted(&mut self, name: &str, data: &[u8]) {
        self.files.push(FileNode::new_file_untrusted(name, data));
    }

    pub fn read(&self, name: &str) -> Option<&[u8]> {
        let file = self.files.iter().find(|f| f.name == name)?;
        if !file.verify() {
            println!(
                "[AXIOM KERNEL] READ BLOCKED: \"{}\" provenance violation",
                name
            );
            return None;
        }
        Some(file.data.as_slice())
    }

    /// Block-level verified ranged read: verifies only the blocks overlapping
    /// the range. Cost scales with bytes read, not total file size.
    pub fn read_range(&self, name: &str, offset: usize, len: usize) -> Option<&[u8]> {
        let file = self.files.iter().find(|f| f.name == name)?;
        let end = core::cmp::min(offset + len, file.data.len());
        if offset >= end {
            return None;
        }
        if !file.verify_range(offset, len) {
            println!(
                "[AXIOM KERNEL] READ BLOCKED: \"{}\" block {} provenance violation",
                name,
                offset / BLOCK_SIZE
            );
            return None;
        }
        Some(&file.data[offset..end])
    }

    /// Overwrite one block of a file (exercises the lazy Merkle path).
    pub fn write_block(&mut self, name: &str, idx: usize, data: &[u8]) -> bool {
        match self.files.iter_mut().find(|f| f.name == name) {
            Some(f) => f.write_block(idx, data),
            None => false,
        }
    }

    /// Current Merkle root for a file (recomputed lazily if dirty).
    pub fn merkle_root(&mut self, name: &str) -> Option<[u8; 32]> {
        self.files
            .iter_mut()
            .find(|f| f.name == name)
            .map(|f| f.merkle_root())
    }

    pub fn verify(&self, name: &str) -> Option<bool> {
        self.files
            .iter()
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
