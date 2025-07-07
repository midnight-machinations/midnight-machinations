import React, { ReactElement, useContext } from "react";
import AuditorMenu from "./RoleSpecificMenus/AuditorMenu";
// import LargeDoomsayerMenu from "./RoleSpecificMenus/LargeDoomsayerMenu";
import Counter from "../../../../components/Counter";
import StyledText from "../../../../components/StyledText";
import translate from "../../../../game/lang";
import SmallPuppeteerMenu from "./RoleSpecificMenus/SmallPuppeteerMenu";
import StewardMenu from "./RoleSpecificMenus/StewardMenu";
import RecruiterMenu from "./RoleSpecificMenus/RecruiterMenu";
import DetailsSummary from "../../../../components/DetailsSummary";
import HypnotistMenu from "./RoleSpecificMenus/HypnotistMenu";
import { usePlayerState } from "../../../../stateContext/useHooks";
import { PhaseState } from "../../../../stateContext/stateType/phaseState";
import { RoleState } from "../../../../stateContext/stateType/roleState";
import { StateContext } from "../../../../stateContext/StateContext";

    

export default function RoleSpecificSection(): ReactElement{
    const gameState = useContext(StateContext)!;

    const roleState = usePlayerState()!.roleState;
    const phaseState = gameState!.phaseState;

    const dayNumber = gameState.dayNumber;
    const numPlayers = gameState.players.length;

    const inner = roleSpecificSectionInner(phaseState, dayNumber, roleState, numPlayers);

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
    numPlayers: number
): ReactElement | null{
    let maxChargesCounter = abilityChargesCounter(numPlayers);

    switch(roleState.type){
        case "auditor":
            return <AuditorMenu roleState={roleState}/>;
        case "hypnotist":
            return <HypnotistMenu roleState={roleState}/>;
        // case "doomsayer":
        //     return <LargeDoomsayerMenu/>;
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
            return <MediumRoleSpecificMenu roleState={roleState}/>;
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
        case "mortician":
            return <Counter
                max={maxChargesCounter}
                current={roleState.cremationsRemaining}
            >
                <StyledText>{translate("role.mortician.roleDataText", roleState.cremationsRemaining)}</StyledText>
            </Counter>
        case "steward":
            return <StewardMenu roleState={roleState}/>;
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

function MediumRoleSpecificMenu(props: Readonly<{
    roleState: RoleState & { type: "medium" }
}>): ReactElement {
    const gameState = useContext(StateContext)!;
    const players = gameState.players;

    const counter = <Counter
        max={3}
        current={props.roleState.seancesRemaining}
    >
        <StyledText>{translate("role.medium.roleDataText.hauntsRemaining", props.roleState.seancesRemaining)}</StyledText>
    </Counter>
    if (props.roleState.seancedTarget === null) {
        return <>
            {counter}
            <div className="role-information">
                <StyledText>{translate("role.medium.roleDataText.nobody")}</StyledText>
            </div>
        </>
    } else {
        return <>
            {counter}
            <div className="role-information">
                <StyledText>{translate("role.medium.roleDataText", 
                    players[props.roleState.seancedTarget].toString(),
                )}</StyledText>
            </div>
        </>;
    }
}

function SpiralMenu(): ReactElement | null {
    const gameState = useContext(StateContext)!;
    const spiralingPlayers = gameState.players.filter(p => p.playerTags.includes("spiraling"))!

    if (spiralingPlayers.length !== 0) {
        return <div className="role-information">
            <StyledText>{translate("role.spiral.roleDataText.cannotSelect")}</StyledText>
        </div>
    } else {
        return null as null;
    }
}