
import { replaceMentions } from "..";
import { Grave, GraveInformation } from "../game/graveState";
import translate from "../game/lang";
import { encodeString } from "./ChatMessage";
import StyledText from "./StyledText";
import React, { ReactElement, useMemo } from "react";
import "./grave.css";
import { useGameState, useLobbyOrGameState } from "./useHooks";
import { ModifierSettings, UnsafeString } from "../game/gameState.d";
import { RoleList, translateRoleSet } from "../game/roleListState.d";

export function translateGraveRole(grave: Grave): string {
    if(grave.information.type === "obscured") {
        return translate("obscured");
    }else{
        return translate(`role.${grave.information.role}.name`);
    }
}

export default function GraveComponent(props: Readonly<{
    grave: Grave, 
    playerNames?: UnsafeString[],
    roleList?: RoleList,
    modifierSettings?: ModifierSettings,
    onClick?: () => void
}>): ReactElement {
    const gamePlayerNames = useGameState(
        gameState => gameState.players.map(player => encodeString(player.toString())),
        ["gamePlayers"]
    )!

    const playerNames = props.playerNames ?? gamePlayerNames;

    const gameRoleList = useLobbyOrGameState(
        gameState => gameState.roleList,
        ["roleList"]
    )!
    const gameModifierSettings = useLobbyOrGameState(
        gameState => gameState.modifierSettings,
        ["modifierSettings"]
    )!

    const roleList = props.roleList ?? gameRoleList;
    const modifierSettings = props.modifierSettings ?? gameModifierSettings;

    if(props.grave.information.type === "obscured") {
        return <ObscuredGrave grave={props.grave} playerNames={playerNames}/>
    } else {
        return <UnobscuredGrave grave={props.grave as any} playerNames={playerNames} roleList={roleList} modifierSettings={modifierSettings}/>;
    }
}

function UnobscuredGrave(props: Readonly<{
    grave: Grave & { information: GraveInformation & { type: "normal" } },
    playerNames: UnsafeString[],
    roleList: RoleList,
    modifierSettings: ModifierSettings,
    onClick?: () => void
}>): ReactElement {
    const graveDeathCause = useMemo(() => {
        if(props.grave.information.deathCause.type === "killers") {
            return props.grave.information.deathCause.killers.map((killer)=>{
                switch(killer.type){
                    case "role":
                        return translate("role."+killer.value+".name");
                    case "roleSet":
                        return translateRoleSet(killer.value, props.modifierSettings);
                    default:
                        return translate("grave.killer."+killer.type);
                }
            }).join(", ") + ".";
        } else if (props.grave.information.deathCause.type === "none") {
            return null;
        } else {
            return translate("grave.deathCause."+props.grave.information.deathCause.type);
        }
    }, [props.grave.information.deathCause, props.modifierSettings]);

    let graveRoleString = translate(`role.${props.grave.information.role}.name`);

    let diedPhaseString = props.grave.diedPhase === "day" ? translate("day") : translate("phase.night");
    let diedPhaseIcon = props.grave.diedPhase === "day" ? translate("day.icon") : translate("night.icon");

    return <div className="grave" onClick={()=>{
        if(props.onClick!==undefined)
            props.onClick();
        }}
    >
        <div><StyledText>{`${diedPhaseString+diedPhaseIcon+props.grave.dayNumber}`}</StyledText></div>
        <div><StyledText>{`${props.playerNames[props.grave.player]+" ("+graveRoleString+")"}`}</StyledText></div>
        {graveDeathCause && <div><StyledText>{`${translate("killedBy")+" "+graveDeathCause}`}</StyledText></div>}
        {(props.grave.information.will as string).length === 0 || <>
            {translate("alibi")}
            <div className="note-area">
                <StyledText>
                    {encodeString(replaceMentions(
                        props.grave.information.will,
                        props.playerNames,
                        props.roleList,
                        props.modifierSettings
                    ))}
                </StyledText>
            </div>
        </>}
        {
            (props.grave.information.deathNotes.length === 0 || props.grave.information.deathNotes.map(note => <>
                {translate("grave.deathNote")}
                <div className="note-area">
                    <StyledText>
                        {encodeString(replaceMentions(
                            note,
                            props.playerNames,
                            props.roleList,
                            props.modifierSettings
                        ))}
                    </StyledText>
                </div>
            </>))
        }
    </div>;
}


function ObscuredGrave(props: Readonly<{grave: Grave, playerNames: UnsafeString[]}>): ReactElement {

    let diedPhaseString = props.grave.diedPhase === "day" ? translate("day") : translate("phase.night");
    let diedPhaseIcon = props.grave.diedPhase === "day" ? translate("day.icon") : translate("night.icon");
    let graveRoleString = translate("obscured");

    return <div className="grave">
        <div><StyledText>{`${diedPhaseString+diedPhaseIcon+props.grave.dayNumber}`}</StyledText></div>
        <div><StyledText>{`${props.playerNames[props.grave.player]+" ("+graveRoleString+")"}`}</StyledText></div>
    </div>;
}