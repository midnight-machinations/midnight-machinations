import type { ReactElement } from "react";
import translate from "../game/lang";
import { getAllRoles } from "../game/roleListState.d";
import type { Role } from "../game/roleState.d";
import Select, { type SelectOptionsSearch } from "./Select";
import StyledText from "./StyledText";

type RoleDropdownProps = {
    enabledRoles?: Role[];
} & (
    | {
          value: Role;
          onChange: (role: Role) => void;
          canChooseNone?: false;
      }
    | {
          value: Role | null;
          onChange: (role: Role | null) => void;
          canChooseNone: true;
      }
);

export default function RoleDropdown(props: RoleDropdownProps): ReactElement {
    const optionMap: SelectOptionsSearch<Role | "none"> = new Map();

    if (props.canChooseNone) {
        optionMap.set("none", [<StyledText noLinks={true}>{translate("none")}</StyledText>, translate("none")]);
    }

    for (const role of getAllRoles()) {
        if (props.enabledRoles === undefined || props.enabledRoles.includes(role)) {
            optionMap.set(role, [
                <StyledText noLinks={true}>{translate("role." + role + ".name")}</StyledText>,
                translate("role." + role + ".name"),
            ]);
        }
    }

    return (
        <Select
            value={convertToLowerValue(props.value)}
            onChange={(value) => {
                if (props.canChooseNone) {
                    const newRole: Role | null = convertToHigherValue(value);
                    props.onChange(newRole);
                } else {
                    props.onChange(value as Role);
                }
            }}
            optionsSearch={optionMap}
        />
    );
}

function convertToLowerValue(value: Role | null): Role | "none" {
    return value === null ? "none" : value;
}
function convertToHigherValue(value: Role | "none"): Role | null {
    return value === "none" ? null : value;
}
