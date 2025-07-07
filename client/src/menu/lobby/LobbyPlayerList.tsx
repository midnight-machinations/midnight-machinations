import React, { ReactElement, useContext, useRef, useState } from "react";
import translate from "../../game/lang";
import "./lobbyMenu.css";
import Icon from "../../components/Icon";
import { Button, RawButton } from "../../components/Button";
import Popover from "../../components/Popover";
import { dropdownPlacementFunction } from "../../components/Select";
import StyledText from "../../components/StyledText";
import { WebsocketContext } from "../WebsocketContext";
import { ClientConnection } from "../../stateContext/stateType/otherState";
import { StateContext } from "../../stateContext/StateContext";

type PlayerDisplayData = {
    id: number,
    clientType: "player" | "spectator",
    connection: ClientConnection,
    ready: boolean | null,
    host: boolean,
    name: string | null,
    displayName: string,
}

export default function LobbyPlayerList(): ReactElement {

    const stateCtx = useContext(StateContext)!;

    let host: boolean = stateCtx.clients.get(stateCtx.myId!)?.ready === "host";
    let players: PlayerDisplayData[] = stateCtx.clients.entries().map(([id, player]) => {
        const name = player.clientType.type === "player" ? player.clientType.name : null;
        return {
            id,
            clientType: player.clientType.type,
            ready: player.ready === "ready",
            connection: player.connection,
            host: player.ready === "host",
            name,
            displayName: name ?? "Spectator",
        }
    });

    return <section className="player-list-menu-colors selector-section">
        <h2>{translate("menu.lobby.players")}</h2>
        <div className="lobby-player-list">
            <ol>
                {players
                    .filter(player => player.clientType === "player")
                    .map(player => <LobbyPlayerListPlayer key={player.id} player={player} host={host}/>)
                }
            </ol>
        </div>
        {host && <>
            <h2>{translate("menu.hostSettings.spectators")}</h2>
            <div className="lobby-player-list">
                <ol>
                    {players
                        .filter(player => player.clientType === "spectator")
                        .map(player => <LobbyPlayerListPlayer key={player.id} player={player} host={host}/>)
                    }
                </ol>
            </div>
        </>}
        {!host && <div className="spectators-ready">
            {translate("menu.lobby.spectatorsReady", 
                [...players.values()].filter(p => p.clientType === "spectator" && p.ready !== null).length,
                [...players.values()].filter(p => p.clientType === "spectator").length
            )}
        </div>}
    </section>
}

function LobbyPlayerListPlayer(props: Readonly<{host: boolean, player: PlayerDisplayData }>): ReactElement {
    const host = props.host;

    const [renameOpen, setRenameOpen] = useState(false);
    const renameButtonRef = useRef<HTMLButtonElement>(null);

    const {sendSetPlayerHostPacket, sendKickPlayerPacket} = useContext(WebsocketContext)!;

    return <li key={props.player.id} className={props.player.connection==="connected" ? "" : "keyword-dead"}>
        <div>
            {props.player.connection === "couldReconnect" && <Icon>signal_cellular_connected_no_internet_4_bar</Icon>}
            {props.player.connection === "disconnected" && <Icon>sentiment_very_dissatisfied</Icon>}
            {props.player.host && <Icon>shield</Icon>}
            {props.player.ready && <Icon>check</Icon>}
            <StyledText>{props.player.displayName}</StyledText>
        </div>
        <div>
            {host && !props.player.host && <button
                onClick={() => sendSetPlayerHostPacket(props.player.id)}
            ><Icon>add_moderator</Icon></button>}
            {host && props.player.connection !== "disconnected" && <button 
                onClick={() => sendKickPlayerPacket(props.player.id)}
            ><Icon>person_remove</Icon></button>}
            {host && props.player.clientType === "player" && <>
                <RawButton
                    ref={renameButtonRef}
                    onClick={() => setRenameOpen(open => !open)}
                ><Icon>edit</Icon></RawButton>
                <Popover
                    open={renameOpen}
                    setOpenOrClosed={setRenameOpen}
                    onRender={dropdownPlacementFunction}
                    anchorForPositionRef={renameButtonRef}
                ><LobbyPlayerListPlayerRename {...props}/></Popover>
            </>}
        </div>
    </li>
}

function LobbyPlayerListPlayerRename(props: Readonly<{ player: PlayerDisplayData }>): ReactElement {
    const [playerName, setPlayerName] = useState(props.player.name ?? "");
    
    const {sendHostSetPlayerNamePacket} = useContext(WebsocketContext)!;

    return <div className="lobby-player-list-player-rename">
        <input 
            value={playerName}
            onInput={e => setPlayerName((e.target as HTMLInputElement).value)}
            onKeyUp={e => {
                if (e.key === "Enter") {
                    const newName = (e.target as HTMLInputElement).value;
                    setPlayerName(newName);
                    sendHostSetPlayerNamePacket(props.player.id, newName);
                }
            }}
        />
        <Button 
            onClick={() => sendHostSetPlayerNamePacket(props.player.id, playerName)}
        >{translate("menu.lobby.button.setName")}</Button>
    </div>
}