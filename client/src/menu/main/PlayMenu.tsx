import React, { ReactElement, useCallback, useContext, useEffect, useState } from "react";
import translate from "../../game/lang";
import { AnchorControllerContext } from "../Anchor";
import GAME_MANAGER from "../..";
import LoadingScreen from "../LoadingScreen";
import "./playMenu.css";
import { StateListener } from "../../game/gameManager.d";
import { LobbyPreviewData } from "../../game/packet";
import LobbyMenu from "../lobby/LobbyMenu";
import PlayMenuJoinPopup from "./PlayMenuJoinPopup";
import { encodeString } from "../../components/ChatMessage";
import { Button } from "../../components/Button";

export default function PlayMenu(): ReactElement {
    const { setContent: setAnchorContent } = useContext(AnchorControllerContext)!;
    
    useEffect(() => {
        GAME_MANAGER.sendLobbyListRequest();
        
        const autoRefresh = setInterval(() => {GAME_MANAGER.sendLobbyListRequest()}, 2500);
        return () => clearInterval(autoRefresh);
    })

    const joinGame = useCallback(
        async (roomCode?: number, playerId?: number): Promise<boolean> => {
            if (roomCode === undefined) return false;
        
            setAnchorContent(<LoadingScreen type="join"/>);
        
            let success: boolean;
            if (playerId === undefined) {
                success = await GAME_MANAGER.sendJoinPacket(roomCode);
            } else {
                success = await GAME_MANAGER.sendRejoinPacket(roomCode, playerId);
            }
        
            if (!success) {
                setAnchorContent(<PlayMenu/>);
            }
        
            return success;
        },
        [setAnchorContent]
    );
    

    return <div className="play-menu">
        <div className="play-menu-browser graveyard-menu-colors">
            <header>
                <h2>
                    {translate("menu.play.title")}
                </h2>
                <div>
                    <button onClick={async () => {
                        setAnchorContent(<LoadingScreen type="host"/>);
                        if (await GAME_MANAGER.sendHostPacket()) {
                            setAnchorContent(<LobbyMenu/>)
                        } else {
                            setAnchorContent(<PlayMenu/>)
                        }
                    }}>
                        {translate("menu.play.button.host")}
                    </button>
                    <button onClick={()=>{GAME_MANAGER.sendLobbyListRequest()}}>
                        {translate("refresh")}
                    </button>
                </div>
            </header>
            <div className="play-menu-center">
                <PlayMenuTable joinGame={joinGame}/>
            </div>
            <PlayMenuFooter joinGame={joinGame}/>
        </div>
    </div>
}

function PlayMenuFooter(props: Readonly<{
    joinGame: (roomCode?: number, playerId?: number) => Promise<boolean>
}>): ReactElement {
    const [roomCode, setRoomCode] = useState<number | undefined>(undefined);
    const [playerID, setPlayerID] = useState<number | undefined>(undefined);

    return <footer>
        <div>
            <label>{translate("menu.play.field.roomCode")}</label>
            <input type="text" value={roomCode?.toString(18)} 
                onChange={(e)=>{
                    const value = e.target.value;
                    if (value === "") {
                        setRoomCode(undefined);
                    } else {
                        try {
                            const code = parseInt(value, 18);
                            if (!isNaN(code)) {
                                setRoomCode(code)
                            }
                        } catch (_) {}
                    }}}
                onKeyUp={(e)=>{
                    if(e.key === 'Enter') {
                        props.joinGame(roomCode);
                    }
                }}
            />
        </div>
        <div>
            <label>{translate("menu.play.field.playerId")}</label>
            <input type="text" value={playerID} 
                onChange={(e)=>{
                    const value = e.target.value;
                    if (value === "") {
                        setPlayerID(undefined);
                    } else {
                        try {
                            const id = parseInt(value);
                            if (!isNaN(id) && id < 256) {
                                setPlayerID(id)
                            }
                        } catch (_) {}
                    }
                }}
                onKeyUp={(e)=>{
                    if(e.key === 'Enter') {
                        props.joinGame(roomCode, playerID);
                    }
                }}
            />
        </div>
        <button onClick={()=>{
            props.joinGame(roomCode, playerID)
        }}>
            {translate("menu.play.button.join")}
        </button>
    </footer>
}

type LobbyMap = Map<number, LobbyPreviewData>;

function PlayMenuTable(props: Readonly<{
    joinGame: (roomCode?: number, playerId?: number) => Promise<boolean>
}>): ReactElement {
    const [lobbies, setLobbies] = useState<LobbyMap>(new Map());
    const { setCoverCard } = useContext(AnchorControllerContext)!;

    useEffect(() => {
        const listener: StateListener = (type) => {
            if (GAME_MANAGER.state.stateType === "outsideLobby" && type === "lobbyList") {
                setLobbies(GAME_MANAGER.state.lobbies);
            }
        }
        GAME_MANAGER.addStateListener(listener);
        return () => GAME_MANAGER.removeStateListener(listener);
    });

    return <table className="play-menu-table">
        <thead>
            <tr>
                <th></th>
                <th>{translate("menu.play.field.name")}</th>
                <th>{translate("players")}</th>
            </tr>
        </thead>
        <tbody>
            {Array.from(lobbies.entries()).map(([roomCode, lobby])=>{

                return <tr key={roomCode}>
                    <td>
                        <button onClick={() => {
                            if(lobby.inGame){
                                setCoverCard(<PlayMenuJoinPopup 
                                    roomCode={roomCode}
                                    lobbyData={lobby}
                                    joinGame={props.joinGame}
                                />);
                            }else{
                                props.joinGame(roomCode);
                            }
                        }}>{translate("menu.play.button.join")}</button>
                    </td>
                    <td>{encodeString(lobby.name)}</td>
                    <td>
                        <div className="play-menu-lobby-player-list">
                            {lobby.players.map(([clientId, clientName])=>{
                                return <Button key={clientName as string} onClick={()=>{
                                    props.joinGame(roomCode, clientId);
                                }}>{encodeString(clientName)}</Button>
                            })}
                        </div>
                    </td>
                </tr>;
            })}
        </tbody>
        <tfoot>
            {new Array(100).fill(0).map((_, i) => {
                return <tr key={i}>
                    <td></td>
                    <td></td>
                    <td></td>
                </tr>
            })}
        </tfoot>
    </table>
}