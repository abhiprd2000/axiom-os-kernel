use alloc::vec::Vec;
use crate::ramdisk::{RamDisk, SECTOR_SIZE, TOTAL_SECTORS};
use crate::println;

// FAT32 constants
#[allow(dead_code)]
const BYTES_PER_SECTOR: u16 = 512;
const SECTORS_PER_CLUSTER: u8 = 8; // 4KB clusters
const RESERVED_SECTORS: u16 = 32;
const NUM_FATS: u8 = 2;
const FAT_SIZE_SECTORS: u32 = 32;
const ROOT_CLUSTER: u32 = 2;
const FAT_EOC: u32 = 0x0FFFFFFF;
#[allow(dead_code)]
const FAT_FREE: u32 = 0x00000000;

pub struct Fat32 {
    disk: RamDisk,
}

impl Fat32 {
    pub fn new() -> Self {
        let mut fs = Fat32 {
            disk: RamDisk::new(),
        };
        fs.format();
        fs
    }

    fn format(&mut self) {
        // Write Boot Sector
        let mut boot = [0u8; SECTOR_SIZE];
        // Jump boot
        boot[0] = 0xEB; boot[1] = 0x58; boot[2] = 0x90;
        // OEM Name
        boot[3..11].copy_from_slice(b"AXIOM OS");
        // Bytes per sector
        boot[11] = 0x00; boot[12] = 0x02;
        // Sectors per cluster
        boot[13] = SECTORS_PER_CLUSTER;
        // Reserved sectors
        boot[14] = (RESERVED_SECTORS & 0xFF) as u8;
        boot[15] = (RESERVED_SECTORS >> 8) as u8;
        // Number of FATs
        boot[16] = NUM_FATS;
        // Total sectors
        let total = TOTAL_SECTORS as u32;
        boot[32] = (total & 0xFF) as u8;
        boot[33] = ((total >> 8) & 0xFF) as u8;
        boot[34] = ((total >> 16) & 0xFF) as u8;
        boot[35] = ((total >> 24) & 0xFF) as u8;
        // FAT size in sectors
        boot[36] = (FAT_SIZE_SECTORS & 0xFF) as u8;
        boot[37] = ((FAT_SIZE_SECTORS >> 8) & 0xFF) as u8;
        boot[38] = ((FAT_SIZE_SECTORS >> 16) & 0xFF) as u8;
        boot[39] = ((FAT_SIZE_SECTORS >> 24) & 0xFF) as u8;
        // Root cluster
        boot[44] = ROOT_CLUSTER as u8;
        // Signature
        boot[510] = 0x55; boot[511] = 0xAA;
        self.disk.write_sector(0, &boot);

        // Initialize FAT tables
        let fat_start = RESERVED_SECTORS as usize;
        let mut fat_sector = [0u8; SECTOR_SIZE];
        // First two entries reserved
        fat_sector[0..4].copy_from_slice(&0x0FFFFFF8u32.to_le_bytes());
        fat_sector[4..8].copy_from_slice(&FAT_EOC.to_le_bytes());
        // Root directory cluster
        fat_sector[8..12].copy_from_slice(&FAT_EOC.to_le_bytes());
        self.disk.write_sector(fat_start, &fat_sector);
        self.disk.write_sector(fat_start + FAT_SIZE_SECTORS as usize, &fat_sector);

        println!("[fat32] Formatted 4MB RAM disk with FAT32");
    }

    fn data_start_sector(&self) -> usize {
        RESERVED_SECTORS as usize + (NUM_FATS as usize * FAT_SIZE_SECTORS as usize)
    }

    fn cluster_to_sector(&self, cluster: u32) -> usize {
        self.data_start_sector() + (cluster as usize - 2) * SECTORS_PER_CLUSTER as usize
    }

