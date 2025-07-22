import translate from "../../game/lang"
import { PlayerIndex } from "./otherState"
import { RoleSet } from "./roleListState"
import { Role } from "./roleState"

export type Grave = {
    player: PlayerIndex,
    diedPhase: GravePhase,
    dayNumber: number,
    information: GraveInformation,
}

export type GraveInformation ={ 
    type: "obscured",
} | {
    type: "normal",
    
    role: Role,
    will: string,
    deathCause: GraveDeathCause,
    deathNotes: string[],
}
export function translateGraveRole(graveInformation: GraveInformation): string{
    switch (graveInformation.type) {
        case "obscured":
            return translate("obscured");
        case "normal":
            return translate("role."+graveInformation.role+".name");
    }
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