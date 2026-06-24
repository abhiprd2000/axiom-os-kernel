#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
extern crate alloc;
use axiom_os::{
    println,
    task::{Task, executor::SimpleExecutor, keyboard::ScancodeStream},
};
use bootloader::{BootInfo, entry_point};
use core::panic::PanicInfo;

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use axiom_os::{allocator, memory};
    use x86_64::VirtAddr;

    axiom_os::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    *axiom_os::PHYS_MEM_OFFSET.lock() = boot_info.physical_memory_offset;
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator =
        unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) };
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap init failed");
    *axiom_os::FRAME_ALLOCATOR.lock() =
        Some(unsafe { memory::BootInfoFrameAllocator::init(&boot_info.memory_map) });

    // Initialize process isolation
    use x86_64::structures::paging::PageTable;
    let kernel_l4 = unsafe {
        let (frame, _) = x86_64::registers::control::Cr3::read();
        let phys = frame.start_address();
        let virt = phys_mem_offset + phys.as_u64();
        &*(virt.as_ptr::<PageTable>())
    };
    {
        let mut pm = axiom_os::PROCESS_MANAGER.lock();
        pm.spawn(
            1,
            process_a,
            &mut frame_allocator,
            phys_mem_offset,
            kernel_l4,
        );
        pm.spawn(
            2,
            process_b,
            &mut frame_allocator,
            phys_mem_offset,
            kernel_l4,
        );
        pm.spawn(
            3,
            process_c,
            &mut frame_allocator,
            phys_mem_offset,
            kernel_l4,
        );
    }
    println!("  ___  __  __ ___ ___  __  __    ___  ___ ");
    println!(" / _ \\|  \\/  |_ _/ _ \\|  \\/  |  / _ \\ ");
    println!("|  __| |\\/| || | (_) | |\\/| | | (_) |__ ");
    println!(" \\___|_|  |_|___\\___/|_|  |_|  \\___/|___/");
    println!("                         AXIOM OS v0.3.0-alpha");
    println!("");
    println!("  Arch:      x86_64 bare metal");
    println!("  Hash:      BLAKE3 (cryptographic)");
    println!("  Heap:      {} KB", axiom_os::allocator::HEAP_SIZE / 1024);
    println!("  Disk:      4 MB FAT32 + 32 MB ATA persistent");
    println!("  Security:  provenance enforced on every read");
    println!("");
    println!("  Type help for commands.");
    println!("");
    let disk_ok = axiom_os::ata::init();
    if disk_ok {
        axiom_os::vga_buffer::println_colored(
            "  Persistent disk: ONLINE",
            axiom_os::vga_buffer::Color::LightGreen,
            axiom_os::vga_buffer::Color::Black,
        );
    } else {
        axiom_os::vga_buffer::println_colored(
            "  Persistent disk: OFFLINE",
            axiom_os::vga_buffer::Color::LightRed,
            axiom_os::vga_buffer::Color::Black,
        );
    }
    println!("");
    let mut executor = SimpleExecutor::new();
    executor.spawn(Task::new(keyboard_task()));
    executor.run();

    axiom_os::hlt_loop();
}

fn process_a() -> ! {
    loop {}
}
fn process_b() -> ! {
    loop {}
}
fn process_c() -> ! {
    loop {}
}

