
import { encodeString } from "../components/ChatMessage";
import { Conclusion, InsiderGroup, PlayerIndex, translateWinCondition, UnsafeString } from "./gameState.d";
import translate from "./lang";
import { Role, roleJsonData } from "./roleState.d";

export type RoleList = RoleOutline[];
export function getRolesFromRoleList(roleList: RoleList): Role[] {

    let set = new Set<Role>();
    for(let roleOutline of roleList){
        for(let role of getRolesFromOutline(roleOutline)){
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



export const ROLE_SETS = [
    "any",
    "town", "townCommon", "townInvestigative", "townProtective", "townKilling", "townSupport", 
    "mafia", "mafiaKilling", "mafiaSupport",
    "neutral", "minions",
    "fiends",
    "cult"
] as const;
export type RoleSet = typeof ROLE_SETS[number];
export function getRolesFromRoleSet(roleSet: RoleSet): Role[] {
    return getAllRoles().filter((role) => {
        return getRoleSetsFromRole(role).includes(roleSet);
    });
}
export function getRoleSetsFromRole(role: Role): RoleSet[] {
    const ROLES = roleJsonData();
    return [...ROLES[role].roleSets, "any"]
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




export function translateRoleOutline(roleOutline: RoleOutline, playerNames: UnsafeString[]): string {
    return roleOutline.map(outline => 
        translateRoleOutlineOption(outline, playerNames)).join(" "+translate("union:var.0")+" "
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
    
    console.log("PlayerPool: Names: ", playerNames);
    console.log("PlayerPool: ",out);
    console.trace();
    
    return out;
}

export function translateRoleOutlineOption(roleOutlineOption: RoleOutlineOption, playerNames: UnsafeString[]): string {
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
        out += translate(roleOutlineOption.roleSet)
    } else {
        out += translate("role."+roleOutlineOption.role+".name")
    }
    return out;
}
export function translateRoleOrRoleSet(roleOrRoleSet: RoleOrRoleSet): string {
    switch (roleOrRoleSet.type) {
        case "roleSet":
            return translate(roleOrRoleSet.roleSet)
        case "role":
            return translate("role."+roleOrRoleSet.role+".name")
    }
}
export function getRolesFromOutline(roleOutline: RoleOutline): Role[] {
    return roleOutline.flatMap((option) => getRolesFromOutlineOption(option));
}
export function getRolesFromOutlineOption(roleOutlineOption: RoleOutlineOption): Role[] {
    if ("roleSet" in roleOutlineOption) {
        return getRolesFromRoleSet(roleOutlineOption.roleSet)
    } else {
        return [roleOutlineOption.role]
    }
}
export function getRolesFromRoleOrRoleSet(roleOrRoleSet: RoleOrRoleSet): Role[] {
    switch (roleOrRoleSet.type) {
        case "roleSet":
            return getRolesFromRoleSet(roleOrRoleSet.roleSet)
        case "role":
            return [roleOrRoleSet.role]
    }
}

export function simplifyRoleOutline(roleOutline: RoleOutline): RoleOutline {
    let newOptions = [...roleOutline];

    newOptions = newOptions.filter((item, index, self) => {
        return index === self.findIndex((t) => deepEqual(item, t));
    });

    for(let optionA of roleOutline){
        for(let optionB of roleOutline){
            if(outlineOptionIsSubset(optionA, optionB) && !deepEqual(optionA, optionB)){
                newOptions = newOptions.filter((option) => option !== optionA);
            }
        }
    }

    newOptions = newOptions.sort(outlineOptionCompare);
    return newOptions;
}
function outlineOptionIsSubset(optionA: RoleOutlineOption, optionB: RoleOutlineOption): boolean {
    let rolesA = getRolesFromOutlineOption(optionA);
    let rolesB = getRolesFromOutlineOption(optionB);
    return rolesA.every((role) => rolesB.includes(role));
}
function outlineOptionCompare(optionA: RoleOutlineOption, optionB: RoleOutlineOption): number {
    let rolesA = getRolesFromOutlineOption(optionA);
    let rolesB = getRolesFromOutlineOption(optionB);
    return rolesB.length - rolesA.length;
}

export function getAllRoles(): Role[] {
    return Object.entries(roleJsonData())
        .sort((a, b) => translate(`role.${a[0]}.name`).localeCompare(translate(`role.${b[0]}.name`)))
        .sort((a, b) => ROLE_SETS.indexOf(a[1].mainRoleSet) - ROLE_SETS.indexOf(b[1].mainRoleSet))
        .map((a) => a[0]) as Role[];
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