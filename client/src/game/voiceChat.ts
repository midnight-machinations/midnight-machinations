import GAME_MANAGER from "../index";
import { LobbyClientID } from "./gameState.d";
import { WebRtcSignalData } from "./packet";

/**
 * Manages WebRTC peer connections for voice chat
 */
class VoiceChatManager {
    private peerConnections: Map<LobbyClientID, RTCPeerConnection> = new Map();
    private localStream: MediaStream | null = null;
    private remoteStreams: Map<LobbyClientID, MediaStream> = new Map();
    private audioElements: Map<LobbyClientID, HTMLAudioElement> = new Map();
    private volumeSettings: Map<LobbyClientID, number> = new Map();
    private pendingIceCandidates: Map<LobbyClientID, RTCIceCandidateInit[]> = new Map();
    private enabled: boolean = false;
    private micEnabled: boolean = false;

    // ICE servers configuration (using free Google STUN servers and a free TURN server)
    private iceServers: RTCConfiguration = {
        iceServers: [
            { urls: 'stun:stun.l.google.com:19302' },
            { urls: 'stun:stun1.l.google.com:19302' },
            // Free TURN server from metered.ca (limited bandwidth, suitable for testing)
            {
                urls: 'turn:openrelay.metered.ca:80',
                username: 'openrelayproject',
                credential: 'openrelayproject',
            },
            {
                urls: 'turn:openrelay.metered.ca:443',
                username: 'openrelayproject',
                credential: 'openrelayproject',
            },
            {
                urls: 'turn:openrelay.metered.ca:443?transport=tcp',
                username: 'openrelayproject',
                credential: 'openrelayproject',
            },
        ]
    };

    constructor() {
        // Bind methods to ensure correct 'this' context
        this.handleSignal = this.handleSignal.bind(this);
    }

    /**
     * Initialize voice chat - request microphone access
     */
    async initialize(): Promise<boolean> {
        if (this.localStream) {
            return true; // Already initialized
        }

        try {
            this.localStream = await navigator.mediaDevices.getUserMedia({ 
                audio: {
                    echoCancellation: true,
                    noiseSuppression: true,
                    autoGainControl: true
                }, 
                video: false 
            });
            
            // Mute by default
            this.localStream.getAudioTracks().forEach(track => {
                track.enabled = false;
            });
            this.micEnabled = false;

            console.log("Voice chat initialized successfully");
            return true;
        } catch (error) {
            console.error("Failed to get user media:", error);
            return false;
        }
    }

    /**
     * Enable voice chat and connect to all players
     */
    async enable(playerIds: LobbyClientID[]): Promise<void> {
        if (!await this.initialize()) {
            console.error("Cannot enable voice chat - microphone access denied");
            return;
        }

        this.enabled = true;

        // Get current player ID safely
        const myId = GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game" 
            ? GAME_MANAGER.state.myId 
            : null;

        // Create peer connections for all other players
        for (const playerId of playerIds) {
            if (playerId !== myId) {
                await this.createPeerConnection(playerId, true);
            }
        }
    }

    /**
     * Disable voice chat and close all connections
     */
    disable(): void {
        this.enabled = false;
        this.closeAllConnections();
    }

    /**
     * Create a peer connection with another player
     */
    private async createPeerConnection(playerId: LobbyClientID, initiator: boolean): Promise<void> {
        // Don't create duplicate connections
        if (this.peerConnections.has(playerId)) {
            return;
        }

        const pc = new RTCPeerConnection(this.iceServers);
        this.peerConnections.set(playerId, pc);

        // Add local stream tracks to the connection
        if (this.localStream) {
            this.localStream.getTracks().forEach(track => {
                pc.addTrack(track, this.localStream!);
            });
        }

        // Handle incoming tracks
        pc.ontrack = (event) => {
            console.log(`Received track from player ${playerId}`);
            const stream = event.streams[0];
            this.remoteStreams.set(playerId, stream);
            this.playRemoteStream(playerId, stream);
        };

        // Handle ICE candidates
        pc.onicecandidate = (event) => {
            if (event.candidate) {
                console.log(`Sending ICE candidate to player ${playerId}`);
                GAME_MANAGER.server.sendPacket({
                    type: "webRtcSignal",
                    targetPlayerId: playerId,
                    signal: {
                        type: "iceCandidate",
                        candidate: event.candidate.candidate,
                        sdpMid: event.candidate.sdpMid,
                        sdpMLineIndex: event.candidate.sdpMLineIndex
                    }
                });
            }
        };

        // Handle connection state changes
        pc.onconnectionstatechange = () => {
            console.log(`Connection state with player ${playerId}: ${pc.connectionState}`);
            if (pc.connectionState === 'failed' || pc.connectionState === 'disconnected' || pc.connectionState === 'closed') {
                this.removePeerConnection(playerId);
            }
        };

        // If we're the initiator, create and send an offer
        if (initiator) {
            try {
                const offer = await pc.createOffer();
                await pc.setLocalDescription(offer);
                
                console.log(`Sending offer to player ${playerId}`);
                GAME_MANAGER.server.sendPacket({
                    type: "webRtcSignal",
                    targetPlayerId: playerId,
                    signal: {
                        type: "offer",
                        sdp: offer.sdp!
                    }
                });
            } catch (error) {
                console.error(`Error creating offer for player ${playerId}:`, error);
            }
        }
    }

