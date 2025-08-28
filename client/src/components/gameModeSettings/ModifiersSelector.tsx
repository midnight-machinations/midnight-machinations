import React, { ReactElement, useCallback, useContext, useEffect, useRef, useState } from "react";
import { MODIFIERS, ModifierID, ModifierState, defaultModifierState, isModifierConfigurable } from "../../game/modifiers";
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
import { getAllRoles, sortRolesCanonically } from "../../game/roleListState.d";
import { useLobbyOrGameState } from "../useHooks";

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
                props.close();
            }}>
                <Icon>autorenew</Icon>
            </Button>
            <Button onClick={() => {
                props.setModifier({
                    type: "customRoleLimits",
                    limits: getAllRoles().map(role => [role, 1])
                });
                props.close();
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
    const enabledRoles = useLobbyOrGameState(
        state => state.enabledRoles,
        ["enabledRoles"]
    )!;

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