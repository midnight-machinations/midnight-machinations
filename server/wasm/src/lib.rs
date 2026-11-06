//! WebAssembly entry points for hosting the game server in a Cloudflare Durable
//! Object (`../worker/index.ts`). The game logic is the unchanged `mafia_server`
//! crate; this only replaces its TCP/WebSocket and timer plumbing.
//!
//! Strings cross the boundary as UTF-8 in linear memory: the host writes input
//! into a buffer from `alloc`, and each call returns the byte length of its
//! output at `output_ptr()`. Output is one line per item:
//!
//! - `<connection id> <ToClientPacket JSON>`: send to that socket
//! - `voice <snapshot JSON>`: a new routing snapshot for the voice service
//!
//! A panic aborts, which traps; the host then starts a fresh instance, like the
//! native server restarting after a panic.

use std::{
    cell::RefCell,
    collections::HashMap,
    net::{IpAddr, Ipv6Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::Duration,
};

use mafia_server::{
    packet::ToClientPacket, voice::VoiceBridge, websocket_connections::connection::Connection,
    websocket_listener::WebsocketListener,
};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

#[link(wasm_import_module = "host")]
unsafe extern "C" {
    fn random(ptr: *mut u8, len: usize);
}

/// The `getrandom` backend selected by `.cargo/config.toml`.
///
/// # Safety
/// `dest` must be valid for `len` bytes of writes.
#[unsafe(no_mangle)]
unsafe extern "Rust" fn __getrandom_v03_custom(dest: *mut u8, len: usize) -> Result<(), getrandom::Error> {
    // SAFETY: the host fills exactly `len` bytes at `dest`.
    unsafe { random(dest, len) };
    Ok(())
}

struct Host {
    listener: WebsocketListener,
    connections: HashMap<u32, (Connection, UnboundedReceiver<ToClientPacket>)>,
    voice_snapshot: Arc<Mutex<Option<serde_json::Value>>>,
    output: String,
}

thread_local! {
    static HOST: RefCell<Option<Host>> = const { RefCell::new(None) };
}

/// Runs `action` against the host, then collects every queued packet into the output.
fn with_host(action: impl FnOnce(&mut Host)) -> usize {
    HOST.with_borrow_mut(|host| {
        let Some(host) = host else { return 0 };
        host.output.clear();
        action(host);
        for (id, (_, packets)) in &mut host.connections {
            while let Ok(packet) = packets.try_recv() {
                if let Ok(json) = serde_json::to_string(&packet) {
                    host.output.push_str(&format!("{id} {json}\n"));
                }
            }
        }
        if let Some(snapshot) = host.voice_snapshot.lock().ok().and_then(|mut s| s.take()) {
            host.output.push_str(&format!("voice {snapshot}\n"));
        }
        host.output.len()
    })
}

/// # Safety
/// `ptr` must point to `len` initialized bytes, or `len` must be 0.
unsafe fn read(ptr: *const u8, len: usize) -> String {
    if len == 0 {
        return String::new();
    }
    // SAFETY: guaranteed by the caller.
    String::from_utf8_lossy(unsafe { std::slice::from_raw_parts(ptr, len) }).into_owned()
}

#[unsafe(no_mangle)]
pub extern "C" fn alloc(len: usize) -> *mut u8 {
    let mut buffer = Vec::<u8>::with_capacity(len);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr
}

/// # Safety
/// `ptr` and `len` must come from one call to `alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn dealloc(ptr: *mut u8, len: usize) {
    // SAFETY: guaranteed by the caller.
    drop(unsafe { Vec::from_raw_parts(ptr, 0, len) });
}

#[unsafe(no_mangle)]
pub extern "C" fn output_ptr() -> *const u8 {
    HOST.with_borrow(|host| host.as_ref().map_or(std::ptr::null(), |host| host.output.as_ptr()))
}

/// Starts the game server. An empty `voice_url` leaves voice disabled; otherwise
/// it is the browser-facing voice URL and snapshots appear in the output.
///
/// # Safety
/// `voice_url` must point to `len` initialized bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn init(voice_url: *const u8, len: usize) {
    // SAFETY: guaranteed by the caller.
    let public_url = unsafe { read(voice_url, len) };
    let voice_snapshot = Arc::new(Mutex::new(None));
    let outbox = voice_snapshot.clone();
    let voice = (!public_url.is_empty()).then(|| {
        VoiceBridge::new(public_url, Box::new(move |snapshot: &serde_json::Value| {
            if let Ok(mut pending) = outbox.lock() { *pending = Some(snapshot.clone()); }
        }))
    });
    HOST.set(Some(Host {
        listener: WebsocketListener::with_voice(voice),
        connections: HashMap::new(),
        voice_snapshot,
        output: String::new(),
    }));
}

#[unsafe(no_mangle)]
pub extern "C" fn connect(id: u32) -> usize {
    with_host(|host| {
        let (sender, packets) = unbounded_channel();
        // The listener keys clients by address; each socket gets a unique fake one.
        let address = SocketAddr::new(IpAddr::V6(Ipv6Addr::from(u128::from(id))), 0);
        let connection = Connection::new(sender, address);
        host.listener.on_connect(&connection);
        host.connections.insert(id, (connection, packets));
    })
}

/// # Safety
/// `ptr` must point to `len` initialized bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn message(id: u32, ptr: *const u8, len: usize) -> usize {
    // SAFETY: guaranteed by the caller.
    let text = unsafe { read(ptr, len) };
    with_host(|host| {
        if let Some((connection, _)) = host.connections.get(&id) {
            host.listener.on_message(connection, &text);
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn disconnect(id: u32) -> usize {
    with_host(|host| {
        if let Some((connection, _)) = host.connections.remove(&id) {
            host.listener.on_disconnect(connection);
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn tick(elapsed_ms: u32) -> usize {
    with_host(|host| host.listener.tick(Duration::from_millis(u64::from(elapsed_ms))))
}

/// Records `{ scope: [room client id] }` from the voice service.
///
/// # Safety
/// `ptr` must point to `len` initialized bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn voice_presence(ptr: *const u8, len: usize) -> usize {
    // SAFETY: guaranteed by the caller.
    let text = unsafe { read(ptr, len) };
    with_host(|host| {
        if let (Some(voice), Ok(presence)) = (host.listener.voice(), serde_json::from_str(&text)) {
            voice.set_presence(presence);
        }
    })
}

/// Queues the current snapshot again, for a voice control link that just connected.
#[unsafe(no_mangle)]
pub extern "C" fn voice_resend() -> usize {
    with_host(|host| {
        if let Some(voice) = host.listener.voice()
            && let Ok(mut pending) = host.voice_snapshot.lock()
        {
            *pending = Some(voice.snapshot().clone());
        }
    })
}