    /**
     * Handle incoming WebRTC signaling messages
     */
    async handleSignal(fromPlayerId: LobbyClientID, signal: WebRtcSignalData): Promise<void> {
        if (!this.enabled) {
            console.log("Ignoring signal - voice chat is disabled");
            return;
        }

        let pc = this.peerConnections.get(fromPlayerId);

        switch (signal.type) {
            case "offer":
                // Create peer connection if it doesn't exist
                if (!pc) {
                    await this.createPeerConnection(fromPlayerId, false);
                    pc = this.peerConnections.get(fromPlayerId);
                }

                if (pc) {
                    try {
                        await pc.setRemoteDescription(new RTCSessionDescription({
                            type: "offer",
                            sdp: signal.sdp
                        }));

                        // Process any pending ICE candidates
                        const pending = this.pendingIceCandidates.get(fromPlayerId);
                        if (pending && pending.length > 0) {
                            console.log(`Processing ${pending.length} pending ICE candidates for player ${fromPlayerId}`);
                            for (const candidate of pending) {
                                try {
                                    await pc.addIceCandidate(new RTCIceCandidate(candidate));
                                } catch (error) {
                                    console.error(`Error adding pending ICE candidate:`, error);
                                }
                            }
                            this.pendingIceCandidates.delete(fromPlayerId);
                        }

                        const answer = await pc.createAnswer();
                        await pc.setLocalDescription(answer);

                        console.log(`Sending answer to player ${fromPlayerId}`);
                        GAME_MANAGER.server.sendPacket({
                            type: "webRtcSignal",
                            targetPlayerId: fromPlayerId,
                            signal: {
                                type: "answer",
                                sdp: answer.sdp!
                            }
                        });
                    } catch (error) {
                        console.error(`Error handling offer from player ${fromPlayerId}:`, error);
                    }
                }
                break;

            case "answer":
                if (pc) {
                    try {
                        await pc.setRemoteDescription(new RTCSessionDescription({
                            type: "answer",
                            sdp: signal.sdp
                        }));
                        console.log(`Set remote description (answer) from player ${fromPlayerId}`);
                        
                        // Process any pending ICE candidates
                        const pending = this.pendingIceCandidates.get(fromPlayerId);
                        if (pending && pending.length > 0) {
                            console.log(`Processing ${pending.length} pending ICE candidates for player ${fromPlayerId}`);
                            for (const candidate of pending) {
                                try {
                                    await pc.addIceCandidate(new RTCIceCandidate(candidate));
                                } catch (error) {
                                    console.error(`Error adding pending ICE candidate:`, error);
                                }
                            }
                            this.pendingIceCandidates.delete(fromPlayerId);
                        }
                    } catch (error) {
                        console.error(`Error handling answer from player ${fromPlayerId}:`, error);
                    }
                }
                break;

            case "iceCandidate":
                if (pc) {
                    try {
                        // Build ICE candidate init object, only including non-null values
                        const candidateInit: RTCIceCandidateInit = {
                            candidate: signal.candidate
                        };
                        if (signal.sdpMid !== null) {
                            candidateInit.sdpMid = signal.sdpMid;
                        }
                        if (signal.sdpMLineIndex !== null) {
                            candidateInit.sdpMLineIndex = signal.sdpMLineIndex;
                        }
                        
                        // Check if remote description is set
                        if (pc.remoteDescription) {
                            await pc.addIceCandidate(new RTCIceCandidate(candidateInit));
                            console.log(`Added ICE candidate from player ${fromPlayerId}`);
                        } else {
                            // Queue the candidate to be added after remote description is set
                            console.log(`Queueing ICE candidate from player ${fromPlayerId} (no remote description yet)`);
                            if (!this.pendingIceCandidates.has(fromPlayerId)) {
                                this.pendingIceCandidates.set(fromPlayerId, []);
                            }
                            this.pendingIceCandidates.get(fromPlayerId)!.push(candidateInit);
                        }
                    } catch (error) {
                        console.error(`Error adding ICE candidate from player ${fromPlayerId}:`, error);
                    }
                }
                break;
        }
    }

