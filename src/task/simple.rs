
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct TaskContext {
    pub rsp: u64,
    r15: u64,
    r14: u64,
    r13: u64,
    r12: u64,
    rbx: u64,
    rbp: u64,
}

impl TaskContext {
    pub const fn new() -> Self {
        TaskContext {
            rsp: 0,
            r15: 0,
            r14: 0,
            r13: 0,
            r12: 0,
            rbx: 0,
            rbp: 0,
        }
    }
}

unsafe extern "C" {
    pub fn switch_context(old: *mut TaskContext, new: *const TaskContext);
}

const STACK_SIZE: usize = 4096 * 5;

pub struct SimpleTask {
    #[allow(dead_code)]
    id: usize,
    stack: [u8; STACK_SIZE],
    context: TaskContext,
}

impl SimpleTask {
    pub fn new(id: usize, entry: fn()) -> Self {
        let mut task = SimpleTask {
            id,
            stack: [0; STACK_SIZE],
            context: TaskContext::new(),
        };

        unsafe {
            let stack_top = task.stack.as_ptr().add(STACK_SIZE) as u64;
            task.context.rsp = stack_top - 8;
            *(task.context.rsp as *mut u64) = entry as u64;
        }

        task
    }

    pub fn context_mut(&mut self) -> *mut TaskContext {
        &mut self.context
    }

    pub fn context(&self) -> *const TaskContext {
        &self.context
    }
}
