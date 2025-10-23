/**
 * Migrations for GameModeStorage data structure.
 * 
 * Each migration transforms the data from one format to the next.
 * Migrations are applied sequentially in registration order.
 */

import { registerMigration } from "./registry";
import { Success, Failure } from "../parse";
import { defaultPhaseTimes } from "../../../../game/gameState";

registerMigration("GameModeStorage", {
    id: "2025-09-28-add-adjournment",
    description: "Add adjournment phase time",
    matches: (json) => json.format === "v5",
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

registerMigration("GameModeStorage", {
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
