import { useGameState, usePlayerNames, usePlayerState } from "../../../../components/useHooks";
import React, { ReactElement } from "react";
import AuditorMenu from "./RoleSpecificMenus/AuditorMenu";
import LargeDoomsayerMenu from "./RoleSpecificMenus/LargeDoomsayerMenu";
import Counter from "../../../../components/Counter";
import StyledText from "../../../../components/StyledText";
import translate from "../../../../game/lang";
import SmallPuppeteerMenu from "./RoleSpecificMenus/SmallPuppeteerMenu";
import StewardMenu from "./RoleSpecificMenus/StewardMenu";
import RecruiterMenu from "./RoleSpecificMenus/RecruiterMenu";
import { RoleState } from "../../../../game/roleState.d";
import { PhaseState, UnsafeString } from "../../../../game/gameState.d";
import DetailsSummary from "../../../../components/DetailsSummary";
import HypnotistMenu from "./RoleSpecificMenus/HypnotistMenu";
import ChatElement, { encodeString } from "../../../../components/ChatMessage";

    

export default function RoleSpecificSection(): ReactElement{
    const roleState = usePlayerState(
        playerState => playerState.roleState,
        ["yourRoleState"]
    )!;
    const phaseState = useGameState(
        gameState => gameState.phaseState,
        ["phase"]
    )!;

    const dayNumber = useGameState(
        gameState => gameState.dayNumber,
        ["phase"]
    )!;
    const numPlayers = useGameState(
        gameState => gameState.players.length,
        ["gamePlayers"]
    )!;

    const playerNames = usePlayerNames();

    const inner = roleSpecificSectionInner(phaseState, dayNumber, roleState, numPlayers, playerNames);

    return <>{inner===null ? null : 
        <DetailsSummary
            summary={<StyledText>{translate("role."+roleState?.type+".name")}</StyledText>}
        >
            {inner}
        </DetailsSummary>
    }</>;
}

function abilityChargesCounter(numPlayers: number): number{
    return Math.ceil(numPlayers/5);
}

