import React, { ReactElement, useContext, useMemo } from "react";
import translate from "../../game/lang";
import GAME_MANAGER from "../../index";
import { FastForwardSetting, PHASES, PhaseState, PhaseType, Player, Verdict } from "../../game/gameState.d";
import { MenuControllerContext, ContentMenu, MENU_THEMES, MENU_TRANSLATION_KEYS } from "./GameScreen";
import "./headerMenu.css";
import StyledText from "../../components/StyledText";
import Icon from "../../components/Icon";
import { Button } from "../../components/Button";
import { useGameState, usePlayerState, useSpectator } from "../../components/useHooks";
import { MobileContext } from "../Anchor";
import { encodeString } from "../../components/ChatMessage";
import Select from "../../components/Select";


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
                    {encodeString(myName ?? "undefined") + " (" + translate("role."+(roleState!.type)+".name") + ")"}
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
    const controllers = usePlayerState(
        playerState => playerState.savedControllers,
        ["yourAllowedControllers", "yourAllowedController"]
    )??[];

    const spectator = useSpectator();

    if(
        props.phaseState.type !== "testimony" &&
        props.phaseState.type !== "judgement" &&
        props.phaseState.type !== "finalWords"
    ){
        return null;
    }

    return <div className="phase-specific">
        <div className="highlighted">
            <StyledText>
                {translate(`${props.phaseState.type}.playerOnTrial`, encodeString(props.players[props.phaseState.playerOnTrial].toString()))}
            </StyledText>
            {(!spectator && props.phaseState.type === "judgement")?<div className="judgement-info">
                {
                    (props.phaseState.playerOnTrial === props.myIndex)?translate("judgement.cannotVote.onTrial"):
                    (!props.players[props.myIndex!].alive)?translate("judgement.cannotVote.dead"):
                    controllers.map(([id, controller])=>{
                        if(
                            id.type !== "judge" ||
                            controller.parameters.available.type !== "integer" ||
                            controller.selection.type !== "integer"
                        ){return null;}
                        const availableVerdicts: Verdict[] = ["innocent", "guilty"];
                        if(2 === controller.parameters.available.selection.max){availableVerdicts.push("abstain")}
                        const selected = controller.selection.selection;

                        return <>
                            {availableVerdicts.map((verdict, idx)=>
                                <VerdictButton key={verdict} verdict={verdict} selected={selected === idx}/>
                            )}
                        </>
                    })
                }
            </div>:null}
        </div>
    </div>
}

function VerdictButton(props: Readonly<{ verdict: Verdict, selected: boolean }>) {
    return <Button
        highlighted={props.selected}
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



type FastForwardSettingString = "none"|"skip"|`phase/${PhaseType}/${number}`
function fastForwardSettingToString(fastForward: FastForwardSetting): string {
    return fastForward.type==="phase"?
        `phase/${fastForward.phase}/${fastForward.day}`:fastForward.type;
}
function stringToFastForwardSetting(string: FastForwardSettingString): FastForwardSetting {
    let split = string.split("/");
    switch(split[0]){
        case "none":
        case "skip":
            return {type:split[0]}
        case "phase":
            return {type:"phase", phase: split[1] as PhaseType, day: parseInt(split[2])}
    }
    return {type:"none"};
}

export function FastForwardButton(props: { spectatorAndHost: boolean }): ReactElement {
    const fastForward = useGameState(
        gameState => gameState.fastForward,
        ["yourVoteFastForwardPhase"]
    )!;
    
    const currentPhase = useGameState(
        gameState => gameState.phaseState.type,
        ["phase"]
    )!;
    const dayNumber = useGameState(
        gameState => gameState.dayNumber,
        ["phase"]
    )!;

    // eslint-disable-next-line
    const optionMap = new Map();

    optionMap.set(
        "none",
        [
            <Icon>play_arrow</Icon>,
            translate("none")
        ]
    );
    optionMap.set(
        "skip",
        [
            <Icon>fast_forward</Icon>,
            translate("wiki.article.standard.fastForward.title")
        ]
    );
    
    for(let i = dayNumber; i < dayNumber+4; i++){
        for(const phase of ["discussion", "nomination", "night"] as PhaseType[]){
            if(dayNumber !== i || PHASES.indexOf(currentPhase) < PHASES.indexOf(phase) ){
                optionMap.set(
                    fastForwardSettingToString({type:"phase",phase:phase,day:i}),
                    [
                        <StyledText noLinks={true}>{translate("phase."+phase)} {i.toString()}</StyledText>,
                        translate("phase."+phase)+" "+(i.toString())
                    ]
                );
            }
        }
    }

    return <Select
        className={"fast-forward-button"+(fastForward.type!=="none"?" highlighted":"")}
        value={fastForwardSettingToString(fastForward)}
        onChange={(e)=>{
            GAME_MANAGER.sendVoteFastForwardPhase(stringToFastForwardSetting(e));
        }}
        optionsSearch={optionMap}
    />
}
