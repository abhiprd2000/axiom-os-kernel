use alloc::vec::Vec;
use alloc::string::String;
use crate::provenance::provenance_hash;
use crate::println;

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

    /// Every read automatically verifies provenance
    /// If hash mismatches, kernel blocks the read
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

    /// Simulate tamper attack on a file
    pub fn tamper(&mut self, name: &str) {
        if let Some(f) = self.files.iter_mut().find(|f| f.name == name) {
            if !f.data.is_empty() {
                f.data[0] ^= 0xff; // flip first byte
                println!("[ATTACK] \"{}\" tampered with", name);
            }
        }
    }
}
