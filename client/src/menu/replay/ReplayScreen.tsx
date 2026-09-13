import React, { ReactElement, useCallback, useContext, useEffect, useState } from "react";
import GAME_MANAGER from "../..";
import { Button } from "../../components/Button";
import { encodeString } from "../../components/ChatMessage";
import Icon from "../../components/Icon";
import Select, { SelectOptionsSearch } from "../../components/Select";
import StyledText from "../../components/StyledText";
import translate from "../../game/lang";
import { getReplayController, REPLAY_SPEEDS, ReplayController, setReplayController } from "../../game/replay/replayController";
import { loadSettingsParsed } from "../../game/localStorage";
import { AnchorControllerContext, MobileContext } from "../Anchor";
import { GameScreenMenus, MenuController, MenuControllerContext, useMenuController } from "../game/GameScreen";
import HeaderMenu, { MenuButtons } from "../game/HeaderMenu";
import LoadingScreen from "../LoadingScreen";
import StartMenu from "../main/StartMenu";
import ReplayMenu, { formatDateTime } from "./ReplayMenu";
import "../game/gameScreen.css";
import "./replayScreen.css";

let CONTENT_CONTROLLER: MenuController | undefined;

export function getReplayScreenContentController(): MenuController | undefined {
    return CONTENT_CONTROLLER;
}

/**
 * Watches a recorded game.
 *
 * The game itself is drawn with the ordinary spectator screen - the replay feeds `GAME_MANAGER`
 * the same packets the server sent when the game was played - with a playback bar added below it.
 */
export default function ReplayScreen(): ReactElement {
    const mobile = useContext(MobileContext)!;
    const { maxMenus, menuOrder } = loadSettingsParsed();

    const contentController = useMenuController(
        maxMenus,
        Object.fromEntries(menuOrder),
        () => CONTENT_CONTROLLER!,
        contentController => CONTENT_CONTROLLER = contentController
    );

    // Stop playback if the screen goes away without the replay having been exited.
    useEffect(() => () => getReplayController()?.pause(), []);

    return <MenuControllerContext.Provider value={contentController}>
        <div className="game-screen spectator-game-screen replay-screen">
            <div className="header">
                <HeaderMenu chatMenuNotification={false}/>
            </div>
            <GameScreenMenus />
            {mobile === true && <MenuButtons chatMenuNotification={false}/>}
            <PlaybackBar />
        </div>
    </MenuControllerContext.Provider>
}

/** Re-renders whenever the replay's playhead, speed or play state changes. */
function useReplayController(): ReplayController | null {
    const [controller] = useState(() => getReplayController());
    const [, setRevision] = useState(0);

    useEffect(() => {
        if (controller === null) return;
        const listener = () => setRevision(revision => revision + 1);
        controller.addListener(listener);
        return () => controller.removeListener(listener);
    }, [controller]);

    return controller;
}

function PlaybackBar(): ReactElement | null {
    const { setContent: setAnchorContent } = useContext(AnchorControllerContext)!;
    const controller = useReplayController();

    const exit = useCallback(async () => {
        setReplayController(null);
        setAnchorContent(<LoadingScreen type="default"/>);
        if (await GAME_MANAGER.setOutsideLobbyState()) {
            setAnchorContent(<ReplayMenu/>);
        } else {
            setAnchorContent(<StartMenu/>);
        }
    }, [setAnchorContent]);

    if (controller === null) return null;

    const recording = controller.replay.recording;

    const speedOptions: SelectOptionsSearch<number> = new Map(REPLAY_SPEEDS.map(speed => [
        speed,
        [`${speed}×`, `${speed}`]
    ]));

    return <div className="replay-playback-bar chat-menu-colors">
        <div className="replay-playback-info">
            <StyledText noLinks={true}>{encodeString(recording.roomName)}</StyledText>
            <span className="replay-playback-date">{formatDateTime(recording.startedAt)}</span>
        </div>

        <div className="replay-playback-controls">
            <Button
                onClick={()=>controller.restart()}
                aria-label={translate("menu.replay.button.restart")}
                tooltip={<>{translate("menu.replay.button.restart")}</>}
            >
                <Icon>replay</Icon>
            </Button>
            <Button
                onClick={()=>controller.previousPhase()}
                aria-label={translate("menu.replay.button.previousPhase")}
                tooltip={<>{translate("menu.replay.button.previousPhase")}</>}
            >
                <Icon>skip_previous</Icon>
            </Button>
            <Button
                onClick={()=>controller.stepBackward()}
                aria-label={translate("menu.replay.button.stepBackward")}
                tooltip={<>{translate("menu.replay.button.stepBackward")}</>}
            >
                <Icon>chevron_left</Icon>
            </Button>
            <Button
                className="brand"
                onClick={()=>controller.togglePlay()}
                aria-label={translate(controller.isPlaying ? "menu.replay.button.pause" : "menu.replay.button.play")}
            >
                <Icon>{controller.isPlaying ? "pause" : "play_arrow"}</Icon>
            </Button>
            <Button
                onClick={()=>controller.stepForward()}
                aria-label={translate("menu.replay.button.stepForward")}
                tooltip={<>{translate("menu.replay.button.stepForward")}</>}
            >
                <Icon>chevron_right</Icon>
            </Button>
            <Button
                onClick={()=>controller.nextPhase()}
                aria-label={translate("menu.replay.button.nextPhase")}
                tooltip={<>{translate("menu.replay.button.nextPhase")}</>}
            >
                <Icon>skip_next</Icon>
            </Button>
            <Select
                className="replay-playback-speed"
                value={controller.playbackSpeed}
                onChange={speed=>controller.setSpeed(speed)}
                optionsSearch={speedOptions}
            />
        </div>

        <div className="replay-playback-seek">
            <input
                type="range"
                min={0}
                max={Math.max(1, controller.durationMs)}
                value={Math.round(controller.elapsedMs)}
                aria-label={translate("menu.replay.seek")}
                onChange={event=>{
                    controller.pause();
                    controller.seekToMs(event.target.valueAsNumber);
                }}
            />
            <div className="replay-playback-readout">
                <span>
                    {formatDuration(controller.elapsedMs)} / {formatDuration(controller.durationMs)}
                </span>
                <span>
                    {translate("menu.replay.step", controller.currentStepCount, controller.stepCount)}
                </span>
                <StyledText noLinks={true}>
                    {controller.currentStep?.description ?? translate("menu.replay.notStarted")}
                </StyledText>
            </div>
        </div>

        <Button onClick={exit} aria-label={translate("menu.replay.button.exit")}>
            <Icon>close</Icon>
        </Button>
    </div>
}

/** `m:ss` for a duration in milliseconds. */
function formatDuration(milliseconds: number): string {
    const totalSeconds = Math.max(0, Math.floor(milliseconds / 1000));
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}
