import { ListMapData } from "../ListMap";
import { UnsafeString } from "./gameState.d";
import { Role } from "./roleState.d";

export const MODIFIERS = [
    "obscuredGraves",
    "skipDay1",
    "deadCanChat", "abstaining",
    "noDeathCause",
    "roleSetGraveKillers", "autoGuilty", 
    "twoThirdsMajority", "noTrialPhases", 
    "noWhispers", "hiddenWhispers",
    "noNightChat", "noChat", 
    "unscheduledNominations",
    "hiddenNominationVotes", "hiddenVerdictVotes",
    "forfeitNominationVote", "randomPlayerNames",
    "customRoleLimits", "customRoleSets"
] as const;

export type ModifierID = (typeof MODIFIERS)[number];

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
    type: "customRoleSets",
    sets: {
        name: UnsafeString,
        roles: Role[]
    }[]
}

export function defaultModifierState(modifierId: ModifierID): ModifierState {
    switch (modifierId) {
        case "customRoleLimits":
            return { type: modifierId, limits: [] };
        case "customRoleSets":
            return { type: modifierId, sets: [] };
        default:
            return { type: modifierId };
    }
}

export function isModifierConfigurable(modifierId: ModifierID): boolean {
    switch (modifierId) {
        case "customRoleLimits":
        case "customRoleSets":
            return true;
        default:
            return false;
    }
}