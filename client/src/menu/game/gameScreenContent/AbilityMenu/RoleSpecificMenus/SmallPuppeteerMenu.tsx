import type { ReactElement } from "react";
import Counter from "../../../../../components/Counter";
import StyledText from "../../../../../components/StyledText";
import type { PhaseType } from "../../../../../game/gameState.d";
import translate from "../../../../../game/lang";

export default function SmallPuppeteerMenu(
    props: Readonly<{ marionettesRemaining: number; maxCharges: number; phase: PhaseType }>,
): ReactElement {
    return (
        <Counter max={props.maxCharges} current={props.marionettesRemaining}>
            <StyledText>
                {translate("role.puppeteer.smallRoleMenu.marionettesRemaining", props.marionettesRemaining)}
            </StyledText>
        </Counter>
    );
}
