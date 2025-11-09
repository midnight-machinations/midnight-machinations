use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::track::track_remote::TrackRemote;
use webrtc::rtp_transceiver::rtp_sender::RTCRtpSender;

use crate::room::RoomClientID;
use crate::packet::ToClientPacket;

/// Manages WebRTC peer connections for voice chat SFU
pub struct WebRtcSfuManager {
    /// WebRTC API instance
    api: Arc<webrtc::api::API>,
    /// Peer connections for each client
    connections: Arc<Mutex<HashMap<RoomClientID, Arc<RTCPeerConnection>>>>,
    /// Audio tracks from each client (for forwarding)
    client_tracks: Arc<Mutex<HashMap<RoomClientID, Arc<TrackRemote>>>>,
    /// Track senders for forwarding audio to clients
    track_senders: Arc<Mutex<HashMap<RoomClientID, Vec<Arc<RTCRtpSender>>>>>,
}

impl WebRtcSfuManager {
    /// Create a new WebRTC SFU manager
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Create a MediaEngine
        let mut media_engine = MediaEngine::default();
        
        // Register default codecs
        media_engine.register_default_codecs()?;

        // Create the API object with the MediaEngine
        let api = APIBuilder::new()
            .with_media_engine(media_engine)
            .build();

        Ok(Self {
            api: Arc::new(api),
            connections: Arc::new(Mutex::new(HashMap::new())),
            client_tracks: Arc::new(Mutex::new(HashMap::new())),
            track_senders: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create a peer connection for a client
    pub async fn create_peer_connection(
        &self,
        client_id: RoomClientID,
        on_ice_candidate: impl Fn(String, Option<String>, Option<u16>) + Send + Sync + 'static,
    ) -> Result<Arc<RTCPeerConnection>, Box<dyn std::error::Error>> {
        // Configure ICE servers (STUN/TURN)
        let config = RTCConfiguration {
            ice_servers: vec![
                RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                    ..Default::default()
                },
                // TURN server removed - credentials were invalid
                // Add your own TURN server if needed for restrictive networks
            ],
            ..Default::default()
        };

        // Create peer connection
        let peer_connection = Arc::new(self.api.new_peer_connection(config).await?);

        // Set up ICE candidate event handler
        peer_connection.on_ice_candidate(Box::new(move |candidate| {
            if let Some(c) = candidate {
                let candidate_str = c.to_json().unwrap_or_default().candidate;
                let sdp_mid = c.to_json().ok().and_then(|j| j.sdp_mid);
                let sdp_mline_index = c.to_json().ok().and_then(|j| j.sdp_mline_index);
                on_ice_candidate(candidate_str, sdp_mid, sdp_mline_index);
            }
            Box::pin(async {})
        }));

        // Set up audio track event handler
        let client_tracks = Arc::clone(&self.client_tracks);
        let cid = client_id;
        
        peer_connection.on_track(Box::new(move |track, _receiver, _transceiver| {
            let tracks2 = Arc::clone(&client_tracks);
            let cid2 = cid;
            
            Box::pin(async move {
                println!("Got audio track from client {}", cid2);
                
                // Store the track for forwarding
                let mut tracks = tracks2.lock().await;
                tracks.insert(cid2, track);
            })
        }));

        // Set up connection state handler
        let cid_for_state = client_id;
        peer_connection.on_peer_connection_state_change(Box::new(move |state: RTCPeerConnectionState| {
            println!("Peer connection state changed for client {}: {:?}", cid_for_state, state);
            Box::pin(async {})
        }));

        // Store peer connection
        let mut connections = self.connections.lock().await;
        connections.insert(client_id, Arc::clone(&peer_connection));

        Ok(peer_connection)
    }

    /// Handle SDP offer from client
    pub async fn handle_offer(
        &self,
        client_id: RoomClientID,
        sdp: String,
        on_ice_candidate: impl Fn(String, Option<String>, Option<u16>) + Send + Sync + 'static,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let peer_connection = match self.get_peer_connection(client_id).await {
            Some(pc) => pc,
            None => self.create_peer_connection(client_id, on_ice_candidate).await?,
        };

        // Set remote description (the offer)
        let offer = RTCSessionDescription::offer(sdp)?;
        peer_connection.set_remote_description(offer).await?;

        // Create answer
        let answer = peer_connection.create_answer(None).await?;
        
        // Set local description (the answer)
        peer_connection.set_local_description(answer.clone()).await?;

        Ok(answer.sdp)
    }

