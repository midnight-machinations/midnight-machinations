/**
 * Migrations for Settings data structure.
 */

import { registerMigration } from "./index";
import { Success } from "../parse";
import { getDefaultSettings } from "../../../../game/localStorage";

// Migration: initial -> v3
// Adds format field and converts from initial settings structure
registerMigration("Settings", {
    id: "2024-01-01-initial-to-v3",
    description: "Add format field and validate settings structure",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === undefined &&
               "volume" in json &&
               "defaultName" in json;
    },
    transform: (json) => {
        return Success({
            format: "v3",
            ...json
        });
    }
});

// Migration: v3 -> v6
// Adds maxMenus and menuOrder fields, removes roleSpecificMenus
registerMigration("Settings", {
    id: "2024-01-02-v3-to-v6",
    description: "Add maxMenus and menuOrder, remove roleSpecificMenus",
    matches: (json) => {
        return typeof json === "object" && 
               !Array.isArray(json) && 
               json.format === "v3";
    },
    transform: (json) => {
        const result: any = { ...json, format: "v6" };

        // Add missing fields from default settings
        if (!("maxMenus" in result)) {
            result.maxMenus = getDefaultSettings().maxMenus;
        }

        if (!("menuOrder" in result)) {
            result.menuOrder = getDefaultSettings().menuOrder;
        } else {
            // Ensure menuOrder has the correct structure for all menus
            for (const key of ['WikiMenu', 'GraveyardMenu', 'PlayerListMenu', 'ChatMenu', 'WillMenu', 'RoleSpecificMenu']) {
                if (!Array.isArray(result.menuOrder[key])) {
                    result.menuOrder[key] = [key, key === "ChatMenu"];
                }
            }
        }

        // Remove deprecated roleSpecificMenus field
        if ("roleSpecificMenus" in result) {
            delete result.roleSpecificMenus;
        }

        return Success(result);
    }
});
