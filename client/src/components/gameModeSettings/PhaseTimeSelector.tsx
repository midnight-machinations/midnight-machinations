import React, { ReactElement, useContext } from "react";
import translate from "../../game/lang";
import "./phaseTimeSelector.css";
import { GameModeContext } from "./GameModesEditor";
import { PHASES, PhaseType } from "../../stateContext/stateType/phaseState";
import { PhaseTimes } from "../../stateContext/stateType/otherState";



export default function PhaseTimesSelector(props: Readonly<{
    disabled?: boolean,
    onChange: (phaseTimes: PhaseTimes) => void,
}>): ReactElement {
    const {phaseTimes} = useContext(GameModeContext);

    const onChange = (phase: Exclude<PhaseType, "recess">, time: number) => {
        let newPhaseTimes = {...phaseTimes};
        newPhaseTimes[phase] = time;
        props.onChange(newPhaseTimes);
    }

    return <section className="phase-times-selector will-menu-colors selector-section">
        <h2>{translate("menu.lobby.timeSettings")}</h2>
        <div className="phase-times">
            {PHASES.map(phase => {
                if (phase === "recess") return null;
                return <PhaseTimeSelector key={phase} disabled={props.disabled} phase={phase} time={phaseTimes[phase]} onChange={onChange}/>
            })}
        </div>
    </section>
}


function PhaseTimeSelector(props: Readonly<{
    disabled?: boolean,
    phase: Exclude<PhaseType, "recess">,
    time: number,
    onChange: (phase: Exclude<PhaseType, "recess">, time: number) => void,
}>): ReactElement {
    const phaseKey = "phase." + props.phase;
    
    return <div className="placard">
        <span>{translate(phaseKey)}</span>
        {props.disabled
            ? props.time
            : <input
                disabled={props.disabled ?? false}
                name={phaseKey}
                type="text"
                value={props.time}
                onChange={(e)=>{
                    const value = Number(e.target.value);

                    if (!isValidPhaseTime(value)) return
                    
                    props.onChange(props.phase, value);
                    
                }}
                onKeyUp={(e)=>{
                    if(e.key !== 'Enter') return;
                    
                    props.onChange(props.phase, props.time);
                }}
            /> 
        }
    </div>
}

export function isValidPhaseTime(time: number) {
    return Number.isSafeInteger(time) && time <= 1000 && 0 <= time;
}