import React, { ReactElement, useCallback, useContext, useEffect, useRef, useState } from "react";
import { CustomRoleSetsModifierState, MODIFIERS, ModifierID, ModifierState, defaultModifierState, isModifierConfigurable } from "../../game/modifiers";
import translate from "../../game/lang";
import StyledText from "../StyledText";
import { GameModeContext } from "./GameModesEditor";
import { Button, RawButton } from "../Button";
import CheckBox from "../CheckBox";
import ListMap, { ListMapData } from "../../ListMap";
import Popover from "../Popover";
import Select, { dropdownPlacementFunction } from "../Select";
import { Role, roleJsonData } from "../../game/roleState.d";
import Icon from "../Icon";
import { BASE_ROLE_SETS, getAllRoles, getRolesFromRoleSet, RoleOrRoleSet, RoleSet, sortRolesCanonically, translateRoleSet } from "../../game/roleListState.d";
import { ModifierSettings, UnsafeString } from "../../game/gameState.d";
import { encodeString } from "../ChatMessage";
import { RoleOrRoleSetSelector } from "./OutlineSelector";

export function ModifiersSelector(props: Readonly<{
    disabled?: boolean,
    setModifiers?: (modifiers: ListMapData<ModifierID, ModifierState>) => void
}>): ReactElement {
    const { modifierSettings } = useContext(GameModeContext);

    return <div className="chat-menu-colors selector-section">
        <h2>{translate("modifiers")}</h2>
        <ModifierSettingsDisplay
            disabled={props.disabled ?? false}
            modifiable={!props.disabled}
            modifierSettings={modifierSettings}
            setModifiers={props.setModifiers!}
        />
    </div>
}

function ModifierButton(props: Readonly<{
    modifier: ModifierID,
    modifiable: boolean,
    state: ModifierState | undefined,
    setModifier: (modifier: ModifierState | undefined) => void
    modifierSettings: Readonly<ModifierSettings>
}>): ReactElement {
    const ref = useRef<HTMLButtonElement>(null);
    const [open, setOpen] = useState(false);

    useEffect(() => {
        if (props.state === undefined) {
            setOpen(false);
        }
    }, [props.state]);

    return <>
        <RawButton
            ref={ref}
            onClick={() => {
                if (!props.modifiable) {
                    // We already know it's configurable, because why else would this be a button
                    setOpen(!open);
                    return;
                }
                if (open) {
                    props.setModifier(undefined);
                    setOpen(false);
                } else {
                    if (props.state === undefined) {
                        props.setModifier(defaultModifierState(props.modifier));
                        if (isModifierConfigurable(props.modifier)) {
                            setOpen(true);
                        }
                    } else {
                        if (isModifierConfigurable(props.modifier)) {
                            setOpen(true);
                        } else {
                            props.setModifier(undefined);
                            setOpen(false);
                        }
                    }
                }
            }}
        >
            <StyledText 
                className={props.state === undefined ? "keyword-disabled" : ""}
                noLinks={true}
            >{translate(props.modifier)}</StyledText>
        </RawButton>
        <Popover
            className="modifier-config-popover"
            open={open}
            setOpenOrClosed={setOpen}
            anchorForPositionRef={ref}
            onRender={dropdownPlacementFunction}
            doNotCloseOnOutsideClick={props.modifiable}
        >
            <ModifierConfigMenu
                modifier={props.modifier}
                state={props.state}
                modifiable={props.modifiable}
                setModifier={props.setModifier}
                close={() => setOpen(false)}
                modifierSettings={props.modifierSettings}
            />
        </Popover>
    </>
}

type ModifierSettingsDisplayProps = {
    modifierSettings: ListMapData<ModifierID, ModifierState>,
} & (
    {
        modifiable: true,
        setModifiers: (modifiers: ListMapData<ModifierID, ModifierState>) => void,
        disabled?: boolean,
    } |
    {
        modifiable?: false,
    }
)