function roleSpecificSectionInner(
    phaseState: PhaseState,
    dayNumber: number,
    roleState: RoleState,
    numPlayers: number,
    playerNames: UnsafeString[]
): ReactElement | null{
    let maxChargesCounter = abilityChargesCounter(numPlayers);

    switch(roleState.type){
        case "auditor":
            return <AuditorMenu roleState={roleState}/>;
        case "hypnotist":
            return <HypnotistMenu roleState={roleState}/>;
        case "doomsayer":
            return <LargeDoomsayerMenu/>;
        case "jailor": 
            return <Counter 
                max={maxChargesCounter} 
                current={roleState.executionsRemaining}
            >
                <StyledText>{translate("role.jailor.roleDataText.executionsRemaining", roleState.executionsRemaining)}</StyledText>
            </Counter>;
        case "kidnapper": 
            return <Counter 
                max={1} 
                current={roleState.executionsRemaining}
            >
                <StyledText>{translate("role.jailor.roleDataText.executionsRemaining", roleState.executionsRemaining)}</StyledText>
            </Counter>;
        case "medium": 
            return <Counter
                max={maxChargesCounter}
                current={roleState.hauntsRemaining}
            >
                <StyledText>{translate("role.medium.roleDataText.hauntsRemaining", roleState.hauntsRemaining)}</StyledText>
            </Counter>
        case "doctor": {
            return <Counter
                max={1}
                current={roleState.selfHealsRemaining}
            >
                <StyledText>{translate("role.doctor.roleDataText", roleState.selfHealsRemaining)}</StyledText>
            </Counter>
        }
        case "bodyguard":
            return <Counter
                max={1}
                current={roleState.selfShieldsRemaining}
            >
                <StyledText>{translate("role.bodyguard.roleDataText", roleState.selfShieldsRemaining)}</StyledText>
            </Counter>
        case "engineer":
            return <div className="role-information">
                <StyledText>{translate("role.engineer.roleDataText." + roleState.trap.type)}</StyledText>
            </div>;
        case "vigilante":
            switch(roleState.state.type){
                case "willSuicide":
                    return <div className="role-information">
                        <StyledText>{translate("role.vigilante.roleDataText.suicide")}</StyledText>
                    </div>
                case "notLoaded":
                    return <div className="role-information">
                        <StyledText>{translate("role.vigilante.roleDataText.notLoaded")}</StyledText>
                    </div>
                case "loaded":
                    return <Counter 
                        max={maxChargesCounter} 
                        current={roleState.state.bullets}
                    >
                        <StyledText>{translate("role.vigilante.roleDataText", roleState.state.bullets)}</StyledText>
                    </Counter>
                default:
                    return null as null
            }
        case "veteran":
            return <Counter
                max={maxChargesCounter}
                current={roleState.alertsRemaining}
            >
                <StyledText>{translate("role.veteran.roleDataText", roleState.alertsRemaining)}</StyledText>
            </Counter>
        case "armorsmith":
            return <Counter
                max={maxChargesCounter}
                current={roleState.openShopsRemaining}
            >
                <StyledText>{translate("role.armorsmith.roleDataText", roleState.openShopsRemaining)}</StyledText>
            </Counter>
        case "marksman": 
            return <MarksmanRoleSpecificMenu roleState={roleState} />;
        case "counterfeiter":
        case "forger":
            return <Counter
                max={maxChargesCounter}
                current={roleState.forgesRemaining}
            >
                <StyledText>{translate("role.forger.roleDataText", roleState.forgesRemaining)}</StyledText>
            </Counter>
        case "cerenovous":
            return <Counter
                max={maxChargesCounter}
                current={roleState.charges}
            >
                <StyledText>{translate("role.cerenovous.roleDataText", roleState.charges)}</StyledText>
            </Counter>
        case "business":
            return <Counter
                max={maxChargesCounter}
                current={roleState.bribesRemaining}
            >
                <StyledText>{translate("role.business.roleDataText", roleState.bribesRemaining)}</StyledText>
            </Counter>
        case "mortician":
            return <Counter
                max={maxChargesCounter}
                current={roleState.cremationsRemaining}
            >
                <StyledText>{translate("role.mortician.roleDataText", roleState.cremationsRemaining)}</StyledText>
            </Counter>
        case "steward":
            return <StewardMenu roleState={roleState}/>;
        case "courtesan":
            return <StyledText>{roleState.previous.map((p)=>encodeString(playerNames[p])).join()}</StyledText>;
        case "spiral": 
            return <SpiralMenu />;
        case "puppeteer":
            return <SmallPuppeteerMenu
                maxCharges={maxChargesCounter}
                marionettesRemaining={roleState.marionettesRemaining}
                phase={phaseState.type}
            />;
        case "yer":
            return <Counter
                max={maxChargesCounter}
                current={roleState.starPassesRemaining}
            >
                <StyledText>{translate("role.yer.shapeshiftsRemaining", roleState.starPassesRemaining)}</StyledText>
            </Counter>;
        case "recruiter":
            return <RecruiterMenu 
                remaining={roleState.recruitsRemaining}
                dayNumber={dayNumber}
                phase={phaseState.type}
            />;
        case "mercenary":
            return <div>
                <Counter
                    max={maxChargesCounter}
                    current={roleState.attacksRemaining}
                >
                    <StyledText>{translate("role.mercenary.roleDataText", roleState.attacksRemaining)}</StyledText>
                </Counter>
                <ChatElement message={{
                    chatGroup: null,
                    variant: {type:"mercenaryHits", roles:roleState.roles??[]}
                }}/>
            </div>
        case "martyr":
            if (roleState.state.type === "stillPlaying") {
                return <>
                    <div className="role-information">
                        <StyledText>{translate("role.martyr.roleDataText.eccentric")}</StyledText>
                    </div>
                    <Counter
                        max={maxChargesCounter}
                        current={roleState.state.bullets}
                    >
                        <StyledText>{translate("role.martyr.roleDataText", roleState.state.bullets)}</StyledText>
                    </Counter>
                </>
            } else {
                return null as null;
            }
        default:
            return null as null;
    }
}

function MarksmanRoleSpecificMenu(props: Readonly<{
    roleState: (RoleState & { type: "marksman" })
}>): ReactElement {
    let stateText = "";
    switch(props.roleState.state.type){
        case "notLoaded":
        case "loaded":
        case "shotTownie":
            stateText = translate("role.marksman.roleDataText."+props.roleState.state.type)
            break;
    }
    
    return <div className="role-information">
        <StyledText>{stateText}</StyledText>
    </div>
}

function SpiralMenu(props: {}): ReactElement | null {
    const spiralingPlayers = useGameState(
        gameState => gameState.players.filter(p => p.playerTags.includes("spiraling")),
        ["yourPlayerTags"]
    )!

    if (spiralingPlayers.length !== 0) {
        return <div className="role-information">
            <StyledText>{translate("role.spiral.roleDataText.cannotSelect")}</StyledText>
        </div>
    } else {
        return null as null;
    }
}