    /// Get existing peer connection
    async fn get_peer_connection(&self, client_id: RoomClientID) -> Option<Arc<RTCPeerConnection>> {
        let connections = self.connections.lock().await;
        connections.get(&client_id).cloned()
    }

    /// Get or create peer connection for a client
    async fn get_or_create_peer_connection(
        &self,
        client_id: RoomClientID,
        on_ice_candidate: impl Fn(String, Option<String>, Option<u16>) + Send + Sync + 'static,
    ) -> Result<Arc<RTCPeerConnection>, Box<dyn std::error::Error>> {
        if let Some(pc) = self.get_peer_connection(client_id).await {
            Ok(pc)
        } else {
            self.create_peer_connection(client_id, on_ice_candidate).await
        }
    }

    /// Add ICE candidate from client
    pub async fn add_ice_candidate(
        &self,
        client_id: RoomClientID,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let peer_connection = match self.get_peer_connection(client_id).await {
            Some(pc) => pc,
            None => {
                println!("Warning: No peer connection for client {} when adding ICE candidate", client_id);
                return Ok(());
            }
        };

        let ice_candidate = webrtc::ice_transport::ice_candidate::RTCIceCandidateInit {
            candidate,
            sdp_mid,
            sdp_mline_index: sdp_m_line_index,
            username_fragment: None,
        };

        peer_connection.add_ice_candidate(ice_candidate).await?;

        Ok(())
    }

    /// Get or create peer connection for a client (legacy method, use get_or_create_peer_connection with callback)
    async fn get_or_create_peer_connection_legacy(
        &self,
        client_id: RoomClientID,
    ) -> Result<Arc<RTCPeerConnection>, Box<dyn std::error::Error>> {
        let connections = self.connections.lock().await;
        
        if let Some(pc) = connections.get(&client_id) {
            Ok(Arc::clone(pc))
        } else {
            drop(connections);
            self.create_peer_connection(client_id, |_, _, _| {}).await
        }
    }

    /// Remove peer connection for a client
    pub async fn remove_client(&self, client_id: RoomClientID) {
        let mut connections = self.connections.lock().await;
        if let Some(pc) = connections.remove(&client_id) {
            let _ = pc.close().await;
        }

        let mut tracks = self.client_tracks.lock().await;
        tracks.remove(&client_id);
        
        let mut senders = self.track_senders.lock().await;
        senders.remove(&client_id);
    }

    /// Get list of clients that should hear a specific client based on chat groups
    /// This will be integrated with the game's chat group system
    pub async fn get_recipients_for_speaker(
        &self,
        speaker_id: RoomClientID,
        _all_clients: &[RoomClientID],
    ) -> Vec<RoomClientID> {
        // TODO: Integrate with chat group permissions
        // For now, forward to all other clients (lobby behavior)
        let connections = self.connections.lock().await;
        connections
            .keys()
            .filter(|&&id| id != speaker_id)
            .copied()
            .collect()
    }
    
    /// Send ICE candidates to a specific client (called by server when peer connection generates them)
    pub fn get_ice_candidate_callback(client_id: RoomClientID, send_packet_fn: impl Fn(ToClientPacket) + Send + Sync + 'static) -> impl Fn(String, Option<String>, Option<u16>) + Send + Sync + 'static {
        move |candidate: String, sdp_mid: Option<String>, sdp_mline_index: Option<u16>| {
            send_packet_fn(ToClientPacket::WebRtcIceCandidate {
                from_player_id: client_id,
                candidate,
                sdp_mid,
                sdp_m_line_index: sdp_mline_index,
            });
        }
    }
}
