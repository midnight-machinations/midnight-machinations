import { PlayerIndex } from "./gameState.d";
import { RoleSet } from "./roleListState.d";
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
    will: string,
    deathCause: GraveDeathCause,
    deathNotes: string[],
}

export type GraveDeathCause = {
    type: "execution" | "leftTown" | "brokenHeart" | "none"
} | {
    type: "killers"
    killers: GraveKiller[]
}
export type GraveKiller = {
    type: "roleSet"
    value: RoleSet
} | {
    type: "suicide"
} | {
    type: "quit"
} | {
    type: "role"
    value: Role
};

export type GravePhase = "day" | "night"