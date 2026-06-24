use crate::println;
use alloc::vec::Vec;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Priority {
    Low = 1,
    Normal = 2,
    High = 3,
    RealTime = 4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProcessState {
    Ready,
    Running,
    Blocked,
}

#[derive(Debug, Clone, Copy)]
pub struct SchedulerEntry {
    pub pid: u64,
    pub priority: Priority,
    pub state: ProcessState,
    pub ticks_remaining: u32,
}

impl SchedulerEntry {
    pub fn new(pid: u64, priority: Priority) -> Self {
        SchedulerEntry {
            pid,
            priority,
            state: ProcessState::Ready,
            ticks_remaining: priority as u32 * 2,
        }
    }
}

pub struct Scheduler {
    entries: Vec<SchedulerEntry>,
    current: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            entries: Vec::new(),
            current: 0,
        }
    }

    pub fn add(&mut self, pid: u64, priority: Priority) {
        self.entries.push(SchedulerEntry::new(pid, priority));
    }

    pub fn next(&mut self) -> Option<u64> {
        if self.entries.is_empty() {
            return None;
        }

        // Find highest priority Ready process
        let next = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, e)| e.state == ProcessState::Ready)
            .max_by_key(|(_, e)| e.priority);

        if let Some((idx, _)) = next {
            if self.current < self.entries.len() {
                self.entries[self.current].state = ProcessState::Ready;
            }
            self.entries[idx].state = ProcessState::Running;
            self.current = idx;
            Some(self.entries[idx].pid)
        } else {
            None
        }
    }

    pub fn block(&mut self, pid: u64) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.pid == pid) {
            e.state = ProcessState::Blocked;
        }
    }

    pub fn unblock(&mut self, pid: u64) {
        if let Some(e) = self.entries.iter_mut().find(|e| e.pid == pid) {
            e.state = ProcessState::Ready;
        }
    }

    pub fn list(&self) {
        for e in &self.entries {
            println!(
                "  PID={} priority={:?} state={:?}",
                e.pid, e.priority, e.state
            );
        }
    }
}

impl Scheduler {
    pub fn remove(&mut self, pid: u64) {
        self.entries.retain(|e| e.pid != pid);
    }
}
