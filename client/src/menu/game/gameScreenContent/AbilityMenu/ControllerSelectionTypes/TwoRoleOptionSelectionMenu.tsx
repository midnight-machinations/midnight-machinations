import { ReactElement } from "react";
import { AvailableTwoRoleOptionSelection, TwoRoleOptionSelection } from "../../../../../game/controllerInput";
import React from "react";
import RoleDropdown from "../../../../../components/RoleDropdown";
import { Role } from "../../../../../game/roleState.d";

export default function TwoRoleOptionSelectionMenu(props: Readonly<{
    input: TwoRoleOptionSelection,
    availableSelection: AvailableTwoRoleOptionSelection,
    onChoose: (input: TwoRoleOptionSelection)=>void
}>): ReactElement {

    const handleOnChange = (index: 0 | 1, roleOption: Role | null) => {
        const newInput: TwoRoleOptionSelection = [props.input[0], props.input[1]];
        newInput[index] = roleOption;
        props.onChoose(newInput);
    }

    
    const enabledRoles = props.availableSelection.availableRoles.filter(role=>role!==null) as Role[];
    const canChooseNone = props.availableSelection.availableRoles.includes(null);

    return <div>
        <Dropdown
            enabledRoles={enabledRoles}
            canChooseNone={canChooseNone}
            index={0}
            input={props.input}
            handleOnChange={handleOnChange}
        />
        <Dropdown
            enabledRoles={enabledRoles}
            canChooseNone={canChooseNone}
            index={1}
            input={props.input}
            handleOnChange={handleOnChange}
        />
    </div>;
}

function Dropdown(props: Readonly<{
    enabledRoles: Role[],
    canChooseNone: boolean,
    index: 0 | 1,
    input: TwoRoleOptionSelection,
    handleOnChange: (index: 0 | 1, roleOption: Role | null) => void,
}>): ReactElement {
    return <>{props.canChooseNone === true ? 
        <RoleDropdown
            value={props.input[props.index]}
            enabledRoles={props.enabledRoles}
            onChange={(roleOption: Role | null)=>{props.handleOnChange(props.index, roleOption)}}
            canChooseNone={true}
        /> : 
        <RoleDropdown
            value={(props.input[props.index] === null ? props.enabledRoles[0] : props.input[props.index]) as Role}
            enabledRoles={props.enabledRoles}
            onChange={(roleOption: Role)=>{props.handleOnChange(props.index, roleOption)}}
            canChooseNone={false}
        />
    }</>
}