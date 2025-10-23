# Migration System

This directory contains the new migration-based system for transforming game mode storage, shareable game modes, and settings data structures.

## Overview

The migration system replaces the old "datafixer" approach with a simpler, more maintainable system based on sequential transformations.

### Key Benefits

1. **No Version Collisions**: Migrations use timestamp-based IDs instead of version numbers
2. **Simpler to Write**: Each migration is just a matcher and transform function
3. **Sequential Application**: Migrations are applied in order until data reaches the current format
4. **Easy to Maintain**: All migrations for a data type are in one file

## How It Works

1. When data is loaded (e.g., from localStorage), it's passed to `parseFromJson()`
2. The system finds the first migration that matches the data
3. The migration transforms the data to the next version
4. This repeats until the data reaches the current format (v6)

## File Structure

- `index.ts` - Core migration infrastructure and runner
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
        return json.format === "v6";  // Match the previous version
    },
    transform: (json) => {
        // Transform the data to the next version
        // ... your transformation logic ...
        
        return Success({
            ...json,
            format: "v7",  // Update to new version
            // ... other changes ...
        });
    }
});
```

3. **Update the current format** in `index.ts`:
   - Change `LATEST_VERSION_STRING` to the new version (e.g., "v7")
   - Update the `CurrentFormat` type in `../index.ts`

## Examples

### Simple Field Addition

```typescript
registerMigration("Settings", {
    id: "2024-10-23-add-dark-mode",
    description: "Add dark mode setting",
    matches: (json) => json.format === "v6",
    transform: (json) => {
        return Success({
            ...json,
            format: "v7",
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
    matches: (json) => json.format === "v6",
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
            format: "v7",
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

## Migration History

- `initial -> v0`: Restructure flat game mode structure
- `v0 -> v1`: Convert disabledRoles to enabledRoles
- `v1 -> v2`: Add enabledModifiers field
- `v2 -> v3`: Convert faction to roleSet in role outlines
- `v3 -> v4`: Simplify role outline structure
- `v4 -> v5`: Convert enabledModifiers to modifierSettings
- `v5 -> v6`: Add adjournment phase time
