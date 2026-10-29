import React, { useSyncExternalStore } from "react";
import GAME_MANAGER from "../../index";
import { useLobbyOrGameState } from "../useHooks";
import translate from "../../game/lang";
import { VOICE } from "../../voice/voiceManager";
import CheckBox from "../CheckBox";

export default function RoomVoiceSetting() {
    useSyncExternalStore(VOICE.subscribe, VOICE.getRevision);
    const room = useLobbyOrGameState(state => ({
        host: state.stateType === "game" ? state.host !== null : state.players.get(state.myId!)?.ready === "host",
    }), ["lobbyClients", "playersHost", "hostData", "acceptJoin", "startGame", "backToLobby"]);
    if (!room || (GAME_MANAGER.state.stateType !== "lobby" && GAME_MANAGER.state.stateType !== "game")) return null;
    return <section className="chat-menu-colors selector-section room-voice-setting">
        <div className="voice-setting-row">
            <span>{translate("voice.enableForRoom")}</span>
            <CheckBox checked={VOICE.roomEnabled} disabled={!room.host} ariaLabel={translate("voice.enableForRoom")}
                onChange={enabled => GAME_MANAGER.server.sendPacket({ type: "setVoiceEnabled", enabled })}/>
        </div>
    </section>;
}
