import { ReactElement, useMemo } from "react";
import { Role } from "../../../../../game/roleState.d";
import RoleDropdown from "../../../../../components/RoleDropdown";
import React from "react";
import { GraveDeathCausesSelection } from "../../../../../game/controllerInput";
import Icon from "../../../../../components/Icon";
import { Button } from "../../../../../components/Button";
import { GraveDeathCause, translateGraveDeathCause, translateGraveDeathCauses } from "../../../../../game/graveState";
import Select, { SelectOptionsSearch } from "../../../../../components/Select";
import translate from "../../../../../game/lang";
import StyledText from "../../../../../components/StyledText";
import { getAllRoles, getAllRoleSets, RoleSet } from "../../../../../game/roleListState.d";

export default function GraveDeathCausesSelectionMenu(props: Readonly<{
    selection: GraveDeathCausesSelection,
    onChoose: (cause: GraveDeathCause[])=>void,
}>): ReactElement {

    const handleSelection = (cause: GraveDeathCause | null, index: number) => {
        let newSelection: GraveDeathCausesSelection = props.selection.slice();

        if(index >= newSelection.length && cause !== null){
            newSelection.push(cause);
        }else{
            if(cause === null){
                newSelection = newSelection.slice(0,index).concat(newSelection.slice(index+1));
            }else{
                newSelection[index] = cause;
            }
        }
        
        props.onChoose(newSelection);
    }

    return <div className="generic-list-controller-menu">
        {
            props.selection.map((p,i)=><GraveDeathCauseDropdown
                key={i}
                canChooseNone={true}
                value={p}
                onChange={(p)=>handleSelection(p, i)}
            />)
        }
        {
            <GraveDeathCauseDropdown
                canChooseNone={true}
                value={null}
                onChange={(p)=>handleSelection(p, props.selection.length)}
            />
        }
        <div>
            {
                props.selection.length > 0
                ?
                    <Button
                        className="flush"
                        onClick={()=>props.onChoose([])}
                    >
                        <Icon>deselect</Icon>
                    </Button>
                :null
            }
        </div>
    </div>
}
type GraveDeathCauseDropdownProps = ({
    value: GraveDeathCause,
    onChange: (cause: GraveDeathCause) => void,
    canChooseNone?: false
} | {
    value: GraveDeathCause | null,
    onChange: (cause: GraveDeathCause | null) => void,
    canChooseNone: true
})
function GraveDeathCauseDropdown(props: Readonly<GraveDeathCauseDropdownProps>): ReactElement {
    // cant use GraveDeathCause here in the map because you cant compare them with == which according to thomas isnt a problem in javascript
    //if we were using a coding language (as opposed to a language created to make people sad) then this next line would say something like
    //const optionMap: SelectOptionsSearch<GraveDeathCause | "none"> = new Map();
    const optionMap: SelectOptionsSearch<string | "none"> = new Map();
    
    if (props.canChooseNone){
        optionMap.set(
            "none", 
            [<StyledText noLinks={true}>{translate("none")}</StyledText>, translate("none")]
        );
    }
    
    for (const cause of getAllGraveDeathCauses()) {
        optionMap.set(
            graveDeathCauseLink(cause),
            [<StyledText noLinks={true}>{translateGraveDeathCause(cause)}</StyledText>, translateGraveDeathCause(cause)]
        );
    }

    return <Select
        value={graveDeathCauseLink(props.value)}
        onChange={value => {
            if(props.canChooseNone){
                const newRole: GraveDeathCause | null = graveDeathCauseFromLink(value);
                props.onChange(newRole)
            }else{
                props.onChange(graveDeathCauseFromLink(value) as GraveDeathCause)
            }
        }}
        optionsSearch={optionMap}
    />
}

function getAllGraveDeathCauses(): GraveDeathCause[] {
    let out: GraveDeathCause[] = [];
    for (const type of ["execution", "ascension", "suicide", "quit"]) {
        out.push({ type: type as "execution" | "ascension" | "suicide" | "quit" });
    }
    for (const role of getAllRoles()) {
        out.push({ type: "role", value: role });
    }
    for (const roleSet of getAllRoleSets()) {
        out.push({ type: "roleSet", value: roleSet });
    }
    return out;
}
function graveDeathCauseLink(cause: GraveDeathCause | null): string | "none" {
    if(cause===null){return "none"}
    switch(cause.type){
        case "ascension":
        case "execution":
        case "quit":
        case "suicide":
            return cause.type;
        case "role":
        case "roleSet":
            return cause.value;
    }
}
function graveDeathCauseFromLink(cause: string): GraveDeathCause | null {
    if(cause === "none"){return null};
    switch(cause){
        case "ascension":
        case "execution":
        case "quit":
        case "suicide":
            return {type: cause}
    }
    if(getAllRoles().includes(cause as Role)){
        return {type: "role", value: cause as any};
    }
    if(getAllRoleSets().includes(cause as RoleSet)){
        return {type: "roleSet", value: cause as any};
    }
    return null;
}