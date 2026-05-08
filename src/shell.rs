use alloc::vec::Vec;
use crate::println;
use crate::vga_buffer::{self, Color};
use crate::vfs::VirtualFS;
use crate::fat32::Fat32;
use crate::scheduler::{Scheduler, Priority};



extern crate blake3;
use spin::Mutex;
use lazy_static::lazy_static;

lazy_static! {
    pub static ref EDITOR_FILE: Mutex<Option<alloc::string::String>> = Mutex::new(None);
    pub static ref EDITOR_ACTIVE: Mutex<Option<crate::editor::Editor>> = Mutex::new(None);
    pub static ref HISTORY: Mutex<[alloc::string::String; 10]> = 
        Mutex::new(core::array::from_fn(|_| alloc::string::String::new()));
    pub static ref HIST_LEN: Mutex<usize> = Mutex::new(0);
    pub static ref HIST_POS: Mutex<usize> = Mutex::new(0);
    pub static ref VFS: Mutex<VirtualFS> = Mutex::new(VirtualFS::new());
    pub static ref FAT32: Mutex<Fat32> = Mutex::new(Fat32::new());

    pub static ref SCHED: Mutex<Scheduler> = Mutex::new({
        let mut s = Scheduler::new();
        s.add(1, Priority::RealTime);
        s.add(2, Priority::High);
        s.add(3, Priority::Normal);
        s
    });
}

