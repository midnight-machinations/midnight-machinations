//! Private control link to the voice service. Audio never travels over the game socket.
use serde::Serialize;

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

/// Holds the routing snapshot for the voice service and the presence it reports.
/// Snapshots go out only when they change; the voice service applies revocations
/// before grants and drops all routes if its control link closes.
pub struct VoiceBridge {
    pub public_url: String,
    snapshot: serde_json::Value,
    /// Hands each new snapshot to whatever carries it to the voice service.
    outbox: Box<dyn FnMut(&serde_json::Value) + Send>,
    /// Room client IDs joined to voice, by scope, as last reported by the voice service.
    presence: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, Vec<u32>>>>,
}
impl VoiceBridge {
    pub fn new(public_url: String, outbox: Box<dyn FnMut(&serde_json::Value) + Send>) -> Self {
        Self { public_url, snapshot: serde_json::json!([]), outbox, presence: Default::default() }
    }
    pub fn in_voice(&self, scope: &str, id: u32) -> bool {
        self.presence
            .lock()
            .is_ok_and(|presence| presence.get(scope).is_some_and(|ids| ids.contains(&id)))
    }
    pub fn set_presence(&self, presence: std::collections::HashMap<String, Vec<u32>>) {
        if let Ok(mut reported) = self.presence.lock() { *reported = presence; }
    }
    /// The current snapshot, to resend when a control link (re)connects.
    pub fn snapshot(&self) -> &serde_json::Value {
        &self.snapshot
    }
    pub fn publish(&mut self, snapshot: serde_json::Value) {
        if snapshot != self.snapshot {
            self.snapshot = snapshot;
            (self.outbox)(&self.snapshot);
        }
    }

    /// Connects to `VOICE_CONTROL_URL` from a background thread, if voice is configured.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_env() -> Option<Self> {
        if std::env::var("FEATURE_VOICE").is_ok_and(|flag| flag == "false") { return None }
        use futures_util::{SinkExt, StreamExt};
        use tokio::sync::watch;
        use tokio_tungstenite::tungstenite::{Message, client::IntoClientRequest};

        let url = std::env::var("VOICE_CONTROL_URL").ok()?;
        let public_url = std::env::var("VOICE_PUBLIC_URL").ok()?;
        let secret = std::env::var("VOICE_CONTROL_SECRET")
            .ok()
            .filter(|s| !s.is_empty())?;
        let (snapshots, mut updates) = watch::channel(serde_json::json!([]));
        let bridge = Self::new(public_url, Box::new(move |snapshot| { snapshots.send_replace(snapshot.clone()); }));
        let reported = bridge.presence.clone();
        // A dedicated runtime keeps the link independent of the game loop's lock.
        std::thread::spawn(move || {
            let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
            else {
                return;
            };
            runtime.block_on(async move {
                let mut version = 0u64;
                loop {
                    let Ok(mut request) = url.as_str().into_client_request() else { return };
                    let Ok(auth) = format!("Bearer {secret}").parse() else { return };
                    request.headers_mut().insert("authorization", auth);
                    if let Ok(Ok((mut socket, _))) = tokio::time::timeout(
                        std::time::Duration::from_secs(3), tokio_tungstenite::connect_async(request)
                    ).await {
                        // The voice service starts empty, so send the current snapshot first.
                        updates.mark_changed();
                        loop {
                            tokio::select! {
                                result = updates.changed() => {
                                    if result.is_err() { return }
                                    version = version.saturating_add(1);
                                    let packet = serde_json::json!({"version": version, "rooms": updates.borrow_and_update().clone()});
                                    if socket.send(Message::text(packet.to_string())).await.is_err() { break }
                                },
                                message = socket.next() => match message {
                                    Some(Ok(Message::Text(data))) => {
                                        let Ok(value) = serde_json::from_str::<serde_json::Value>(&data) else { continue };
                                        if let Some(scopes) = value.get("presence")
                                            && let Ok(scopes) = serde_json::from_value(scopes.clone())
                                            && let Ok(mut reported) = reported.lock()
                                        { *reported = scopes; }
                                    },
                                    Some(Ok(Message::Ping(data))) => {
                                        if socket.send(Message::Pong(data)).await.is_err() { break }
                                    },
                                    Some(Ok(Message::Close(_))) | Some(Err(_)) | None => break,
                                    _ => {},
                                },
                            }
                        }
                        if let Ok(mut reported) = reported.lock() { reported.clear(); }
                        let _ = socket.close(None).await;
                    }
                    tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                }
            });
        });
        Some(bridge)
    }
}
