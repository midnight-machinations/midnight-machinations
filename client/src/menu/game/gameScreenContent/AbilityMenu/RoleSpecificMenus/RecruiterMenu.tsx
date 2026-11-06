import type { ReactElement } from "react";
import Counter from "../../../../../components/Counter";
import StyledText from "../../../../../components/StyledText";
import type { PhaseType } from "../../../../../game/gameState.d";
import translate from "../../../../../game/lang";

export default function RecruiterMenu(props: { remaining: number; phase: PhaseType; dayNumber: number }): ReactElement {
    return (
        <Counter max={Math.max(5, props.remaining)} current={props.remaining}>
            <StyledText>{translate("role.recruiter.smallRoleMenu.recruitsRemaining", props.remaining)}</StyledText>
        </Counter>
    );
}
