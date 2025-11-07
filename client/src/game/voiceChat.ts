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
    private playerVolumes: Map<LobbyClientID, number> = new Map();
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
        if (!this.enabled) {
            return;
        }

        try {
            // Convert number array back to Uint8Array
            const uint8Array = new Uint8Array(audioData);
            
            // Create a Blob from the data with proper MIME type
            const blob = new Blob([uint8Array], { type: 'audio/webm;codecs=opus' });
            
            // Create an object URL from the blob
            const audioUrl = URL.createObjectURL(blob);
            
            // Play the audio using HTML Audio element
            this.playAudioUrl(fromPlayerId, audioUrl);

        } catch (error) {
            console.error(`Error handling voice data from player ${fromPlayerId}:`, error);
        }
    }

    /**
     * Play audio from a URL for a specific player
     */
    private playAudioUrl(playerId: LobbyClientID, audioUrl: string): void {
        try {
            // Create audio element
            const audio = new Audio(audioUrl);
            
            // Apply volume setting
            const volume = this.getPlayerVolume(playerId);
            audio.volume = volume;
            
            // Play the audio
            audio.play().catch(err => {
                console.error(`Error playing audio from player ${playerId}:`, err);
            });

            // Clean up the object URL after playback
            audio.onended = () => {
                URL.revokeObjectURL(audioUrl);
            };
            
            // Also clean up on error
            audio.onerror = () => {
                URL.revokeObjectURL(audioUrl);
            };

        } catch (error) {
            console.error(`Error playing audio from player ${playerId}:`, error);
        }
    }

    /**
     * Play an audio buffer for a specific player (legacy method, not currently used)
     */
    private playAudioBuffer(playerId: LobbyClientID, audioBuffer: AudioBuffer): void {
        if (!this.audioContext) {
            return;
        }

        try {
            // Create audio source
            const source = this.audioContext.createBufferSource();
            source.buffer = audioBuffer;

            // Connect and play
            source.connect(this.audioContext.destination);
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
        // Clear volume settings
        this.playerVolumes.clear();
    }

    /**
     * Set volume for a specific player
     */
    setPlayerVolume(playerId: LobbyClientID, volume: number): void {
        // Clamp volume between 0 and 1
        volume = Math.max(0, Math.min(1, volume));
        
        // Store volume setting for future audio playback
        this.playerVolumes.set(playerId, volume);
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
        // Clean up volume setting for this player
        this.playerVolumes.delete(playerId);

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
