//! The few things that differ between the native server and the WebAssembly
//! build hosted by `worker/`, which supplies them as `host` imports.

#[cfg(not(target_arch = "wasm32"))]
pub use std::time::Instant;

#[cfg(not(target_arch = "wasm32"))]
pub fn print(line: &str) {
    println!("{line}");
}

#[cfg(not(target_arch = "wasm32"))]
pub fn timestamp() -> String {
    chrono::Local::now().format("%m.%d %I:%M:%S").to_string()
}

#[cfg(target_arch = "wasm32")]
pub use wasm::{Instant, print, timestamp};

#[cfg(target_arch = "wasm32")]
mod wasm {
    use std::time::Duration;

    #[link(wasm_import_module = "host")]
    unsafe extern "C" {
        /// Milliseconds from a monotonic-enough clock (`Date.now()`).
        fn now_ms() -> f64;
        fn log(ptr: *const u8, len: usize);
    }

    #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
    pub struct Instant(f64);
    impl Instant {
        pub fn now() -> Self {
            // SAFETY: provided by the host with no preconditions.
            Self(unsafe { now_ms() })
        }
        pub fn saturating_duration_since(&self, earlier: Self) -> Duration {
            Duration::from_secs_f64((self.0 - earlier.0).max(0.0) / 1000.0)
        }
        pub fn duration_since(&self, earlier: Self) -> Duration {
            self.saturating_duration_since(earlier)
        }
        pub fn elapsed(&self) -> Duration {
            Self::now().saturating_duration_since(*self)
        }
    }

    pub fn print(line: &str) {
        // SAFETY: the host only reads `len` bytes from `ptr` during the call.
        unsafe { log(line.as_ptr(), line.len()) }
    }

    /// The host's log already carries timestamps.
    pub fn timestamp() -> &'static str {
        ""
    }
}
