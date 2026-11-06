/**
 * Feature flags from the root config.toml `[features]` table, compiled in as
 * VITE_FEATURE_<NAME> by `mask config` (or set directly for production builds).
 * An unset flag keeps its default.
 */
function flag(value: string | undefined, fallback: boolean): boolean {
    return value === undefined || value === "" ? fallback : value === "true";
}

export const FEATURES = {
    voice: flag(import.meta.env.VITE_FEATURE_VOICE, true),
} as const;
