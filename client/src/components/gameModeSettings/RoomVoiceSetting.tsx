import type React from "react";
import { useSyncExternalStore } from "react";
import { FEATURES } from "../../features";
import translate from "../../game/lang";
import GAME_MANAGER from "../../index";
import { VOICE } from "../../voice/voiceManager";
import CheckBox from "../CheckBox";
import { useLobbyOrGameState } from "../useHooks";

export default function RoomVoiceSetting(props: Readonly<{ children?: React.ReactNode }>) {
    useSyncExternalStore(VOICE.subscribe, VOICE.getRevision);
    const room = useLobbyOrGameState(
        (state) => ({
            host: state.stateType === "game" ? state.host !== null : state.players.get(state.myId!)?.ready === "host",
        }),
        ["lobbyClients", "playersHost", "hostData", "acceptJoin", "startGame", "backToLobby"],
    );
    if (!FEATURES.voice) return <>{props.children}</>;
    if (!room || (GAME_MANAGER.state.stateType !== "lobby" && GAME_MANAGER.state.stateType !== "game")) return null;
    return (
        <section className="chat-menu-colors selector-section room-voice-setting">
            <div className="voice-setting-row">
                <span>{translate("voice.enableForRoom")}</span>
                <CheckBox
                    checked={VOICE.roomEnabled}
                    disabled={!room.host}
                    ariaLabel={translate("voice.enableForRoom")}
                    onChange={(enabled) => GAME_MANAGER.server.sendPacket({ type: "setVoiceEnabled", enabled })}
                />
            </div>
            {props.children}
        </section>
    );
}
