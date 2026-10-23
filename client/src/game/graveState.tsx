import { PlayerIndex, UnsafeString } from "./gameState.d";
import translate from "./lang";
import { RoleList, RoleSet } from "./roleListState.d";
import { Role } from "./roleState.d";

export type GraveIndex = number;

export type Grave = {
    player: PlayerIndex,
    diedPhase: GravePhase,
    dayNumber: number,
    information: GraveInformation,
}

export type GraveInformation = {
    type: "obscured",
} | {
    type: "normal",
    
    role: Role,
    alibi: UnsafeString,
    deathCauses: GraveDeathCause[],
    callingCards: UnsafeString[],
}

export type GraveDeathCause = {
    type: "execution" | "ascension" | "suicide" | "quit"
} | {
    type: "roleSet"
    value: RoleSet
} | {
    type: "role"
    value: Role
} | {
    type: "quit"
};

export type GravePhase = "day" | "night"

export function translateGraveDeathCause(deathCause: GraveDeathCause): string {
    switch(deathCause.type){
        case "role":
            return translate("role."+deathCause.value+".name");
        case "roleSet":
            return translate(deathCause.value);
        default:
            return translate("grave.deathCause."+deathCause.type);
    }
}
export function translateGraveDeathCauses(deathCauses: GraveDeathCause[]): string {
    return deathCauses.map((cause) =>  translateGraveDeathCause(cause)).join(", ");
}