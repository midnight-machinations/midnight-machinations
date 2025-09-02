import { PlayerIndex, UnsafeString } from "./gameState.d"
import { RoleSet } from "./roleListState.d"
import ROLES from "./../resources/roles.json";
import { ChatMessageVariant } from "../components/ChatMessage";
import { AuditorResult } from "../menu/game/gameScreenContent/AbilityMenu/RoleSpecificMenus/AuditorMenu";
import { Doomsayer } from "../menu/game/gameScreenContent/AbilityMenu/RoleSpecificMenus/LargeDoomsayerMenu";
import { TwoRoleOptionSelection } from "./controllerInput";
import { Hypnotist } from "../menu/game/gameScreenContent/AbilityMenu/RoleSpecificMenus/HypnotistMenu";

export type RoleState = {
    type: "jailor",
    executionsRemaining: number,
    jailedTargetRef: number | null
} | {
    type: "villager"
} | {
    type: "courtesan",
    previous: PlayerIndex[]
} | {
    type: "mayor"
} | {
    type: "transporter"
} | {
    type: "porter"
} | {
    type: "polymath"
} | {
    type: "detective"
} | {
    type: "lookout"
} | {
    type: "spy"
} | {
    type: "pyrolisk"
} | {
    type: "spiral"
} | {
    type: "tracker"
} | {
    type: "philosopher"
} | {
    type: "psychic"
} | {
    type: "auditor",
    previouslyGivenResults: [number, AuditorResult][]
} | {
    type: "snoop",
} | {
    type: "gossip",
} | {
    type: "tallyClerk"
} | {
    type: "percolator"
} | {
    type: "doctor",
    selfHealsRemaining: number,
} | {
    type: "bodyguard",
    selfShieldsRemaining: number,
} | {
    type: "cop",
} | {
    type: "bouncer"
} | {
    type: "engineer",
    trap: {type: "dismantled"} | {type: "ready"} | {type: "set"}
} | {
    type: "armorsmith",
    openShopsRemaining: number,
} | {
    type: "steward",
    stewardProtectsRemaining: number,
    previousRoleChosen: TwoRoleOptionSelection
} | {
    type: "vigilante",
    state: {type:"notLoaded"} | {type:"willSuicide"} | {type:"loaded",bullets:number} | {type:"suicided"}
} | {
    type: "veteran"
    alertsRemaining: number,
} | {
    type: "marksman"
    state: {type:"notLoaded"} | {type:"shotTownie"} | {type: "loaded"}
} | {
    type: "deputy"
} | {
    type: "rabblerouser"
} | {
    type: "escort"
} | {
    type: "medium",
    hauntsRemaining: number,
    seancedTarget: PlayerIndex | null
    hauntedTarget: PlayerIndex | null
} | {
    type: "retributionist"
} | {
    type: "reporter",
    public: boolean,
    report: UnsafeString,
    interviewedTarget: PlayerIndex | null
} | {
    type: "godfather"
} | {
    type: "impostor"
} | {
    type: "counterfeiter",
    forgesRemaining: number,
} | {
    type: "recruiter",
    recruitsRemaining: number
    backup: PlayerIndex | null
} | {
    type: "mafioso"
} | {
    type: "mafiaKillingWildcard"
    role: Role
} | {
    type: "goon"
} |
(Hypnotist & {type: "hypnotist"})
 | {
    type: "consort"
} | {
    type: "blackmailer",
    previous: PlayerIndex | null
} | {
    type: "cerenovous",
    previous: PlayerIndex | null,
    currentlyBrained: PlayerIndex | null,
    charges: number,
} | {
    type: "informant",
} | {
    type: "mortician",
    obscuredPlayers: PlayerIndex[],
    cremationsRemaining: number
} | {
    type: "forger",
    forgesRemaining: number,
} | {
    type: "disguiser",
    currentTarget: PlayerIndex | null,
    disguisedRole: Role,
} | {
    type: "reeducator",
    convertChargesRemaining: boolean,
    convertRole: Role,
} | {
    type: "framer"
} | {
    type: "necromancer"
} | {
    type: "mafiaSupportWildcard"
    role: Role
} | {
    type: "jester"
} | {
    type: "revolutionary"
} | 
Doomsayer 
| {
    type: "mercenary",
    roles: Role[],
    attacksRemaining: number,
    won: boolean
} | {
    type: "politician"
} | {
    type: "santaClaus",
    abilityUsedLastNight: "naughty" | "nice" | null
} | {
    type: "krampus",
    lastUsedAbility: "doNothing" | "kill" | null,
    ability: "doNothing" | "kill"
} | {
    type: "witch"
} | {
    type: "scarecrow"
} | {
    type: "warper"
} | {
    type: "kidnapper"
    executionsRemaining: number,
    jailedTargetRef: number | null
} | {
    type: "wildcard"
    role: Role
} | {
    type: "trueWildcard"
    role: Role
} | {
    type: "martyr",
    state: {
        type: "won"
    } | {
        type: "leftTown"
    } | {
        type: "stillPlaying",
        bullets: number
    }
} | {
    type: "arsonist"
} | {
    type: "werewolf",
} | {
    type: "ojo"
} | {
    type: "puppeteer"
    marionettesRemaining: number
} | {
    type: "warden"
} | {
    type: "yer",
    starPassesRemaining: number
} | {
    type: "kira"
} | {
    type: "fiendsWildcard"
} | {
    type: "apostle"
} | {
    type: "disciple"
} | {
    type: "zealot"
} | {
    type: "serialKiller"
}


export type Role = keyof typeof ROLES;
export type SingleRoleJsonData = {
    mainRoleSet: RoleSet,
    roleSets: RoleSet[],
    armor: boolean,
    aura: null | "innocent" | "suspicious",
    maxCount: null | number,
    canWriteDeathNote: boolean,
    canBeConvertedTo: Role[],
    chatMessages: ChatMessageVariant[] 
}
export type RoleJsonData = Record<Role, SingleRoleJsonData>

export function getMainRoleSetFromRole(role: Role): RoleSet {
    return roleJsonData()[role].mainRoleSet as RoleSet;
}

export function roleJsonData(): RoleJsonData {
    return ROLES as RoleJsonData;
}

export function getSingleRoleJsonData(role: Role): SingleRoleJsonData {
    return roleJsonData()[role];
}