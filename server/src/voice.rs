//! Private control link to the SFU. Audio never travels over the game socket.
use futures_util::{SinkExt, StreamExt};
use serde::Serialize;
use tokio::sync::watch;
use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoicePeer {
    pub id: u32,
    pub name: String,
    pub player_index: Option<u8>,
    /// Whether this peer has joined voice. Only shared in lobbies; always false in games.
    pub in_voice: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceSession {
    pub url: String,
    pub token: String,
    pub scope: String,
    pub self_id: u32,
    pub send_groups: Vec<String>,
    pub receive_groups: Vec<String>,
    pub forced_silenced: bool,
    pub peers: Vec<VoicePeer>,
}

pub fn random_token() -> String {
    format!(
        "{:032x}{:032x}",
        rand::random::<u128>(),
        rand::random::<u128>()
    )
}

pub struct VoiceBridge {
    pub public_url: String,
    snapshots: watch::Sender<serde_json::Value>,
    commands: tokio::sync::mpsc::UnboundedSender<Option<std::sync::mpsc::SyncSender<()>>>,
    active: std::sync::Arc<std::sync::atomic::AtomicBool>,
    frozen: std::sync::Arc<std::sync::atomic::AtomicBool>,
    lease_until: std::sync::Arc<std::sync::Mutex<std::time::Instant>>,
    /// Room client IDs joined to voice, by scope, as last reported by the SFU.
    presence: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Vec<u32>>>>,
}
impl VoiceBridge {
    pub fn from_env() -> Option<Self> {
        use std::sync::{
            Arc, Mutex,
            atomic::{AtomicBool, Ordering},
        };
        let url = std::env::var("VOICE_CONTROL_URL").ok()?;
        let public_url = std::env::var("VOICE_PUBLIC_URL").ok()?;
        let secret = std::env::var("VOICE_CONTROL_SECRET")
            .ok()
            .filter(|s| !s.is_empty())?;
        let (snapshots, mut updates) = watch::channel(serde_json::json!([]));
        let (commands, mut requests) =
            tokio::sync::mpsc::unbounded_channel::<Option<std::sync::mpsc::SyncSender<()>>>();
        let active = Arc::new(AtomicBool::new(false));
        let frozen = Arc::new(AtomicBool::new(false));
        let lease_until = Arc::new(Mutex::new(std::time::Instant::now()));
        let presence = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let (connected, paused, lease, reported) = (active.clone(), frozen.clone(), lease_until.clone(), presence.clone());
        // A dedicated runtime keeps permission acknowledgements progressing while the
        // synchronous game loop holds its lock and waits for a forwarding barrier.
        std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(async move {
                loop {
                    let Ok(mut request) = url.as_str().into_client_request() else { return };
                    let Ok(auth) = format!("Bearer {secret}").parse() else { return };
                    request.headers_mut().insert("authorization", auth);
                    if let Ok(Ok((mut socket, _))) = tokio::time::timeout(
                        std::time::Duration::from_secs(3), tokio_tungstenite::connect_async(request)
                    ).await {
                        connected.store(true, Ordering::SeqCst);
                        let mut heartbeat = tokio::time::interval(std::time::Duration::from_secs(1));
                        let mut version = 0u64;
                        let mut barrier = 0u64;
                        let mut acknowledgement = None;
                        loop {
                            tokio::select! {
                                result = updates.changed() => {
                                    if result.is_err() { return }
                                },
                                command = requests.recv() => {
                                    match command {
                                        Some(Some(reply)) => {
                                            barrier = barrier.saturating_add(1);
                                            acknowledgement = Some(reply);
                                            if socket.send(Message::text(serde_json::json!({"freeze": barrier}).to_string())).await.is_err() { break }
                                            continue;
                                        },
                                        Some(None) => break,
                                        None => return,
                                    }
                                },
                                _ = heartbeat.tick() => {},
                                message = socket.next() => {
                                    match message {
                                        Some(Ok(Message::Text(data))) => {
                                            let Ok(value) = serde_json::from_str::<serde_json::Value>(&data) else { continue };
                                            if let Some(scopes) = value.get("presence")
                                                && let Ok(scopes) = serde_json::from_value(scopes.clone())
                                                && let Ok(mut reported) = reported.lock()
                                            { *reported = scopes; }
                                            if value.get("frozen").and_then(serde_json::Value::as_u64) == Some(barrier)
                                                && let Some(reply) = acknowledgement.take()
                                            { let _ = reply.send(()); }
                                            continue;
                                        },
                                        Some(Ok(Message::Ping(data))) => {
                                            if socket.send(Message::Pong(data)).await.is_err() { break }
                                            continue;
                                        },
                                        Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                                        _ => continue,
                                    }
                                }
                            }
                            if let Ok(mut deadline) = lease.lock() {
                                *deadline = std::time::Instant::now().checked_add(std::time::Duration::from_secs(5)).expect("five-second media lease fits Instant");
                            }
                            let packet = if paused.load(Ordering::SeqCst) {
                                serde_json::json!({"heartbeat": true})
                            } else {
                                version = version.saturating_add(1);
                                serde_json::json!({"version": version, "rooms": updates.borrow_and_update().clone()})
                            };
                            if socket.send(Message::text(packet.to_string())).await.is_err() { break }
                        }
                        connected.store(false, Ordering::SeqCst);
                        if let Ok(mut reported) = reported.lock() { reported.clear(); }
                        let _ = socket.close(None).await;
                    }
                    if updates.has_changed().is_err() { return }
                    // No new media authority may resume an old policy while a game
                    // mutation is waiting for the previous authority's lease to expire.
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            });
        });
        Some(Self {
            public_url,
            snapshots,
            commands,
            active,
            frozen,
            lease_until,
            presence,
        })
    }
    pub fn in_voice(&self, scope: &str, id: u32) -> bool {
        self.presence
            .lock()
            .is_ok_and(|presence| presence.get(scope).is_some_and(|ids| ids.contains(&id)))
    }
    /// Stop forwarding before a game mutation, including for modified clients.
    pub fn freeze(&self) {
        use std::sync::atomic::Ordering;
        self.frozen.store(true, Ordering::SeqCst);
        if self.active.load(Ordering::SeqCst) {
            let (reply, result) = std::sync::mpsc::sync_channel(1);
            if self.commands.send(Some(reply)).is_ok()
                && result
                    .recv_timeout(std::time::Duration::from_secs(2))
                    .is_ok()
            {
                return;
            }
            // Abort a stalled socket. The SFU's heartbeat lease bounds how long
            // it could retain forwarding permissions if the close cannot arrive.
            let _ = self.commands.send(None);
        }
        let deadline = self
            .lease_until
            .lock()
            .map(|deadline| *deadline)
            .unwrap_or_else(|_| {
                std::time::Instant::now()
                    .checked_add(std::time::Duration::from_secs(5))
                    .expect("five-second media lease fits Instant")
            });
        std::thread::sleep(deadline.saturating_duration_since(std::time::Instant::now()));
    }
    pub fn publish(&self, snapshot: serde_json::Value) {
        // Install the new policy before releasing the barrier. A heartbeat must
        // never resume forwarding using the pre-mutation snapshot.
        self.snapshots.send_replace(snapshot.clone());
        self.frozen
            .store(false, std::sync::atomic::Ordering::SeqCst);
        // Wake the bridge again if it observed the first update while frozen.
        self.snapshots.send_replace(snapshot);
    }
}
