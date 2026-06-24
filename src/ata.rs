use crate::println;
use x86_64::instructions::port::Port;

const ATA_DATA: u16 = 0x170;
#[allow(dead_code)]
const ATA_ERROR: u16 = 0x171;
const ATA_SECTOR_CNT: u16 = 0x172;
const ATA_LBA_LO: u16 = 0x173;
const ATA_LBA_MID: u16 = 0x174;
const ATA_LBA_HI: u16 = 0x175;
const ATA_DRIVE_HEAD: u16 = 0x176;
const ATA_STATUS: u16 = 0x177;
const ATA_COMMAND: u16 = 0x177;

const CMD_READ: u8 = 0x20;
const CMD_WRITE: u8 = 0x30;

#[allow(dead_code)]
const STATUS_BSY: u8 = 0x80;
const STATUS_DRQ: u8 = 0x08;
const STATUS_ERR: u8 = 0x01;

#[allow(dead_code)]
fn wait_ready() -> bool {
    let mut status: Port<u8> = unsafe { Port::new(ATA_STATUS) };
    for _ in 0..100_000 {
        let s = unsafe { status.read() };
        if s & STATUS_BSY == 0 && s & STATUS_DRQ != 0 {
            return true;
        }
        if s & STATUS_ERR != 0 {
            return false;
        }
    }
    false
}

#[allow(dead_code)]
fn wait_not_busy() {
    let mut status: Port<u8> = unsafe { Port::new(ATA_STATUS) };
    for _ in 0..100_000 {
        if unsafe { status.read() } & STATUS_BSY == 0 {
            return;
        }
    }
}

#[allow(dead_code)]
fn select_slave_lba(lba: u32) {
    unsafe {
        Port::<u8>::new(ATA_DRIVE_HEAD).write(0xE0 | ((lba >> 24) as u8 & 0x0F));
    }
}

pub fn read_sector(lba: u32, buf: &mut [u8; 512]) -> bool {
    unsafe {
        // Select slave, set LBA
        Port::<u8>::new(ATA_DRIVE_HEAD).write(0xE0 | ((lba >> 24) as u8 & 0x0F));
        Port::<u8>::new(ATA_SECTOR_CNT).write(1);
        Port::<u8>::new(ATA_LBA_LO).write(lba as u8);
        Port::<u8>::new(ATA_LBA_MID).write((lba >> 8) as u8);
        Port::<u8>::new(ATA_LBA_HI).write((lba >> 16) as u8);
        Port::<u8>::new(ATA_COMMAND).write(CMD_READ);
        // Fixed delay - read status 400 times
        let mut sp = Port::<u8>::new(ATA_STATUS);
        for _ in 0..400usize {
            sp.read();
        }
        let s = sp.read();
        if s & STATUS_ERR != 0 || s & STATUS_DRQ == 0 {
            return false;
        }
        let mut data: Port<u16> = Port::new(ATA_DATA);
        for i in 0..256usize {
            let word = data.read();
            buf[i * 2] = (word & 0xFF) as u8;
            buf[i * 2 + 1] = (word >> 8) as u8;
        }
        true
    }
}

pub fn write_sector(lba: u32, buf: &[u8; 512]) -> bool {
    unsafe {
        Port::<u8>::new(ATA_DRIVE_HEAD).write(0xE0 | ((lba >> 24) as u8 & 0x0F));
        Port::<u8>::new(ATA_SECTOR_CNT).write(1);
        Port::<u8>::new(ATA_LBA_LO).write(lba as u8);
        Port::<u8>::new(ATA_LBA_MID).write((lba >> 8) as u8);
        Port::<u8>::new(ATA_LBA_HI).write((lba >> 16) as u8);
        Port::<u8>::new(ATA_COMMAND).write(CMD_WRITE);
        let mut sp = Port::<u8>::new(ATA_STATUS);
        for _ in 0..400usize {
            sp.read();
        }
        let s = sp.read();
        if s & STATUS_ERR != 0 || s & STATUS_DRQ == 0 {
            return false;
        }
        let mut data: Port<u16> = Port::new(ATA_DATA);
        for i in 0..256usize {
            let word = (buf[i * 2] as u16) | ((buf[i * 2 + 1] as u16) << 8);
            data.write(word);
        }
        true
    }
}

pub fn detect() -> bool {
    unsafe {
        // Select slave drive (0xB0)
        Port::<u8>::new(ATA_DRIVE_HEAD).write(0xA0);
        // Read status 4 times to let drive respond
        for _ in 0..4 {
            Port::<u8>::new(ATA_STATUS).read();
        }
        let status = Port::<u8>::new(ATA_STATUS).read();
        // 0xFF = floating bus = no drive, 0x00 = also no drive
        status != 0xFF && status != 0x00
    }
}

pub fn init() -> bool {
    unsafe {
        // Check secondary channel master (our axiom-disk.img is index=1 = secondary master)
        Port::<u8>::new(ATA_DRIVE_HEAD).write(0xA0);
        for _ in 0..15usize {
            Port::<u8>::new(ATA_STATUS).read();
        }
        let s1 = Port::<u8>::new(ATA_STATUS).read();
        let s2 = Port::<u8>::new(ATA_STATUS).read();
        let s3 = Port::<u8>::new(ATA_STATUS).read();
        println!("[ata] sec-master s1={:#x} s2={:#x} s3={:#x}", s1, s2, s3);

        // Accept any non-0xFF, non-0x00 status as drive present
        let found = s3 != 0xFF && s3 != 0x00;
        if found {
            println!("[ata] persistent disk ONLINE (32 MB)");
        } else {
            println!("[ata] persistent disk OFFLINE");
        }
        found
    }
}
