use crate::println;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct Message {
    pub from: u64,
    pub to: u64,
    pub data: String,
}

impl Message {
    pub fn new(from: u64, to: u64, data: &str) -> Self {
        Message {
            from,
            to,
            data: String::from(data),
        }
    }
}

pub struct MessageQueue {
    messages: Vec<Message>,
}

impl MessageQueue {
    pub fn new() -> Self {
        MessageQueue {
            messages: Vec::new(),
        }
    }

    pub fn send(&mut self, from: u64, to: u64, data: &str) {
        println!("[ipc] PID {} -> PID {}: \"{}\"", from, to, data);
        self.messages.push(Message::new(from, to, data));
    }

    pub fn receive(&mut self, pid: u64) -> Option<Message> {
        if let Some(idx) = self.messages.iter().position(|m| m.to == pid) {
            Some(self.messages.remove(idx))
        } else {
            None
        }
    }

    pub fn pending(&self, pid: u64) -> usize {
        self.messages.iter().filter(|m| m.to == pid).count()
    }
}
