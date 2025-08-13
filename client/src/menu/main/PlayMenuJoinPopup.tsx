import React, { ReactElement } from "react";
import { LobbyPreviewData } from "../../game/packet";
import translate from "../../game/lang";
import "./playMenuJoinPopup.css";
import { encodeString } from "../../components/ChatMessage";

export default function PlayMenuJoinPopup(props: Readonly<{
    roomCode: number,
    lobbyData: LobbyPreviewData,
    joinGame: (roomCode?: number, playerId?: number) => void
}>): ReactElement {

    return <div className="play-menu-join-popup">
        <button onClick={() => props.joinGame(props.roomCode)}>{translate("menu.play.button.spectate")}</button>
        
        <div className="rejoinColumn">
            {props.lobbyData.players.map((player, j)=>{
                return <button key={player[1] as string} onClick={()=>{
                    props.joinGame(props.roomCode, player[0]);
                }}>{encodeString(player[1])}</button>
            })}
        </div>
    </div>
}