pub fn interpret_command(command: &str) {
    let cmd = command.trim();
    let parts: Vec<&str> = cmd.splitn(2, ' ').collect();
    let verb = parts[0];
    let arg  = if parts.len() > 1 { parts[1].trim() } else { "" };

    if !cmd.is_empty() && cmd != "!!" {
        let mut hist = HISTORY.lock();
        let mut len  = HIST_LEN.lock();
        let mut pos  = HIST_POS.lock();
        let last_idx = if *len == 0 { 10 } else { (*pos + 10 - 1) % 10 };
        if *len == 0 || hist[last_idx] != cmd {
            hist[*pos] = alloc::string::String::from(cmd);
            *pos = (*pos + 1) % 10;
            if *len < 10 { *len += 1; }
        }
    }

    match verb {
        "help" => {
            println!("=== AXIOM OS v0.3.0 Commands ===");
            println!("  help                - show this menu");
            println!("  clear               - clear screen");
            println!("  info                - system info");
            println!("  ls                  - list VFS files");
            println!("  trust <name> <data> - store trusted file");
            println!("  verify <name>       - verify provenance");
            println!("  tamper <name>       - simulate attack");
            println!("  diskwrite <n> <d>   - write to FAT32 disk");
            println!("  diskread <name>     - read from FAT32 disk");
            println!("  diskls              - list FAT32 files");
            println!("  disktamper <name>   - simulate disk attack");
            println!("  axiom               - about");
            println!("  ps                  - list processes");
            println!("  kill <pid>          - terminate process");
            println!("  spawn <pid>         - spawn new process");
            println!("  hash <text>         - compute BLAKE3 hash");
            println!("  bench               - run benchmarks");
            println!("  mitra <code>        - execute Mitra language");
            println!("  sysinfo             - full system information");
            println!("  edit <filename>     - open text editor");
            println!("  save <name>         - save VFS file to persistent disk");
            println!("  load <name>         - load file from persistent disk to VFS");
            println!("  edit <filename>     - open text editor");
            println!("  save <name>         - save VFS file to persistent disk");
            println!("  load <name>         - load file from persistent disk to VFS");
            println!("  run <script.mtr>    - run Mitra script from FAT32 disk");
            println!("  echo <text>         - print text");
            println!("  write <name> <data>  - write file without provenance");
            println!("  cat <name>          - read file (VFS or FAT32)");
            println!("  history             - show last 10 commands");
            println!("  !!                  - re-run last command");
            println!("  !n <n>              - re-run command #n");
        }
        "clear" => { vga_buffer::clear_screen(); }
        "info" => {
            println!("Axiom OS v0.2.0-alpha | Arch: x86_64 + aarch64 | Bare Metal");
            println!("Hash: BLAKE3 | Storage: FAT32 RAM Disk (4MB)");
            println!("Hardware-enforced data provenance active.");
        }
        "axiom" => {
            println!("AXIOM OS - The kernel that makes tampering");
            println!("architecturally impossible.");
        }
        "ls" => {
            let long = arg == "-l";
            let vfs = VFS.lock();
            if vfs.files.is_empty() {
                println!("  (no files)");
            } else if long {
                println!("{:<16} {:>8} {:>10}  {}", "NAME", "SIZE", "HASH", "STATUS");
                println!("{}", "----------------------------------------");
                for f in &vfs.files {
                    let verified = crate::provenance::provenance_hash(&f.data) == f.provenance_hash;
                    let h = f.provenance_hash;
                    let hash_short = alloc::format!("{:02x}{:02x}{:02x}{:02x}", h[0], h[1], h[2], h[3]);
                    let status = if verified { "[OK]" } else { "[TAMPERED]" };
                    if verified {
                        crate::vga_buffer::println_colored(
                            &alloc::format!("{:<16} {:>8} {:>10}  {}", f.name, f.data.len(), hash_short, status),
                            crate::vga_buffer::Color::LightGreen, crate::vga_buffer::Color::Black);
                    } else {
                        crate::vga_buffer::println_colored(
                            &alloc::format!("{:<16} {:>8} {:>10}  {}", f.name, f.data.len(), hash_short, status),
                            crate::vga_buffer::Color::LightRed, crate::vga_buffer::Color::Black);
                    }
                }
            } else {
                vfs.list();
            }
        }
        "trust" => {
            let sub: Vec<&str> = arg.splitn(2, ' ').collect();
            if sub.len() < 2 {
                println!("[error] usage: trust <name> <content>");
                return;
            }
            let name    = sub[0].trim();
            let content = sub[1].trim().as_bytes();
            let mut vfs = VFS.lock();
            vfs.create(name, content);
            println!("[axiom] trusted file stored: {}", name);
            println!("[axiom] provenance hash computed and locked");
        }
        "verify" => {
            if arg.is_empty() {
                println!("[error] usage: verify <filename>");
                return;
            }
            let vfs = VFS.lock();
            match vfs.verify(arg) {
                Some(true)  => {
                    vga_buffer::println_colored(&alloc::format!("[AXIOM KERNEL] VERIFIED: {}", arg), Color::LightGreen, Color::Black);
                    vga_buffer::println_colored("[AXIOM KERNEL] Hash matches - data authentic", Color::LightGreen, Color::Black);
                }
                Some(false) => {
                    vga_buffer::println_colored(&alloc::format!("[AXIOM KERNEL] TAMPERED: {}", arg), Color::LightRed, Color::Black);
                    vga_buffer::println_colored("[AXIOM KERNEL] Hash mismatch - BLOCKED", Color::LightRed, Color::Black);
                }
                None => { println!("[error] file not found: {}", arg); }
            }
        }
        "tamper" => {
            if arg.is_empty() {
                println!("[error] usage: tamper <filename>");
                return;
            }
            let mut vfs = VFS.lock();
            vfs.tamper(arg);
            println!("[attack] {} tampered", arg);
            println!("[attack] now run: verify {}", arg);
        }
        "diskwrite" => {
            let sub: Vec<&str> = arg.splitn(2, ' ').collect();
            if sub.len() < 2 {
                println!("[error] usage: diskwrite <name> <data>");
                return;
            }
            let name = sub[0].trim();
            let data = sub[1].trim().as_bytes();
            FAT32.lock().write_file(name, data);
        }
        "diskread" => {
            if arg.is_empty() {
                println!("[error] usage: diskread <name>");
                return;
            }
            match FAT32.lock().read_file(arg) {
                Some(data) => {
                    let s = core::str::from_utf8(&data).unwrap_or("binary");
                    println!("[fat32] {}: {}", arg, s);
                }
                None => { println!("[error] file not found: {}", arg); }
            }
        }
        "disktamper" => {
            if arg.is_empty() { println!("[error] usage: disktamper <name>"); return; }
            if FAT32.lock().tamper_file(arg) {
                println!("[attack] now run: diskverify {}", arg);
            } else {
                println!("[error] file not found: {}", arg);
            }
        }
        "diskverify" => {
            if arg.is_empty() { println!("[error] usage: diskverify <name>"); return; }
            let result = FAT32.lock().verify_file(arg);
            match result {
                Some(true)  => { println!("[AXIOM KERNEL] DISK VERIFIED: {}", arg); println!("[AXIOM KERNEL] FAT32 hash matches - authentic"); }
                Some(false) => { println!("[AXIOM KERNEL] DISK TAMPERED: {}", arg); println!("[AXIOM KERNEL] FAT32 hash mismatch - BLOCKED"); }
                None => { println!("[error] file not found: {}", arg); }
            }
        }
        "diskls" => {
            println!("=== FAT32 Disk Files ===");
            FAT32.lock().list_files();
        }
        "ps" => {
            println!("=== Process List ===");
            SCHED.lock().list();
            println!("--- Process Manager ---");
            crate::PROCESS_MANAGER.lock().list();
            println!("Total: {} processes", crate::PROCESS_MANAGER.lock().count());
        }
        "spawn" => {
            if arg.is_empty() {
                println!("[error] usage: spawn <pid>");
                return;
            }
            if let Ok(pid) = arg.parse::<u64>() {
                let phys_offset = x86_64::VirtAddr::new(*crate::PHYS_MEM_OFFSET.lock());
                let kernel_l4 = unsafe {
                    let (frame, _) = x86_64::registers::control::Cr3::read();
                    let phys = frame.start_address();
                    let virt = phys_offset + phys.as_u64();
                    &*(virt.as_ptr::<x86_64::structures::paging::PageTable>())
                };
                let mut fa = crate::FRAME_ALLOCATOR.lock();
                if let Some(ref mut allocator) = *fa {
                    crate::PROCESS_MANAGER.lock().spawn(
                        pid, shell_process, allocator, phys_offset, kernel_l4
                    );
                    SCHED.lock().add(pid, crate::scheduler::Priority::Normal);
                    println!("[kernel] spawned PID={} priority=Normal", pid);
                } else {
                    println!("[error] frame allocator not available");
                }
            } else {
                println!("[error] invalid PID: {}", arg);
            }
        }
        "kill" => {
            if arg.is_empty() {
                println!("[error] usage: kill <pid>");
                return;
            }
            if let Ok(pid) = arg.parse::<u64>() {
                if crate::PROCESS_MANAGER.lock().kill(pid) {
                    SCHED.lock().remove(pid);
                    println!("[kernel] PID {} terminated", pid);
                } else {
                    println!("[error] PID {} not found", pid);
                }
            } else {
                println!("[error] invalid PID: {}", arg);
            }
        }
        "hash" => {
            if arg.is_empty() {
                println!("[error] usage: hash <text>");
                return;
            }
            let h = blake3::hash(arg.as_bytes());
            println!("[blake3] input: {}", arg);
            let bytes = h.as_bytes();
            println!("[blake3] hash: {:02x}{:02x}{:02x}{:02x}...{:02x}{:02x}", bytes[0], bytes[1], bytes[2], bytes[3], bytes[30], bytes[31]);
        }
        "bench" => {
            use crate::benchmark::Benchmark;
            println!("=== Axiom OS Benchmarks ===");
            let mut b = Benchmark::new("blake3_hash");
            b.run(1000, || { blake3::hash(b"Journalist report from Jharkhand"); });
            b.report();
            let mut b2 = Benchmark::new("vfs_read");
            b2.run(100, || {
                let mut v = crate::vfs::VirtualFS::new();
                v.create("t", b"test data");
                v.read("t");
            });
            b2.report();
            println!("[bench] CPU cycles are real hardware measurements");
        }
        "mitra" => {
            if arg.is_empty() {
                println!("[error] usage: mitra <code>");
                println!("[hint]  example: mitra trust x = hello");
                return;
            }
            use crate::mitra::lexer::Lexer;
            use crate::mitra::parser::Parser;
            use crate::mitra::interpreter::Interpreter;
            use crate::ipc::MessageQueue;
            let mut mq = MessageQueue::new();
            let mut lexer = Lexer::new(arg);
            let tokens = lexer.tokenize();
            let mut parser = Parser::new(tokens);
            let ast = parser.parse();
            if ast.is_empty() {
                println!("[mitra] no valid statements parsed");
                return;
            }
            println!("[mitra] executing: {}", arg);
            let mut vfs = VFS.lock();
            let mut interp = Interpreter::new(&mut *vfs, &mut mq);
            interp.execute(ast);
        }
        "sysinfo" => {
            println!("=== AXIOM OS System Information ===");
            println!("  OS:        Axiom OS v0.3.0");
            println!("  Arch:      x86_64 + aarch64 (dual architecture)");
            println!("  Hash:      BLAKE3 (cryptographic)");
            println!("  Heap:      {} KB mapped at {:#x}",
                crate::allocator::HEAP_SIZE / 1024,
                crate::allocator::HEAP_START);
            println!("  RAM Disk:  4 MB FAT32");
            println!("  Processes: {}", crate::PROCESS_MANAGER.lock().count());
            println!("  Syscalls:  SYS_EXIT/YIELD/SPAWN/WRITE/VERIFY");
            println!("  Shell:     trust/verify/tamper/disk*/ps/kill/hash/bench/mitra");
            println!("  Security:  provenance enforced on every VFS+FAT32 read");
            println!("  Benchmark: BLAKE3={} cycles/op", {
                use crate::benchmark::read_tsc;
                let start = read_tsc();
                for _ in 0..100 {
                    blake3::hash(b"Journalist report from Jharkhand");
                }
                let end = read_tsc();
                (end.wrapping_sub(start)) / 100
            });
        }
        "edit" => {
            if arg.is_empty() {
                println!("[error] usage: edit <filename>");
                return;
            }
            // Signal keyboard task to enter editor mode
            *EDITOR_FILE.lock() = Some(alloc::string::String::from(arg));
        }
        "save" => {
            // Usage: save <filename>
            // Reads file from VFS and writes to persistent ATA disk
            if arg.is_empty() {
                println!("[error] usage: save <filename>");
                return;
            }
            let data = {
                let vfs = VFS.lock();
                vfs.read(arg).map(|d| d.to_vec())
            };
            match data {
                None => println!("[error] file not found in VFS: {}", arg),
                Some(d) => {
                    let mut sector = [0u8; 512];
                    let len = d.len().min(500);
                    // Store filename length + name + data
                    let name = arg.as_bytes();
                    let nlen = name.len().min(11) as u8;
                    sector[0] = nlen;
                    sector[1..1+nlen as usize].copy_from_slice(&name[..nlen as usize]);
                    sector[12] = len as u8;
                    sector[13..13+len].copy_from_slice(&d[..len]);
                    if crate::ata::write_sector(1, &sector) {
                        vga_buffer::println_colored(
                            &alloc::format!("[ata] SAVED: {} ({} bytes) to persistent disk", arg, len),
                            Color::LightGreen, Color::Black
                        );
                    } else {
                        println!("[error] write failed - is persistent disk online?");
                    }
                }
            }
        }
        "load" => {
            // Reads from persistent ATA disk back into VFS
            if arg.is_empty() {
                println!("[error] usage: load <filename>");
                return;
            }
            let mut sector = [0u8; 512];
            if !crate::ata::read_sector(1, &mut sector) {
                println!("[error] read failed - is persistent disk online?");
                return;
            }
            let nlen = sector[0] as usize;
            let stored_name = core::str::from_utf8(&sector[1..1+nlen]).unwrap_or("").trim();
            if stored_name.eq_ignore_ascii_case(arg.trim()) {
                let dlen = sector[12] as usize;
                let data = &sector[13..13+dlen];
                VFS.lock().create(arg, data);
                vga_buffer::println_colored(
                    &alloc::format!("[ata] LOADED: {} ({} bytes) from persistent disk", arg, dlen),
                    Color::LightGreen, Color::Black
                );
            } else {
                println!("[error] file '{}' not found on disk (found: '{}')", arg, stored_name);
            }
        }
        "run" => {
            if arg.is_empty() {
                println!("[error] usage: run <script.mtr>");
                return;
            }
            // Load from FAT32
            let script = FAT32.lock().read_file(arg);
            match script {
                None => {
                    println!("[error] script not found: {}", arg);
                    return;
                }
                Some(data) => {
                    // Verify provenance before execution
                    let verified = FAT32.lock().verify_file(arg);
                    match verified {
                        Some(false) => {
                            vga_buffer::println_colored(&alloc::format!("[AXIOM KERNEL] SCRIPT BLOCKED: {}", arg), Color::LightRed, Color::Black);
                            vga_buffer::println_colored("[AXIOM KERNEL] Provenance violation - tampered script refused", Color::LightRed, Color::Black);
                            return;
                        }
                        None => {
                            println!("[error] could not verify: {}", arg);
                            return;
                        }
                        Some(true) => {
                            vga_buffer::println_colored(&alloc::format!("[AXIOM KERNEL] SCRIPT VERIFIED: {}", arg), Color::LightGreen, Color::Black);
                            println!("[mitra] loading {} ({} bytes)", arg, data.len());
                        }
                    }
                    // Parse and execute
                    let src = match core::str::from_utf8(&data) {
                        Ok(s) => s,
                        Err(_) => {
                            println!("[error] script is not valid UTF-8");
                            return;
                        }
                    };
                    use crate::mitra::lexer::Lexer;
                    use crate::mitra::parser::Parser;
                    use crate::mitra::interpreter::Interpreter;
                    use crate::ipc::MessageQueue;
                    let mut mq = MessageQueue::new();
                    let mut lexer = Lexer::new(src);
                    let tokens = lexer.tokenize();
                    let mut parser = Parser::new(tokens);
                    let ast = parser.parse();
                    println!("[mitra] executing {} statements", ast.len());
                    let mut vfs = VFS.lock();
                    let mut interp = Interpreter::new(&mut *vfs, &mut mq);
                    interp.execute(ast);
                    println!("[mitra] done");
                }
            }
        }
        "echo" => {
            println!("{}", arg);
        }
        "write" => {
            // Usage: write <name> <content>
            let sub: Vec<&str> = arg.splitn(2, ' ').collect();
            if sub.len() < 2 {
                println!("[error] usage: write <name> <content>");
                return;
            }
            let name    = sub[0].trim();
            let content = sub[1].trim().as_bytes();
            let mut vfs = VFS.lock();
            vfs.create(name, content);
            println!("[vfs] written: {} ({} bytes) [no provenance]", name, content.len());
            println!("[hint] use trust to write with provenance enforcement");
        }
        "cat" => {
            if arg.is_empty() {
                println!("[error] usage: cat <filename>");
                return;
            }
            // Try VFS first
            {
                let vfs = VFS.lock();
                if let Some(data) = vfs.read(arg) {
                    let s = core::str::from_utf8(data).unwrap_or("[binary]");
                    println!("{}", s);
                    return;
                }
            }
            // Try FAT32 disk
            match FAT32.lock().read_file(arg) {
                Some(data) => {
                    let s = core::str::from_utf8(&data).unwrap_or("[binary]");
                    println!("{}", s);
                }
                None => {
                    println!("[error] file not found: {} (checked VFS and FAT32)", arg);
                }
            }
        }
        "history" => {
            let hist = HISTORY.lock();
            let len  = *HIST_LEN.lock();
            let pos  = *HIST_POS.lock();
            if len == 0 {
                println!("  (no history)");
            } else {
                let start = (pos + 10 - len) % 10;
                for i in 0..len {
                    let idx = (start + i) % 10;
                    println!("  {} {}", i + 1, hist[idx]);
                }
            }
        }
        "!!" => {
            let hist = HISTORY.lock();
            let len  = *HIST_LEN.lock();
            let pos  = *HIST_POS.lock();
            if len == 0 {
                println!("[error] no previous command");
            } else {
                let last = (pos + 10 - 1) % 10;
                let prev = hist[last].clone();
                drop(hist);
                println!("> {}", prev);
                crate::shell::interpret_command(&prev);
            }
        }
        "!n" => {
            if arg.is_empty() {
                println!("[error] usage: !n <number>");
                return;
            }
            if let Ok(n) = arg.parse::<usize>() {
                let hist = HISTORY.lock();
                let len  = *HIST_LEN.lock();
                let pos  = *HIST_POS.lock();
                if n == 0 || n > len {
                    println!("[error] no command at position {}", n);
                } else {
                    let start = (pos + 10 - len) % 10;
                    let idx = (start + n - 1) % 10;
                    let cmd_to_run = hist[idx].clone();
                    drop(hist);
                    println!("> {}", cmd_to_run);
                    crate::shell::interpret_command(&cmd_to_run);
                }
            } else {
                println!("[error] invalid number: {}", arg);
            }
        }
        "calc" => {
            if arg.is_empty() {
                println!("[error] usage: calc <expr>");
            } else {
                match crate::calc::evaluate(arg) {
                    Ok(v)  => crate::vga_buffer::println_colored(&alloc::format!("= {}", crate::calc::format_result(v)), crate::vga_buffer::Color::LightGreen, crate::vga_buffer::Color::Black),
                    Err(e) => crate::vga_buffer::println_colored(&alloc::format!("[calc error] {}", e), crate::vga_buffer::Color::LightRed, crate::vga_buffer::Color::Black),
                }
            }
        }
        "" => {}
        _ => { println!("Unknown: {} - type help", cmd); }
    }
}

fn shell_process() -> ! {
    crate::println!("[process] spawned from shell - running");
    loop {}
}
