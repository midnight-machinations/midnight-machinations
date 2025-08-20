import { ReactElement } from "react";
import React from "react";
import { ControllerID, BooleanSelection, controllerIdToLink } from "../../../../../game/controllerInput";
import Select from "../../../../../components/Select";
import translate, { translateChecked } from "../../../../../game/lang";
import StyledText from "../../../../../components/StyledText";
import CheckBox from "../../../../../components/CheckBox";

export default function BooleanSelectionMenu(props: Readonly<{
    id?: ControllerID
    selection: BooleanSelection,
    onChoose: (bool: BooleanSelection)=>void,
}>): ReactElement {

    if(
        props.id === undefined ||
        translateChecked("controllerId."+controllerIdToLink(props.id).replace(/\//g, ".") + ".boolean.true") === null ||
        translateChecked("controllerId."+controllerIdToLink(props.id).replace(/\//g, ".") + ".boolean.false") === null
    ){
        return <BooleanSelectionMenuUnnamed {...props}/>;
    }else{
        return <BooleanSelectionMenuNamed {...props as { id: ControllerID, selection: BooleanSelection, onChoose: (bool: BooleanSelection) => void }}/>;
    }
}

function BooleanSelectionMenuUnnamed(props: Readonly<{
    selection: BooleanSelection,
    onChoose: (bool: BooleanSelection)=>void,
}>): ReactElement {
    
    return <div>
        <CheckBox
            checked={props.selection}
            onChange={(x)=>{
                props.onChoose(x);
            }}
        />
    </div>
}

function BooleanSelectionMenuNamed(props: Readonly<{
    id: ControllerID
    selection: BooleanSelection,
    onChoose: (bool: BooleanSelection)=>void,
}>): ReactElement {
    ///make array with numbers from available.min to available.max
    let optionsSearch = new Map<boolean, [React.ReactNode, string]>();

    let trueText = translate("controllerId."+controllerIdToLink(props.id).replace(/\//g, ".") + ".boolean.true");
    optionsSearch.set(true, [<StyledText noLinks={true}>{trueText}</StyledText>, trueText]);
    
    let falseText = translate("controllerId."+controllerIdToLink(props.id).replace(/\//g, ".") + ".boolean.false");
    optionsSearch.set(false, [<StyledText noLinks={true}>{falseText}</StyledText>, falseText]);
    
    return <div>
        <Select
            optionsSearch={optionsSearch}
            value={props.selection}
            onChange={(s: boolean)=>props.onChoose(s)}
        />
    </div>
}