import React, { ReactElement, useContext, useMemo } from "react";
import translate from "../../game/lang";
import GAME_MANAGER from "../../index";
import { PhaseState, Player, Verdict } from "../../game/gameState.d";
import { MenuControllerContext, ContentMenu, MENU_THEMES, MENU_TRANSLATION_KEYS } from "./GameScreen";
import "./headerMenu.css";
import StyledText from "../../components/StyledText";
import Icon from "../../components/Icon";
import { Button } from "../../components/Button";
import { useGameState, usePlayerState, useSpectator } from "../../components/useHooks";
import { MobileContext } from "../Anchor";


export default function HeaderMenu(props: Readonly<{
    chatMenuNotification: boolean
}>): ReactElement {
    const mobile = useContext(MobileContext)!;
    
    const phaseState = useGameState(
        gameState => gameState.phaseState,
        ["phase"]
    )!

    const backgroundStyle = 
        phaseState.type === "briefing" ? "background-none" :
        (phaseState.type === "night" || phaseState.type === "obituary") ? "background-night" : 
        "background-day";

    const host = useGameState(
        state => state.host !== null,
        ["playersHost"]
    )!;

    const spectator = useSpectator()!;


    return <div className={"header-menu " + backgroundStyle}>
        {!(spectator && !host) && <FastForwardButton spectatorAndHost={spectator && host}/>}
        <Information />
        {!mobile && <MenuButtons chatMenuNotification={props.chatMenuNotification}/>}
        <Timer />
    </div>
}

function Timer(): ReactElement {
    const timeLeftMs = useGameState(
        gameState => gameState.timeLeftMs,
        ["phaseTimeLeft", "tick"]
    )!
    const phaseLength = useGameState(
        gameState => {
            if (gameState.phaseState.type === "recess") return 0;
            return gameState.phaseTimes[gameState.phaseState.type]
        },
        ["phase"]
    )!

    const timerStyle = {
        height: "100%",
        backgroundColor: 'red',
        width: `${timeLeftMs / (phaseLength * 10)}%`,
        margin: '0 auto', // Center the timer horizontally
    };

    return <div className="timer-box">
        <div style={timerStyle}/>
    </div>
}

function Information(): ReactElement {
    const dayNumber = useGameState(
        gameState => gameState.dayNumber,
        ["phase"]
    )!
    const timeLeftMs = useGameState(
        gameState => gameState.timeLeftMs,
        ["phaseTimeLeft", "tick"]
    ) ?? null;
    const phaseState = useGameState(
        gameState => gameState.phaseState,
        ["phase"]
    )!
    const players = useGameState(
        gameState => gameState.players,
        ["gamePlayers"]
    )!

    const myIndex = usePlayerState(
        gameState => gameState.myIndex,
        ["yourPlayerIndex"]
    )
    const roleState = usePlayerState(
        clientState => clientState.roleState,
        ["yourRoleState"]
    )
    const myName = useMemo(() => {
        return myIndex === undefined ? undefined : players[myIndex]?.toString()
    }, [myIndex, players])


    const timeLeftText = useMemo(() => {
        if (timeLeftMs === null) {
            return "∞"
        } else {
            return Math.floor(timeLeftMs/1000);
        }
    }, [timeLeftMs])

    const dayNumberText = useMemo(() => {
        if (phaseState.type === "recess") {
            return "";
        } else {
            return ` ${dayNumber}`;
        }
    }, [dayNumber, phaseState.type])

    const spectator = useSpectator();
    

    return <div className="information"> 
        <div className="my-information">
            <div>
                <h3>
                    <div>
                        {translate("phase."+phaseState.type)}{dayNumberText}⏳{timeLeftText}
                    </div>
                </h3>
                {spectator || <StyledText>
                    {myName + " (" + translate("role."+(roleState!.type)+".name") + ")"}
                </StyledText>}
            </div>
        </div>
        <PhaseSpecificInformation players={players} myIndex={myIndex} phaseState={phaseState}/>
    </div>
}

export function PhaseSpecificInformation(props: Readonly<{
    phaseState: PhaseState,
    players: Player[],
    myIndex: number | undefined
}>): ReactElement | null {
    const enabledModifiers = useGameState(
        gameState => gameState.enabledModifiers,
        ["enabledModifiers"]
    )!

    const spectator = useSpectator();

    if (
        props.phaseState.type === "testimony"
        || props.phaseState.type === "finalWords"
        || props.phaseState.type === "judgement"
    ) {
        return <div className="phase-specific">
            <div className="highlighted">
                <StyledText>
                    {translate(`${props.phaseState.type}.playerOnTrial`, props.players[props.phaseState.playerOnTrial].toString())}
                </StyledText>
                {!spectator && props.phaseState.type === "judgement" && <div className="judgement-info">
                    {(() => {
                        if (props.phaseState.playerOnTrial === props.myIndex) {
                            return translate("judgement.cannotVote.onTrial");
                        } else if (!props.players[props.myIndex!].alive) {
                            return translate("judgement.cannotVote.dead");
                        } else {
                            return (
                                enabledModifiers.includes("abstaining") ? 
                                    ["guilty", "abstain", "innocent"] as const :
                                    ["guilty", "innocent"] as const 
                                ).map((verdict) => {
                                return <VerdictButton key={verdict} verdict={verdict}/>
                            })
                        }
                    })()}
                </div>}
            </div>
        </div>
        
    } else {
        return null;
    }
}

function VerdictButton(props: Readonly<{ verdict: Verdict }>) {
    const judgement = usePlayerState(
        clientState => clientState.judgement,
        ["yourJudgement"]
    )!

    return <Button
        highlighted={judgement === props.verdict}
        onClick={()=>{GAME_MANAGER.sendJudgementPacket(props.verdict)}}
    >
        <StyledText noLinks={true}>
            {translate("verdict." + props.verdict)}
        </StyledText>
    </Button>
}

export function MenuButtons(props: Readonly<{ chatMenuNotification: boolean }>): ReactElement | null {
    const menuController = useContext(MenuControllerContext)!;

    return <div className="menu-buttons">
        {menuController.menus().map(menu => {
            return <Button key={menu} className={MENU_THEMES[menu] ?? ""}
                highlighted={menuController.menusOpen().includes(menu)} 
                onClick={()=>menuController.closeOrOpenMenu(menu)}
            >
                {menu === ContentMenu.ChatMenu
                    && props.chatMenuNotification
                    && <div className="chat-notification highlighted">!</div>
                }
                {translate(MENU_TRANSLATION_KEYS[menu] + ".icon")}
                <span className="mobile-hidden">{translate(MENU_TRANSLATION_KEYS[menu] + ".title")}</span>
            </Button>
        })}
    </div>
}

export function FastForwardButton(props: { spectatorAndHost: boolean }): ReactElement {
    const fastForward = useGameState(
        gameState => gameState.fastForward,
        ["yourVoteFastForwardPhase"]
    )!

    return <Button 
        onClick={() => {
            if (props.spectatorAndHost) {
                GAME_MANAGER.sendHostSkipPhase()
            } else {
                GAME_MANAGER.sendVoteFastForwardPhase(!fastForward)
            }
        }}
        className="fast-forward-button"
        highlighted={fastForward}
    >
        <Icon>double_arrow</Icon>
    </Button>
}
