/**
 * Migrations for GameModeStorage data structure.
 * 
 * Each migration transforms the data from one format to the next.
 * Migrations are applied sequentially in registration order.
 */

import { registerMigration } from "./index";
import { Success, Failure, isFailure, ParseResult, ParseSuccess } from "../parse";
import { getRolesComplement } from "../../../../game/roleListState.d";
import { PHASES, PhaseTimes } from "../../../../game/gameState.d";
import { defaultPhaseTimes } from "../../../../game/gameState";

// Migration: initial -> v0
// Restructures the flat game mode structure into a grouped structure
registerMigration("GameModeStorage", {
    id: "2024-01-01-initial-to-v0",
    description: "Convert initial flat structure to v0 grouped structure",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === undefined &&
               !("gameModes" in json);
    },
    transform: (json) => {
        const gameModes = [];

        for (const gameMode of Object.values(json) as any[]) {
            if (typeof gameMode !== "object" || !gameMode || !gameMode.name) continue;
            
            let name = gameMode.name;
            const indexOfNumber = name.search(/\d*$/);
            if (indexOfNumber > 0 && name.charAt(indexOfNumber - 1) === ' ') {
                name = name.substring(0, indexOfNumber - 1);
            }

            const existingEntry = gameModes.find(entry => entry.name === name);

            if (existingEntry !== undefined) {
                existingEntry.data[gameMode.roleList.length] = {
                    roleList: gameMode.roleList,
                    phaseTimes: gameMode.phaseTimes,
                    disabledRoles: gameMode.disabledRoles
                };
            } else {
                gameModes.push({
                    name: name,
                    data: {
                        [gameMode.roleList.length]: {
                            roleList: gameMode.roleList,
                            phaseTimes: gameMode.phaseTimes,
                            disabledRoles: gameMode.disabledRoles
                        }
                    }
                });
            }
        }

        return Success({
            format: "v0",
            gameModes
        });
    }
});

// Migration: v0 -> v1
// Converts disabledRoles to enabledRoles
registerMigration("GameModeStorage", {
    id: "2024-01-02-v0-to-v1",
    description: "Convert disabledRoles to enabledRoles",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v0";
    },
    transform: (json) => {
        if (!json.gameModes || !Array.isArray(json.gameModes)) {
            return Failure("gameModesMissing", json);
        }

        const gameModes = json.gameModes.map((gameMode: any) => {
            const newData: Record<number, any> = {};
            
            for (const [players, data] of Object.entries(gameMode.data) as [string, any][]) {
                const disabledRoles = data.disabledRoles || [];
                const enabledRoles = getRolesComplement(disabledRoles);
                
                newData[players as any] = {
                    roleList: data.roleList,
                    phaseTimes: data.phaseTimes,
                    enabledRoles: enabledRoles
                };
            }

            return {
                name: gameMode.name,
                data: newData
            };
        });

        return Success({
            format: "v1",
            gameModes
        });
    }
});

// Migration: v1 -> v2
// Adds enabledModifiers field (initially empty array)
registerMigration("GameModeStorage", {
    id: "2024-01-03-v1-to-v2",
    description: "Add enabledModifiers field",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v1";
    },
    transform: (json) => {
        if (!json.gameModes || !Array.isArray(json.gameModes)) {
            return Failure("gameModesMissing", json);
        }

        const gameModes = json.gameModes.map((gameMode: any) => {
            const newData: Record<number, any> = {};
            
            for (const [players, data] of Object.entries(gameMode.data) as [string, any][]) {
                newData[players as any] = {
                    ...data,
                    enabledModifiers: []
                };
            }

            return {
                name: gameMode.name,
                data: newData
            };
        });

        return Success({
            format: "v2",
            gameModes
        });
    }
});

