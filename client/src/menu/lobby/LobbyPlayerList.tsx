import React, { ReactElement, useRef, useState } from "react";
import translate from "../../game/lang";
import GAME_MANAGER from "../../index";
import "./lobbyMenu.css";
import { ClientConnection } from "../../game/gameState.d";
import Icon from "../../components/Icon";
import { useLobbyOrGameState } from "../../components/useHooks";
import { Button, RawButton } from "../../components/Button";
import Popover from "../../components/Popover";
import { dropdownPlacementFunction } from "../../components/Select";
import StyledText from "../../components/StyledText";
import { encodeString } from "../../components/ChatMessage";

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
    const players: PlayerDisplayData[] = useLobbyOrGameState(
        state => {
            if (state.stateType === "lobby") {
                return state.players.entries().map(([id, player]) => {
                    const name = player.clientType.type === "player" ? encodeString(player.clientType.name) : null;
                    return {
                        id,
                        clientType: player.clientType.type,
                        ready: player.ready === "ready",
                        connection: player.connection,
                        host: player.ready === "host",
                        name,
                        displayName: name ?? "Spectator",
                    }
                })
            } else if (state.host !== null) {
                return state.host.clients.entries().map(([id, player]) => {
                    return {
                        id,
                        clientType: player.clientType.type,
                        connection: player.connection,
                        ready: null,
                        host: player.host,
                        name: player.clientType.type === "player"
                            ? encodeString(state.players[player.clientType.index].name)
                            : player.clientType.index.toString(),
                        displayName: player.clientType.type === "player"
                            ? encodeString(state.players[player.clientType.index].toString())
                            : player.clientType.index.toString(),
                    }
                })
            }
        },
        ["playersHost", "playersLostConnection", "lobbyClients", "playersReady", "hostData", "gamePlayers"]
    ) ?? [];

    const host = useLobbyOrGameState(
        state => {
            if (state.stateType === "lobby")
                return state.players.get(state.myId!)?.ready === "host"
            else
                return state.host !== null
        },
        ["playersHost", "lobbyClients", "yourId", "playersReady", "hostData"]
    )!;

    return <section className="player-list-menu-colors selector-section">
        <h2>{translate("menu.lobby.players")}</h2>
        <div className="lobby-player-list">
            <ol>
                {players
                    .filter(player => player.clientType === "player")
                    .map(player => <LobbyPlayerListPlayer key={player.id} player={player}/>)
                }
            </ol>
        </div>
        {host && <>
            <h2>{translate("menu.hostSettings.spectators")}</h2>
            <div className="lobby-player-list">
                <ol>
                    {players
                        .filter(player => player.clientType === "spectator")
                        .map(player => <LobbyPlayerListPlayer key={player.id} player={player}/>)
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

function LobbyPlayerListPlayer(props: Readonly<{ player: PlayerDisplayData }>): ReactElement {
    const host = useLobbyOrGameState(
        state => {
            if (state.stateType === "lobby")
                return state.players.get(state.myId!)?.ready === "host"
            else
                return state.host !== null
        },
        ["playersHost", "lobbyClients", "yourId", "playersReady", "hostData"]
    )!;

    const [renameOpen, setRenameOpen] = useState(false);
    const renameButtonRef = useRef<HTMLButtonElement>(null);

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
                onClick={() => GAME_MANAGER.sendSetPlayerHostPacket(props.player.id)}
            ><Icon>add_moderator</Icon></button>}
            {host && props.player.connection !== "disconnected" && <button 
                onClick={() => GAME_MANAGER.sendKickPlayerPacket(props.player.id)}
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
    
    return <div className="lobby-player-list-player-rename">
        <input 
            value={playerName}
            onInput={e => setPlayerName((e.target as HTMLInputElement).value)}
            onKeyUp={e => {
                if (e.key === "Enter") {
                    const newName = (e.target as HTMLInputElement).value;
                    setPlayerName(newName);
                    GAME_MANAGER.sendHostSetPlayerNamePacket(props.player.id, newName);
                }
            }}
        />
        <Button 
            onClick={() => GAME_MANAGER.sendHostSetPlayerNamePacket(props.player.id, playerName)}
        >{translate("menu.lobby.button.setName")}</Button>
    </div>
}