export function ModifierSettingsDisplay(props: ModifierSettingsDisplayProps): ReactElement {
    const isEnabled = useCallback((modifier: ModifierID) => {
        return new ListMap(props.modifierSettings).get(modifier) !== null;
    }, [props.modifierSettings]);

    const modifierTextElement = (modifier: ModifierID) => {
        return <StyledText 
            noLinks={props.modifiable ?? false}
            className={!isEnabled(modifier) ? "keyword-disabled" : undefined}
        >
            {translate(modifier)}
        </StyledText>
    }

    const [hideDisabled, setHideDisabled] = useState(true);

    const modifierSettingsForReading: Readonly<ModifierSettings> = new ListMap(props.modifierSettings);

    return <div>
        {!props.modifiable && <label className="centered-label">
            {translate("hideDisabled")}
            <CheckBox
                checked={hideDisabled}
                onChange={checked => setHideDisabled(checked)}
            />
        </label>}
        <div className="modifier-settings-display">
            {MODIFIERS
                .filter(role => isEnabled(role) || !hideDisabled || props.modifiable)
                .sort((a, b) => props.modifiable ? 0 : (isEnabled(a) ? -1 : 1) - (isEnabled(b) ? -1 : 1))
                .map((modifier) => 
                    (props.modifiable || isModifierConfigurable(modifier)) 
                        ? <ModifierButton 
                            key={modifier} 
                            modifier={modifier} 
                            modifiable={props.modifiable ?? false}
                            state={new ListMap(props.modifierSettings).get(modifier) ?? undefined} 
                            setModifier={(state) => {
                                const newModifiers = new ListMap(props.modifierSettings);
                                if (state) {
                                    // Add or update
                                    newModifiers.insert(modifier, state);
                                } else {
                                    // Remove
                                    newModifiers.delete(modifier);
                                }
                                if (props.modifiable) {
                                    props.setModifiers(newModifiers.list);
                                }
                            }}
                            modifierSettings={modifierSettingsForReading}
                        />
                        : <div key={modifier} className={"placard" + (!isEnabled(modifier) ? " disabled" : "")}>
                            {modifierTextElement(modifier)}
                        </div>
            )}
        </div>
    </div>
}

export function ModifierConfigMenu(props: Readonly<{
    modifier: ModifierID,
    modifiable: boolean,
    state: ModifierState | undefined,
    setModifier: (modifier: ModifierState | undefined) => void,
    close: () => void,
    modifierSettings: Readonly<ModifierSettings>,
}>): ReactElement | null {
    // This will prevent it from opening until the server sends us the state
    if (props.state === undefined) {
        return null;
    }

    switch (props.modifier) {
        case "customRoleLimits": {
            return <CustomRoleLimitsConfigMenu
                state={props.state as ModifierState & { type: "customRoleLimits" }}
                modifiable={props.modifiable}
                setModifier={props.setModifier}
                close={props.close}
            />
        }
        case "customRoleSets": {
            return <CustomRoleSetsConfigMenu
                state={props.state as ModifierState & { type: "customRoleSets" }}
                modifiable={props.modifiable}
                setModifier={props.setModifier}
                close={props.close}
                modifierSettings={props.modifierSettings}
            />
        }
        default:
            return null;
    }
}

