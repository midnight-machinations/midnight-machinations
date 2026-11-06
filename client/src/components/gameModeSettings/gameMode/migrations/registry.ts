import type { Settings } from "../../../../game/localStorage";
import type { GameModeStorage, ShareableGameMode } from "..";
import type { ParseResult, UntypedJson } from "../parse";

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
export type Migration<T = unknown> = {
    id: string;
    description: string;

    // Check if this migration applies to the given data
    matches: (json: UntypedJson) => boolean;

    // Transform the data to the next version
    transform: (json: UntypedJson) => ParseResult<T>;
};

export type MigrationRegistry = {
    GameModeStorage: Migration<GameModeStorage>[];
    ShareableGameMode: Migration<ShareableGameMode>[];
    Settings: Migration<Settings>[];
};

// Registry of all migrations
export const MIGRATIONS: MigrationRegistry = {
    GameModeStorage: [],
    ShareableGameMode: [],
    Settings: [],
};

/**
 * Register a migration for a specific data type.
 */
export function registerMigration<T extends keyof MigrationRegistry>(
    type: T,
    migration: MigrationRegistry[T][number],
): void {
    // TypeScript cannot tie MIGRATIONS[type] to T, so name the element type it already has
    (MIGRATIONS[type] as MigrationRegistry[T][number][]).push(migration);
}
