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
    private mediaSourceMap: Map<LobbyClientID, { mediaSource: MediaSource, audio: HTMLAudioElement, sourceBuffer: SourceBuffer | null, queue: Uint8Array[] }> = new Map();
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
            
            // Get or create MediaSource for this player
            let playerMedia = this.mediaSourceMap.get(fromPlayerId);
            if (!playerMedia) {
                playerMedia = this.createMediaSourceForPlayer(fromPlayerId);
                this.mediaSourceMap.set(fromPlayerId, playerMedia);
            }
            
            // Add chunk to queue
            playerMedia.queue.push(uint8Array);
            
            // Process queue if source buffer is ready
            this.processAudioQueue(fromPlayerId);

        } catch (error) {
            console.error(`Error handling voice data from player ${fromPlayerId}:`, error);
        }
    }

    /**
     * Create MediaSource and Audio element for a player
     */
    private createMediaSourceForPlayer(playerId: LobbyClientID): { mediaSource: MediaSource, audio: HTMLAudioElement, sourceBuffer: SourceBuffer | null, queue: Uint8Array[] } {
        const mediaSource = new MediaSource();
        const audio = new Audio();
        audio.src = URL.createObjectURL(mediaSource);
        
        // Apply volume setting
        const volume = this.getPlayerVolume(playerId);
        audio.volume = volume;
        
        const playerMedia = {
            mediaSource,
            audio,
            sourceBuffer: null as SourceBuffer | null,
            queue: [] as Uint8Array[]
        };
        
        mediaSource.addEventListener('sourceopen', () => {
            try {
                if (mediaSource.readyState === 'open') {
                    const sourceBuffer = mediaSource.addSourceBuffer('audio/webm; codecs="opus"');
                    playerMedia.sourceBuffer = sourceBuffer;
                    
                    sourceBuffer.addEventListener('updateend', () => {
                        this.processAudioQueue(playerId);
                    });
                    
                    // Start playback
                    audio.play().catch(err => {
                        console.error(`Error starting audio playback for player ${playerId}:`, err);
                    });
                    
                    // Process any queued data
                    this.processAudioQueue(playerId);
                }
            } catch (error) {
                console.error(`Error setting up source buffer for player ${playerId}:`, error);
            }
        });
        
        return playerMedia;
    }
    
    /**
     * Process queued audio data for a player
     */
    private processAudioQueue(playerId: LobbyClientID): void {
        const playerMedia = this.mediaSourceMap.get(playerId);
        if (!playerMedia || !playerMedia.sourceBuffer) {
            return;
        }
        
        const { sourceBuffer, queue } = playerMedia;
        
        // Only append if not currently updating and we have data
        if (!sourceBuffer.updating && queue.length > 0) {
            const chunk = queue.shift()!;
            try {
                // Cast to ArrayBuffer to satisfy TypeScript
                sourceBuffer.appendBuffer(chunk.buffer as ArrayBuffer);
            } catch (error) {
                console.error(`Error appending buffer for player ${playerId}:`, error);
            }
        }
    }

    /**
     * Play audio from a URL for a specific player (legacy, not used)
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
        // Clean up all MediaSource instances
        for (const [playerId, playerMedia] of this.mediaSourceMap.entries()) {
            try {
                playerMedia.audio.pause();
                playerMedia.audio.src = '';
                if (playerMedia.mediaSource.readyState === 'open') {
                    playerMedia.mediaSource.endOfStream();
                }
                URL.revokeObjectURL(playerMedia.audio.src);
            } catch (e) {
                // Ignore cleanup errors
            }
        }
        this.mediaSourceMap.clear();
        
        // Clear volume settings
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
        const playerMedia = this.mediaSourceMap.get(playerId);
        if (playerMedia) {
            playerMedia.audio.volume = volume;
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
        // Clean up MediaSource for this player
        const playerMedia = this.mediaSourceMap.get(playerId);
        if (playerMedia) {
            try {
                playerMedia.audio.pause();
                playerMedia.audio.src = '';
                if (playerMedia.mediaSource.readyState === 'open') {
                    playerMedia.mediaSource.endOfStream();
                }
            } catch (e) {
                // Ignore cleanup errors
            }
            this.mediaSourceMap.delete(playerId);
        }
        
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
