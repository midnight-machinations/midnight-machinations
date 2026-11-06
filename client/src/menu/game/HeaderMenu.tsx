import { type ReactElement, useContext, useMemo } from "react";
import type { PhaseState, Player, Verdict } from "../../game/gameState.d";
import translate from "../../game/lang";
import GAME_MANAGER from "../../index";
import { ContentMenu, MENU_THEMES, MENU_TRANSLATION_KEYS, MenuControllerContext } from "./GameScreen";
import "./headerMenu.css";
import { Button } from "../../components/Button";
import { encodeString } from "../../components/ChatMessage";
import FastForwardSelect from "../../components/FastForwardButton";
import StyledText from "../../components/StyledText";
import { useGameState, usePlayerState, useSpectator } from "../../components/useHooks";
import { loadSettingsParsed } from "../../game/localStorage";
import { MobileContext } from "../Anchor";

export default function HeaderMenu(
    props: Readonly<{
        chatMenuNotification: boolean;
    }>,
): ReactElement {
    const mobile = useContext(MobileContext)!;

    const phaseState = useGameState((gameState) => gameState.phaseState, ["phase"])!;

    const backgroundStyle =
        phaseState.type === "briefing"
            ? "background-none"
            : phaseState.type === "night" || phaseState.type === "obituary"
              ? "background-night"
              : "background-day";

    const host = useGameState((state) => state.host !== null, ["playersHost", "hostData"])!;

    const spectator = useSpectator()!;

    const reducedHeader = loadSettingsParsed().headerEnabled === false;

    return (
        <div className={"header-menu " + backgroundStyle + (reducedHeader ? " reduced-header" : "")}>
            {!(spectator && !host) && <FastForwardButton spectatorAndHost={spectator && host} />}
            {reducedHeader || <Information />}
            {!mobile && <MenuButtons chatMenuNotification={props.chatMenuNotification} />}
            <Timer />
        </div>
    );
}

function Timer(): ReactElement {
    const timeLeftMs = useGameState((gameState) => gameState.timeLeftMs, ["phaseTimeLeft", "tick"])!;
    const phaseLength = useGameState(
        (gameState) => {
            if (gameState.phaseState.type === "recess") return 0;
            return gameState.phaseTimes[gameState.phaseState.type];
        },
        ["phase"],
    )!;

    const timerStyle = {
        height: "100%",
        backgroundColor: "red",
        width: `${timeLeftMs / (phaseLength * 10)}%`,
        margin: "0 auto", // Center the timer horizontally
    };

    return (
        <div className="timer-box">
            <div style={timerStyle} />
        </div>
    );
}

function Information(): ReactElement {
    const dayNumber = useGameState((gameState) => gameState.dayNumber, ["phase"])!;
    const timeLeftMs = useGameState((gameState) => gameState.timeLeftMs, ["phaseTimeLeft", "tick"]) ?? null;
    const phaseState = useGameState((gameState) => gameState.phaseState, ["phase"])!;
    const players = useGameState((gameState) => gameState.players, ["gamePlayers"])!;

    const myIndex = usePlayerState((gameState) => gameState.myIndex, ["yourPlayerIndex"]);
    const myRole = usePlayerState((clientState) => clientState.myRole, ["yourRole"])!;
    const myName = useMemo(() => {
        return myIndex === undefined ? undefined : players[myIndex]?.toString();
    }, [myIndex, players]);

    const timeLeftText = useMemo(() => {
        if (timeLeftMs === null) {
            return "∞";
        } else {
            return Math.floor(timeLeftMs / 1000);
        }
    }, [timeLeftMs]);

    const dayNumberText = useMemo(() => {
        if (phaseState.type === "recess") {
            return "";
        } else {
            return ` ${dayNumber}`;
        }
    }, [dayNumber, phaseState.type]);

    const spectator = useSpectator();

    return (
        <div className="information">
            <div className="my-information">
                <div>
                    <h3>
                        <div>{`${translate("phase." + phaseState.type)}${dayNumberText}⏳${timeLeftText}`}</div>
                    </h3>
                    {spectator || (
                        <StyledText>
                            {encodeString(myName ?? "undefined") + " (" + translate("role." + myRole + ".name") + ")"}
                        </StyledText>
                    )}
                </div>
            </div>
            <PhaseSpecificInformation players={players} myIndex={myIndex} phaseState={phaseState} />
        </div>
    );
}

