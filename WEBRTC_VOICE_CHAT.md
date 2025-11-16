# WebRTC SFU Voice Chat Implementation

## Architecture Evolution

### v1 (Attempted): Peer-to-Peer WebRTC
- Direct peer connections between clients
- **Problem**: Server couldn't control who hears whom based on chat groups

### v2 (Implemented, High Latency): Server-Mediated MediaSource
- Audio flows through WebSocket server
- MediaSource API for playback
- **Problem**: High latency (~500ms+) due to buffering overhead

### v3 (Current, In Progress): WebRTC SFU (Selective Forwarding Unit)
- WebRTC peer connections between each client and server
- Server acts as SFU: receives tracks, selectively forwards to recipients
- **Benefits**: Low latency (~50-100ms) + server control over routing
- **Status**: Infrastructure in place, needs completion

## What Has Been Implemented

### Backend (Rust/Server)

1. **Voice Chat Setting**
   - Added `voice_chat_enabled` boolean field to the `Settings` struct in `server/src/game/settings.rs`
   - Defaults to `false` (disabled)

2. **WebRTC SFU Infrastructure** (`server/src/webrtc_sfu.rs`)
   - `WebRtcSfuManager` struct using the `webrtc` Rust crate
   - Peer connection management for each client
   - SDP offer/answer handling
   - ICE candidate exchange
   - Audio track reception and storage
   - **TODO**: Wire into message handlers, implement track forwarding

3. **Packet Types**
   - `VoiceChatEnabled` - sync voice chat enabled/disabled state
   - `SetVoiceChatEnabled` - host toggles voice chat
   - `WebRtcOffer` - client sends SDP offer to server
   - `WebRtcAnswer` - server sends SDP answer to client
   - `WebRtcIceCandidate` - bidirectional ICE candidate exchange
   
4. **Message Handlers**
   - Lobby and game handlers acknowledge WebRTC packets but don't fully process yet
   - **TODO**: Call WebRtcSfuManager methods, forward tracks to recipients

### Frontend (TypeScript/React)

1. **State Management**
   - Added `voiceChatEnabled` boolean to `LobbyState` type
   - Added to `createLobbyState()` default initialization (defaults to `false`)
   - Message listener handles `voiceChatEnabled` packet to update state

2. **Voice Chat Manager** (`client/src/game/voiceChat.ts`)
   - **Current Implementation**: MediaSource-based (high latency)
     - Microphone access via `getUserMedia()`
     - Audio capture using MediaRecorder API with Opus codec
     - MediaSource API for audio playback
     - Per-player volume controls
   - **TODO**: Replace with WebRTC PeerConnection API
     - Create RTCPeerConnection to server
     - Send offer, handle answer
     - Exchange ICE candidates  
     - Add local audio track
     - Receive and play remote audio tracks
     - Much lower latency
     - Microphone mute/unmute
     - Audio buffering and playback management
   - Uses Opus codec for efficient audio compression
   - Supports echo cancellation, noise suppression, and auto gain control

3. **UI Components**
   - **VoiceChatToggle** (`client/src/components/gameModeSettings/VoiceChatToggle.tsx`):
     - Checkbox in lobby settings for host to enable/disable voice chat
     - Only visible and editable by host
   - **VoiceChatControls** (`client/src/components/VoiceChatControls.tsx`):
     - Displays in lobby when voice chat is enabled
     - Microphone toggle button (shows mic on/off state)
     - Per-player volume sliders (0-100%)
     - Dynamically updates when players join/leave
     - Lazy-loads voice chat manager to avoid loading WebRTC code when not needed

4. **Translations**
   - Added English translations in `client/src/resources/lang/en_us.json`:
     - `menu.lobby.voiceChat` - "Voice Chat"
     - `menu.lobby.voiceChatEnabled` - "Enable voice chat in lobby"
     - `voiceChat.title` - "Voice Chat"
     - `voiceChat.micOn` - "Mute"
     - `voiceChat.micOff` - "Unmute"
     - `voiceChat.playerVolumes` - "Player Volumes"

5. **Styling**
   - Created `client/src/components/voiceChatControls.css` for voice chat UI styling
   - Styled microphone button with enabled/disabled states
   - Responsive volume sliders

## Architecture

### Signaling Flow

1. When voice chat is enabled in lobby, each client:
   - Requests microphone access
   - Creates peer connections to all other players
   - Sends WebRTC offers via the server

2. Server acts as signaling relay:
   - Forwards `WebRtcSignal` packets between clients
   - Does not process audio itself (peer-to-peer)

3. Clients exchange:
   - SDP offers and answers for session negotiation
   - ICE candidates for NAT traversal (queued if remote description not yet set)
   - Audio streams flow directly between peers

4. ICE candidate handling:
   - Candidates arriving before remote description is set are queued
   - After setRemoteDescription, all queued candidates are processed
   - This prevents "Either sdpMid or sdpMLineIndex must be specified" errors

5. NAT Traversal:
   - Uses STUN servers for direct peer-to-peer connections
   - Falls back to TURN servers when direct connection fails (firewall/NAT restrictions)
   - Free TURN server from openrelay.metered.ca included for testing

### Lobby Voice Chat (Implemented)

- **Single channel**: All players in lobby can hear each other
- **Host control**: Only host can enable/disable voice chat
- **Player controls**: Each player can:
  - Mute their own microphone
  - Adjust volume of other players independently
- **Cleanup**: Voice chat automatically disables when game starts

## What Still Needs Implementation

### In-Game Voice Chat

1. **Chat Group Integration**
   - Needs to respect existing chat groups system
   - When player can send to chat group → their voice goes to that group
   - When player can read from chat group → they hear voices from that group
   - Implementation approach:
     - Track which chat groups each player can send to / read from
     - Dynamically connect/disconnect peer connections based on permissions
     - Update connections when:
       - Phase changes (day/night)
       - Player roles change chat permissions
       - Player dies (dead chat, etc.)

