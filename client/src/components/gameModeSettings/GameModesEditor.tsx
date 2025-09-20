import { ReactElement, createContext, useCallback, useState } from "react";
import React from "react";
import { OutlineListSelector } from "./OutlineSelector";
import { getAllRoles, RoleList, RoleOutline } from "../../game/roleListState.d";
import translate from "../../game/lang";
import "./gameModesEditor.css";
import PhaseTimesSelector from "./PhaseTimeSelector";
import { PhaseTimes } from "../../game/gameState.d";
import EnabledRoleSelector from "./EnabledRoleSelector";
import { Role } from "../../game/roleState.d";
import "./selectorSection.css";
import { defaultPhaseTimes } from "../../game/gameState";
import { GameModeSelector } from "./GameModeSelector";
import { ShareableGameMode } from "./gameMode";
import { ModifiersSelector } from "./ModifiersSelector";
import { ListMapData } from "../../ListMap";
import { ModifierID, ModifierState } from "../../game/modifiers";

const GameModeContext = createContext({
    roleList: [] as RoleList,
    phaseTimes: defaultPhaseTimes(),
    enabledRoles: [] as Role[],
    modifierSettings: [] as ListMapData<ModifierID, ModifierState>
});
export {GameModeContext};


export default function GameModesEditor(props: Readonly<{
    initialGameMode?: ShareableGameMode
}>): ReactElement {

    const [roleList, setRoleList] = useState<RoleList>(()=>{
        if(props.initialGameMode){
            return props.initialGameMode.roleList;
        }
        return [];
    });
    const [phaseTimes, setPhaseTimes] = useState<PhaseTimes>(()=>{
        if(props.initialGameMode){
            return props.initialGameMode.phaseTimes;
        }
        return defaultPhaseTimes()
    });
    const [enabledRoles, setEnabledRoles] = useState<Role[]>(()=>{
        if(props.initialGameMode){
            return props.initialGameMode.enabledRoles;
        }
        return [];
    });
    const [modifierSettings, setModifierSettings] = useState<ListMapData<ModifierID, ModifierState>>(()=>{
        if(props.initialGameMode){
            return props.initialGameMode.modifierSettings;
        }
        return [] as ListMapData<ModifierID, ModifierState>;
    });


    const onChangeRolePicker = useCallback((value: RoleOutline, index: number) => {
        const newRoleList = [...roleList];
        newRoleList[index] = value;
        setRoleList(newRoleList);
    }, [roleList]);
    
    const addOutline = () => {
        setRoleList([...roleList, [{ roleSet: { type: "any" } }]]);
    }
    const removeOutline = (index: number) => {
        let newRoleList = [...roleList];
        newRoleList.splice(index, 1);
        setRoleList(newRoleList);
    }


    const onEnableRoles = (roles: Role[]) => {
        const newEnabledRoles = [...enabledRoles];
        for(const role of roles){
            if(!newEnabledRoles.includes(role)){
                newEnabledRoles.push(role);
            }
        }
        setEnabledRoles(newEnabledRoles);
    }
    const onDisableRoles = (roles: Role[]) => {
        setEnabledRoles(enabledRoles.filter((role) => !roles.includes(role)));
    }
    const onEnableAll = () => {
        setEnabledRoles(getAllRoles());
    }

    const setModifiers = (modifiers: ListMapData<ModifierID, ModifierState>) => {
        setModifierSettings(modifiers);
    }
    
    
    return <div className="game-modes-editor">
        <header>
            <h1>{translate("menu.globalMenu.gameSettingsEditor")}</h1>
        </header>
        <GameModeContext.Provider value={{roleList, phaseTimes, enabledRoles, modifierSettings}}>
            <main>
                <div>
                    <GameModeSelector 
                        canModifySavedGameModes={true}
                        loadGameMode={gameMode => {
                            setRoleList(gameMode.roleList);
                            setEnabledRoles(gameMode.enabledRoles);
                            setPhaseTimes(gameMode.phaseTimes);
                            setModifierSettings(gameMode.modifierSettings);
                        }}
                    />
                    <PhaseTimesSelector 
                        onChange={(newPhaseTimes) => {
                            setPhaseTimes(newPhaseTimes);
                        }}            
                    />
                </div>
                <div>
                    <ModifiersSelector
                        disabled={false}
                        setModifiers={setModifiers}
                    />
                    <OutlineListSelector
                        onChangeRolePicker={onChangeRolePicker}
                        onAddNewOutline={addOutline}
                        onRemoveOutline={removeOutline}
                        setRoleList={setRoleList}
                    />
                    <EnabledRoleSelector
                        onDisableRoles={onDisableRoles}
                        onEnableRoles={onEnableRoles}
                        onIncludeAll={onEnableAll}         
                    />
                </div>
            </main>
        </GameModeContext.Provider>
    </div>
}
