# Migration System

This directory contains the migration-based system for transforming game mode storage, shareable game modes, and settings data structures.

## Overview

The migration system uses a simple, sequential approach where each migration transforms data from one format to the next. The format field in your data contains the ID of the last migration that was applied.

### Key Benefits

1. **No Version Collisions**: Migrations use timestamp-based IDs instead of version numbers
2. **Simpler to Write**: Each migration is just a matcher and transform function
3. **Sequential Application**: Migrations are applied in order until data reaches the current format
4. **Easy to Maintain**: All migrations for a data type are in one file
5. **Flexible Format Tracking**: The format field directly tells you which migration was last applied

## How It Works

1. When data is loaded (e.g., from localStorage), it's passed to `parseFromJson()`
2. The system finds the first migration that matches the data
3. The migration transforms the data and sets the format to its migration ID
4. This repeats until the data reaches the latest migration ID
5. The format field contains the ID of the last migration applied (e.g., `"2024-01-07-v5-to-v6"`)

## File Structure

- `index.ts` - Core migration infrastructure and runner
- `registry.ts` - Migration registry (separate to avoid circular dependencies)
- `gameModeStorageMigrations.ts` - Migrations for GameModeStorage data
- `shareableGameModeMigrations.ts` - Migrations for ShareableGameMode data
- `settingsMigrations.ts` - Migrations for Settings data

## Adding a New Migration

When you need to add a new migration (e.g., changing the data structure):

1. **Choose the appropriate file** based on what data type you're migrating
2. **Add a new migration** using `registerMigration()`:

    ```typescript
    registerMigration("GameModeStorage", {
        id: "YYYY-MM-DD-description",  // Use today's date
        description: "Brief description of what this migration does",
        matches: (json) => {
            // Return true if this migration should run on this data
            // Match the format field from the PREVIOUS migration
            return json.format === "2024-01-07-v5-to-v6";
        },
        transform: (json) => {
            // Transform the data to the next version
            // ... your transformation logic ...
            
            return Success({
                ...json,
                format: "YYYY-MM-DD-description",  // Set to THIS migration's ID
                // ... other changes ...
            });
        }
    });
    ```

3. **No need to update version constants** - The system automatically uses the last registered migration as the current format

## Examples

### Simple Field Addition

```typescript
registerMigration("Settings", {
    id: "2024-10-23-add-dark-mode",
    description: "Add dark mode setting",
    matches: (json) => json.format === "2024-01-02-settings-v3-to-v6",
    transform: (json) => {
        return Success({
            ...json,
            format: "2024-10-23-add-dark-mode",
            darkMode: false  // Add new field with default value
        });
    }
});
```

### Data Transformation

```typescript
registerMigration("GameModeStorage", {
    id: "2024-10-23-restructure-roles",
    description: "Restructure role list format",
    matches: (json) => json.format === "2024-01-07-v5-to-v6",
    transform: (json) => {
        const gameModes = json.gameModes.map((gameMode: any) => ({
            ...gameMode,
            data: Object.fromEntries(
                Object.entries(gameMode.data).map(([k, v]: [string, any]) => [
                    k,
                    {
                        ...v,
                        roleList: v.roleList.map(transformRole)
                    }
                ])
            )
        }));
        
        return Success({
            format: "2024-10-23-restructure-roles",
            gameModes
        });
    }
});
```

## Best Practices

1. **Use descriptive IDs**: Include the date and a clear description
2. **Keep migrations small**: Each migration should do one thing
3. **Test thoroughly**: Test with real data from previous versions
4. **Don't modify old migrations**: Once deployed, migrations are immutable
5. **Use type safety**: TypeScript will help catch errors in your transformations
6. **Match the previous format**: Always match against the format ID of the immediately previous migration

## How Format Tracking Works

Instead of using version strings like "v6", the format field now contains the migration ID:

**Before:**

```json
{
  "format": "v6",
  "gameModes": [...]
}
```

**After:**

```json
{
  "format": "2024-01-07-v5-to-v6",
  "gameModes": [...]
}
```

This means:

- You can tell exactly which migration was last applied
- No need to maintain a separate version constant
- New migrations just reference the previous migration's ID
- The system automatically knows the latest format by looking at the last registered migration
