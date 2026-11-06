use std::sync::{Mutex, Arc, MutexGuard};

pub mod connection;
#[cfg(not(target_arch = "wasm32"))]
pub mod websocket_server;

pub trait ForceLock {
    type Inner;

    /// Ignore poison errors and keep running.
    /// Be careful where you use this!! 
    /// Sometimes an error means the server should shut down, not carry on.
    fn force_lock(self) -> Self::Inner;
}

impl<'a, T> ForceLock for &'a Arc<Mutex<T>> {
    type Inner = MutexGuard<'a, T>;

    fn force_lock(self) -> Self::Inner {
        match self.lock() {
            Ok(inner) => inner,
            Err(err) => {
                self.clear_poison();
                err.into_inner()
            }
        }
    }
}