export function PhaseSpecificInformation(
    props: Readonly<{
        phaseState: PhaseState;
        players: Player[];
        myIndex: number | undefined;
    }>,
): ReactElement | null {
    const controllers =
        usePlayerState(
            (playerState) => playerState.savedControllers,
            ["yourAllowedControllers", "yourAllowedController"],
        ) ?? [];

    const spectator = useSpectator();

    switch (props.phaseState.type) {
        case "nomination":
        case "adjournment": {
            const trials = props.phaseState.trialsLeft;

            return (
                <div className="phase-specific">
                    <div>
                        <StyledText>
                            {translate(trials === 1 ? "trialsRemaining.1" : "trialsRemaining", trials)}
                        </StyledText>
                    </div>
                </div>
            );
        }
        case "testimony":
        case "judgement":
        case "finalWords":
            break;
        default:
            return null;
    }

    // Trial
    return (
        <div className="phase-specific">
            <div>
                <StyledText>
                    {translate(
                        `${props.phaseState.type}.playerOnTrial`,
                        encodeString(props.players[props.phaseState.playerOnTrial].toString()),
                    )}
                </StyledText>
                {!spectator && props.phaseState.type === "judgement" ? (
                    <div className="judgement-info">
                        {props.phaseState.playerOnTrial === props.myIndex ? (
                            <StyledText noLinks={true}>{translate("judgement.cannotVote.onTrial")}</StyledText>
                        ) : !props.players[props.myIndex!]?.alive &&
                          !controllers.some(
                              ([id, controller]) =>
                                  id.type === "judge" &&
                                  id.player === props.myIndex &&
                                  !controller.parameters.grayedOut,
                          ) ? (
                            <StyledText noLinks={true}>{translate("judgement.cannotVote.dead")}</StyledText>
                        ) : (
                            controllers.map(([id, controller]) => {
                                if (
                                    id.type !== "judge" ||
                                    id.player !== props.myIndex ||
                                    controller.parameters.grayedOut ||
                                    !controller.parameters.allowedPlayers.includes(props.myIndex) ||
                                    controller.parameters.available.type !== "integer" ||
                                    controller.selection.type !== "integer"
                                ) {
                                    return null;
                                }
                                const availableVerdicts: Verdict[] = ["innocent", "guilty"];
                                if (2 === controller.parameters.available.selection.max) {
                                    availableVerdicts.push("abstain");
                                }
                                const selected = controller.selection.selection;

                                return (
                                    <>
                                        {availableVerdicts.map((verdict, idx) => (
                                            <VerdictButton
                                                key={verdict}
                                                verdict={verdict}
                                                selected={selected === idx}
                                            />
                                        ))}
                                    </>
                                );
                            })
                        )}
                    </div>
                ) : null}
            </div>
        </div>
    );
}

function VerdictButton(props: Readonly<{ verdict: Verdict; selected: boolean }>) {
    return (
        <Button
            highlighted={props.selected}
            onClick={() => {
                GAME_MANAGER.sendJudgementPacket(props.verdict);
            }}
        >
            <StyledText noLinks={true}>{translate("verdict." + props.verdict)}</StyledText>
        </Button>
    );
}

export function MenuButtons(props: Readonly<{ chatMenuNotification: boolean }>): ReactElement | null {
    const menuController = useContext(MenuControllerContext)!;

    return (
        <div className="menu-buttons">
            {menuController.menus().map((menu) => {
                if (
                    GAME_MANAGER.state.stateType === "game" &&
                    GAME_MANAGER.state.clientState.type === "spectator" &&
                    (menu === ContentMenu.WillMenu || menu === ContentMenu.RoleSpecificMenu)
                ) {
                    return null;
                }
                return (
                    <Button
                        key={menu}
                        className={MENU_THEMES[menu] ?? ""}
                        highlighted={menuController.menusOpen().includes(menu)}
                        onClick={() => menuController.closeOrOpenMenu(menu)}
                    >
                        {menu === ContentMenu.ChatMenu && props.chatMenuNotification && (
                            <div className="chat-notification highlighted">!</div>
                        )}
                        {translate(MENU_TRANSLATION_KEYS[menu] + ".icon")}
                        <span className="mobile-hidden">{translate(MENU_TRANSLATION_KEYS[menu] + ".title")}</span>
                    </Button>
                );
            })}
        </div>
    );
}

export function FastForwardButton(props: { spectatorAndHost: boolean }): ReactElement {
    const fastForward = useGameState(
        (gameState) => (props.spectatorAndHost ? gameState.host?.fastForward : gameState.fastForward),
        ["yourVoteFastForwardPhase", "hostData"],
    ) ?? { type: "none" };
    return (
        <FastForwardSelect
            value={fastForward}
            onChange={(setting) => {
                if (props.spectatorAndHost) GAME_MANAGER.sendHostSkipPhase(setting);
                else GAME_MANAGER.sendVoteFastForwardPhase(setting);
            }}
        />
    );
}
