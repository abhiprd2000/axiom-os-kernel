use core::{future::Future, pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;
use crossbeam_queue::ArrayQueue;
use conquer_once::spin::OnceCell;
use crate::vfs::CachedVfsBlock;

pub mod executor;
pub mod keyboard;
pub mod simple;

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

#[derive(Clone, Copy)]
pub struct CryptoVerificationJob {
    pub block_ptr: *mut CachedVfsBlock,
    pub expected_hash: [u8; 32], 
}

// Explicitly implement Send and Sync to allow transmission through the lock-free ArrayQueue
unsafe impl Send for CryptoVerificationJob {}
unsafe impl Sync for CryptoVerificationJob {}

// Global thread-safe async queue for block validation
pub static VERIFICATION_QUEUE: OnceCell<ArrayQueue<CryptoVerificationJob>> = OnceCell::uninit();

/// Publicly exposes queue initialization to external modules like main.rs
pub fn init_queue() {
    VERIFICATION_QUEUE.init_once(|| ArrayQueue::new(32));
}

pub fn spawn_provenance_worker() {
    loop {
        if let Some(queue) = VERIFICATION_QUEUE.get() {
            if let Some(job) = queue.pop() {
                unsafe {
                    crate::provenance::process_next_provenance_job(job);
                }
            }
        }
        core::hint::spin_loop();
    }
}

pub fn yield_current_thread() {
    core::hint::spin_loop();
}