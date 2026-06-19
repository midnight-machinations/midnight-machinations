import React, { ReactElement, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import { PhaseType, PhaseTimes } from "../../game/gameState.d";
import translate from "../../game/lang";
import { isValidPhaseTime } from "../../game/gameManager";
import "./phaseTimeSelector.css";
import { GameModeContext } from "./GameModesEditor";
import Popover from "../Popover";
import { dropdownPlacementFunction } from "../Select";
import { setWikiSearchPage } from "../Wiki";
import { AnchorControllerContext, MobileContext } from "../../menu/Anchor";
import { WikiArticleLink } from "../WikiArticleLink";
import StyledText from "../StyledText";
import ListMap from "../../ListMap";
import { ModifierID, ModifierState } from "../../game/modifiers";



export default function PhaseTimesSelector(props: Readonly<{
    disabled?: boolean,
    onChange: (phaseTimes: PhaseTimes) => void,
}>): ReactElement {
    const {phaseTimes} = useContext(GameModeContext);

    const onChange = (phase: Exclude<PhaseType, "recess">, time: number) => {
        if (!isValidPhaseTime(time)) return

        let newPhaseTimes = {...phaseTimes};
        newPhaseTimes[phase] = time;
        props.onChange(newPhaseTimes);
    }

    return <section className="phase-times-selector will-menu-colors selector-section">
        <h2>{translate("menu.lobby.timeSettings")}</h2>
        <PhaseTimesVisualizer phaseTimes={phaseTimes} disabled={props.disabled} onChange={onChange} />
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

function PhaseTimesVisualizer(props: Readonly<{
    phaseTimes: PhaseTimes
    disabled?: boolean
    onChange: (phase: Exclude<PhaseType, "recess">, time: number) => void
}>): ReactElement {
    const { phaseTimes } = props;

    const gameModeContext = useContext(GameModeContext);

    const modifiers = useMemo(() => {
        return new ListMap(gameModeContext.modifierSettings);
    }, [gameModeContext.modifierSettings]);

    const phaseOrder = useMemo(() => getPhaseOrder(modifiers), [modifiers]);

    const totalPhaseTime = useMemo(() => {
        return phaseOrder.reduce((acc, phase) => acc + getRealPhaseTime(phase, phaseTimes), 0);
    }, [phaseTimes, phaseOrder]);

    const phaseEntries = useMemo(() => {
        const entries: [Exclude<PhaseType, "recess">, {
            ratio: number,
            baseColor: string,
            key: number
        }][] = [];

        for (const { index, phase } of phaseOrder.map((phase, index) => ({ index, phase }))) {
            const time = getRealPhaseTime(phase, phaseTimes);
            const baseColor = getPhaseBaseColor(phase);
            entries.push([phase, { ratio: time / totalPhaseTime, baseColor, key: index }]);
        }

        return entries;
    }, [phaseTimes, phaseOrder, totalPhaseTime]);

    return <div className="phase-times-visualizer">
        <div className="phase-times-visualizer-scroll">
            {phaseEntries.map(([phase, { ratio, baseColor, key }]) => (
                <PhaseTimesVisualizerPhase
                    key={key}
                    phase={phase}
                    ratio={ratio}
                    baseColor={baseColor}
                    disabled={props.disabled}
                    time={phaseTimes[phase]}
                    onChange={props.onChange}
                />
            ))}
        </div>
    </div>
}

function getRealPhaseTime(phase: Exclude<PhaseType, "recess">, phaseTimes: PhaseTimes): number {
    let time = phaseTimes[phase];
    return time;
}

function getPhaseOrder(modifiers: ListMap<ModifierID, ModifierState>): Exclude<PhaseType, "recess">[] {
    const preTrialPhases: Exclude<PhaseType, "recess">[] = [
        "briefing",
        "dusk",
        "night",
        "obituary",
        "discussion",
    ];

    if (modifiers.get("noTrialPhases") !== null) {
        return preTrialPhases;
    }

    return [
        ...preTrialPhases,
        "nomination",
        "adjournment",
        "nomination",
        "adjournment",
        "nomination",
        "testimony",
        "judgement",
        "finalWords",
    ]
}

function PhaseTimesVisualizerPhase(props: Readonly<{
    phase: Exclude<PhaseType, "recess">,
    ratio: number,
    baseColor: string
    onChange: (phase: Exclude<PhaseType, "recess">, time: number) => void
    disabled?: boolean
    time: number
}>): ReactElement {
    const anchorController = useContext(AnchorControllerContext)!;

    const isMobile = useContext(MobileContext);

    const { phase, ratio, time, baseColor } = props;
    
    const [open, setOpen] = useState<"show" | "edit" | "closed">("closed");

    const ref = useRef<HTMLButtonElement>(null);

    const inputRef = useRef<HTMLInputElement | null>(null);

    const focusInput = useCallback((input: HTMLInputElement | null) => {
        inputRef.current = input;

        if (!input) return;

        requestAnimationFrame(() => {
            input.focus();
            input.select();
        });
    }, []);

    useEffect(() => {
        if (open !== "edit") return;

        const timeout = setTimeout(() => {
            inputRef.current?.focus();
            inputRef.current?.select();
        });

        return () => clearTimeout(timeout);
    }, [open]);

    const onClick = useCallback(() => {
        if (props.disabled) {
            if (isMobile) {
                setOpen(current => current === "closed" ? "show" : "closed");
                return;
            }
            setWikiSearchPage('standard/' + phase as WikiArticleLink, anchorController);
            return;
        }

        setOpen(current => current === "edit" ? "closed" : "edit");
    }, [props.disabled, phase, anchorController]);

    const onMouseEnter = useCallback(() => {
        if (isMobile) return;
        setOpen(current => current === "closed" ? "show" : current);
    }, []);

    const onMouseLeave = useCallback(() => {
        if (isMobile) return;
        setOpen(current => current === "show" ? "closed" : current);
    }, []);

    const handleMouseDown = useDragToChangeValue(time, time => {
        if (isMobile) return;
        setOpen(current => current === "edit" ? current : "show");
        props.onChange(props.phase, time)
    });

    return <>
        <button style={{
            width: `${ratio * 100}%`,
            backgroundColor: baseColor
        }}
            onClick={onClick}
            onMouseEnter={onMouseEnter}
            onMouseLeave={onMouseLeave}
            onMouseDown={(e) => {
                if (isMobile) return;
                setOpen(current => current === "closed" ? "show" : current);
                handleMouseDown(e)
            }}
            onFocus={() => {
                if (isMobile) return;
                setOpen(current => current === "closed" ? "show" : current)
            }}
            onBlur={() => {
                if (isMobile) return;
                setOpen(current => current === "show" ? "closed" : current)
            }}
            ref={ref}
        >{translate(`phase.${phase}`)}</button>
        <Popover
            open={open !== "closed"}
            setOpenOrClosed={(open) => open ? setOpen(current => current === "closed" ? "show" : current) : setOpen("closed")}
            anchorForPositionRef={ref}
            onRender={dropdownPlacementFunction}
        >
            {open !== "edit" && <div className="placard">
                <StyledText noLinks={true}>{translate(`phase.${phase}`)}: {"" + time}</StyledText>
            </div>}
            {open === "edit" && <div className="placard">
                <input
                    className="phase-time-input"
                    disabled={props.disabled}
                    defaultValue={time}
                    style={{ display: open === "edit" ? undefined : "none"}}
                    type="number"
                    ref={focusInput}
                    autoFocus
                    onKeyUp={(e) => {
                        if (e.key !== "Enter") return;

                        const value = Number.parseInt((e.target as HTMLInputElement).value);
                        if (!isValidPhaseTime(value)) return

                        props.onChange(props.phase, value)
                        setOpen("closed");
                    }}
                />
            </div>}
        </Popover>
    </>
}

function getPhaseBaseColor(phase: Exclude<PhaseType, "recess">): string {
    switch (phase) {
        case "briefing":
            return "#725548";
        case "dusk":
            return "#5b4292";
        case "night":
            return "#430752";
        case "discussion":
            return "#5058ce";
        case "obituary":
            return "#352d56";
        case "nomination":
            return "#42853c";
        case "adjournment":
            return "#2b8599";
        case "testimony":
            return "#a89225";
        case "judgement":
            return "#613f26";
        case "finalWords":
            return "#971111";
    }
}

function useDragToChangeValue(startValue: number, onChange: (value: number) => void) {
    const [startY, setStartY] = useState<number | null>(null);

    useEffect(() => {
        if (startY === null) return;
        
        // Add global listeners so dragging works even if the cursor leaves the button
        globalThis.addEventListener('mousemove', handleMouseMove);
        globalThis.addEventListener('mouseup', handleMouseUp);

        return () => {
            globalThis.removeEventListener('mousemove', handleMouseMove);
            globalThis.removeEventListener('mouseup', handleMouseUp);
        };
    }, [startY]);

    const handleMouseDown = (e: React.MouseEvent<HTMLButtonElement>) => {
        setStartY(e.clientY);
    };

    const handleMouseMove = (e: MouseEvent) => {
        if (startY === null) return;
        const deltaY = e.clientY - startY;
        onChange(startValue - Math.floor(deltaY / 10));
    };

    const handleMouseUp = () => {
        globalThis.removeEventListener('mousemove', handleMouseMove);
        globalThis.removeEventListener('mouseup', handleMouseUp);
    };

    return handleMouseDown;
}