    pub fn write_file(&mut self, filename: &str, data: &[u8]) {
        // Find free cluster (start at 3)
        let fat_start = RESERVED_SECTORS as usize;
        let mut fat_sector = [0u8; SECTOR_SIZE];
        self.disk.read_sector(fat_start, &mut fat_sector);

        // Use cluster 3 for simplicity (single file demo)
        let file_cluster: u32 = 3;
        let eoc = FAT_EOC.to_le_bytes();
        fat_sector[12..16].copy_from_slice(&eoc);
        self.disk.write_sector(fat_start, &fat_sector);

        // Write data to cluster
        let data_sector = self.cluster_to_sector(file_cluster);
        let mut buf = [0u8; SECTOR_SIZE];
        let write_len = data.len().min(SECTOR_SIZE);
        buf[..write_len].copy_from_slice(&data[..write_len]);
        self.disk.write_sector(data_sector, &buf);

        // Write directory entry in root cluster
        let root_sector = self.cluster_to_sector(ROOT_CLUSTER);
        let mut dir_sector = [0u8; SECTOR_SIZE];
        self.disk.read_sector(root_sector, &mut dir_sector);

        // FAT32 directory entry (32 bytes)
        let mut entry = [0u8; 32];
        // Filename (8.3 format) - split on dot
        let (base, ext) = match filename.find('.') {
            Some(pos) => (&filename[..pos], &filename[pos+1..]),
            None      => (filename, ""),
        };
        let name_bytes = base.as_bytes();
        let copy_len = name_bytes.len().min(8);
        entry[..copy_len].copy_from_slice(&name_bytes[..copy_len]);
        for i in copy_len..8 { entry[i] = 0x20; }
        // Extension
        let ext_bytes = ext.as_bytes();
        let ext_len = ext_bytes.len().min(3);
        entry[8..8+ext_len].copy_from_slice(&ext_bytes[..ext_len]);
        for i in 8+ext_len..11 { entry[i] = 0x20; }
        // Attributes: archive
        entry[11] = 0x20;
        // First cluster
        entry[26] = (file_cluster & 0xFF) as u8;
        entry[27] = ((file_cluster >> 8) & 0xFF) as u8;
        entry[20] = ((file_cluster >> 16) & 0xFF) as u8;
        entry[21] = ((file_cluster >> 24) & 0xFF) as u8;
        // File size
        let size = data.len() as u32;
        entry[28] = (size & 0xFF) as u8;
        entry[29] = ((size >> 8) & 0xFF) as u8;
        entry[30] = ((size >> 16) & 0xFF) as u8;
        entry[31] = ((size >> 24) & 0xFF) as u8;

        // Store first 8 bytes of BLAKE3 hash in reserved FAT32 bytes [12..20]
        let hash = blake3::hash(data);
        entry[12..20].copy_from_slice(&hash.as_bytes()[..8]);
        dir_sector[..32].copy_from_slice(&entry);
        self.disk.write_sector(root_sector, &dir_sector);

        println!("[fat32] Written: {} ({} bytes) at cluster {}", filename, data.len(), file_cluster);
    }

