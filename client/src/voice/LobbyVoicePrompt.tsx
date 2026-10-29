import React, { useSyncExternalStore } from "react";
import { Button } from "../components/Button";
import Icon from "../components/Icon";
import translate from "../game/lang";
import { VOICE } from "./voiceManager";

export default function LobbyVoicePrompt() {
    useSyncExternalStore(VOICE.subscribe, VOICE.getRevision);
    if (!VOICE.roomEnabled || VOICE.status === "connected") return null;
    const connecting = VOICE.status === "connecting";
    return <Button className="lobby-join-voice" disabled={connecting || !VOICE.session} onClick={() => void VOICE.join()}>
        <Icon size="small">headset</Icon>
        {translate(connecting ? "voice.status.connecting" : "voice.join")}
    </Button>;
}
