import React, { useSyncExternalStore } from "react";
import Icon from "../components/Icon";
import translate from "../game/lang";
import { VOICE } from "./voiceManager";
import "./voice.css";

/** Only local playback preferences and received speech affect remote icons. */
export default function SpeakingIndicator(props: { roomClientId?: number; playerIndex?: number; className?: string }) {
    useSyncExternalStore(VOICE.subscribe, VOICE.getRevision);
    if (VOICE.status !== "connected") return null;
    const id = props.roomClientId ?? VOICE.session?.peers.find(peer => peer.playerIndex === props.playerIndex)?.id;
    const muted = id !== undefined && VOICE.playerMuted(id);
    const speaking = !muted && VOICE.isSpeaking(id);
    const name = VOICE.session?.peers.find(peer => peer.id === id)?.name ?? "";
    const action = translate(muted ? "voice.unmute" : "voice.mute");
    const state = translate(muted ? "voice.muted" : speaking ? "voice.speaking" : "voice.notSpeaking");
    return <button type="button" className={props.className}
        disabled={id === undefined || !VOICE.session} aria-pressed={muted} aria-label={`${action} ${name}: ${state}`}
        title={`${action} ${name}: ${state}`} onClick={() => { if (id !== undefined) void VOICE.togglePlayerMute(id); }}>
        <Icon>{muted ? "mic_off" : speaking ? "mic" : "mic_none"}</Icon>
    </button>;
}
