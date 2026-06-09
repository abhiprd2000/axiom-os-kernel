use core::{future::Future, pin::Pin, task::{Context, Poll}};
use alloc::boxed::Box;

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

use crate::vfs::CachedVfsBlock;

#[derive(Clone, Copy)]
pub struct CryptoVerificationJob {
    pub block_ptr: *mut CachedVfsBlock,
    pub expected_hash: [u8; 32], 
}

pub struct VerificationQueue {
    pub jobs: [Option<CryptoVerificationJob>; 32],
    pub head: usize,
    pub tail: usize,
}

pub fn spawn_provenance_worker() {
    
    loop {
        if let Some(queue) = VERIFICATION_QUEUE.get() {
            if let Some(job) = queue.pop() {
                // Execute the cryptographic hashing off the main thread path
                unsafe {
                    crate::provenance::process_next_provenance_job(job);
                }
            }
        }
        // Prevent raw CPU hogging in simulation
        core::hint::spin_loop();
    }
}