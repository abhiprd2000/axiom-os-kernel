use alloc::boxed::Box;
use alloc::vec::Vec;
use x86_64::{
    VirtAddr,
    registers::control::Cr3,
    structures::paging::{FrameAllocator, PageTable, PhysFrame, Size4KiB},
};
#[allow(dead_code)]
fn process_exit() -> ! {
    crate::println!("Process exited cleanly");
    loop {}
}

use crate::task::simple::{TaskContext, switch_context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessId(u64);

impl ProcessId {
    pub fn new(id: u64) -> Self {
        ProcessId(id)
    }
    pub fn as_u64(&self) -> u64 {
        self.0
    }
}

pub struct Process {
    pub id: ProcessId,
    pub stack: Box<[u8; 4096 * 4]>,
    pub page_table_frame: PhysFrame,
    pub task_context: TaskContext,
}

impl Process {
    pub fn new(
        id: u64,
        entry_point: fn() -> !,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
        physical_memory_offset: VirtAddr,
        kernel_page_table: &PageTable,
    ) -> Self {
        let frame = frame_allocator
            .allocate_frame()
            .expect("failed to allocate page table frame");

        let phys = frame.start_address();
        let virt = physical_memory_offset + phys.as_u64();
        let new_table: &mut PageTable = unsafe { &mut *(virt.as_mut_ptr()) };
        new_table.zero();
        for i in 256..512 {
            new_table[i] = kernel_page_table[i].clone();
        }

        // Box the stack so it has a stable address that won't move
        let stack = Box::new([0u8; 4096 * 4]);
        let stack_end = stack.as_ptr() as u64 + (4096 * 4) as u64;
        let stack_top = stack_end - 8;
        unsafe {
            *(stack_top as *mut u64) = entry_point as u64;
        }

        let mut ctx = TaskContext::new();
        ctx.rsp = stack_top;

        Process {
            id: ProcessId::new(id),
            stack,
            page_table_frame: frame,
            task_context: ctx,
        }
    }

    pub fn context_mut(&mut self) -> *mut TaskContext {
        &mut self.task_context as *mut TaskContext
    }

    pub fn context(&self) -> *const TaskContext {
        &self.task_context as *const TaskContext
    }

    pub unsafe fn activate(&self) {
        unsafe {
            Cr3::write(self.page_table_frame, Cr3::read().1);
        }
    }
}

pub struct ProcessManager {
    processes: Vec<Process>,
    current: usize,
}

impl ProcessManager {
    pub fn new() -> Self {
        ProcessManager {
            processes: Vec::new(),
            current: 0,
        }
    }

    pub fn spawn(
        &mut self,
        id: u64,
        entry_point: fn() -> !,
        frame_allocator: &mut impl FrameAllocator<Size4KiB>,
        physical_memory_offset: VirtAddr,
        kernel_page_table: &PageTable,
    ) {
        let p = Process::new(
            id,
            entry_point,
            frame_allocator,
            physical_memory_offset,
            kernel_page_table,
        );
        self.processes.push(p);
    }

    pub fn current_id(&self) -> Option<u64> {
        self.processes.get(self.current).map(|p| p.id.as_u64())
    }

    pub fn count(&self) -> usize {
        self.processes.len()
    }

    pub fn switch_to_next(&mut self) {
        if self.processes.len() < 2 {
            return;
        }
        let next = (self.current + 1) % self.processes.len();
        let old_ctx = self.processes[self.current].context_mut();
        let new_ctx = self.processes[next].context();
        self.current = next;
        unsafe {
            switch_context(old_ctx, new_ctx);
        }
    }
}

impl ProcessManager {
    pub fn kill(&mut self, pid: u64) -> bool {
        if let Some(pos) = self.processes.iter().position(|p| p.id.as_u64() == pid) {
            self.processes.remove(pos);
            if self.current >= self.processes.len() && self.current > 0 {
                self.current = self.processes.len() - 1;
            }
            return true;
        }
        false
    }

    pub fn list(&self) {
        if self.processes.is_empty() {
            crate::println!("  (no processes)");
            return;
        }
        for p in &self.processes {
            let marker = if p.id.as_u64() == self.current_id().unwrap_or(0) {
                "*"
            } else {
                " "
            };
            crate::println!(
                "  {}PID={} ctx_rsp={:#x}",
                marker,
                p.id.as_u64(),
                p.task_context.rsp
            );
        }
    }
}
