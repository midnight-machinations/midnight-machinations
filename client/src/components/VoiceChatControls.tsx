import React, { ReactElement, useEffect, useState } from "react";
import translate from "../game/lang";
import { Button } from "./Button";
import Icon from "./Icon";
import { useLobbyState } from "./useHooks";
import { LobbyState } from "../game/gameState.d";
import "./voiceChatControls.css";

export function VoiceChatControls(): ReactElement | null {
    const voiceChatEnabled = useLobbyState(
        (lobbyState: LobbyState) => lobbyState.voiceChatEnabled,
        ["voiceChatEnabled"]
    ) ?? false;

    const players = useLobbyState(
        (lobbyState: LobbyState) => lobbyState.players,
        ["lobbyClients"]
    );

    const myId = useLobbyState(
        (lobbyState: LobbyState) => lobbyState.myId,
        ["yourId"]
    );

    const [micEnabled, setMicEnabled] = useState(false);
    const [volumeLevels, setVolumeLevels] = useState<Map<number, number>>(new Map());
    const [initialized, setInitialized] = useState(false);

    // Initialize voice chat when enabled
    useEffect(() => {
        if (voiceChatEnabled && !initialized) {
            const initVoiceChat = async () => {
                const { voiceChatManager } = await import("../game/voiceChat");
                const playerIds = players ? Array.from(players.keys()) : [];
                await voiceChatManager.enable(playerIds);
                setInitialized(true);
            };
            initVoiceChat();
        } else if (!voiceChatEnabled && initialized) {
            const disableVoiceChat = async () => {
                const { voiceChatManager } = await import("../game/voiceChat");
                voiceChatManager.disable();
                setInitialized(false);
                setMicEnabled(false);
            };
            disableVoiceChat();
        }
    }, [voiceChatEnabled, initialized, players]);

    // Handle player joins/leaves - simplified for server-mediated architecture
    useEffect(() => {
        if (!voiceChatEnabled || !initialized || !players) return;

        const updatePlayers = async () => {
            const { voiceChatManager } = await import("../game/voiceChat");
            const currentPlayerIds = Array.from(players.keys());

            // Just notify about player changes (server handles routing)
            for (const playerId of currentPlayerIds) {
                if (playerId !== myId) {
                    voiceChatManager.addPlayer(playerId);
                }
            }
        };

        updatePlayers();
    }, [players, voiceChatEnabled, initialized, myId]);

    const toggleMicrophone = async () => {
        const { voiceChatManager } = await import("../game/voiceChat");
        const newState = voiceChatManager.toggleMicrophone();
        setMicEnabled(newState);
    };

    const setPlayerVolume = async (playerId: number, volume: number) => {
        const { voiceChatManager } = await import("../game/voiceChat");
        voiceChatManager.setPlayerVolume(playerId, volume);
        setVolumeLevels(prev => {
            const newMap = new Map(prev);
            newMap.set(playerId, volume);
            return newMap;
        });
    };

    const getPlayerVolume = async (playerId: number): Promise<number> => {
        const { voiceChatManager } = await import("../game/voiceChat");
        return voiceChatManager.getPlayerVolume(playerId);
    };

    if (!voiceChatEnabled) {
        return null;
    }

    const otherPlayers = players ? Array.from(players.entries()).filter(
        ([id, _player]) => id !== myId
    ) : [];

    return <div className="voice-chat-controls">
        <div className="voice-chat-header">
            <h3>{translate("voiceChat.title")}</h3>
            <Button 
                onClick={toggleMicrophone}
                className={micEnabled ? "mic-enabled" : "mic-disabled"}
            >
                <Icon size="small">{micEnabled ? "mic" : "mic_off"}</Icon>
                {micEnabled ? translate("voiceChat.micOn") : translate("voiceChat.micOff")}
            </Button>
        </div>

        {otherPlayers.length > 0 && (
            <div className="voice-chat-players">
                <h4>{translate("voiceChat.playerVolumes")}</h4>
                {otherPlayers.map(([playerId, player]: [number, any]) => {
                    const playerName = player.clientType.type === "player" 
                        ? player.clientType.name 
                        : `Spectator ${playerId}`;
                    const volume = volumeLevels.get(playerId) ?? 1.0;

                    return (
                        <div key={playerId} className="voice-chat-player">
                            <span className="player-name">{playerName}</span>
                            <input
                                type="range"
                                min="0"
                                max="100"
                                value={volume * 100}
                                onChange={(e) => setPlayerVolume(playerId, parseInt(e.target.value) / 100)}
                                className="volume-slider"
                            />
                            <span className="volume-value">{Math.round(volume * 100)}%</span>
                        </div>
                    );
                })}
            </div>
        )}
    </div>
}
