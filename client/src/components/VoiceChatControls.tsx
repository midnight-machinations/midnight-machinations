import React, { ReactElement, useEffect, useState } from "react";
import { voiceChatManager } from "../../game/voiceChat";
import { useLobbyState } from "../useHooks";
import translate from "../../game/lang";
import { Button } from "../Button";
import Icon from "../Icon";
import type { LobbyClientID, LobbyClient, LobbyState } from "../../game/gameState.d.tsx";
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
            const playerIds = Array.from(players?.keys() ?? []);
            voiceChatManager.enable(playerIds).then(() => {
                setInitialized(true);
            });
        } else if (!voiceChatEnabled && initialized) {
            voiceChatManager.disable();
            setInitialized(false);
            setMicEnabled(false);
        }
    }, [voiceChatEnabled, initialized, players]);

    // Handle player joins/leaves
    useEffect(() => {
        if (!voiceChatEnabled || !initialized) return;

        const currentPlayerIds = Array.from(players?.keys() ?? []);
        const managedPlayerIds = new Set(voiceChatManager["peerConnections"].keys());

        // Add new players
        for (const playerId of currentPlayerIds) {
            if (playerId !== myId && !managedPlayerIds.has(playerId)) {
                voiceChatManager.addPlayer(playerId);
            }
        }

        // Remove disconnected players
        for (const playerId of managedPlayerIds) {
            if (!currentPlayerIds.includes(playerId)) {
                voiceChatManager.removePlayer(playerId);
            }
        }
    }, [players, voiceChatEnabled, initialized, myId]);

    const toggleMicrophone = () => {
        const newState = voiceChatManager.toggleMicrophone();
        setMicEnabled(newState);
    };

    const setPlayerVolume = (playerId: number, volume: number) => {
        voiceChatManager.setPlayerVolume(playerId, volume);
        setVolumeLevels(prev => {
            const newMap = new Map(prev);
            newMap.set(playerId, volume);
            return newMap;
        });
    };

    if (!voiceChatEnabled) {
        return null;
    }

    const playerEntries = players ? Array.from(players.entries()) : [];
    const otherPlayers = playerEntries.filter(
        ([id, _player]) => id !== myId
    ) as Array<[LobbyClientID, LobbyClient]>;

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
                {otherPlayers.map(([playerId, player]: [LobbyClientID, LobbyClient]) => {
                    const playerName = player.clientType.type === "player" 
                        ? player.clientType.name 
                        : `Spectator ${playerId}`;
                    const volume = volumeLevels.get(playerId) ?? voiceChatManager.getPlayerVolume(playerId);

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
