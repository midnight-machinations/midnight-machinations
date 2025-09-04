import { ReactElement } from "react";
import React from "react";
import { AvailableIntegerSelection, ControllerID, controllerIdToLink, IntegerSelection } from "../../../../../game/controllerInput";
import Select from "../../../../../components/Select";
import { translateChecked } from "../../../../../game/lang";
import StyledText from "../../../../../components/StyledText";

export default function IntegerSelectionMenu(props: Readonly<{
    id?: ControllerID
    selection: IntegerSelection,
    available: AvailableIntegerSelection,
    onChoose: (number: number)=>void,
}>): ReactElement {

    ///make array with numbers from available.min to available.max
    let options = [];
    for(let i = props.available.min; i <= props.available.max; i++){
        options.push(i);
    }

    let optionsSearch = new Map<number, [React.ReactNode, string]>();
    for(let i = props.available.min; i <= props.available.max; i++){
        let out: [React.ReactNode, string] = [<>{i}</>, i.toString()];

        if(props.id !== undefined){
            let text = translateChecked("controllerId."+controllerIdToLink(props.id).replace(/\//g, ".") + ".integer." + i);
            
            if(text !== null)
                out = [<StyledText noLinks={true}>{text}</StyledText>, text];
        }

        optionsSearch.set(i, out);
    }

    return <div>
        <Select
            optionsSearch={optionsSearch}
            value={props.selection}
            onChange={(s: number)=>props.onChoose(s)}
        />
    </div>
}