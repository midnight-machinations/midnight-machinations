
import { encodeString } from "../components/ChatMessage";
import { Conclusion, InsiderGroup, ModifierSettings, PlayerIndex, translateWinCondition, UnsafeString } from "./gameState.d";
import translate from "./lang";
import { ModifierState } from "./modifiers";
import { Role, roleJsonData } from "./roleState.d";

export type RoleList = RoleOutline[];
export function getRolesFromRoleList(roleList: RoleList, modifierSettings: ModifierSettings): Role[] {

    let set = new Set<Role>();
    for(let roleOutline of roleList){
        for(let role of getRolesFromOutline(roleOutline, modifierSettings)){
            set.add(role);
        }
    }

    return Array.from(set);
}

export function getRolesComplement(roleList: Role[]): Role[] {
    return getAllRoles().filter((role) => {
        return !roleList.includes(role);
    });
}



export const BASE_ROLE_SETS = [
    "any",
    "town", "townCommon", "townInvestigative", "townProtective", "townKilling", "townSupport", 
    "mafia", "mafiaKilling", "mafiaSupport",
    "neutral", "minions",
    "fiends",
    "cult"
] as const;
export type BaseRoleSet = (typeof BASE_ROLE_SETS)[number];
export type RoleSet = { type: BaseRoleSet } | { type: "custom", id: number };
export function getRolesFromBaseRoleSet(roleSet: BaseRoleSet): Role[] {
    return getAllRoles().filter((role) => {
        return getBaseRoleSetsFromRole(role).includes(roleSet);
    });
}
export function getRolesFromRoleSet(roleSet: RoleSet, modifierSettings: ModifierSettings): Role[] {
    return getAllRoles().filter((role) => {
        return getRoleSetsFromRole(role, modifierSettings).some((rs) => deepEqual(rs, roleSet));
    });
}
export function getBaseRoleSetsFromRole(role: Role): BaseRoleSet[] {
    const ROLES = roleJsonData();
    return [...ROLES[role].roleSets, "any"]
}
export function getRoleSetsFromRole(role: Role, modifierSettings: ModifierSettings): RoleSet[] {
    const roleSets: RoleSet[] = getBaseRoleSetsFromRole(role).map((baseRoleSet) => ({ type: baseRoleSet }));

    const customRoleSetsModifier = modifierSettings.get("customRoleSets");
    if (customRoleSetsModifier !== null) {
        (customRoleSetsModifier as ModifierState & { type: "customRoleSets" }).sets.forEach((set, index) => {
            if (set.roles.includes(role)) {
                roleSets.push({ type: "custom", id: index });
            }
        });
    }

    return roleSets
}


export type RoleOutline = RoleOutlineOption[];

export type RoleOutlineOption = ({
    roleSet: RoleSet
} | {
    role: Role
}) & {
    winIfAny?: Conclusion[],
    insiderGroups?: InsiderGroup[]
    playerPool?: PlayerIndex[]
}

export type RoleOrRoleSet = ({
    type: "roleSet",
    roleSet: RoleSet
} | {
    type: "role",
    role: Role
})




export function translateRoleOutline(roleOutline: RoleOutline, playerNames: UnsafeString[], modifierSettings: ModifierSettings): string {
    return roleOutline.map(outline => 
        translateRoleOutlineOption(outline, playerNames, modifierSettings)).join(" "+translate("union:var.0")+" "
    )
}

export function translatePlayerPool(playerPool: PlayerIndex[], playerNames: UnsafeString[]): string {
    let out = '';
    if (playerPool.length === 0) {
        out += translate("nobody");
    }
    out += playerPool
        .map(playerNumber => encodeString(
            playerNames.at(playerNumber) ?? translate("player.unknown", playerNumber)
        ))
        .join(' ' + translate("union") + ' ')
    
    return out;
}

export function translateRoleSet(roleSet: RoleSet, modifierSettings?: ModifierSettings): string {
    switch (roleSet.type) {
        case "custom":
            const customRoleSetsModifier = modifierSettings?.get("customRoleSets");
            if (customRoleSetsModifier !== null && customRoleSetsModifier !== undefined) {
                const set = (customRoleSetsModifier as ModifierState & { type: "customRoleSets" }).sets[roleSet.id];
                if (set !== undefined) {
                    return encodeString(set.name);
                }
            }
            return translate("roleSet.customUnnamed", roleSet.id);
        default:
            return translate(roleSet.type);
    }
}

