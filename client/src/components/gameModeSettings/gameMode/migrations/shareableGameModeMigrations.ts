/**
 * Migrations for ShareableGameMode data structure.
 * 
 * These migrations parallel the GameModeStorage migrations but for single game modes
 * that are shared between users.
 */

import { registerMigration } from "./registry";
import { Success } from "../parse";
import { defaultPhaseTimes } from "../../../../game/gameState";

registerMigration("ShareableGameMode", {
    id: "2025-09-28-add-adjournment",
    description: "Add adjournment phase time",
    matches: (json) => json.format === "v5",
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

registerMigration("ShareableGameMode", {
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
