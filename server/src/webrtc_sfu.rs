use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::APIBuilder;
use webrtc::ice_transport::ice_server::RTCIceServer;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::sdp::session_description::RTCSessionDescription;
use webrtc::peer_connection::RTCPeerConnection;
use webrtc::track::track_remote::TrackRemote;

use crate::room::RoomClientID;

/// Manages WebRTC peer connections for voice chat SFU
pub struct WebRtcSfuManager {
    /// WebRTC API instance
    api: Arc<webrtc::api::API>,
    /// Peer connections for each client
    connections: Arc<Mutex<HashMap<RoomClientID, Arc<RTCPeerConnection>>>>,
    /// Audio tracks from each client (for forwarding)
    client_tracks: Arc<Mutex<HashMap<RoomClientID, Arc<TrackRemote>>>>,
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
        })
    }

    /// Create a peer connection for a client
    pub async fn create_peer_connection(
        &self,
        client_id: RoomClientID,
    ) -> Result<Arc<RTCPeerConnection>, Box<dyn std::error::Error>> {
        // Configure ICE servers (STUN/TURN)
        let config = RTCConfiguration {
            ice_servers: vec![
                RTCIceServer {
                    urls: vec!["stun:stun.l.google.com:19302".to_owned()],
                    ..Default::default()
                },
                RTCIceServer {
                    urls: vec!["turn:openrelay.metered.ca:80".to_owned()],
                    username: "openrelayproject".to_owned(),
                    credential: "openrelayproject".to_owned(),
                    ..Default::default()
                },
            ],
            ..Default::default()
        };

        // Create peer connection
        let peer_connection = Arc::new(self.api.new_peer_connection(config).await?);

        // Set up event handlers
        let _pc = Arc::clone(&peer_connection);
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
    ) -> Result<String, Box<dyn std::error::Error>> {
        let peer_connection = self.get_or_create_peer_connection(client_id).await?;

        // Set remote description (the offer)
        let offer = RTCSessionDescription::offer(sdp)?;
        peer_connection.set_remote_description(offer).await?;

        // Create answer
        let answer = peer_connection.create_answer(None).await?;
        
        // Set local description (the answer)
        peer_connection.set_local_description(answer.clone()).await?;

        Ok(answer.sdp)
    }

    /// Add ICE candidate from client
    pub async fn add_ice_candidate(
        &self,
        client_id: RoomClientID,
        candidate: String,
        sdp_mid: Option<String>,
        sdp_m_line_index: Option<u16>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let peer_connection = self.get_or_create_peer_connection(client_id).await?;

        let ice_candidate = webrtc::ice_transport::ice_candidate::RTCIceCandidateInit {
            candidate,
            sdp_mid,
            sdp_mline_index: sdp_m_line_index,
            username_fragment: None,
        };

        peer_connection.add_ice_candidate(ice_candidate).await?;

        Ok(())
    }

    /// Get or create peer connection for a client
    async fn get_or_create_peer_connection(
        &self,
        client_id: RoomClientID,
    ) -> Result<Arc<RTCPeerConnection>, Box<dyn std::error::Error>> {
        let connections = self.connections.lock().await;
        
        if let Some(pc) = connections.get(&client_id) {
            Ok(Arc::clone(pc))
        } else {
            drop(connections);
            self.create_peer_connection(client_id).await
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
}
