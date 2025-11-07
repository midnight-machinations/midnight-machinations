import GAME_MANAGER from "../index";
import { LobbyClientID } from "./gameState.d";

/**
 * Manages server-mediated voice chat
 * Audio flows through the WebSocket server instead of peer-to-peer
 */
class VoiceChatManager {
    private localStream: MediaStream | null = null;
    private mediaRecorder: MediaRecorder | null = null;
    private audioContext: AudioContext | null = null;
    private audioBuffers: Map<LobbyClientID, AudioBuffer[]> = new Map();
    private audioSources: Map<LobbyClientID, AudioBufferSourceNode> = new Map();
    private volumeSettings: Map<LobbyClientID, GainNode> = new Map();
    private enabled: boolean = false;
    private micEnabled: boolean = false;
    private sequence: number = 0;

    constructor() {
        this.handleVoiceData = this.handleVoiceData.bind(this);
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
                    sampleRate: 48000,
                }, 
                video: false 
            });
            
            // Initialize AudioContext for playback
            this.audioContext = new AudioContext({ sampleRate: 48000 });

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

        // Start recording and sending audio
        this.startAudioCapture();

        console.log("Voice chat enabled");
    }

    /**
     * Disable voice chat and stop all audio processing
     */
    disable(): void {
        this.enabled = false;
        this.stopAudioCapture();
        this.clearAllAudioBuffers();
        console.log("Voice chat disabled");
    }

    /**
     * Start capturing audio and sending it to the server
     */
    private startAudioCapture(): void {
        if (!this.localStream) {
            return;
        }

        try {
            // Use MediaRecorder with Opus codec for efficient compression
            const options: MediaRecorderOptions = {
                mimeType: 'audio/webm;codecs=opus',
                audioBitsPerSecond: 24000, // 24 kbps for voice
            };

            this.mediaRecorder = new MediaRecorder(this.localStream, options);

            // Send audio data chunks to server
            this.mediaRecorder.ondataavailable = (event) => {
                if (event.data.size > 0 && this.micEnabled && this.enabled) {
                    // Convert Blob to ArrayBuffer
                    event.data.arrayBuffer().then(arrayBuffer => {
                        const audioData = Array.from(new Uint8Array(arrayBuffer));
                        
                        GAME_MANAGER.server.sendPacket({
                            type: "voiceData",
                            audioData,
                            sequence: this.sequence++
                        });
                    });
                }
            };

            this.mediaRecorder.onerror = (event) => {
                console.error("MediaRecorder error:", event);
            };

            // Request data every 100ms for low latency
            this.mediaRecorder.start(100);

            console.log("Audio capture started");
        } catch (error) {
            console.error("Failed to start audio capture:", error);
        }
    }

    /**
     * Stop capturing audio
     */
    private stopAudioCapture(): void {
        if (this.mediaRecorder && this.mediaRecorder.state !== 'inactive') {
            this.mediaRecorder.stop();
            this.mediaRecorder = null;
        }
    }

    /**
     * Handle incoming voice data from server
     */
    async handleVoiceData(fromPlayerId: LobbyClientID, audioData: number[], sequence: number): Promise<void> {
        if (!this.enabled || !this.audioContext) {
            return;
        }

        try {
            // Convert number array back to Uint8Array
            const uint8Array = new Uint8Array(audioData);
            
            // Create a Blob from the data
            const blob = new Blob([uint8Array], { type: 'audio/webm;codecs=opus' });
            
            // Decode audio data
            const arrayBuffer = await blob.arrayBuffer();
            const audioBuffer = await this.audioContext.decodeAudioData(arrayBuffer);

            // Play the audio
            this.playAudioBuffer(fromPlayerId, audioBuffer);

        } catch (error) {
            console.error(`Error handling voice data from player ${fromPlayerId}:`, error);
        }
    }

    /**
     * Play an audio buffer for a specific player
     */
    private playAudioBuffer(playerId: LobbyClientID, audioBuffer: AudioBuffer): void {
        if (!this.audioContext) {
            return;
        }

        try {
            // Create audio source
            const source = this.audioContext.createBufferSource();
            source.buffer = audioBuffer;

            // Create or get gain node for volume control
            let gainNode = this.volumeSettings.get(playerId);
            if (!gainNode) {
                gainNode = this.audioContext.createGain();
                gainNode.gain.value = 1.0; // Default volume
                gainNode.connect(this.audioContext.destination);
                this.volumeSettings.set(playerId, gainNode);
            }

            // Connect and play
            source.connect(gainNode);
            source.start(0);

            // Clean up after playback
            source.onended = () => {
                source.disconnect();
            };

        } catch (error) {
            console.error(`Error playing audio from player ${playerId}:`, error);
        }
    }

    /**
     * Clear all audio buffers
     */
    private clearAllAudioBuffers(): void {
        this.audioBuffers.clear();
        
        // Stop all active audio sources
        for (const source of this.audioSources.values()) {
            try {
                source.stop();
                source.disconnect();
            } catch (e) {
                // Ignore errors from already stopped sources
            }
        }
        this.audioSources.clear();

        // Disconnect gain nodes
        for (const gainNode of this.volumeSettings.values()) {
            try {
                gainNode.disconnect();
            } catch (e) {
                // Ignore errors
            }
        }
        this.volumeSettings.clear();
    }

    /**
     * Set volume for a specific player
     */
    setPlayerVolume(playerId: LobbyClientID, volume: number): void {
        // Clamp volume between 0 and 1
        volume = Math.max(0, Math.min(1, volume));
        
        let gainNode = this.volumeSettings.get(playerId);
        if (!gainNode && this.audioContext) {
            gainNode = this.audioContext.createGain();
            gainNode.connect(this.audioContext.destination);
            this.volumeSettings.set(playerId, gainNode);
        }
        
        if (gainNode) {
            gainNode.gain.value = volume;
        }
    }

    /**
     * Get volume for a specific player
     */
    getPlayerVolume(playerId: LobbyClientID): number {
        const gainNode = this.volumeSettings.get(playerId);
        return gainNode ? gainNode.gain.value : 1.0;
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
        // Clean up any audio state for this player
        this.audioBuffers.delete(playerId);
        
        const source = this.audioSources.get(playerId);
        if (source) {
            try {
                source.stop();
                source.disconnect();
            } catch (e) {
                // Ignore
            }
            this.audioSources.delete(playerId);
        }

        const gainNode = this.volumeSettings.get(playerId);
        if (gainNode) {
            try {
                gainNode.disconnect();
            } catch (e) {
                // Ignore
            }
            this.volumeSettings.delete(playerId);
        }

        console.log(`Player ${playerId} removed from voice chat`);
    }

    /**
     * Clean up resources
     */
    cleanup(): void {
        this.stopAudioCapture();
        this.clearAllAudioBuffers();
        
        if (this.localStream) {
            this.localStream.getTracks().forEach(track => track.stop());
            this.localStream = null;
        }

        if (this.audioContext) {
            this.audioContext.close();
            this.audioContext = null;
        }

        this.enabled = false;
        this.micEnabled = false;
        this.sequence = 0;

        console.log("Voice chat cleaned up");
    }
}

// Export singleton instance
export const voiceChatManager = new VoiceChatManager();
