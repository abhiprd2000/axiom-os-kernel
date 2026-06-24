use crate::println;
use alloc::vec;
use alloc::vec::Vec;

pub const DISK_SIZE: usize = 4 * 1024 * 1024; // 4MB
pub const SECTOR_SIZE: usize = 512;
pub const TOTAL_SECTORS: usize = DISK_SIZE / SECTOR_SIZE;

pub struct RamDisk {
    data: Vec<u8>,
}

impl RamDisk {
    pub fn new() -> Self {
        println!("[ramdisk] Initializing 4MB RAM disk");
        RamDisk {
            data: vec![0u8; DISK_SIZE],
        }
    }

    pub fn read_sector(&self, sector: usize, buf: &mut [u8]) {
        let start = sector * SECTOR_SIZE;
        buf.copy_from_slice(&self.data[start..start + SECTOR_SIZE]);
    }

    pub fn write_sector(&mut self, sector: usize, buf: &[u8]) {
        let start = sector * SECTOR_SIZE;
        self.data[start..start + SECTOR_SIZE].copy_from_slice(buf);
    }

    pub fn size_mb(&self) -> usize {
        DISK_SIZE / (1024 * 1024)
    }
}