function CustomRoleLimitsConfigMenu(props: Readonly<{
    state: ModifierState & { type: "customRoleLimits" },
    modifiable: boolean,
    setModifier: (modifier: ModifierState | undefined) => void,
    close: () => void,
}>): ReactElement {
    const limits = new ListMap<Role, number>(props.state.limits);

    return <div>
        {!props.modifiable && <StyledText>{translate(`customRoleLimits`)}</StyledText>}
        <div className="role-list-setter-list">
            {limits.entries().sort(([roleA, _a], [roleB, _b]) => sortRolesCanonically(roleA, roleB)).map(([role, limit]) => 
                <CustomRoleLimitSelection
                    key={role}
                    role={role}
                    limit={limit}
                    modifiable={props.modifiable}
                    onChange={(newRole, newLimit) => {
                        if (newRole !== role) {
                            limits.delete(role);
                        }
                        limits.insert(newRole, newLimit);
                        props.setModifier({ type: "customRoleLimits", limits: limits.list });
                    }}
                    remove={() => {
                        limits.delete(role);
                        props.setModifier({ type: "customRoleLimits", limits: limits.list });
                    }}
                />
            )}
            {props.modifiable && <div>
                <Button onClick={() => {
                    const role = getAllRoles().find(role => !limits.keys().includes(role)) ?? "villager"
                    limits.insert(role, 1);
                    props.setModifier({ type: "customRoleLimits", limits: limits.list });
                }}
                ><Icon>add</Icon></Button>
            </div>}
        </div>
        {props.modifiable && <div>
            <Button onClick={() => {
                props.setModifier({
                    type: "customRoleLimits",
                    limits: Object.entries(roleJsonData())
                        .filter(([_role, data]) => data.maxCount !== null)
                        .map(([role, data]) => [role as Role, data.maxCount!])
                })
            }}>
                <Icon>autorenew</Icon>
            </Button>
            <Button onClick={() => {
                props.setModifier({
                    type: "customRoleLimits",
                    limits: getAllRoles().map(role => [role, 1])
                });
            }}>
                <Icon>filter_1</Icon>
            </Button>
            <Button onClick={() => {
                props.setModifier(undefined)
                props.close();
            }}>
                <Icon>delete</Icon>
            </Button>
            <Button onClick={() => {
                props.close();
            }}>
                <Icon>expand_less</Icon>
            </Button>

        </div>}
    </div>;
}

function CustomRoleLimitSelection(props: Readonly<{
    role: Role,
    limit: number,
    modifiable: boolean,
    onChange: (role: Role, limit: number) => void,
    remove: () => void,
}>): ReactElement {
    const enabledRoles = useContext(GameModeContext).enabledRoles;

    const optionsSearch = new Map<Role, [ReactElement, string]>();

    getAllRoles().forEach(role => {
        optionsSearch.set(role, [
            <StyledText
                key={role}
                noLinks={true}
                className={!enabledRoles.includes(role) ? "keyword-disabled" : ""}
            >
                {translate(`role.${role}.name`)}
            </StyledText>,
            translate(`role.${role}.name`)
        ]);
    });

    return <div className="custom-role-limit-selection">
        {props.modifiable ? <>
            <Select
                className="role-outline-option-selector"
                value={props.role}
                onChange={role => props.onChange(role, props.limit)}
                optionsSearch={optionsSearch}
            />
            <input
                type="text"
                value={props.limit}
                onChange={(e)=>{
                    const value = Number(e.target.value);

                    if (value < 0 || value > 255 || Math.round(value) !== value) return;

                    props.onChange(props.role, value);
                }}
                onKeyUp={(e)=>{
                    if(e.key !== 'Enter') return;

                    props.onChange(props.role, props.limit);
                }}
            />
            <Button
                onClick={props.remove}
            >
                <Icon>delete</Icon>
            </Button>
        </> : <>
            <StyledText>{translate(`role.${props.role}.name`)}</StyledText>: {props.limit}
        </>}
    </div>
}

function CustomRoleSetsConfigMenu(props: Readonly<{
    state: ModifierState & { type: "customRoleSets" },
    modifiable: boolean,
    setModifier: (modifier: ModifierState | undefined) => void,
    close: () => void,
    modifierSettings: Readonly<ModifierSettings>,
}>): ReactElement {
    return <div>
        {!props.modifiable && <StyledText>{translate(`customRoleSets`)}</StyledText>}
        <div className="role-list-setter-list">
            {props.state.sets.map((set, index) => {
                return <CustomRoleSetSelection
                    key={set.name as string}
                    name={set.name}
                    setName={(newName) => {
                        const sets = [...props.state.sets];
                        sets[index] = { ...set, name: newName };
                        props.setModifier({ type: "customRoleSets", sets: sets });
                    }}
                    setId={index}
                    subRoleSets={set.roleSets ?? []}
                    roles={set.roles ?? []}
                    modifiable={props.modifiable}
                    onChange={(newSubRoleSets, newRoles) => {
                        const sets = [...props.state.sets];
                        sets[index] = { name: set.name, roleSets: newSubRoleSets, roles: newRoles };
                        props.setModifier({ type: "customRoleSets", sets: sets });
                    }}
                    remove={() => {
                        const sets = [...props.state.sets];
                        sets.splice(index, 1);
                        props.setModifier({ type: "customRoleSets", sets: sets });
                    }}
                    modifierSettings={props.modifierSettings}
                />
            })}
            {props.modifiable && <div>
                <Button onClick={() => {
                    const sets = [...props.state.sets];
                    const newSetName = translate("roleSet.customUnnamed", sets.length + 1);

                    sets.push({ name: newSetName as UnsafeString, roles: [] });
                    props.setModifier({ type: "customRoleSets", sets: sets });
                }}
                ><Icon>add</Icon></Button>
            </div>}
        </div>
        {props.modifiable && <div>
            <Button onClick={() => {
                props.setModifier(undefined)
                props.close();
            }}>
                <Icon>delete</Icon>
            </Button>
            <Button onClick={() => {
                props.close();
            }}>
                <Icon>expand_less</Icon>
            </Button>

        </div>}
    </div>;
}

