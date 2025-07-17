
export const MODIFIERS = [
    "obscuredGraves",
    "skipDay1",
    "deadCanChat", "abstaining",
    "noDeathCause",
    "roleSetGraveKillers", "autoGuilty", 
    "twoThirdsMajority", "noTrialPhases", 
    "noWhispers", "hiddenWhispers",
    "noNightChat", "noChat", 
    "unscheduledNominations"
] as const;
export type ModifierType = (typeof MODIFIERS)[number];