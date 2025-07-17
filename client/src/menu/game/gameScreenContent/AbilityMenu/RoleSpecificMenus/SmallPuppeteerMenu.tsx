import { ReactElement } from "react";
import Counter from "../../../../../components/Counter";
import React from "react";
import StyledText from "../../../../../components/StyledText";
import translate from "../../../../../game/lang";
import { PhaseType } from "../../../../../stateContext/stateType/phaseState";

export default function SmallPuppeteerMenu(props: Readonly<{
    marionettesRemaining: number,
    maxCharges: number,
    phase: PhaseType
}>): ReactElement {
    return <>
        <Counter
            max={props.maxCharges}
            current={props.marionettesRemaining}
        >
            <StyledText>{translate("role.puppeteer.smallRoleMenu.marionettesRemaining", props.marionettesRemaining)}</StyledText>
        </Counter>
    </>
}