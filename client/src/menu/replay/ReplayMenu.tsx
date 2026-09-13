import React, { ReactElement, useCallback, useContext, useEffect, useState } from "react";
import GAME_MANAGER from "../..";
import { Button } from "../../components/Button";
import { encodeString } from "../../components/ChatMessage";
import Icon from "../../components/Icon";
import StyledText from "../../components/StyledText";
import { ReplayPreviewData } from "../../game/packet";
import { StateListener } from "../../game/gameManager.d";
import { translateConclusion } from "../../game/gameState.d";
import translate from "../../game/lang";
import { buildReplay } from "../../game/replay/replayEngine";
import { ReplayController, setReplayController } from "../../game/replay/replayController";
import { AnchorControllerContext } from "../Anchor";
import LoadingScreen from "../LoadingScreen";
import PlayMenu from "../main/PlayMenu";
import StartMenu from "../main/StartMenu";
import ReplayScreen from "./ReplayScreen";
import "../main/playMenu.css";
import "./replayMenu.css";

/**
 * The replay browser: every game log the server has, laid out like the room browser, with Watch in
 * place of Join.
 */
export default function ReplayMenu(): ReactElement {
    const { setContent: setAnchorContent, pushErrorCard } = useContext(AnchorControllerContext)!;

    useEffect(() => {
        GAME_MANAGER.sendReplayListRequest();
    }, []);

    const watch = useCallback(async (fileName: string) => {
        setAnchorContent(<LoadingScreen type="join"/>);

        const log = await GAME_MANAGER.sendReplayRequest(fileName);

        if (log === null) {
            pushErrorCard({
                title: translate("menu.replay.notification.couldNotLoad"),
                body: fileName
            });
            setAnchorContent(<ReplayMenu/>);
            return;
        }

        setReplayController(new ReplayController(buildReplay(fileName, log)));
        setAnchorContent(<ReplayScreen/>);
    }, [setAnchorContent, pushErrorCard]);

    return <div className="play-menu">
        <div className="play-menu-browser replay-menu-browser graveyard-menu-colors">
            <header>
                <h2>{translate("menu.replay.title")}</h2>
                <div>
                    <Button className="flush" onClick={()=>{GAME_MANAGER.sendReplayListRequest()}}>
                        <Icon>refresh</Icon>
                    </Button>
                    <Button onClick={()=>setAnchorContent(<PlayMenu/>)}>
                        <Icon>play_arrow</Icon> {translate("menu.play.title")}
                    </Button>
                    <Button onClick={()=>setAnchorContent(<StartMenu/>)}>
                        <Icon>arrow_back</Icon> {translate("menu.globalMenu.quitToMenu")}
                    </Button>
                </div>
            </header>
            <div className="play-menu-center">
                <ReplayMenuTable watch={watch}/>
            </div>
        </div>
    </div>
}

function ReplayMenuTable(props: Readonly<{
    watch: (fileName: string) => Promise<void>
}>): ReactElement {
    const [replays, setReplays] = useState<ReplayPreviewData[]>(() =>
        GAME_MANAGER.state.stateType === "outsideLobby" ? GAME_MANAGER.state.replays : []
    );

    useEffect(() => {
        const listener: StateListener = (type) => {
            if (GAME_MANAGER.state.stateType === "outsideLobby" && type === "replayList") {
                setReplays(GAME_MANAGER.state.replays);
            }
        }
        GAME_MANAGER.addStateListener(listener);
        return () => GAME_MANAGER.removeStateListener(listener);
    }, []);

    return <table className="play-menu-table">
        <thead>
            <tr>
                <th></th>
                <th>{translate("menu.play.field.name")}</th>
                <th>{translate("menu.replay.field.date")}</th>
                <th>{translate("players")}</th>
            </tr>
        </thead>
        <tbody>
            {replays.length === 0 && <tr>
                <td></td>
                <td className="replay-menu-empty">{translate("menu.replay.empty")}</td>
                <td></td>
                <td></td>
            </tr>}
            {replays.map(replay => <tr key={replay.fileName}>
                <td>
                    <button onClick={()=>props.watch(replay.fileName)}>
                        {translate("menu.replay.button.watch")}
                    </button>
                </td>
                <td>{encodeString(replay.roomName)}</td>
                <td className="replay-menu-date">
                    <span>{formatDateTime(replay.startedAt)}</span>
                    {replay.conclusion !== null && <StyledText noLinks={true}>
                        {translate("menu.replay.winner", translateConclusion(replay.conclusion))}
                    </StyledText>}
                </td>
                <td>
                    <div className="play-menu-lobby-player-list">
                        {replay.players.map(([playerIndex, playerName])=>
                            <span key={playerIndex} className="replay-menu-player">
                                <StyledText noLinks={true}>{encodeString(playerName)}</StyledText>
                            </span>
                        )}
                    </div>
                </td>
            </tr>)}
        </tbody>
        <tfoot>
            {new Array(100).fill(0).map((_, i) => {
                return <tr key={i}>
                    <td></td>
                    <td></td>
                    <td></td>
                    <td></td>
                </tr>
            })}
        </tfoot>
    </table>
}

/** The exact date and time the game started, in the viewer's own locale and time zone. */
export function formatDateTime(isoTimestamp: string): string {
    const date = new Date(isoTimestamp);
    if (Number.isNaN(date.getTime())) return isoTimestamp;

    return date.toLocaleString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit"
    });
}
