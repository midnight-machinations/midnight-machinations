import type { ReactElement } from "react";
import { type FastForwardSetting, PHASES, type PhaseType } from "../game/gameState.d";
import translate from "../game/lang";
import Icon from "./Icon";
import Select, { type SelectOptionsSearch } from "./Select";
import StyledText from "./StyledText";
import { useGameState } from "./useHooks";
import "./fastForwardButton.css";

type FastForwardSettingString = "none" | "skip" | `phase/${PhaseType}/${number}`;
function fastForwardSettingToString(fastForward: FastForwardSetting): FastForwardSettingString {
    return fastForward.type === "phase" ? `phase/${fastForward.phase}/${fastForward.day}` : fastForward.type;
}
function stringToFastForwardSetting(string: FastForwardSettingString): FastForwardSetting {
    const split = string.split("/");
    switch (split[0]) {
        case "none":
        case "skip":
            return { type: split[0] };
        case "phase":
            return { type: "phase", phase: split[1] as PhaseType, day: parseInt(split[2]) };
    }
    return { type: "none" };
}

export default function FastForwardButton(props: {
    value: FastForwardSetting;
    onChange: (value: FastForwardSetting) => void;
}): ReactElement {
    const fastForward = props.value;
    const currentPhase = useGameState((gameState) => gameState.phaseState.type, ["phase"])!;
    const dayNumber = useGameState((gameState) => gameState.dayNumber, ["phase"])!;

    const optionMap: SelectOptionsSearch<FastForwardSettingString> = new Map();

    optionMap.set("none", [<Icon>play_arrow</Icon>, translate("none")]);
    optionMap.set("skip", [<Icon>fast_forward</Icon>, translate("wiki.article.standard.fastForward.title")]);

    for (let i = dayNumber; i < Math.min(dayNumber + 4, 255); i++) {
        for (const phase of ["discussion", "nomination", "night"] as PhaseType[]) {
            if (
                (i > 1 || phase === "night") &&
                (dayNumber !== i || PHASES.indexOf(currentPhase) < PHASES.indexOf(phase))
            ) {
                optionMap.set(fastForwardSettingToString({ type: "phase", phase: phase, day: i }), [
                    <>
                        <Icon>fast_forward</Icon>{" "}
                        <StyledText noLinks={true}>
                            {translate("phase." + phase)} {i.toString()}
                        </StyledText>
                    </>,
                    translate("phase." + phase) + " " + i.toString(),
                ]);
            }
        }
    }

    return (
        <Select
            className={"brand fast-forward-button" + (fastForward.type !== "none" ? " highlighted" : "")}
            value={fastForwardSettingToString(fastForward)}
            onChange={(e) => {
                props.onChange(stringToFastForwardSetting(e));
            }}
            optionsSearch={optionMap}
            hideArrow
        />
    );
}
