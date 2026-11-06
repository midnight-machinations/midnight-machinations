import type React from "react";
import type { ReactElement } from "react";
import Select from "../../../../../components/Select";
import StyledText from "../../../../../components/StyledText";
import {
    type AvailableIntegerSelection,
    type ControllerID,
    controllerIdToLink,
    type IntegerSelection,
} from "../../../../../game/controllerInput";
import { translateChecked } from "../../../../../game/lang";

export default function IntegerSelectionMenu(
    props: Readonly<{
        id?: ControllerID;
        selection: IntegerSelection;
        available: AvailableIntegerSelection;
        onChoose: (number: number) => void;
    }>,
): ReactElement {
    ///make array with numbers from available.min to available.max
    const options = [];
    for (let i = props.available.min; i <= props.available.max; i++) {
        options.push(i);
    }

    const optionsSearch = new Map<number, [React.ReactNode, string]>();
    for (let i = props.available.min; i <= props.available.max; i++) {
        let out: [React.ReactNode, string] = [i, i.toString()];

        if (props.id !== undefined) {
            const text = translateChecked(
                "controllerId." + controllerIdToLink(props.id).replace(/\//g, ".") + ".integer." + i,
            );

            if (text !== null) out = [<StyledText noLinks={true}>{text}</StyledText>, text];
        }

        optionsSearch.set(i, out);
    }

    return (
        <div>
            <Select optionsSearch={optionsSearch} value={props.selection} onChange={(s: number) => props.onChoose(s)} />
        </div>
    );
}
