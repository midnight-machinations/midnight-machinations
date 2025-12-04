import { ListMapData } from "../ListMap";
import { Role } from "./roleState.d";

export const MODIFIERS = [
    "obscuredGraves",
    "skipDay1",
    "deadCanChat", "abstaining",
    "noDeathCause",
    "roleSetGraveKillers", "autoGuilty", 
    "twoThirdsMajority", "noMajority", "noTrialPhases", 
    "noWhispers", "hiddenWhispers",
    "noNightChat", "noChat", 
    "unscheduledNominations",
    "hiddenNominationVotes", "hiddenVerdictVotes",
    "forfeitNominationVote", "randomPlayerNames",
    "customRoleLimits",
    "gravity"
] as const;

export type ModifierID = (typeof MODIFIERS)[number];

export type GravityLevel = "zeroGravity" | "antiGravity";

export type ModifierState = {
    type: "obscuredGraves"
} | {
    type: "skipDay1"
} | {
    type: "deadCanChat"
} | {
    type: "abstaining"
} | {
    type: "noDeathCause"
} | {
    type: "roleSetGraveKillers"
} | {
    type: "autoGuilty"
} | {
    type: "twoThirdsMajority"
} | {
    type: "noMajority"
} | {
    type: "noTrialPhases"
} | {
    type: "noWhispers"
} | {
    type: "hiddenWhispers"
} | {
    type: "noNightChat"
} | {
    type: "noChat"
} | {
    type: "unscheduledNominations"
} | {
    type: "hiddenNominationVotes"
} | {
    type: "hiddenVerdictVotes"
} | {
    type: "forfeitNominationVote"
} | {
    type: "randomPlayerNames"
} | {
    type: "customRoleLimits",
    limits: ListMapData<Role, number>
} | {
    type: "gravity",
    level: GravityLevel
}

export function defaultModifierState(modifierId: ModifierID): ModifierState {
    switch (modifierId) {
        case "customRoleLimits":
            return { type: modifierId, limits: [] };
        case "gravity":
            return { type: modifierId, level: "zeroGravity" };
        default:
            return { type: modifierId };
    }
}

export function isModifierConfigurable(modifierId: ModifierID): boolean {
    switch (modifierId) {
        case "customRoleLimits":
        case "gravity":
            return true;
        default:
            return false;
    }
}