export function translateRoleOutlineOption(roleOutlineOption: RoleOutlineOption, playerNames: UnsafeString[], modifierSettings: ModifierSettings): string {
    let out = "";
    if (roleOutlineOption.playerPool) {
        out += translatePlayerPool(roleOutlineOption.playerPool, playerNames) + ': ';
    }
    if (roleOutlineOption.insiderGroups) {
        if (roleOutlineOption.insiderGroups.length === 0) {
            out += translate("chatGroup.all.icon")
        }
        out += roleOutlineOption.insiderGroups
            .map(insiderGroup => translate(`chatGroup.${insiderGroup}.icon`))
            .join(' ' + translate("union") + ' ');
        out += ', '
    }
    if (roleOutlineOption.winIfAny) {
        out += `${translateWinCondition({ type: "gameConclusionReached", winIfAny: roleOutlineOption.winIfAny })}, `;
    }
    if ("roleSet" in roleOutlineOption) {
        out += translateRoleSet(roleOutlineOption.roleSet, modifierSettings);
    } else {
        out += translate("role."+roleOutlineOption.role+".name")
    }
    return out;
}
export function translateRoleOrRoleSet(roleOrRoleSet: RoleOrRoleSet, modifierSettings: ModifierSettings): string {
    switch (roleOrRoleSet.type) {
        case "roleSet":
            return translateRoleSet(roleOrRoleSet.roleSet, modifierSettings);
        case "role":
            return translate("role."+roleOrRoleSet.role+".name")
    }
}
export function getRolesFromOutline(roleOutline: RoleOutline, modifierSettings: ModifierSettings): Role[] {
    return roleOutline.flatMap((option) => getRolesFromOutlineOption(option, modifierSettings));
}
export function getRolesFromOutlineOption(roleOutlineOption: RoleOutlineOption, modifierSettings: ModifierSettings): Role[] {
    if ("roleSet" in roleOutlineOption) {
        return getRolesFromRoleSet(roleOutlineOption.roleSet, modifierSettings)
    } else {
        return [roleOutlineOption.role]
    }
}
export function getRolesFromRoleOrRoleSet(roleOrRoleSet: RoleOrRoleSet, modifierSettings: ModifierSettings): Role[] {
    switch (roleOrRoleSet.type) {
        case "roleSet":
            return getRolesFromRoleSet(roleOrRoleSet.roleSet, modifierSettings)
        case "role":
            return [roleOrRoleSet.role]
    }
}

export function simplifyRoleOutline(roleOutline: RoleOutline, modifierSettings: ModifierSettings): RoleOutline {
    let newOptions = [...roleOutline];

    newOptions = newOptions.filter((item, index, self) => {
        return index === self.findIndex((t) => deepEqual(item, t));
    });

    for(let optionA of roleOutline){
        for(let optionB of roleOutline){
            if(outlineOptionIsSubset(optionA, optionB, modifierSettings) && !deepEqual(optionA, optionB)){
                newOptions = newOptions.filter((option) => option !== optionA);
            }
        }
    }

    newOptions = newOptions.sort((a, b) => outlineOptionCompare(a, b, modifierSettings));
    return newOptions;
}
function outlineOptionIsSubset(optionA: RoleOutlineOption, optionB: RoleOutlineOption, modifierSettings: ModifierSettings): boolean {
    let rolesA = getRolesFromOutlineOption(optionA, modifierSettings);
    let rolesB = getRolesFromOutlineOption(optionB, modifierSettings);
    return rolesA.every((role) => rolesB.includes(role));
}
function outlineOptionCompare(optionA: RoleOutlineOption, optionB: RoleOutlineOption, modifierSettings: ModifierSettings): number {
    let rolesA = getRolesFromOutlineOption(optionA, modifierSettings);
    let rolesB = getRolesFromOutlineOption(optionB, modifierSettings);
    return rolesB.length - rolesA.length;
}

export function getAllRoles(): Role[] {
    return (Object.keys(roleJsonData()) as Role[])
        .sort(sortRolesCanonically);
}

export function sortRolesCanonically(a: Role, b: Role): number {
    const roleJson = roleJsonData()
    const roleSetA = BASE_ROLE_SETS.indexOf(roleJson[a].mainRoleSet)
    const roleSetB = BASE_ROLE_SETS.indexOf(roleJson[b].mainRoleSet)
    if (roleSetA !== roleSetB) {
        return roleSetA - roleSetB
    } else {
        return translate(`role.${a}.name`).localeCompare(translate(`role.${b}.name`))
    }
}


function deepEqual(obj1: any, obj2: any): boolean {
    // Check if the objects are strictly equal
    if (obj1 === obj2) {
        return true;
    }
  
    // if both are null or undefined then return true
    if (obj1 == null && obj2 == null) {
        return true;
    }


    // Check if both objects are objects and not null
    if (typeof obj1 !== "object" || obj1 === null ||
        typeof obj2 !== "object" || obj2 === null) {
        return false;
    }
  
    // Check if the objects have the same number of keys
    const keys1 = Object.keys(obj1);
    const keys2 = Object.keys(obj2);
    if (keys1.length !== keys2.length) {
        return false;
    }
  
    // Recursively compare each key-value pair
    for (const key of keys1) {
        if (!deepEqual(obj1[key], obj2[key])) {
            return false;
        }
    }
  
    return true;
}