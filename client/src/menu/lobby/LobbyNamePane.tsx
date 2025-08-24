import React, { ReactElement } from "react";
import GAME_MANAGER from "../..";
import translate from "../../game/lang";
import Icon from "../../components/Icon";
import { useLobbyState } from "../../components/useHooks";
import { Button } from "../../components/Button";
import { encodeString } from "../../components/ChatMessage";



export default function LobbyNamePane(): ReactElement {
    const isSpectator = useLobbyState(
        lobbyState => lobbyState.players.get(lobbyState.myId!)?.clientType.type === "spectator",
        ["lobbyClients", "yourId"]
    )!;

    const ready = useLobbyState(
        lobbyState => lobbyState.players.get(lobbyState.myId!)?.ready,
        ["lobbyClients", "playersHost", "playersReady", "yourId"]
    )!;

    return <section className="player-list-menu-colors selector-section lobby-name-pane">
        {!isSpectator && <NameSelector/>}
        <div className="name-pane-buttons">
            <Button onClick={() => GAME_MANAGER.sendSetSpectatorPacket(!isSpectator)}>
                {isSpectator
                    ? <><Icon>sports_esports</Icon> {translate("switchToPlayer")}</>
                    : <><Icon>visibility</Icon> {translate("switchToSpectator")}</>}
            </Button>
            {ready === "host" && <button
                onClick={() => GAME_MANAGER.sendRelinquishHostPacket()}
            ><Icon>remove_moderator</Icon> {translate("menu.lobby.button.relinquishHost")}</button>}
            {ready !== "host" && <Button
                onClick={() => {GAME_MANAGER.sendReadyUpPacket(ready === "notReady")}}
            >
                {ready === "ready"
                    ? <><Icon>clear</Icon> {translate("menu.lobby.button.unready")}</>
                    : <><Icon>check</Icon> {translate("menu.lobby.button.readyUp")}</>}
            </Button>}
        </div>
    </section>
}

function NameSelector(): ReactElement {
    const myName = useLobbyState(
        state => {
            const client = state.players.get(state.myId!);
            if(client === undefined || client === null) return undefined;
            if(client.clientType.type === "spectator") return undefined;
            return client.clientType.name;
        },
        ["lobbyClients", "yourId"]
    );
    const [enteredName, setEnteredName] = React.useState("");

    return <div className="name-pane-selector">
        <div className="lobby-name">
            {/* This might be a little crazy */}
            <section><h2 dangerouslySetInnerHTML={{ __html: encodeString(myName ?? "") }} /></section>
        </div>
        <div className="name-box">
            <input type="text" value={enteredName}
                onChange={(e)=>{setEnteredName(e.target.value)}}
                placeholder={translate("menu.lobby.field.namePlaceholder")}
                onKeyUp={(e)=>{
                    if(e.key === 'Enter')
                        GAME_MANAGER.sendSetNamePacket(enteredName);
                }}
            />
            <button onClick={()=>{
                GAME_MANAGER.sendSetNamePacket(enteredName)
            }}>{translate("menu.lobby.button.setName")}</button>
        </div>
    </div>
}