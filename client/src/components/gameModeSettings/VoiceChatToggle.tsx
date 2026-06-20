import React, { ReactElement } from "react";
import translate from "../../game/lang";
import CheckBox from "../CheckBox";
import { useLobbyState } from "../useHooks";
import GAME_MANAGER from "../../index";

export function VoiceChatToggle(props: Readonly<{
    disabled?: boolean
}>): ReactElement {
    const voiceChatEnabled = useLobbyState(
        lobbyState => lobbyState.voiceChatEnabled,
        ["voiceChatEnabled"]
    ) ?? false;

    const handleToggle = () => {
        if (!props.disabled) {
            GAME_MANAGER.server.sendPacket({
                type: "setVoiceChatEnabled",
                enabled: !voiceChatEnabled
            });
        }
    };

    return <div className="selector-section">
        <h2>{translate("menu.lobby.voiceChat")}</h2>
        <div style={{ display: 'flex', alignItems: 'center', gap: '0.5rem', opacity: props.disabled ? 0.5 : 1 }}>
            <CheckBox
                checked={voiceChatEnabled}
                onChange={handleToggle}
            />
            <span>{translate("menu.lobby.voiceChatEnabled")}</span>
        </div>
    </div>
}
