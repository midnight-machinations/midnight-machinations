import { CurrentFormat, GameModeStorage, ShareableGameMode } from "..";
import { Settings } from "../../../../game/localStorage";
import { ParseResult, Success, Failure, isFailure } from "../parse";

/**
 * A migration transforms data from one format to another.
 * Migrations are applied sequentially in the order they are registered.
 * 
 * Each migration should:
 * - Have a unique ID (timestamp-based to avoid collisions)
 * - Include a matcher to identify if it should run
 * - Include a transform function to modify the data
 * - Set the format field to the next version
 */
export type Migration<T = any> = {
    id: string;
    description: string;
    
    // Check if this migration applies to the given data
    matches: (json: NonNullable<any>) => boolean;
    
    // Transform the data to the next version
    transform: (json: NonNullable<any>) => ParseResult<T>;
};

type MigrationRegistry = {
    GameModeStorage: Migration<GameModeStorage>[];
    ShareableGameMode: Migration<ShareableGameMode>[];
    Settings: Migration<Settings>[];
};

type ConverterMap = {
    GameModeStorage: GameModeStorage;
    ShareableGameMode: ShareableGameMode;
    Settings: Settings;
};

// Registry of all migrations
const MIGRATIONS: MigrationRegistry = {
    GameModeStorage: [],
    ShareableGameMode: [],
    Settings: []
};

/**
 * Register a migration for a specific data type.
 */
export function registerMigration<T extends keyof MigrationRegistry>(
    type: T,
    migration: Migration
): void {
    MIGRATIONS[type].push(migration);
}

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
import "./gameModeStorageMigrations";
import "./shareableGameModeMigrations";
import "./settingsMigrations";
