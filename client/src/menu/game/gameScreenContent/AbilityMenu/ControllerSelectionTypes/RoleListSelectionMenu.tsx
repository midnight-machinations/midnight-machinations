import { ReactElement } from "react";
import { Role } from "../../../../../game/roleState.d";
import RoleDropdown from "../../../../../components/RoleDropdown";
import React from "react";
import { AvailableRoleListSelection, RoleListSelection } from "../../../../../game/controllerInput";

export default function RoleListSelectionMenu(props: Readonly<{
    selection: RoleListSelection,
    availableSelection?: AvailableRoleListSelection,
    onChoose: (role: Role[])=>void,
}>): ReactElement {
    const handleSelection = (player: Role | null, index: number) => {
        let newSelection: RoleListSelection = props.selection.slice();

        if(index >= newSelection.length && player !== null){
            newSelection.push(player);
        }else{
            if(player === null){
                newSelection = newSelection.slice(0,index).concat(newSelection.slice(index+1));
            }else{
                newSelection[index] = player;
            }
        }
        
        props.onChoose(newSelection);
    }

    return <div>
        {
            props.selection.map((p,i)=><RoleDropdown
                enabledRoles={props.availableSelection?.availableRoles}
                canChooseNone={true}
                value={p}
                onChange={(p)=>handleSelection(p, i)}
            />)
        }
        {
            (props.availableSelection?.maxRoles??Infinity) > props.selection.length ? <RoleDropdown
                enabledRoles={props.availableSelection?.availableRoles}
                canChooseNone={true}
                value={null}
                onChange={(p)=>handleSelection(p, props.selection.length)}
            /> : null
        }
    </div>
}