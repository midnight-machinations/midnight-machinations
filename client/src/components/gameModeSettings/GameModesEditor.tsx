import { ReactElement, createContext, useCallback, useState } from "react";
import React from "react";
import { OutlineListSelector } from "./OutlineSelector";
import { getAllRoles, RoleList, RoleOutline } from "../../stateContext/stateType/roleListState";
import translate from "../../game/lang";
import "./gameModesEditor.css";
import PhaseTimesSelector from "./PhaseTimeSelector";
import EnabledRoleSelector from "./EnabledRoleSelector";
import "./selectorSection.css";
import { GameModeSelector } from "./GameModeSelector";
import { ShareableGameMode } from "./gameMode";
import { EnabledModifiersSelector } from "./EnabledModifiersSelector";
import { defaultPhaseTimes, PhaseTimes } from "../../stateContext/stateType/otherState";
import { ModifierType } from "../../stateContext/stateType/modifiersState";
import { Role } from "../../stateContext/stateType/roleState";

const GameModeContext = createContext({
    roleList: [] as RoleList,
    phaseTimes: defaultPhaseTimes(),
    enabledRoles: [] as Role[],
    enabledModifiers: [] as ModifierType[]
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
    const [enabledModifiers, setEnabledModifiers] = useState<ModifierType[]>(()=>{
        if(props.initialGameMode){
            return props.initialGameMode.enabledModifiers;
        }
        return [];
    });


    const onChangeRolePicker = useCallback((value: RoleOutline, index: number) => {
        const newRoleList = [...roleList];
        newRoleList[index] = value;
        setRoleList(newRoleList);
    }, [roleList]);
    
    const addOutline = () => {
        setRoleList([...roleList, [{ roleSet: "any" }]]);
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

    const onSetEnabledModifiers = (modifiers: ModifierType[]) => {
        setEnabledModifiers(modifiers);
    }
    
    
    return <div className="game-modes-editor">
        <header>
            <h1>{translate("menu.globalMenu.gameSettingsEditor")}</h1>
        </header>
        <GameModeContext.Provider value={{roleList, phaseTimes, enabledRoles, enabledModifiers}}>
            <main>
                <div>
                    <GameModeSelector 
                        canModifySavedGameModes={true}
                        loadGameMode={gameMode => {
                            setRoleList(gameMode.roleList);
                            setEnabledRoles(gameMode.enabledRoles);
                            setPhaseTimes(gameMode.phaseTimes);
                            setEnabledModifiers(gameMode.enabledModifiers);
                        }}
                    />
                    <PhaseTimesSelector 
                        onChange={(newPhaseTimes) => {
                            setPhaseTimes(newPhaseTimes);
                        }}            
                    />
                </div>
                <div>
                    <EnabledModifiersSelector
                        disabled={false}
                        onChange={onSetEnabledModifiers}
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
