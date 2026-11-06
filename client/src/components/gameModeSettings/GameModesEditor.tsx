import { createContext, type ReactElement, useCallback, useMemo, useState } from "react";
import { getAllRoles, type RoleList, type RoleOutline } from "../../game/roleListState.d";
import { OutlineListSelector } from "./OutlineSelector";
import "./gameModesEditor.css";
import type { PhaseTimes } from "../../game/gameState.d";
import type { Role } from "../../game/roleState.d";
import EnabledRoleSelector from "./EnabledRoleSelector";
import PhaseTimesSelector from "./PhaseTimeSelector";
import "./selectorSection.css";
import { defaultPhaseTimes } from "../../game/gameState";
import type { ModifierID, ModifierState } from "../../game/modifiers";
import type { ListMapData } from "../../ListMap";
import { GameModeSelector } from "./GameModeSelector";
import type { ShareableGameMode } from "./gameMode";
import { ModifiersSelector } from "./ModifiersSelector";

const GameModeContext = createContext({
    roleList: [] as RoleList,
    phaseTimes: defaultPhaseTimes(),
    enabledRoles: [] as Role[],
    modifierSettings: [] as ListMapData<ModifierID, ModifierState>,
});

export { GameModeContext };

export default function GameModesEditor(
    props: Readonly<{
        initialGameMode?: ShareableGameMode;
    }>,
): ReactElement {
    const [roleList, setRoleList] = useState<RoleList>(() => {
        if (props.initialGameMode) {
            return props.initialGameMode.roleList;
        }
        return [];
    });
    const [phaseTimes, setPhaseTimes] = useState<PhaseTimes>(() => {
        if (props.initialGameMode) {
            return props.initialGameMode.phaseTimes;
        }
        return defaultPhaseTimes();
    });
    const [enabledRoles, setEnabledRoles] = useState<Role[]>(() => {
        if (props.initialGameMode) {
            return props.initialGameMode.enabledRoles;
        }
        return [];
    });
    const [modifierSettings, setModifierSettings] = useState<ListMapData<ModifierID, ModifierState>>(() => {
        if (props.initialGameMode) {
            return props.initialGameMode.modifierSettings;
        }
        return [] as ListMapData<ModifierID, ModifierState>;
    });

    const onChangeRolePicker = useCallback(
        (value: RoleOutline, index: number) => {
            const newRoleList = [...roleList];
            newRoleList[index] = value;
            setRoleList(newRoleList);
        },
        [roleList],
    );

    const addOutline = () => {
        setRoleList([...roleList, [{ roleSet: "any" }]]);
    };
    const removeOutline = (index: number) => {
        const newRoleList = [...roleList];
        newRoleList.splice(index, 1);
        setRoleList(newRoleList);
    };

    const onEnableRoles = (roles: Role[]) => {
        const newEnabledRoles = [...enabledRoles];
        for (const role of roles) {
            if (!newEnabledRoles.includes(role)) {
                newEnabledRoles.push(role);
            }
        }
        setEnabledRoles(newEnabledRoles);
    };
    const onDisableRoles = (roles: Role[]) => {
        setEnabledRoles(enabledRoles.filter((role) => !roles.includes(role)));
    };
    const onEnableAll = () => {
        setEnabledRoles(getAllRoles());
    };

    const setModifiers = (modifiers: ListMapData<ModifierID, ModifierState>) => {
        setModifierSettings(modifiers);
    };

    const contextValue = useMemo(
        () => ({
            roleList,
            phaseTimes,
            enabledRoles,
            modifierSettings,
        }),
        [roleList, phaseTimes, enabledRoles, modifierSettings],
    );

    return (
        <div className="game-modes-editor">
            <GameModeContext.Provider value={contextValue}>
                <header>
                    <GameModeSelector
                        loadGameMode={(gameMode) => {
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
                </header>
                <main>
                    <div>
                        <EnabledRoleSelector
                            onDisableRoles={onDisableRoles}
                            onEnableRoles={onEnableRoles}
                            onIncludeAll={onEnableAll}
                        />
                        <ModifiersSelector disabled={false} setModifiers={setModifiers} />
                    </div>
                    <div>
                        <OutlineListSelector
                            onChangeRolePicker={onChangeRolePicker}
                            onAddNewOutline={addOutline}
                            onRemoveOutline={removeOutline}
                            setRoleList={setRoleList}
                        />
                    </div>
                </main>
            </GameModeContext.Provider>
        </div>
    );
}
