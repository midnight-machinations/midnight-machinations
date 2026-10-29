import React, { useSyncExternalStore } from "react";
import Icon from "../components/Icon";
import translate from "../game/lang";
import { VOICE } from "./voiceManager";
import "./voice.css";

/** Voice state for one player-list entry. Speaking shows as an outline on the entry itself. */
export function useVoicePlayer(target: { roomClientId?: number; playerIndex?: number }) {
    useSyncExternalStore(VOICE.subscribe, VOICE.getRevision);
    const session = VOICE.session;
    const id = target.roomClientId ?? session?.peers.find(peer => peer.playerIndex === target.playerIndex)?.id;
    const connected = VOICE.status === "connected";
    const self = id !== undefined && id === session?.selfId;
    return {
        id,
        // Whether this room has voice, so presence icons have meaning.
        available: session !== null,
        // The server only reports presence in lobbies.
        inVoice: id !== undefined && session?.peers.find(peer => peer.id === id)?.inVoice === true,
        speaking: connected && id !== undefined && !VOICE.playerMuted(id) && VOICE.isSpeaking(id),
        // Your own mic button, or the unmute button for a player you muted.
        showMuteButton: connected && id !== undefined && (self || VOICE.person(id).muted),
    };
}

/** A mute button styled like the lobby player list's other buttons. */
export function LobbyVoiceMuteButton(props: Readonly<{ roomClientId: number }>) {
    const voice = useVoicePlayer(props);
    if (!voice.showMuteButton) return null;
    const muted = VOICE.playerMuted(props.roomClientId);
    const name = VOICE.session?.peers.find(peer => peer.id === props.roomClientId)?.name ?? "";
    const label = `${translate(muted ? "voice.unmute" : "voice.mute")} ${name}`;
    return <button aria-pressed={muted} aria-label={label} title={label}
        onClick={() => void VOICE.togglePlayerMute(props.roomClientId)}>
        <Icon>{muted ? "mic_off" : "mic"}</Icon>
    </button>;
}