    pub fn read_file(&mut self, filename: &str) -> Option<Vec<u8>> {
        let root_sector = self.cluster_to_sector(ROOT_CLUSTER);
        let mut dir_sector = [0u8; SECTOR_SIZE];
        self.disk.read_sector(root_sector, &mut dir_sector);

        // Search directory entries
        for i in 0..16 {
            let entry = &dir_sector[i*32..(i+1)*32];
            if entry[0] == 0 { break; }
            let stored_name = core::str::from_utf8(&entry[..8])
                .unwrap_or("").trim();
            // Strip extension from search name for 8.3 comparison
            let search_name = filename.trim().splitn(2, '.').next().unwrap_or(filename.trim());
            if stored_name.eq_ignore_ascii_case(search_name) {
                let cluster = u32::from_le_bytes([entry[26], entry[27], entry[20], entry[21]]);
                let size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]) as usize;
                let data_sector = self.cluster_to_sector(cluster);
                let mut buf = [0u8; SECTOR_SIZE];
                self.disk.read_sector(data_sector, &mut buf);
                let data = buf[..size].to_vec();
                // Auto-verify provenance on every read
                let current_hash = blake3::hash(&data);
                let stored: [u8; 8] = entry[12..20].try_into().ok()?;
                let computed: [u8; 8] = current_hash.as_bytes()[..8].try_into().ok()?;
                if stored != computed {
                    println!("[AXIOM KERNEL] READ BLOCKED: \"{}\" FAT32 provenance violation", filename);
                    return None;
                }
                return Some(data);
            }
        }
        None
    }

    pub fn list_files(&mut self) {
        let root_sector = self.cluster_to_sector(ROOT_CLUSTER);
        let mut dir_sector = [0u8; SECTOR_SIZE];
        self.disk.read_sector(root_sector, &mut dir_sector);
        let mut found = false;
        for i in 0..16 {
            let entry = &dir_sector[i*32..(i+1)*32];
            if entry[0] == 0 { break; }
            if entry[0] == 0xE5 { continue; }
            let name = core::str::from_utf8(&entry[..8]).unwrap_or("?").trim();
            let size = u32::from_le_bytes([entry[28], entry[29], entry[30], entry[31]]);
            println!("  {} ({} bytes) [FAT32]", name, size);
            found = true;
        }
        if !found {
            println!("  (no files on disk)");
        }
    }
}

impl Fat32 {
    pub fn verify_file(&mut self, filename: &str) -> Option<bool> {
        // Read file data
        let data = self.read_file(filename)?;
        // Recompute hash
        let current_hash = blake3::hash(&data);
        // Read stored hash from directory entry reserved bytes
        let root_sector = self.cluster_to_sector(ROOT_CLUSTER);
        let mut dir_sector = [0u8; SECTOR_SIZE];
        self.disk.read_sector(root_sector, &mut dir_sector);
        for i in 0..16 {
            let entry = &dir_sector[i*32..(i+1)*32];
            if entry[0] == 0 { break; }
            let stored_name = core::str::from_utf8(&entry[..8])
                .unwrap_or("").trim();
            // Strip extension from search name for 8.3 comparison
            let search_name = filename.trim().splitn(2, '.').next().unwrap_or(filename.trim());
            if stored_name.eq_ignore_ascii_case(search_name) {
                // Stored hash is in bytes 12-19 (first 8 bytes of BLAKE3)
                let stored: [u8; 8] = entry[12..20].try_into().ok()?;
                let current: [u8; 8] = current_hash.as_bytes()[..8].try_into().ok()?;
                return Some(stored == current);
            }
        }
        None
    }

}
impl Fat32 {
    pub fn tamper_file(&mut self, filename: &str) -> bool {
        let root_sector = self.cluster_to_sector(ROOT_CLUSTER);
        let mut dir_sector = [0u8; SECTOR_SIZE];
        self.disk.read_sector(root_sector, &mut dir_sector);
        for i in 0..16 {
            let entry = &dir_sector[i*32..(i+1)*32];
            if entry[0] == 0 { break; }
            let stored_name = core::str::from_utf8(&entry[..8]).unwrap_or("").trim();
            let search_name = filename.trim().splitn(2, '.').next().unwrap_or(filename.trim());
            if stored_name.eq_ignore_ascii_case(search_name) {
                let cluster = u32::from_le_bytes([entry[26], entry[27], entry[20], entry[21]]);
                let data_sector = self.cluster_to_sector(cluster);
                let mut buf = [0u8; SECTOR_SIZE];
                self.disk.read_sector(data_sector, &mut buf);
                buf[0] ^= 0xFF; // flip first byte
                self.disk.write_sector(data_sector, &buf);
                crate::println!("[attack] FAT32 file '{}' tampered - byte 0 flipped", filename);
                return true;
            }
        }
        false
    }
}