function CustomRoleSetSelection(props: Readonly<{
    name: UnsafeString,
    setName: (name: UnsafeString) => void,
    setId: number,
    subRoleSets: { roleSet: RoleSet, excludedRoles: Role[] }[],
    roles: Role[],
    modifiable: boolean,
    onChange: (subRoleSets: { roleSet: RoleSet, excludedRoles: Role[] }[], roles: Role[]) => void,
    remove: () => void,
    modifierSettings: Readonly<ModifierSettings>,
}>): ReactElement {
    const enabledRoles = useContext(GameModeContext).enabledRoles;

    const roleOptionsSearch = new Map<Role, [ReactElement, string]>();

    getAllRoles().forEach(role => {
        roleOptionsSearch.set(role, [
            <StyledText
                key={role}
                noLinks={true}
                className={!enabledRoles.includes(role) ? "keyword-disabled" : ""}
            >
                {translate(`role.${role}.name`)}
            </StyledText>,
            translate(`role.${role}.name`)
        ]);
    });

    const roleSetOptionsSearch = new Map<RoleSet, [ReactElement, string]>();

    BASE_ROLE_SETS.forEach(roleSet => {
        roleSetOptionsSearch.set({ type: roleSet }, [
            <StyledText
                key={roleSet}
                noLinks={true}
            >
                {translate(roleSet)}
            </StyledText>,
            translate(roleSet)
        ]);
    });

    (props.modifierSettings.get("customRoleSets") as CustomRoleSetsModifierState | undefined)?.sets.forEach((set, id) => {
        roleSetOptionsSearch.set({ type: "custom", id }, [
            <StyledText
                key={set.name as string}
                noLinks={true}
            >
                {encodeString(set.name)}
            </StyledText>,
            encodeString(set.name)
        ]);
    });

    const [nameField, setNameField] = useState(props.name as string);

    const [roleOrRoleSetToAdd, setRoleOrRoleSetToAdd] = useState<RoleOrRoleSet>({ type: "roleSet", roleSet: { type: "any" } });

    const [open, setOpen] = useState<boolean>(false);
    const ref = useRef<HTMLButtonElement>(null);

    return <div>
        <RawButton
            ref={ref}
            onClick={() => setOpen(open => !open)}
        ><StyledText noLinks={true}>{encodeString(props.name)}</StyledText> {props.modifiable && <Icon>edit</Icon>}</RawButton>
        {props.modifiable && <Button
            onClick={props.remove}
        ><Icon>delete</Icon></Button>}
        <Popover
            open={open}
            setOpenOrClosed={setOpen}
            anchorForPositionRef={ref}
            doNotCloseOnOutsideClick={true}
            onRender={dropdownPlacementFunction}
        >
            <div className="custom-role-set-selection">
                {props.modifiable ? <>
                    <input
                        type="text"
                        value={nameField}
                        onChange={(e) => {
                            setNameField(e.target.value);
                        }}
                        onBlur={(e) => {
                            props.setName(e.target.value as UnsafeString);
                        }}
                        onKeyUp={(e) => {
                            if (e.key !== 'Enter') return;

                            props.setName((e.target as React.ChangeEvent<HTMLInputElement>['target']).value as UnsafeString);
                        }}
                    />
                </> : <>
                    <StyledText>{encodeString(props.name)}</StyledText>
                </>}
                <StyledText>{translate("customRoleSets.subRoleSets")}</StyledText>
                <div className="role-list-setter-list">
                    {props.subRoleSets.length > 0 ? props.subRoleSets.map((subRoleSet, i) =>
                        <div key={subRoleSet.roleSet + '.' + subRoleSet.excludedRoles.join(',')} className="custom-role-limit-selection">
                            <CustomRoleSetSelectionSubRoleSet
                                setId={props.setId}
                                subRoleSet={subRoleSet}
                                modifiable={props.modifiable}
                                roleSetOptionsSearch={roleSetOptionsSearch}
                                onChange={(roleSet, excludedRoles) => {
                                    const newSubRoleSets = [...props.subRoleSets];
                                    newSubRoleSets[i] = { roleSet, excludedRoles };
                                    props.onChange(newSubRoleSets, props.roles);
                                }}
                                onDelete={() => {
                                    const newSubRoleSets = [...props.subRoleSets];
                                    newSubRoleSets.splice(i, 1);
                                    props.onChange(newSubRoleSets, props.roles)
                                }}
                                modifierSettings={props.modifierSettings}
                            />
                        </div>
                    ) : translate("none")}
                </div>
                <StyledText>{translate("customRoleSets.roles")}</StyledText>
                <div className="role-list-setter-list">
                    {props.roles.length > 0 ? props.roles.map((role, i) => 
                        <div key={role} className="custom-role-limit-selection">
                            {props.modifiable ? <>
                                <Select
                                    className="role-outline-option-selector"
                                    value={role}
                                    onChange={r => {
                                        const newRoles = [...props.roles];
                                        newRoles[i] = r
                                        props.onChange(props.subRoleSets, newRoles);
                                    }}
                                    optionsSearch={roleOptionsSearch}
                                />
                                <Button
                                    onClick={() => {
                                        const newRoles = [...props.roles];
                                        newRoles.splice(i, 1);
                                        props.onChange(props.subRoleSets, newRoles);
                                    }}
                                >
                                    <Icon>delete</Icon>
                                </Button>
                            </> : <>
                                <StyledText
                                    noLinks={true}
                                    className={!enabledRoles.includes(role) ? "keyword-disabled" : ""}
                                >
                                    {translate(`role.${role}.name`)}
                                </StyledText>
                            </>}
                        </div>
                    ) : translate("none")}
                </div>
                {props.modifiable && <StyledText>{translate("customRoleSets.addRoleOrRoleSet")}</StyledText>}
                {props.modifiable && <div className="select-role-to-add">
                    <RoleOrRoleSetSelector
                        roleOrRoleSet={roleOrRoleSetToAdd}
                        onChange={setRoleOrRoleSetToAdd}
                    />
                    <Button
                        onClick={() => {
                            if (roleOrRoleSetToAdd.type === "role") {
                                const newRoles = [...props.roles];
                                newRoles.push(roleOrRoleSetToAdd.role);
                                props.onChange(props.subRoleSets, newRoles);
                            } else {
                                const newRoleSets = [...props.subRoleSets];
                                newRoleSets.push({ roleSet: roleOrRoleSetToAdd.roleSet, excludedRoles: [] });
                                props.onChange(newRoleSets, props.roles);
                            }
                        }}
                    >
                        <Icon>add</Icon>
                    </Button>
                </div>}
            </div>
        </Popover>
    </div>
}

