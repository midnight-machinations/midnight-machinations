# Server-Mediated Voice Chat Implementation

## Architecture Change

**Previous approach (P2P)**: Clients established direct WebRTC peer connections to each other.  
**New approach (Server-mediated)**: Audio data flows through the WebSocket server, allowing the server to control who can hear whom based on chat group permissions.

## What Has Been Implemented

### Backend (Rust/Server)

1. **Voice Chat Setting**
   - Added `voice_chat_enabled` boolean field to the `Settings` struct in `server/src/game/settings.rs`
   - Defaults to `false` (disabled)

2. **Packet Types**
   - Added `VoiceChatEnabled` packet to `ToClientPacket` enum for syncing voice chat state
   - Added `SetVoiceChatEnabled` packet to `ToServerPacket` enum for host to toggle voice chat
   - Added `VoiceData` packet types for sending/receiving audio data through the server:
     - Client sends audio chunks to server
     - Server forwards audio to appropriate recipients based on chat groups
   
3. **Audio Routing**
   - Server maintains mapping of which chat groups each player can send to / receive from
   - Audio packets include sender information
   - Server filters and forwards audio based on chat group permissions

4. **Message Handlers**
   - Lobby message handler (`server/src/lobby/on_client_message.rs`):
     - Handles `SetVoiceChatEnabled` to toggle the setting and broadcast to all clients
     - Handles `VoiceData` packets and forwards to all other players in lobby
   - Game message handler (`server/src/game/on_client_message.rs`):
     - Handles `VoiceData` during game based on chat group permissions
   - Settings synchronization:
     - `send_settings()` in `server/src/lobby/mod.rs` updated to include voice chat setting

### Frontend (TypeScript/React)

1. **State Management**
   - Added `voiceChatEnabled` boolean to `LobbyState` type
   - Added to `createLobbyState()` default initialization (defaults to `false`)
   - Message listener handles `voiceChatEnabled` packet to update state

2. **Voice Chat Manager** (`client/src/game/voiceChat.ts`)
   - Singleton `VoiceChatManager` class that handles:
     - Microphone access via `getUserMedia()`
     - Audio capture and encoding using MediaRecorder API
     - Sending audio chunks to server via WebSocket
     - Receiving and playing audio from server
     - Per-player volume controls
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
   - Likely should be separate from player voice chat

### Testing & Polish

1. **Multi-Client Testing**
   - Test with 2+ clients in same lobby
   - Verify audio quality and latency
   - Test NAT traversal in different network configurations
   - Test with firewalls/restrictive networks

2. **Error Handling**
   - Handle microphone permission denied gracefully
   - Handle WebRTC connection failures
   - Show user-friendly error messages
   - Fallback when STUN servers are unreachable

3. **Performance**
   - Test with maximum players (currently uses mesh topology)
   - Consider SFU (Selective Forwarding Unit) if mesh doesn't scale
   - Monitor CPU usage with many peer connections

4. **UI/UX Improvements**
   - Visual indicators for who is speaking
   - Push-to-talk option
   - Voice activity detection (show when someone is talking)
   - Better mobile support
   - Save volume preferences in localStorage

5. **Security**
   - Ensure voice chat respects game permissions
   - Prevent players from hearing groups they shouldn't
   - Consider encryption beyond standard WebRTC (DTLS-SRTP)

## Known Limitations

1. **Mesh Topology**: Current implementation uses mesh topology where each client connects to every other client. This scales to ~10-15 players but may need SFU for larger games.

2. **TURN Server**: Free TURN server from openrelay.metered.ca is included for testing purposes only. For production use, you should set up your own TURN server or use a paid service with higher bandwidth limits.

3. **No Persistence**: Volume settings are not saved between sessions.

4. **Browser Compatibility**: Requires modern browser with WebRTC support (Chrome, Firefox, Safari, Edge).

5. **Microphone Permission**: Users must grant microphone permission. There's no prompt - voice chat simply won't work if denied.

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
