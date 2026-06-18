import { ReactElement, useCallback, useContext, useState } from "react"
import translate from "../../game/lang"
import StyledText from "../StyledText"
import { ROLE_SETS, RoleOrRoleSet, getAllRoles, getRolesFromRoleOrRoleSet } from "../../game/roleListState.d";
import { Role, roleJsonData } from "../../game/roleState.d";
import { RoleOrRoleSetSelector } from "./OutlineSelector";
import "./disabledRoleSelector.css"
import { Button } from "../Button";
import { GameModeContext } from "./GameModesEditor";
import CheckBox from "../CheckBox";




export default function EnabledRoleSelector(props: Readonly<{
    disabled?: boolean,
    onDisableRoles: (role: Role[]) => void,
    onEnableRoles: (role: Role[]) => void,
    onIncludeAll: () => void
}>): ReactElement {
    const {enabledRoles} = useContext(GameModeContext);

    const toggleRoleOrRoleSet = useCallback((roleOrRoleSet: RoleOrRoleSet) => {
        let enabled = false;
        if (roleOrRoleSet.type === "role") {
            enabled = enabledRoles.includes(roleOrRoleSet.role);
        } else {
            enabled = getRolesFromRoleOrRoleSet(roleOrRoleSet).some(role => enabledRoles.includes(role));
        }

        if (enabled) {
            props.onDisableRoles(getRolesFromRoleOrRoleSet(roleOrRoleSet));
        } else {
            props.onEnableRoles(getRolesFromRoleOrRoleSet(roleOrRoleSet));
        }
    }, [enabledRoles]);

    return <div className="role-specific-colors selector-section">
        <h2>{translate("menu.lobby.enabledRoles")}</h2>
        {(props.disabled !== true) && 
            <RoleOrRoleSetSelector
                disabled={props.disabled}
                displayValue={["toggle", [
                    <StyledText key="toggle" noLinks={true}>
                        {translate("menu.enabledRoles.toggle")}
                    </StyledText>,
                    translate("menu.enabledRoles.toggle")
                ]]}
                onChange={toggleRoleOrRoleSet}
                noCloseOnKeyboardSelect
            />}

        <EnabledRolesDisplay 
            enabledRoles={enabledRoles}
            modifiable={!props.disabled}
            onDisableRoles={props.onDisableRoles}
            onEnableRoles={props.onEnableRoles}
            disabled={props.disabled}
        />
    </div>
}

type EnabledRolesDisplayProps = {
    enabledRoles: Role[],
} & (
    {
        modifiable: true,
        onDisableRoles: (role: Role[]) => void,
        onEnableRoles: (role: Role[]) => void,
        disabled?: boolean,
    } |
    {
        modifiable?: false,
    }
)

export function EnabledRolesDisplay(props: EnabledRolesDisplayProps): ReactElement {
    const isEnabled = useCallback((role: Role) => props.enabledRoles.includes(role), [props.enabledRoles]);

    const roleTextElement = (role: Role) => {

        return <StyledText 
            noLinks={props.modifiable ?? false}
            className={!isEnabled(role) ? "keyword-disabled" : undefined}
        >
            {translate("role."+role+".name")}
        </StyledText>
    }

    const [hideDisabled, setHideDisabled] = useState(true);

    return <div>
        {!props.modifiable && <label className="centered-label">
            {translate("hideDisabled")}
            <CheckBox
                checked={hideDisabled}
                onChange={checked => setHideDisabled(checked)}
            />
        </label>}
        <div>
            {getAllRoles()
                .filter(role => isEnabled(role) || !hideDisabled || props.modifiable)
                .sort((a, b) => props.modifiable ? 0 : (isEnabled(a) ? -1 : 1) - (isEnabled(b) ? -1 : 1))
                .sort((a, b) => ROLE_SETS.indexOf(roleJsonData()[a].mainRoleSet) - ROLE_SETS.indexOf(roleJsonData()[b].mainRoleSet))
                .map((role, i) => 
                    props.modifiable 
                        ? <Button key={i}
                            disabled={props.disabled}
                            onClick={() => (!isEnabled(role) ? props.onEnableRoles : props.onDisableRoles)([role])}
                        >
                            {roleTextElement(role)}
                        </Button> 
                        : <div key={i} className={"placard" + (!isEnabled(role) ? " disabled" : "")}>
                            {roleTextElement(role)}
                        </div>)}
        </div>
    </div>
}