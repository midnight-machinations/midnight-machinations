import ListMap, { ListMapData } from "../ListMap";
import { ModifierSettings, UnsafeString } from "./gameState.d";
import { getRolesFromRoleSet, RoleSet } from "./roleListState.d";
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
    type: "customRoleSets",
    sets: {
        name: UnsafeString,
        roleSets?: {
            roleSet: RoleSet,
            excludedRoles: Role[]
        }[],
        roles?: Role[]
    }[]
}

export type CustomRoleSetsModifierState = Extract<ModifierState, { type: "customRoleSets" }>;

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

export function customRoleSetRoles(
    setId: number,
    modifierSettings: ModifierSettings
): Role[] {
    const newModifierSettings = new ListMap([...modifierSettings.list]);

    const customRoleSets: CustomRoleSetsModifierState | null = newModifierSettings.get("customRoleSets") as any;

    if (customRoleSets === null) {
        return [];
    }

    const set = customRoleSets.sets.at(setId);

    // Recursive calls should "forget" this custom role set, to prevent infinite recursion.
    newModifierSettings.insert("customRoleSets", {
        ...customRoleSets,
        sets: customRoleSets.sets.filter((s, id) => id !== setId)
    });

    if (set === undefined) {
        return []
    }

    const subRoleSets = set.roleSets ?? [];
    const roles = set.roles ?? [];

    const out: Role[] = [];

    for (const subRoleSet of subRoleSets) {
        const rolesFromRoleSet = getRolesFromRoleSet(subRoleSet.roleSet, newModifierSettings);
        out.push(...rolesFromRoleSet.filter(role => !subRoleSet.excludedRoles.includes(role)));
    }

    out.push(...roles);

    return out;
}