function CustomRoleSetSelectionSubRoleSet(props: Readonly<{
    setId: number,
    subRoleSet: { roleSet: RoleSet, excludedRoles: Role[] },
    roleSetOptionsSearch: Map<RoleSet, [ReactElement, string]>,
    modifiable: boolean,
    onChange: (roleSet: RoleSet, excludedRoles: Role[]) => void,
    onDelete: () => void,
    modifierSettings: ModifierSettings
}>): ReactElement {
    const enabledRoles = useContext(GameModeContext).enabledRoles;

    const rolesFromRoleSet = getRolesFromRoleSet(props.subRoleSet.roleSet, props.modifierSettings);
    const rolesToAdd = rolesFromRoleSet.filter(role => !props.subRoleSet.excludedRoles.includes(role));

    const roleOptionsSearch = new Map<Role, [ReactElement, string]>();

    rolesFromRoleSet.forEach(role => {
        roleOptionsSearch.set(role, [
            <StyledText
                key={role}
                noLinks={true}
                className={!enabledRoles.includes(role) ? "keyword-disabled" : ""}
            >
                {translate(`role.${role}.name`)}
            </StyledText>,
            translate(`role.${role}.name`)
        ]);
    });

    const [open, setOpen] = useState<boolean>(false);
    const ref = useRef<HTMLButtonElement>(null);

    const buttonText = (
        `${translateRoleSet(props.subRoleSet.roleSet, props.modifierSettings)} ` +
        (props.subRoleSet.excludedRoles.length > 0 ? (
            `${translate("customRoleSets.subRoleSet.minus")} ` +
            `${props.subRoleSet.excludedRoles.map(role => translate("role." + role + ".name")).join(', ')}`
        ) : '')
    );

    const maxLength = 25;

    const truncatedText = buttonText.length < maxLength ? buttonText : (buttonText.substring(0, Math.min(buttonText.length, maxLength - 3)) + "...");

    return props.modifiable ? <>
        <RawButton ref={ref} onClick={() => setOpen(!open)}>
            <StyledText noLinks={true}>{truncatedText}</StyledText>
        </RawButton>
        <Button onClick={props.onDelete}><Icon>delete</Icon></Button>
        <Popover
            open={open}
            setOpenOrClosed={setOpen}
            anchorForPositionRef={ref}
            doNotCloseOnOutsideClick={true}
            onRender={dropdownPlacementFunction}
        >
            <div className="sub-role-set">
                <div className="sub-role-set-role-set">
                    <Select
                        className="role-outline-option-selector"
                        value={props.subRoleSet.roleSet}
                        onChange={rs => props.onChange(rs, [])}
                        optionsSearch={props.roleSetOptionsSearch}
                        compareFn={(a, b) => a.type === b.type && (a.type !== "custom" || b.type !== "custom" || a.id === b.id)}
                    />
                </div>
                {translate("customRoleSets.subRoleSet.minus")}
                <div className="role-list-setter-list">
                    {props.subRoleSet.excludedRoles.map((role, index) => <div key={role}>
                        <Select
                            className="role-outline-option-selector"
                            value={role}
                            onChange={r => {
                                const newExcludedRoles = [...props.subRoleSet.excludedRoles];
                                newExcludedRoles[index] = r;
                                props.onChange(props.subRoleSet.roleSet, newExcludedRoles);
                            }}
                            optionsSearch={roleOptionsSearch}
                        />
                        <Button
                            onClick={() => {
                                const newRoles = [...props.subRoleSet.excludedRoles];
                                newRoles.splice(index, 1);
                                props.onChange(props.subRoleSet.roleSet, newRoles);
                            }}
                        >
                            <Icon>delete</Icon>
                        </Button>
                    </div>) || translate("noExclusions")}
                    <div>
                        {rolesToAdd.length > 0 && <Button onClick={() => {
                            props.onChange(props.subRoleSet.roleSet, [
                                ...props.subRoleSet.excludedRoles,
                                rolesToAdd[0]
                            ]);
                        }}><Icon>add</Icon></Button>}
                    </div>
                </div>
            </div>
        </Popover>
    </> : <>
        <div>
            <StyledText>{translateRoleSet(props.subRoleSet.roleSet, props.modifierSettings)}</StyledText>
            {props.subRoleSet.excludedRoles.length > 0 && " " + translate("customRoleSets.subRoleSet.minus") + " "}
            {props.subRoleSet.excludedRoles.map(role => <>
                <StyledText
                    key={role}
                    noLinks={true}
                    className={!enabledRoles.includes(role) ? "keyword-disabled" : ""}
                >
                    {translate(`role.${role}.name`)}
                </StyledText>
            </>)}
        </div>
    </>
}