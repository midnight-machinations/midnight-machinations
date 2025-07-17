
import { replaceMentions } from "..";
import translate from "../game/lang";
import { sanitizePlayerMessage } from "./ChatMessage";
import StyledText from "./StyledText";
import React, { ReactElement, useContext, useMemo } from "react";
import "./grave.css";
import { Grave, GraveInformation } from "../stateContext/stateType/grave";
import { StateContext } from "../stateContext/StateContext";

export function translateGraveRole(graveInformation: GraveInformation): string {
    if(graveInformation.type === "obscured") {
        return translate("obscured");
    }else{
        return translate(`role.${graveInformation.role}.name`);
    }
}

export default function GraveComponent(props: Readonly<{
    grave: Grave, 
    playerNames?: string[]
    onClick?: () => void
}>): ReactElement {
    const gamePlayerNames = useContext(StateContext)!.players.map(player => player.toString());

    const playerNames = props.playerNames ?? gamePlayerNames;


    if(props.grave.information.type === "obscured") {
        return <ObscuredGrave grave={props.grave} playerNames={playerNames}/>
    } else {
        return <UnobscuredGrave grave={props.grave as any} playerNames={playerNames}/>;
    }
}

function UnobscuredGrave(props: Readonly<{
    grave: Grave & { information: GraveInformation & { type: "normal" } },
    playerNames: string[]
    onClick?: () => void
}>): ReactElement {
    const graveDeathCause = useMemo(() => {
        if(props.grave.information.deathCause.type === "killers") {
            return props.grave.information.deathCause.killers.map((killer)=>{
                switch(killer.type){
                    case "role":
                        return translate("role."+killer.value+".name");
                    case "roleSet":
                        return translate(killer.value);
                    default:
                        return translate("grave.killer."+killer.type);
                }
            }).join(", ") + ".";
        } else if (props.grave.information.deathCause.type === "none") {
            return null;
        } else {
            return translate("grave.deathCause."+props.grave.information.deathCause.type);
        }
    }, [props.grave.information.deathCause]);

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
        {props.grave.information.will.length === 0 || <>
            {translate("alibi")}
            <div className="note-area">
                <StyledText>
                    {sanitizePlayerMessage(replaceMentions(
                        props.grave.information.will,
                        props.playerNames
                    ))}
                </StyledText>
            </div>
        </>}
        {
            (props.grave.information.deathNotes.length === 0 || props.grave.information.deathNotes.map(note => <>
                {translate("grave.deathNote")}
                <div className="note-area">
                    <StyledText>
                        {sanitizePlayerMessage(replaceMentions(
                            note,
                            props.playerNames
                        ))}
                    </StyledText>
                </div>
            </>))
        }
    </div>;
}


function ObscuredGrave(props: Readonly<{grave: Grave, playerNames: string[]}>): ReactElement {

    let diedPhaseString = props.grave.diedPhase === "day" ? translate("day") : translate("phase.night");
    let diedPhaseIcon = props.grave.diedPhase === "day" ? translate("day.icon") : translate("night.icon");
    let graveRoleString = translate("obscured");

    return <div className="grave">
        <div><StyledText>{`${diedPhaseString+diedPhaseIcon+props.grave.dayNumber}`}</StyledText></div>
        <div><StyledText>{`${props.playerNames[props.grave.player]+" ("+graveRoleString+")"}`}</StyledText></div>
    </div>;
}