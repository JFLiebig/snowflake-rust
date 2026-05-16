use std::sync::Arc;
use tokio::sync::Semaphore;
use std::sync::atomic::{AtomicI64, Ordering};

pub struct Tokens {
    semaphore: Option<Arc<Semaphore>>,
    clients: AtomicI64,
}

impl Tokens {
    pub fn new(capacity: u32) -> Self {
        let semaphore = if capacity > 0 {
            Some(Arc::new(Semaphore::new(capacity as usize)))
        } else {
            None
        };
        Self {
            semaphore,
            clients: AtomicI64::new(0),
        }
    }

    pub async fn get(&self) -> Option<tokio::sync::SemaphorePermit<'_>> {
        self.clients.fetch_add(1, Ordering::SeqCst);
        if let Some(ref sem) = self.semaphore {
            Some(sem.acquire().await.unwrap())
        } else {
            None
        }
    }

    pub fn ret(&self) {
        self.clients.fetch_sub(1, Ordering::SeqCst);
        // Permit is automatically returned when dropped if we were to keep it,
        // but here the caller holds the permit.
    }

    pub fn count(&self) -> i64 {
        self.clients.load(Ordering::SeqCst)
    }
}
