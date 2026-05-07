use alloc::vec::Vec;
use super::parser::AstNode;
use crate::{vfs::VirtualFS, ipc::MessageQueue, provenance::TrustedData, println};

pub struct Interpreter<'a> {
    vfs: &'a mut VirtualFS,
    mq: &'a mut MessageQueue,
}

impl<'a> Interpreter<'a> {
    pub fn new(vfs: &'a mut VirtualFS, mq: &'a mut MessageQueue) -> Self {
        Interpreter { vfs, mq }
    }

    pub fn execute(&mut self, nodes: Vec<AstNode>) {
        for node in nodes {
            match node {
                AstNode::Trust { name, value } => {
                    println!("[mitra] trust {} = \"{}\"", name, value);
                    self.vfs.create(&name, value.as_bytes());
                    println!("[mitra] file \"{}\" stored with provenance hash", value);
                }
                AstNode::TrustedData { name, content } => {
                    println!("[mitra] trusted_data {}", name);
                    let td = TrustedData::new("mitra_trusted", content.as_bytes());
                    let ok = td.verify_or_halt();
                    if ok {
                        println!("[mitra] trusted_data \"{}\" approved for execution", name);
                    } else {
                        println!("[mitra] trusted_data \"{}\" BLOCKED", name);
                    }
                }
                AstNode::Verify { name } => {
                    println!("[mitra] verifying \"{}\"...", name);
                    match self.vfs.verify(&name) {
                        Some(true)  => println!("[mitra] VERIFIED: \"{}\" is authentic", name),
                        Some(false) => println!("[mitra] TAMPERED: \"{}\" hash mismatch!", name),
                        None        => println!("[mitra] NOT FOUND: \"{}\"", name),
                    }
                }
                AstNode::Spawn { pid } => {
                    println!("[mitra] spawn process PID={}", pid);
                }
                AstNode::Send { from, to, msg } => {
                    println!("[mitra] send {} -> {}: \"{}\"", from, to, msg);
                    self.mq.send(from, to, &msg);
                }
                AstNode::If { condition, body } => {
                    println!("[mitra] if {}: executing {} nodes", condition, body.len());
                    self.execute(body);
                }
            }
        }
    }
}
