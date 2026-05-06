use alloc::vec::Vec;
use crate::ramdisk::{RamDisk, SECTOR_SIZE, TOTAL_SECTORS};
use crate::println;

#[allow(dead_code)]
const BYTES_PER_SECTOR: u16 = 512;
const SPC: u8 = 8;
const RSVD: u16 = 32;
const NFATS: u8 = 2;
const FAT_SZ: u32 = 32;
const ROOT_CLU: u32 = 2;
const EOC: u32 = 0x0FFFFFFF;
#[allow(dead_code)]
const FREE: u32 = 0;
const HSTORE: usize = 1;

pub struct Fat32 { disk: RamDisk }

impl Fat32 {
    pub fn new() -> Self {
        let mut fs = Fat32 { disk: RamDisk::new() };
        fs.format();
        fs
    }

    fn format(&mut self) {
        let mut boot = [0u8; SECTOR_SIZE];
        boot[0] = 0xEB; boot[1] = 0x58; boot[2] = 0x90;
        boot[3..11].copy_from_slice(b"AXIOM OS");
        boot[11] = 0x00; boot[12] = 0x02;
        boot[13] = SPC;
        boot[14] = (RSVD & 0xFF) as u8;
        boot[15] = (RSVD >> 8) as u8;
        boot[16] = NFATS;
        let t = TOTAL_SECTORS as u32;
        boot[32] = (t & 0xFF) as u8;
        boot[33] = ((t >> 8) & 0xFF) as u8;
        boot[34] = ((t >> 16) & 0xFF) as u8;
        boot[35] = ((t >> 24) & 0xFF) as u8;
        boot[36] = (FAT_SZ & 0xFF) as u8;
        boot[37] = ((FAT_SZ >> 8) & 0xFF) as u8;
        boot[38] = ((FAT_SZ >> 16) & 0xFF) as u8;
        boot[39] = ((FAT_SZ >> 24) & 0xFF) as u8;
        boot[44] = ROOT_CLU as u8;
        boot[510] = 0x55; boot[511] = 0xAA;
        self.disk.write_sector(0, &boot);

        let fs = RSVD as usize;
        let mut fat = [0u8; SECTOR_SIZE];
        fat[0..4].copy_from_slice(&0x0FFFFFF8u32.to_le_bytes());
        fat[4..8].copy_from_slice(&EOC.to_le_bytes());
        fat[8..12].copy_from_slice(&EOC.to_le_bytes());
        self.disk.write_sector(fs, &fat);
        self.disk.write_sector(fs + FAT_SZ as usize, &fat);
        println!("[fat32] 4MB disk ready");
    }

    fn data_start(&self) -> usize {
        RSVD as usize + NFATS as usize * FAT_SZ as usize
    }

    fn clu2sec(&self, c: u32) -> usize {
        self.data_start() + (c as usize - 2) * SPC as usize
    }

    fn hstore_write(&mut self, name: &str, hash: &[u8; 32]) {
        let mut s = [0u8; SECTOR_SIZE];
        self.disk.read_sector(HSTORE, &mut s);
        let key = name.splitn(2, '.').next().unwrap_or(name).trim();
        for i in 0..12 {
            let off = i * 40;
            let n = core::str::from_utf8(&s[off..off+8]).unwrap_or("").trim();
            if s[off] == 0 || n.eq_ignore_ascii_case(key) {
                s[off..off+8].fill(0x20);
                let b = key.as_bytes();
                let bl = b.len().min(8);
                s[off..off+bl].copy_from_slice(&b[..bl]);
                s[off+8..off+40].copy_from_slice(hash);
                self.disk.write_sector(HSTORE, &s);
                return;
            }
        }
    }

    fn hstore_read(&mut self, name: &str) -> Option<[u8; 32]> {
        let mut s = [0u8; SECTOR_SIZE];
        self.disk.read_sector(HSTORE, &mut s);
        let key = name.splitn(2, '.').next().unwrap_or(name).trim();
        for i in 0..12 {
            let off = i * 40;
            if s[off] == 0 { break; }
            let n = core::str::from_utf8(&s[off..off+8]).unwrap_or("").trim();
            if n.eq_ignore_ascii_case(key) {
                let mut h = [0u8; 32];
                h.copy_from_slice(&s[off+8..off+40]);
                return Some(h);
            }
        }
        None
    }