// Migration: v2 -> v3
// Converts faction-based role outlines to roleSet-based
registerMigration("GameModeStorage", {
    id: "2024-01-04-v2-to-v3",
    description: "Convert faction to roleSet in role outlines",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v2";
    },
    transform: (json) => {
        if (!json.gameModes || !Array.isArray(json.gameModes)) {
            return Failure("gameModesMissing", json);
        }

        const gameModes = json.gameModes.map((gameMode: any) => {
            const newData: Record<number, any> = {};
            
            for (const [players, data] of Object.entries(gameMode.data) as [string, any][]) {
                const roleList = data.roleList.map((outline: any) => {
                    if (outline.type === "any") {
                        return outline;
                    }
                    if (outline.type === "roleOutlineOptions") {
                        return {
                            type: "roleOutlineOptions",
                            options: outline.options.map((option: any) => {
                                if (option.type === "faction") {
                                    return {
                                        type: "roleSet",
                                        roleSet: option.faction
                                    };
                                }
                                return option;
                            })
                        };
                    }
                    return outline;
                });

                newData[players as any] = {
                    ...data,
                    roleList
                };
            }

            return {
                name: gameMode.name,
                data: newData
            };
        });

        return Success({
            format: "v3",
            gameModes
        });
    }
});

// Migration: v3 -> v4
// Simplifies role outline structure
registerMigration("GameModeStorage", {
    id: "2024-01-05-v3-to-v4",
    description: "Simplify role outline structure",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v3";
    },
    transform: (json) => {
        if (!json.gameModes || !Array.isArray(json.gameModes)) {
            return Failure("gameModesMissing", json);
        }

        const gameModes = json.gameModes.map((gameMode: any) => {
            const newData: Record<number, any> = {};
            
            for (const [players, data] of Object.entries(gameMode.data) as [string, any][]) {
                const roleList = data.roleList.map((outline: any) => {
                    if (outline.type === "any") {
                        return [{ roleSet: "any" }];
                    }
                    if (outline.type === "roleOutlineOptions") {
                        return outline.options.map((option: any) => {
                            if (option.type === "role") {
                                return { role: option.role };
                            }
                            if (option.type === "roleSet") {
                                return { roleSet: option.roleSet };
                            }
                            return option;
                        });
                    }
                    return outline;
                });

                newData[players as any] = {
                    ...data,
                    roleList
                };
            }

            return {
                name: gameMode.name,
                data: newData
            };
        });

        return Success({
            format: "v4",
            gameModes
        });
    }
});

// Migration: v4 -> v5
// Converts enabledModifiers array to modifierSettings map
registerMigration("GameModeStorage", {
    id: "2024-01-06-v4-to-v5",
    description: "Convert enabledModifiers to modifierSettings",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v4";
    },
    transform: (json) => {
        // Import defaultModifierState dynamically to avoid circular deps
        const { defaultModifierState } = require("../../../../game/modifiers");

        if (!json.gameModes || !Array.isArray(json.gameModes)) {
            return Failure("gameModesMissing", json);
        }

        const gameModes = json.gameModes.map((gameMode: any) => {
            const newData: Record<number, any> = {};
            
            for (const [players, data] of Object.entries(gameMode.data) as [string, any][]) {
                const enabledModifiers = data.enabledModifiers || [];
                const modifierSettings = enabledModifiers.map((modifierId: any) => 
                    [modifierId, defaultModifierState(modifierId)]
                );

                newData[players as any] = {
                    roleList: data.roleList,
                    phaseTimes: data.phaseTimes,
                    enabledRoles: data.enabledRoles,
                    modifierSettings
                };
            }

            return {
                name: gameMode.name,
                data: newData
            };
        });

        return Success({
            format: "v5",
            gameModes
        });
    }
});

// Migration: v5 -> v6
// Adds adjournment phase time if missing
registerMigration("GameModeStorage", {
    id: "2024-01-07-v5-to-v6",
    description: "Add adjournment phase time",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v5";
    },
    transform: (json) => {
        if (!json.gameModes || !Array.isArray(json.gameModes)) {
            return Failure("gameModesMissing", json);
        }

        const gameModes = json.gameModes.map((gameMode: any) => {
            const newData: Record<number, any> = {};
            
            for (const [players, data] of Object.entries(gameMode.data) as [string, any][]) {
                const phaseTimes = { ...data.phaseTimes };
                if (phaseTimes.adjournment === undefined) {
                    phaseTimes.adjournment = defaultPhaseTimes().adjournment;
                }

                newData[players as any] = {
                    ...data,
                    phaseTimes
                };
            }

            return {
                name: gameMode.name,
                data: newData
            };
        });

        return Success({
            format: "v6",
            gameModes
        });
    }
});
