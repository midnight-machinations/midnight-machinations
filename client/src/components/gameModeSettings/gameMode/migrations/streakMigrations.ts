/**
 * Migrations for Streaks
 */

import { registerMigration } from "./registry";
import { Success } from "../parse";

registerMigration("Streak", {
    id: "2026-07-05-weekly-streak",
    description: "Add weekly results streak tracking",
    matches: (json) => typeof json === "object" && json !== null && json.format === "initial",
    transform: (json) => {
        return Success({
            format: "2026-07-05-weekly-streak",
            lastPlayed: typeof json.lastPlayed === "number" ? json.lastPlayed : null,
            length: typeof json.length === "number" ? json.length : 0,
        });
    }
});
