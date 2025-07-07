import React, { ReactElement, useCallback, useContext, useEffect, useState } from "react";
import translate from "../../game/lang";
import "./playMenu.css";
import PlayMenuJoinPopup from "./PlayMenuJoinPopup";
import { Button } from "../../components/Button";
import { AppContext } from "../AppContext";
import { WebsocketContext } from "../WebsocketContext";
import { LobbyPreviewData } from "../../stateContext/stateType/gameBrowserState";
import { StateContext } from "../../stateContext/StateContext";

export default function PlayMenu(): ReactElement {
    const { setContent: setAnchorContent } = useContext(AppContext)!;
    const { sendLobbyListRequest, sendJoinPacket, sendRejoinPacket, sendHostPacket } = useContext(WebsocketContext)!;
    
    useEffect(() => {
        let autoRefreshInterval: NodeJS.Timeout;
        sendLobbyListRequest();
        
        const FIVE_SECONDS = 5000;
        autoRefreshInterval = setInterval(sendLobbyListRequest, FIVE_SECONDS);
        return () => {
            if (autoRefreshInterval) {
                clearInterval(autoRefreshInterval)
            }
        };
    }, [])

    const joinGame = useCallback(
        async (roomCode?: number, playerId?: number) => {
            if (roomCode === undefined) return false;
        
            setAnchorContent({type:"loading", loadingType:"join"});
        
            if (playerId === undefined) {
                sendJoinPacket(roomCode);
            } else {
                sendRejoinPacket(roomCode, playerId);
            }
        },
        [setAnchorContent, sendJoinPacket, sendRejoinPacket]
    );
    

    return <div className="play-menu">
        <div className="play-menu-browser graveyard-menu-colors">
            <header>
                <h2>
                    {translate("menu.play.title")}
                </h2>
                <div>
                    <Button onClick={async () => {
                        setAnchorContent({type:"loading", loadingType:"host"});
                        // stateCtx.setGameBrowser();
                        sendHostPacket();
                    }}>
                        {translate("menu.play.button.host")}
                    </Button>
                    <Button onClick={()=>{sendLobbyListRequest()}}>
                        {translate("refresh")}
                    </Button>
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
    joinGame: (roomCode?: number, playerId?: number)=>void
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

function PlayMenuTable(props: Readonly<{
    joinGame: (roomCode?: number, playerId?: number)=>void
}>): ReactElement {
    const { setCoverCard } = useContext(AppContext)!;
    const stateCtx = useContext(StateContext)!;

    return <table className="play-menu-table">
        <thead>
            <tr>
                <th></th>
                <th>{translate("menu.play.field.name")}</th>
                <th>{translate("players")}</th>
            </tr>
        </thead>
        <tbody>
            {(Array.from(stateCtx.lobbies.entries())).map((entry)=>{
                const roomCode = entry[0];
                const lobby: LobbyPreviewData = entry[1];

                return <tr key={roomCode}>
                    <td>
                        <Button onClick={() => {
                            if(lobby.inGame){
                                setCoverCard(<PlayMenuJoinPopup 
                                    roomCode={roomCode}
                                    lobbyData={lobby}
                                    joinGame={props.joinGame}
                                />);
                            }else{
                                props.joinGame(roomCode);
                            }
                        }}>{translate("menu.play.button.join")}</Button>
                    </td>
                    <td>{lobby.name}</td>
                    <td>
                        <div className="play-menu-lobby-player-list">
                            {lobby.players.map((player)=>{
                                return <Button key={player[1]} onClick={()=>{
                                    props.joinGame(roomCode, player[0]);
                                }}>{player[1]}</Button>
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