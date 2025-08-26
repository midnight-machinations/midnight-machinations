import React, { ReactElement, useCallback, useContext, useState } from "react";
import { MODIFIERS, ModifierID, ModifierState } from "../../game/gameState.d";
import translate from "../../game/lang";
import StyledText from "../StyledText";
import { GameModeContext } from "./GameModesEditor";
import { Button } from "../Button";
import CheckBox from "../CheckBox";
import ListMap, { ListMapData } from "../../ListMap";

export function ModifiersSelector(props: Readonly<{
    disabled?: boolean,
    modifierSettings?: ListMapData<ModifierID, ModifierState>,
    onEnableModifiers?: (modifiers: ModifierID[]) => void,
    onDisableModifiers?: (modifiers: ModifierID[]) => void,
    onModifierStateChange?: (modifiers: ListMapData<ModifierID, ModifierState>) => void
}>): ReactElement {
    let { modifierSettings } = useContext(GameModeContext);
    modifierSettings = props.modifierSettings ?? modifierSettings;

    return <div className="chat-menu-colors selector-section">
        <h2>{translate("modifiers")}</h2>
        <ModifierSettingsDisplay
            disabled={props.disabled===undefined ? false : props.disabled}
            modifiable={!props.disabled}
            modifierSettings={modifierSettings}
            onEnableModifiers={(modifiers: ModifierID[]) => {
                if (props.onEnableModifiers) {
                    props.onEnableModifiers(modifiers)
                }
            }}
            onDisableModifiers={(modifiers: ModifierID[]) => {
                if (props.onDisableModifiers) {
                    props.onDisableModifiers(modifiers)
                }
            }}
        />
    </div>
}

type EnabledModifiersDisplayProps = {
    modifierSettings: ListMapData<ModifierID, ModifierState>,
} & (
    {
        modifiable: true,
        onDisableModifiers: (modifiers: ModifierID[]) => void,
        onEnableModifiers: (modifiers: ModifierID[]) => void,
        onModifierStateChange: (modifiers: ListMapData<ModifierID, ModifierState>) => void,
        disabled?: boolean,
    } |
    {
        modifiable?: false,
    }
)

export function ModifierSettingsDisplay(props: EnabledModifiersDisplayProps): ReactElement {
    const isEnabled = useCallback((modifier: ModifierID) => {
        return props.modifierSettings.some(([id, _]) => id === modifier)
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
        <div>
            {MODIFIERS
                .filter(role => isEnabled(role) || !hideDisabled || props.modifiable)
                .sort((a, b) => props.modifiable ? 0 : (isEnabled(a) ? -1 : 1) - (isEnabled(b) ? -1 : 1))
                .map((modifier, i) => 
                props.modifiable 
                    ? <Button key={modifier}
                        disabled={props.disabled}
                        onClick={() => (!isEnabled(modifier) ? props.onEnableModifiers : props.onDisableModifiers)([modifier])}
                    >
                        {modifierTextElement(modifier)}
                    </Button> 
                    : <div key={modifier} className={"placard" + (!isEnabled(modifier) ? " disabled" : "")}>
                        {modifierTextElement(modifier)}
                    </div>
            )}
        </div>
    </div>
}