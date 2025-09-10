import { ReactElement } from "react";
import { Role } from "../../../../../game/roleState.d";
import RoleDropdown from "../../../../../components/RoleDropdown";
import React from "react";
import { AvailableRoleListSelection, RoleListSelection } from "../../../../../game/controllerInput";
import Icon from "../../../../../components/Icon";
import { Button } from "../../../../../components/Button";

export default function RoleListSelectionMenu(props: Readonly<{
    selection: RoleListSelection,
    availableSelection: AvailableRoleListSelection,
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

    const newChoosableRoles = props.availableSelection.availableRoles.filter((p)=>
        props.availableSelection.canChooseDuplicates || !props.selection.includes(p)
    ) as Role[];

    return <div className="generic-list-controller-menu">
        <div className="generic-list-controller-menu-items">
            {
                props.selection.map((p,i)=><RoleDropdown
                    enabledRoles={props.availableSelection.availableRoles.filter((p)=>
                        props.availableSelection.canChooseDuplicates || !props.selection.includes(p) || p === props.selection[i]
                    ) as Role[]}
                    canChooseNone={true}
                    value={p}
                    onChange={(p)=>handleSelection(p, i)}
                />)
            }
            {
                (props.availableSelection?.maxRoles??Infinity) > props.selection.length ? <RoleDropdown
                    enabledRoles={newChoosableRoles}
                    canChooseNone={true}
                    value={null}
                    onChange={(p)=>handleSelection(p, props.selection.length)}
                /> : null
            }
        </div>
        <div className="generic-list-controller-menu-buttons">
            {
                ((props.availableSelection.maxRoles??Infinity) >= props.availableSelection.availableRoles.length) && newChoosableRoles.length !== 0
                ?
                    <Button
                        onClick={()=>props.onChoose(props.availableSelection.availableRoles)}
                    >
                        <Icon>select_all</Icon>
                    </Button>
                :null
            }
            {
                ((props.availableSelection.maxRoles??Infinity) > 1) &&
                props.availableSelection.availableRoles.length > 1 &&
                props.selection.length > 0
                ?
                    <Button
                        onClick={()=>props.onChoose([])}
                    >
                        <Icon>deselect</Icon>
                    </Button>
                :null
            }
        </div>
    </div>
}