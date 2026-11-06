import type { ReactElement } from "react";
import translate from "../../game/lang";
import type { LobbyPreviewData } from "../../game/packet";
import "./playMenuJoinPopup.css";
import { encodeString } from "../../components/ChatMessage";

export default function PlayMenuJoinPopup(
    props: Readonly<{
        roomCode: number;
        lobbyData: LobbyPreviewData;
        joinGame: (roomCode?: number, playerId?: number) => void;
    }>,
): ReactElement {
    return (
        <div className="play-menu-join-popup">
            <button type="button" onClick={() => props.joinGame(props.roomCode)}>
                {translate("menu.play.button.spectate")}
            </button>

            <div className="rejoinColumn">
                {props.lobbyData.players.map((player, _j) => {
                    return (
                        <button
                            type="button"
                            key={player[1] as string}
                            onClick={() => {
                                props.joinGame(props.roomCode, player[0]);
                            }}
                        >
                            {encodeString(player[1])}
                        </button>
                    );
                })}
            </div>
        </div>
    );
}
