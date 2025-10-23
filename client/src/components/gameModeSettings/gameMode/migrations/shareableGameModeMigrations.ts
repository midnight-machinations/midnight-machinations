/**
 * Migrations for ShareableGameMode data structure.
 * 
 * These migrations parallel the GameModeStorage migrations but for single game modes
 * that are shared between users.
 */

import { registerMigration } from "./index";
import { Success, Failure } from "../parse";
import { getRolesComplement } from "../../../../game/roleListState.d";
import { defaultPhaseTimes } from "../../../../game/gameState";

// Migration: initial -> v0
// Adds format field and validates structure
registerMigration("ShareableGameMode", {
    id: "2024-01-01-initial-to-v0",
    description: "Add format field to shareable game mode",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === undefined &&
               "name" in json &&
               "roleList" in json;
    },
    transform: (json) => {
        return Success({
            format: "v0",
            ...json
        });
    }
});

// Migration: v0 -> v1
// Converts disabledRoles to enabledRoles
registerMigration("ShareableGameMode", {
    id: "2024-01-02-v0-to-v1",
    description: "Convert disabledRoles to enabledRoles",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v0";
    },
    transform: (json) => {
        const disabledRoles = json.disabledRoles || [];
        const enabledRoles = getRolesComplement(disabledRoles);
        
        const result: any = {
            format: "v1",
            name: json.name,
            roleList: json.roleList,
            phaseTimes: json.phaseTimes,
            enabledRoles: enabledRoles
        };

        return Success(result);
    }
});

// Migration: v1 -> v2
// Adds enabledModifiers field
registerMigration("ShareableGameMode", {
    id: "2024-01-03-v1-to-v2",
    description: "Add enabledModifiers field",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v1";
    },
    transform: (json) => {
        return Success({
            ...json,
            format: "v2",
            enabledModifiers: []
        });
    }
});

// Migration: v2 -> v3
// Converts faction to roleSet in role outlines
registerMigration("ShareableGameMode", {
    id: "2024-01-04-v2-to-v3",
    description: "Convert faction to roleSet in role outlines",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v2";
    },
    transform: (json) => {
        const roleList = json.roleList.map((outline: any) => {
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

        return Success({
            ...json,
            format: "v3",
            roleList
        });
    }
});

// Migration: v3 -> v4
// Simplifies role outline structure
registerMigration("ShareableGameMode", {
    id: "2024-01-05-v3-to-v4",
    description: "Simplify role outline structure",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v3";
    },
    transform: (json) => {
        const roleList = json.roleList.map((outline: any) => {
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

        return Success({
            ...json,
            format: "v4",
            roleList
        });
    }
});

// Migration: v4 -> v5
// Converts enabledModifiers to modifierSettings
registerMigration("ShareableGameMode", {
    id: "2024-01-06-v4-to-v5",
    description: "Convert enabledModifiers to modifierSettings",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v4";
    },
    transform: (json) => {
        const { defaultModifierState } = require("../../../../game/modifiers");

        const enabledModifiers = json.enabledModifiers || [];
        const modifierSettings = enabledModifiers.map((modifierId: any) => 
            [modifierId, defaultModifierState(modifierId)]
        );

        return Success({
            format: "v5",
            name: json.name,
            roleList: json.roleList,
            phaseTimes: json.phaseTimes,
            enabledRoles: json.enabledRoles,
            modifierSettings
        });
    }
});

// Migration: v5 -> v6
// Adds adjournment phase time if missing
registerMigration("ShareableGameMode", {
    id: "2024-01-07-v5-to-v6",
    description: "Add adjournment phase time",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v5";
    },
    transform: (json) => {
        const phaseTimes = { ...json.phaseTimes };
        if (phaseTimes.adjournment === undefined) {
            phaseTimes.adjournment = defaultPhaseTimes().adjournment;
        }

        return Success({
            ...json,
            format: "v6",
            phaseTimes
        });
    }
});
