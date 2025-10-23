import { CurrentFormat, GameModeStorage, ShareableGameMode } from "..";
import { Settings } from "../../../../game/localStorage";
import { ParseResult, Success, Failure, isFailure } from "../parse";
import { MIGRATIONS, registerMigration } from "./registry";

// Re-export for convenience
export type { Migration } from "./registry";
export { registerMigration } from "./registry";

type ConverterMap = {
    GameModeStorage: GameModeStorage;
    ShareableGameMode: ShareableGameMode;
    Settings: Settings;
};

/**
 * Apply all necessary migrations to bring data to the current format.
 * Migrations are applied sequentially until no more migrations match.
 */
export function applyMigrations<T extends keyof ConverterMap>(
    type: T,
    json: NonNullable<any>
): ParseResult<ConverterMap[T]> {
    const MAX_ITERATIONS = 1000;
    let currentJson = json;
    let migrationsApplied = 0;

    for (let i = 0; i < MAX_ITERATIONS; i++) {
        // Find the first migration that matches
        const migration = MIGRATIONS[type].find(m => m.matches(currentJson));

        if (!migration) {
            // No more migrations to apply - check if we're at the current format
            if (typeof currentJson === "object" && 
                !Array.isArray(currentJson) && 
                currentJson.format === getCurrentFormat(type)) {
                return Success(currentJson as ConverterMap[T]);
            }
            
            // Data doesn't match any migration and isn't current format
            return Failure("unsupportedFormat", currentJson);
        }

        // Apply the migration
        const result = migration.transform(currentJson);
        if (isFailure(result)) {
            return result;
        }

        currentJson = result.value;
        migrationsApplied++;
    }

    return Failure("tooManyMigrations", { migrationsApplied, data: json });
}

/**
 * Get the current format version for a data type.
 */
function getCurrentFormat(type: keyof ConverterMap): CurrentFormat {
    return "v6"; // This is the latest format
}

/**
 * Main entry point for parsing and migrating data.
 */
export default function parseFromJson<T extends keyof ConverterMap>(
    type: T,
    json: NonNullable<any>
): ParseResult<ConverterMap[T]> {
    return applyMigrations(type, json);
}

// Export the current format for use by other modules
export const LATEST_VERSION_STRING: CurrentFormat = "v6";

// Import all migrations to register them
// These imports are at the end of the file to ensure MIGRATIONS is initialized first
import "./gameModeStorageMigrations";
import "./shareableGameModeMigrations";
import "./settingsMigrations";
