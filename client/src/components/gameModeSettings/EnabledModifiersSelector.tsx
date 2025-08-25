import React, { ReactElement, useContext } from "react";
import { ModifierType, toModifierType } from "../../game/gameState.d";
import translate from "../../game/lang";
import { GameModeContext } from "./GameModesEditor";
import CheckBox from "../CheckBox";
import Select, { SelectOptionsSearch, set_option_typical as setOptionTypical, set_options_typical as setOptionsTypical } from "../Select";
import "./enabledModifiersSelector.css";

export function EnabledModifiersSelector(props: Readonly<{
    disabled?: boolean,
    enabledModifiers?: ModifierType[],
    onChange?: (modifiers: ModifierType[]) => void,
}>): ReactElement {
    let { enabledModifiers } = useContext(GameModeContext);
    enabledModifiers = props.enabledModifiers ?? enabledModifiers;
    

    return <div className="chat-menu-colors selector-section modifier-selectors">
        <h2>{translate("modifiers")}</h2>
        <EnabledModifiersDisplay 
            disabled={props.disabled===undefined ? false : props.disabled}
            modifiable={!props.disabled}
            enabledModifiers={enabledModifiers}
            onEnableDisableModifiers={(enable: ModifierType[], disable: ModifierType[])=>{
                if (props.onChange) {
                    props.onChange(
                        enabledModifiers
                            .filter(modifier => !disable.includes(modifier))
                            .concat(enable)
                    )
                }
            }}
            onDisableModifiers={(modifiers: ModifierType[]) => {
                if (props.onChange) {
                    props.onChange(enabledModifiers.filter(modifier => !modifiers.includes(modifier)))
                }
            }}
            onEnableModifiers={(modifiers: ModifierType[]) => {
                if (props.onChange) {
                    props.onChange(
                        enabledModifiers.concat(modifiers)
                    )
                }
            }}
        />
    </div>
}

type EnabledModifiersDisplayProps = {
    enabledModifiers: ModifierType[],
} & (
    {
        modifiable: true,
        onEnableDisableModifiers:(enable: ModifierType[], disable: ModifierType[]) => void,
        onDisableModifiers: (modifiers: ModifierType[]) => void,
        onEnableModifiers: (modifiers: ModifierType[]) => void,
        disabled?: boolean,
    } | {
        modifiable?: false,
    }
)

type ModifierSelectionType = {
    type: "dropdown",
    modifiers: ModifierType[],
} | {
    type: "checkbox",
    modifier: ModifierType,
    inverted: boolean,
} | {
    type: "boolean",
    modifier: ModifierType, //the two options are determined by modifierMenu.<name>.true & modifierMenu.<name>.false
}

class ModifierSelection {
    name: string
    data: ModifierSelectionType
    constructor(name: string, data: ModifierSelectionType) {
        this.name = name
        this.data = data
    }
    generateDisplay(props: EnabledModifiersDisplayProps) {
        switch(this.data.type){
            case "dropdown": {
                const data = this.data;
                const defaultValue = "modifierMenu."+this.name+".fake";

                function selectedOption(): ModifierType | typeof defaultValue {
                    let selected = data.modifiers.find(modifier => {return props.enabledModifiers.includes(modifier)});
                    return selected === undefined ? defaultValue : selected
                }
                
                const optionsMap: SelectOptionsSearch<ModifierType | typeof defaultValue> = new Map();
                setOptionTypical(optionsMap, defaultValue)
                setOptionsTypical(optionsMap, data.modifiers);
                return <div className="placard modifier-selector">
                    <span>{translate("modifierMenu."+this.name)}</span>
                    <span><Select 
                        value={selectedOption()}
                        onChange={value => {
                            if(props.modifiable === true){
                                const modifier = toModifierType(value);
                                if(modifier !== undefined) {
                                    props.onEnableDisableModifiers([modifier], data.modifiers)
                                } else {
                                    props.onDisableModifiers(data.modifiers);
                                }
                            }
                        }}
                        optionsSearch = {optionsMap}
                    /></span>
                </div>
            }

            case "checkbox": {
                const data = this.data;
                return <div className="placard">
                    <span>{translate("modifierMenu."+this.name)}</span>
                    <CheckBox
                        checked={props.enabledModifiers.includes(data.modifier) !== data.inverted}
                        onChange={checked => {
                            if (props.modifiable) {
                                if (checked !== data.inverted) {
                                    props.onEnableModifiers([data.modifier]);
                                } else {
                                    props.onDisableModifiers([data.modifier]);
                                }
                            }
                    } } />
                </div>
            }
            
            case "boolean": {
                const data = this.data;

                const falseSelection = "modifierMenu."+this.name+".false";
                const trueSelection = "modifierMenu."+this.name+".true";

                const optionsMap: SelectOptionsSearch<ModifierType> = new Map();
                setOptionTypical(optionsMap, falseSelection);
                setOptionTypical(optionsMap, trueSelection);

                return <div className="placard modifier-selector">
                    <span>{translate("modifierMenu."+this.name)}</span>
                    <span><Select 
                        value={
                            props.enabledModifiers.includes(data.modifier) ? 
                            trueSelection :
                            falseSelection
                        }
                        onChange={value => {
                            if(props.modifiable === true){
                                if(value.endsWith("true")) {
                                    props.onEnableModifiers([data.modifier])
                                } else {
                                    props.onDisableModifiers([data.modifier])
                                }
                            }
                        }}
                        optionsSearch = {optionsMap}
                    /></span>
                </div>
            }
        }
    }
    static dropdown(name: string, modifiers: ModifierType[]): ModifierSelection {
        return new ModifierSelection(
                name, 
            {
                type: "dropdown",
                modifiers
            }
        )
    }
    static checkbox(modifier: ModifierType, inverted: boolean, name: string =modifier): ModifierSelection {
        return new ModifierSelection(
                name, 
            {
                type: "checkbox",
                modifier,
                inverted,
            }
        )
    }
    static boolean(modifier: ModifierType): ModifierSelection {
        return new ModifierSelection(
            modifier,
            {
                type:"boolean",
                modifier
            }
        )
    }
}

const MODIFIER_SELECTIONS = [
    ModifierSelection.dropdown("trialPhases", ["unscheduledNominations", "noTrialPhases"]),
    ModifierSelection.boolean("abstaining"),
    ModifierSelection.dropdown("guiltyVoteRequirement", ["twoThirdsMajority", "autoGuilty"]),
    ModifierSelection.dropdown("graveInfo", ["noDeathCause", "roleSetGraveKillers", "obscuredGraves"]),
    ModifierSelection.dropdown("chat", ["noNightChat", "noChat"]),
    ModifierSelection.boolean("deadCanChat"),
    ModifierSelection.dropdown("whispers", ["hiddenWhispers", "noWhispers"]),
    ModifierSelection.boolean("skipDay1"),
    ModifierSelection.boolean("hiddenNominationVotes"),
    ModifierSelection.boolean("hiddenVerdictVotes"),
    ModifierSelection.boolean("forfeitNominationVote"),
    ModifierSelection.boolean("randomPlayerNames"),
] as const

export function EnabledModifiersDisplay(props: EnabledModifiersDisplayProps): ReactElement {
    let display = <></>
    for(const selector of MODIFIER_SELECTIONS){
        display = <>{display}{selector.generateDisplay(props)}</>
    }
    return <div className="modifier-selector">
        {display}
    </div>
}