import GAME_MANAGER from "../index";
import { LobbyClientID } from "./gameState.d";

/**
 * Manages WebRTC SFU voice chat
 * Uses WebRTC peer connection to server for low-latency audio
 */
class VoiceChatManager {
    private localStream: MediaStream | null = null;
    private peerConnection: RTCPeerConnection | null = null;
    private playerVolumes: Map<LobbyClientID, number> = new Map();
    private remoteAudioElements: Map<LobbyClientID, HTMLAudioElement> = new Map();
    private enabled: boolean = false;
    private micEnabled: boolean = false;
    private iceCandidateQueue: RTCIceCandidateInit[] = [];
    private remoteDescriptionSet: boolean = false;

    constructor() {
        this.handleWebRtcSignal = this.handleWebRtcSignal.bind(this);
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
                    autoGainControl: true,
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
     * Enable voice chat and start capturing/sending audio
     */
    async enable(playerIds: LobbyClientID[]): Promise<void> {
        if (!await this.initialize()) {
            console.error("Cannot enable voice chat - microphone access denied");
            return;
        }

        this.enabled = true;

        // Create WebRTC peer connection to server
        await this.setupPeerConnection();

        console.log("Voice chat enabled");
    }

    /**
     * Disable voice chat and stop all audio processing
     */
    disable(): void {
        this.enabled = false;
        this.closePeerConnection();
        this.clearAllAudioElements();
        console.log("Voice chat disabled");
    }

    /**
     * Set up WebRTC peer connection to server
     */
    private async setupPeerConnection(): Promise<void> {
        if (!this.localStream) {
            return;
        }

        try {
            // Create peer connection with STUN/TURN servers
            const configuration: RTCConfiguration = {
                iceServers: [
                    { urls: 'stun:stun.l.google.com:19302' },
                    {
                        urls: 'turn:openrelay.metered.ca:80',
                        username: 'openrelayproject',
                        credential: 'openrelayproject'
                    }
                ]
            };

            this.peerConnection = new RTCPeerConnection(configuration);

            // Add local audio track to peer connection
            this.localStream.getTracks().forEach(track => {
                if (this.peerConnection && this.localStream) {
                    this.peerConnection.addTrack(track, this.localStream);
                }
            });

            // Handle ICE candidates
            this.peerConnection.onicecandidate = (event) => {
                if (event.candidate) {
                    console.log("Sending ICE candidate to server");
                    GAME_MANAGER.server.sendPacket({
                        type: "webRtcIceCandidate",
                        candidate: event.candidate.candidate,
                        sdpMid: event.candidate.sdpMid,
                        sdpMLineIndex: event.candidate.sdpMLineIndex
                    });
                }
            };

            // Handle remote tracks (audio from other players)
            this.peerConnection.ontrack = (event) => {
                console.log("Received remote track");
                // Remote tracks will be associated with players via track events
                // For now, play all remote audio
                const audio = new Audio();
                audio.srcObject = event.streams[0];
                audio.play().catch(err => console.error("Error playing remote audio:", err));
            };

            // Handle connection state changes
            this.peerConnection.onconnectionstatechange = () => {
                if (this.peerConnection) {
                    console.log("Peer connection state:", this.peerConnection.connectionState);
                }
            };

            // Create and send offer to server
            const offer = await this.peerConnection.createOffer();
            await this.peerConnection.setLocalDescription(offer);

            console.log("Sending offer to server");
            GAME_MANAGER.server.sendPacket({
                type: "webRtcOffer",
                sdp: offer.sdp || ""
            });

        } catch (error) {
            console.error("Failed to setup peer connection:", error);
        }
    }

    /**
     * Handle WebRTC signaling from server (offers, answers, ICE candidates)
     */
    async handleWebRtcSignal(type: "offer" | "answer" | "iceCandidate", data: any): Promise<void> {
        if (!this.peerConnection) {
            console.warn("Received WebRTC signal but no peer connection exists");
            return;
        }

        try {
            if (type === "answer") {
                // Server sent answer to our offer
                const answer = new RTCSessionDescription({
                    type: "answer",
                    sdp: data.sdp
                });
                await this.peerConnection.setRemoteDescription(answer);
                this.remoteDescriptionSet = true;
                console.log("Set remote description (answer from server)");

                // Process queued ICE candidates
                while (this.iceCandidateQueue.length > 0) {
                    const candidate = this.iceCandidateQueue.shift()!;
                    await this.peerConnection.addIceCandidate(candidate);
                }
            } else if (type === "iceCandidate") {
                // Server sent ICE candidate
                const candidate: RTCIceCandidateInit = {
                    candidate: data.candidate,
                    sdpMid: data.sdpMid,
                    sdpMLineIndex: data.sdpMLineIndex
                };

                if (this.remoteDescriptionSet) {
                    await this.peerConnection.addIceCandidate(candidate);
                    console.log("Added ICE candidate from server");
                } else {
                    // Queue candidates until remote description is set
                    this.iceCandidateQueue.push(candidate);
                    console.log("Queued ICE candidate from server");
                }
            }
        } catch (error) {
            console.error("Error handling WebRTC signal:", error);
        }
    }

    /**
     * Close peer connection
     */
    private closePeerConnection(): void {
        if (this.peerConnection) {
            this.peerConnection.close();
            this.peerConnection = null;
        }
        this.remoteDescriptionSet = false;
        this.iceCandidateQueue = [];
    }

    /**
     * Clear all audio elements
     */
    private clearAllAudioElements(): void {
        for (const [playerId, audio] of this.remoteAudioElements.entries()) {
            audio.pause();
            audio.srcObject = null;
        }
        this.remoteAudioElements.clear();
        this.playerVolumes.clear();
    }

    /**
     * Set volume for a specific player
     */
    setPlayerVolume(playerId: LobbyClientID, volume: number): void {
        // Clamp volume between 0 and 1
        volume = Math.max(0, Math.min(1, volume));
        
        // Store volume setting
        this.playerVolumes.set(playerId, volume);
        
        // Apply to existing audio element if it exists
        const audio = this.remoteAudioElements.get(playerId);
        if (audio) {
            audio.volume = volume;
        }
    }

    /**
     * Get volume for a specific player
     */
    getPlayerVolume(playerId: LobbyClientID): number {
        return this.playerVolumes.get(playerId) ?? 1.0;
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

        console.log(`Microphone ${this.micEnabled ? 'enabled' : 'disabled'}`);
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
     * Add a new player to voice chat (no-op for server-mediated)
     */
    async addPlayer(playerId: LobbyClientID): Promise<void> {
        // Server handles routing, nothing to do on client
        console.log(`Player ${playerId} added to voice chat`);
    }

    /**
     * Remove a player from voice chat
     */
    removePlayer(playerId: LobbyClientID): void {
        // Clean up audio element for this player
        const audio = this.remoteAudioElements.get(playerId);
        if (audio) {
            audio.pause();
            audio.srcObject = null;
            this.remoteAudioElements.delete(playerId);
        }
        
        // Clean up volume setting for this player
        this.playerVolumes.delete(playerId);

        console.log(`Player ${playerId} removed from voice chat`);
    }

    /**
     * Clean up resources
     */
    cleanup(): void {
        this.closePeerConnection();
        this.clearAllAudioElements();
        
        if (this.localStream) {
            this.localStream.getTracks().forEach(track => track.stop());
            this.localStream = null;
        }

        this.enabled = false;
        this.micEnabled = false;

        console.log("Voice chat cleaned up");
    }
}

// Export singleton instance
export const voiceChatManager = new VoiceChatManager();
