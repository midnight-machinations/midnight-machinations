import { ReactElement } from "react"
import { Role } from "../game/roleState.d"
import React from "react"
import Select, { SelectOptionsSearch, set_option_typical } from "./Select"
import { getAllRoles } from "../game/roleListState.d"


type RoleDropdownProps = {
    enabledRoles?: Role[]
} & ({
    value: Role,
    onChange: (role: Role) => void,
    canChooseNone?: false
} | {
    value: Role | null,
    onChange: (role: Role | null) => void,
    canChooseNone: true
})

export default function RoleDropdown(props: RoleDropdownProps): ReactElement {

    const optionMap: SelectOptionsSearch<Role | "none"> = new Map();

    if (props.canChooseNone) {
        set_option_typical(optionMap, "none")
    }
    
    for (const role of getAllRoles()) {
        if (props.enabledRoles === undefined || props.enabledRoles.includes(role)) {
            set_option_typical(optionMap, role, "role."+role+".name")
        }
    }


    return <Select
        value={convertToLowerValue(props.value)}
        onChange={value => {
            if(props.canChooseNone){
                const newRole: Role | null = convertToHigherValue(value);
                props.onChange(newRole)
            }else{
                props.onChange(value as Role)
            }
        }}
        optionsSearch={optionMap}
    />
}

function convertToLowerValue(value: Role | null): Role | "none" {
    return value === null ? "none" : value;
}
function convertToHigherValue(value: Role | "none"): Role | null {
    return value === "none" ? null : value;
}