import { ReactElement } from "react";
import Counter from "../../../../../components/Counter";
import React from "react";
import StyledText from "../../../../../components/StyledText";
import translate from "../../../../../game/lang";
import { PhaseType } from "../../../../../stateContext/stateType/phaseState";

export default function RecruiterMenu(props: {
    remaining: number, 
    phase: PhaseType, 
    dayNumber: number
}): ReactElement {
    return <>
        <Counter
            max={Math.max(5, props.remaining)}
            current={props.remaining}
        >
            <StyledText>{translate("role.recruiter.smallRoleMenu.recruitsRemaining", props.remaining)}</StyledText>
        </Counter>
    </>
}