async fn keyboard_task() {
    use alloc::string::String;
    use futures_util::stream::StreamExt;
    use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::MapLettersToUnicode,
    );
    let mut input_buf = String::new();
    let mut hist_cursor: usize = 0;
    axiom_os::print!("> ");
    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                // Check if editor was just launched
                {
                    let mut ef = axiom_os::shell::EDITOR_FILE.lock();
                    if let Some(fname) = ef.take() {
                        let ed = axiom_os::editor::Editor::new(&fname);
                        *axiom_os::shell::EDITOR_ACTIVE.lock() = Some(ed);
                        axiom_os::print!("  ");
                    }
                }
                // If editor is active, send keys to it
                {
                    let mut ea = axiom_os::shell::EDITOR_ACTIVE.lock();
                    if let Some(ref mut editor) = *ea {
                        let quit = match key {
                            DecodedKey::Unicode(c) => editor.handle_char(c),
                            DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowUp) => {
                                editor.handle_char('\0');
                                false
                            }
                            _ => false,
                        };
                        if quit {
                            *ea = None;
                            axiom_os::vga_buffer::clear_screen();
                            axiom_os::println!("AXIOM OS v0.3.0-alpha");
                            axiom_os::print!("> ");
                        }
                        continue;
                    }
                }
                match key {
                    DecodedKey::Unicode('\n') | DecodedKey::Unicode('\r') => {
                        axiom_os::println!();
                        axiom_os::shell::interpret_command(&input_buf);
                        input_buf.clear();
                        hist_cursor = 0;
                        axiom_os::print!("> ");
                    }
                    DecodedKey::Unicode('\x08') => {
                        if !input_buf.is_empty() {
                            input_buf.pop();
                            axiom_os::vga_buffer::WRITER.lock().backspace();
                        }
                    }
                    DecodedKey::Unicode('\t') => {
                        let commands = [
                            "trust",
                            "verify",
                            "tamper",
                            "cat",
                            "ls",
                            "write",
                            "echo",
                            "diskwrite",
                            "diskread",
                            "diskls",
                            "diskverify",
                            "disktamper",
                            "ps",
                            "kill",
                            "spawn",
                            "hash",
                            "bench",
                            "mitra",
                            "run",
                            "sysinfo",
                            "history",
                            "help",
                            "clear",
                            "info",
                            "axiom",
                        ];
                        let matches: alloc::vec::Vec<&str> = commands
                            .iter()
                            .filter(|c| c.starts_with(input_buf.as_str()))
                            .copied()
                            .collect();
                        if matches.len() == 1 {
                            let completed = matches[0];
                            for _ in 0..input_buf.len() {
                                axiom_os::vga_buffer::WRITER.lock().backspace();
                            }
                            input_buf = alloc::string::String::from(completed);
                            axiom_os::print!("{}", completed);
                        } else if matches.len() > 1 {
                            axiom_os::println!();
                            for m in &matches {
                                axiom_os::print!("  {}", m);
                            }
                            axiom_os::println!();
                            axiom_os::print!("> {}", input_buf);
                        }
                    }
                    DecodedKey::Unicode(c) => {
                        input_buf.push(c);
                        axiom_os::print!("{}", c);
                    }
                    DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowUp) => {
                        let hist = axiom_os::shell::HISTORY.lock();
                        let len = *axiom_os::shell::HIST_LEN.lock();
                        let pos = *axiom_os::shell::HIST_POS.lock();
                        if len == 0 {
                            continue;
                        }
                        if hist_cursor < len {
                            hist_cursor += 1;
                        }
                        let idx = (pos + 10 - hist_cursor) % 10;
                        let entry = hist[idx].clone();
                        drop(hist);
                        // Clear current line
                        for _ in 0..input_buf.len() {
                            axiom_os::vga_buffer::WRITER.lock().backspace();
                        }
                        input_buf = entry.clone();
                        axiom_os::print!("{}", entry);
                    }
                    DecodedKey::RawKey(pc_keyboard::KeyCode::ArrowDown) => {
                        let hist = axiom_os::shell::HISTORY.lock();
                        let pos = *axiom_os::shell::HIST_POS.lock();
                        // Clear current line
                        for _ in 0..input_buf.len() {
                            axiom_os::vga_buffer::WRITER.lock().backspace();
                        }
                        if hist_cursor > 1 {
                            hist_cursor -= 1;
                            let idx = (pos + 10 - hist_cursor) % 10;
                            let entry = hist[idx].clone();
                            drop(hist);
                            input_buf = entry.clone();
                            axiom_os::print!("{}", entry);
                        } else {
                            hist_cursor = 0;
                            drop(hist);
                            input_buf.clear();
                        }
                    }

                    DecodedKey::RawKey(_) => {}
                }
            }
        }
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}