    pub fn write_file(&mut self, name: &str, data: &[u8]) {
        let fs = RSVD as usize;
        let mut fat = [0u8; SECTOR_SIZE];
        self.disk.read_sector(fs, &mut fat);
        let clu: u32 = 3;
        fat[12..16].copy_from_slice(&EOC.to_le_bytes());
        self.disk.write_sector(fs, &fat);

        let mut buf = [0u8; SECTOR_SIZE];
        let wl = data.len().min(SECTOR_SIZE);
        buf[..wl].copy_from_slice(&data[..wl]);
        self.disk.write_sector(self.clu2sec(clu), &buf);

        let rs = self.clu2sec(ROOT_CLU);
        let mut dir = [0u8; SECTOR_SIZE];
        self.disk.read_sector(rs, &mut dir);

        let mut e = [0u8; 32];
        let (base, ext) = match name.find('.') {
            Some(p) => (&name[..p], &name[p+1..]),
            None => (name, ""),
        };
        let nb = base.as_bytes();
        let nl = nb.len().min(8);
        e[..nl].copy_from_slice(&nb[..nl]);
        for i in nl..8 { e[i] = 0x20; }
        let eb = ext.as_bytes();
        let el = eb.len().min(3);
        e[8..8+el].copy_from_slice(&eb[..el]);
        for i in 8+el..11 { e[i] = 0x20; }
        e[11] = 0x20;
        e[26] = (clu & 0xFF) as u8;
        e[27] = ((clu >> 8) & 0xFF) as u8;
        e[20] = ((clu >> 16) & 0xFF) as u8;
        e[21] = ((clu >> 24) & 0xFF) as u8;
        let sz = data.len() as u32;
        e[28] = (sz & 0xFF) as u8;
        e[29] = ((sz >> 8) & 0xFF) as u8;
        e[30] = ((sz >> 16) & 0xFF) as u8;
        e[31] = ((sz >> 24) & 0xFF) as u8;
        let hash = blake3::hash(data);
        let full: [u8; 32] = *hash.as_bytes();
        e[12..20].copy_from_slice(&full[..8]);
        dir[..32].copy_from_slice(&e);
        self.disk.write_sector(rs, &dir);
        self.hstore_write(name, &full);
        println!("[fat32] wrote: {} ({} bytes)", name, data.len());
    }

    pub fn read_file(&mut self, name: &str) -> Option<Vec<u8>> {
        let rs = self.clu2sec(ROOT_CLU);
        let mut dir = [0u8; SECTOR_SIZE];
        self.disk.read_sector(rs, &mut dir);
        let key = name.trim().splitn(2, '.').next().unwrap_or(name.trim());
        for i in 0..16 {
            let e = &dir[i*32..(i+1)*32];
            if e[0] == 0 { break; }
            let sn = core::str::from_utf8(&e[..8]).unwrap_or("").trim();
            if sn.eq_ignore_ascii_case(key) {
                let clu = u32::from_le_bytes([e[26], e[27], e[20], e[21]]);
                let sz = u32::from_le_bytes([e[28], e[29], e[30], e[31]]) as usize;
                let mut buf = [0u8; SECTOR_SIZE];
                self.disk.read_sector(self.clu2sec(clu), &mut buf);
                let data = buf[..sz].to_vec();
                let cur = blake3::hash(&data);
                match self.hstore_read(name) {
                    Some(stored) => {
                        if stored != *cur.as_bytes() {
                            println!("[AXIOM KERNEL] READ BLOCKED: {} provenance fail", name);
                            return None;
                        }
                    }
                    None => {
                        let s: [u8; 8] = e[12..20].try_into().ok()?;
                        if s != cur.as_bytes()[..8] {
                            println!("[AXIOM KERNEL] READ BLOCKED: {} provenance fail", name);
                            return None;
                        }
                    }
                }
                return Some(data);
            }
        }
        None
    }

    pub fn list_files(&mut self) {
        let rs = self.clu2sec(ROOT_CLU);
        let mut dir = [0u8; SECTOR_SIZE];
        self.disk.read_sector(rs, &mut dir);
        let mut found = false;
        for i in 0..16 {
            let e = &dir[i*32..(i+1)*32];
            if e[0] == 0 { break; }
            if e[0] == 0xE5 { continue; }
            let n = core::str::from_utf8(&e[..8]).unwrap_or("?").trim();
            let sz = u32::from_le_bytes([e[28], e[29], e[30], e[31]]);
            println!("  {} ({} bytes)", n, sz);
            found = true;
        }
        if !found { println!("  (empty)"); }
    }

    pub fn verify_file(&mut self, name: &str) -> Option<bool> {
        self.read_file(name)?;
        Some(true)
    }

    pub fn tamper_file(&mut self, name: &str) -> bool {
        let rs = self.clu2sec(ROOT_CLU);
        let mut dir = [0u8; SECTOR_SIZE];
        self.disk.read_sector(rs, &mut dir);
        let key = name.trim().splitn(2, '.').next().unwrap_or(name.trim());
        for i in 0..16 {
            let e = &dir[i*32..(i+1)*32];
            if e[0] == 0 { break; }
            let sn = core::str::from_utf8(&e[..8]).unwrap_or("").trim();
            if sn.eq_ignore_ascii_case(key) {
                let clu = u32::from_le_bytes([e[26], e[27], e[20], e[21]]);
                let mut buf = [0u8; SECTOR_SIZE];
                self.disk.read_sector(self.clu2sec(clu), &mut buf);
                buf[0] ^= 0xFF;
                self.disk.write_sector(self.clu2sec(clu), &buf);
                println!("[attack] {} tampered", name);
                return true;
            }
        }
        false
    }
}