    /**
     * Play a remote audio stream
     */
    private playRemoteStream(playerId: LobbyClientID, stream: MediaStream): void {
        // Remove existing audio element if any
        const existingAudio = this.audioElements.get(playerId);
        if (existingAudio) {
            existingAudio.pause();
            existingAudio.srcObject = null;
            existingAudio.remove();
        }

        // Create new audio element
        const audio = new Audio();
        audio.srcObject = stream;
        audio.autoplay = true;
        
        // Apply volume setting if exists
        const volume = this.volumeSettings.get(playerId) ?? 1.0;
        audio.volume = volume;

        this.audioElements.set(playerId, audio);
        
        // Start playing
        audio.play().catch(error => {
            console.error(`Error playing audio from player ${playerId}:`, error);
        });
    }

    /**
     * Remove a peer connection
     */
    private removePeerConnection(playerId: LobbyClientID): void {
        const pc = this.peerConnections.get(playerId);
        if (pc) {
            pc.close();
            this.peerConnections.delete(playerId);
        }

        const audio = this.audioElements.get(playerId);
        if (audio) {
            audio.pause();
            audio.srcObject = null;
            audio.remove();
            this.audioElements.delete(playerId);
        }

        this.remoteStreams.delete(playerId);
        this.pendingIceCandidates.delete(playerId);
    }

    /**
     * Close all peer connections
     */
    private closeAllConnections(): void {
        for (const playerId of this.peerConnections.keys()) {
            this.removePeerConnection(playerId);
        }
        this.pendingIceCandidates.clear();
    }

    /**
     * Set volume for a specific player
     */
    setPlayerVolume(playerId: LobbyClientID, volume: number): void {
        // Clamp volume between 0 and 1
        volume = Math.max(0, Math.min(1, volume));
        
        this.volumeSettings.set(playerId, volume);
        
        const audio = this.audioElements.get(playerId);
        if (audio) {
            audio.volume = volume;
        }
    }

    /**
     * Get volume for a specific player
     */
    getPlayerVolume(playerId: LobbyClientID): number {
        return this.volumeSettings.get(playerId) ?? 1.0;
    }

    /**
     * Toggle microphone on/off
     */
    toggleMicrophone(): boolean {
        if (!this.localStream) {
            return false;
        }

        this.micEnabled = !this.micEnabled;
        this.localStream.getAudioTracks().forEach(track => {
            track.enabled = this.micEnabled;
        });

        return this.micEnabled;
    }

    /**
     * Set microphone enabled state
     */
    setMicrophoneEnabled(enabled: boolean): void {
        if (!this.localStream) {
            return;
        }

        this.micEnabled = enabled;
        this.localStream.getAudioTracks().forEach(track => {
            track.enabled = enabled;
        });
    }

    /**
     * Check if microphone is enabled
     */
    isMicrophoneEnabled(): boolean {
        return this.micEnabled;
    }

    /**
     * Check if voice chat is enabled
     */
    isEnabled(): boolean {
        return this.enabled;
    }

    /**
     * Add a new player to voice chat
     */
    async addPlayer(playerId: LobbyClientID): Promise<void> {
        if (!this.enabled) {
            return;
        }

        const myId = GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game" 
            ? GAME_MANAGER.state.myId 
            : null;

        if (playerId !== myId) {
            await this.createPeerConnection(playerId, true);
        }
    }

    /**
     * Remove a player from voice chat
     */
    removePlayer(playerId: LobbyClientID): void {
        this.removePeerConnection(playerId);
    }

    /**
     * Clean up resources
     */
    cleanup(): void {
        this.closeAllConnections();
        
        if (this.localStream) {
            this.localStream.getTracks().forEach(track => track.stop());
            this.localStream = null;
        }

        this.enabled = false;
        this.micEnabled = false;
    }
}

// Export singleton instance
export const voiceChatManager = new VoiceChatManager();
