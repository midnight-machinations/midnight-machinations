/**
 * Migrations for Settings data structure.
 */

import { registerMigration } from "./registry";
import { Success } from "../parse";
import { getDefaultSettings } from "../../../../game/localStorage";

registerMigration("Settings", {
    id: "2024-12-28-new-menu-settings",
    description: "Add maxMenus and menuOrder, remove roleSpecificMenus",
    matches: (json) => json.format === "v3",
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

registerMigration("Settings", {
    id: "2025-10-23-change-to-migration-id-format",
    description: "Change to migration ID format",
    matches: (json) => json.format === "v6",
    transform: (json) => {
        return Success({
            ...json,
            format: "2025-10-23-change-to-migration-id-format",
        });
    }
});