2. **Dynamic Channel Switching**
   - Monitor `YourSendChatGroups` packet from server
   - When chat groups change:
     - Add peer connections for new groups
     - Remove peer connections for old groups
   - Needs integration with game state chat group system

3. **Spectator Voice Chat**
   - Decide if spectators should have voice chat
   ## Next Steps to Complete WebRTC SFU

### Server-Side

1. **Wire Up WebRTC Manager to Message Handlers**
   - Create WebRtcSfuManager instance in WebsocketListener
   - Pass reference to Lobby and Game states
   - Handle `WebRtcOffer`: call `manager.handle_offer()`, send back answer
   - Handle `WebRtcIceCandidate`: call `manager.add_ice_candidate()`
   - Send server's ICE candidates to clients

2. **Implement Track Forwarding Logic**
   - When a client's track is received, determine recipients based on chat groups
   - Use `manager.get_recipients_for_speaker()` with chat group filtering
   - Forward tracks to appropriate recipients
   - Integrate with game's existing chat group system

3. **Handle Client Disconnection**
   - Call `manager.remove_client()` when client leaves
   - Close peer connections and clean up resources

### Client-Side

1. **Replace MediaSource with WebRTC PeerConnection**
   - Create `RTCPeerConnection` instance to server
   - Configure with STUN servers (Google STUN)
   - Add local audio track from `getUserMedia()`
   - Generate SDP offer, send to server via `WebRtcOffer` packet
   - Handle server's SDP answer
   - Send ICE candidates to server via `WebRtcIceCandidate` packet
   - Handle ICE candidates from server

2. **Handle Remote Audio Tracks**
   - Listen for `ontrack` event
   - Receive remote audio tracks from server
   - Play tracks with volume controls
   - Map tracks to player IDs

3. **Remove MediaSource Code**
   - Delete MediaRecorder audio chunk sending
   - Delete MediaSource playback logic
   - Keep volume control UI (works with WebRTC tracks too)

### Integration & Testing

1. **Chat Group Filtering**
   - Implement `get_recipients_for_speaker()` using game's chat group system
   - Mafia hears Mafia at night
   - Dead hear dead
   - Jailor and jailed hear each other
   - Etc.

2. **Testing**
   - Multi-client local testing
   - Different network conditions
   - Various chat group scenarios
   - Performance testing

## Current State Summary

**What Works:**
- ✅ Voice chat toggle in lobby settings
- ✅ Microphone capture and MediaSource playback (high latency)
- ✅ Per-player volume controls
- ✅ WebRTC SFU infrastructure on server
- ✅ Packet types defined

**What Needs Work:**
- ⏳ Wire WebRTC manager to message handlers
- ⏳ Implement track forwarding logic
- ⏳ Replace client MediaSource with WebRTC PeerConnection
- ⏳ Chat group filtering for in-game voice
- ⏳ Testing with multiple clients

**Estimated Remaining Work:**
- Server wiring: ~2-3 hours
- Client WebRTC implementation: ~3-4 hours
- Testing & debugging: ~2-3 hours
- **Total: ~7-10 hours for complete WebRTC SFU**

## Alternative: Optimize Current Approach

If full SFU is too complex, the current MediaSource approach can be optimized:

1. Reduce MediaRecorder chunk interval: 100ms → 20ms
2. Reduce MediaSource buffer size
3. Add jitter buffer tuning
4. **Expected latency improvement: ~200-300ms** (vs current ~500ms+)
5. **Time required: ~1-2 hours**

This won't be as good as WebRTC SFU (~50-100ms) but is much simpler.

## Testing Locally

To test voice chat:

1. Build and run the server:
   ```bash
   cd server
   cargo run
   ```

2. Build and run the client:
   ```bash
   cd client
   pnpm install
   pnpm dev
   ```

3. Open multiple browser tabs/windows (localhost:port)
4. Create a lobby as host
5. In lobby settings, enable "Voice Chat"
6. Join with other tabs/windows
7. Test microphone toggle and volume controls
8. Verify audio flows between clients

## Future Enhancements

- Push-to-talk mode
- Voice activity detection with visual indicators
- Spatial audio (positional audio based on game mechanics)
- Voice chat in different languages/accents detection
- Recording/replay of voice chat (with consent)
- Admin/host mute capabilities
- Bandwidth usage controls
- Audio quality settings

## Files Modified/Created

### Server
- `server/src/game/settings.rs` - Added voice_chat_enabled field
- `server/src/packet.rs` - Added WebRTC packets
- `server/src/lobby/mod.rs` - Send voice chat setting
- `server/src/lobby/on_client_message.rs` - Handle voice chat messages
- `server/src/game/on_client_message.rs` - Forward WebRTC signals in-game

### Client
- `client/src/game/gameState.d.tsx` - Added voiceChatEnabled to LobbyState
- `client/src/game/gameState.tsx` - Initialize voice chat setting
- `client/src/game/packet.tsx` - Added WebRTC packet types
- `client/src/game/messageListener.tsx` - Handle voice chat packets, cleanup on game start
- `client/src/game/voiceChat.ts` - WebRTC connection manager (NEW)
- `client/src/components/VoiceChatControls.tsx` - Voice chat UI controls (NEW)
- `client/src/components/voiceChatControls.css` - Voice chat styling (NEW)
- `client/src/components/gameModeSettings/VoiceChatToggle.tsx` - Host toggle (NEW)
- `client/src/menu/lobby/LobbyMenu.tsx` - Integrate voice chat UI
- `client/src/resources/lang/en_us.json` - Add translations
