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

registerMigration("ShareableGameMode", {
    id: "2026-06-19-remove-recess-phase-time",
    description: "Remove recess from phase time settings",
    matches: (json) => json.format === "2025-10-23-change-to-migration-id-format",
    transform: (json) => {
        const { recess, ...phaseTimes } = json.phaseTimes ?? {};

        return Success({
            ...json,
            format: "2026-06-19-remove-recess-phase-time",
            phaseTimes
        });
    }
});
