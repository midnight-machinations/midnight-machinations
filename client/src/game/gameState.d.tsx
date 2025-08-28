import { Grave, GraveIndex } from "./graveState";
import { ChatMessage, ChatMessageIndex } from "../components/ChatMessage";
import { Role, RoleState } from "./roleState.d";
import { RoleList } from "./roleListState.d";
import { LobbyPreviewData } from "./packet";
import { ChatFilter } from "../menu/game/gameScreenContent/ChatMenu";
import { ControllerID, SavedController } from "./controllerInput";
import translate from "./lang";
import ListMap, { ListMapData } from "../ListMap";
import { ModifierID, ModifierState } from "./modifiers";

export type State = Disconnected | OutsideLobbyState | LobbyState | GameState;

export type Disconnected = {
    stateType: "disconnected"
}

export type OutsideLobbyState = {
    stateType: "outsideLobby",

    selectedRoomCode: string | null,
    lobbies: Map<number, LobbyPreviewData>,
}


//Change this to use PlayerID for player map and playerID for who I AM instead of myName and host
export type LobbyState = {
    stateType: "lobby"
    roomCode: number,
    lobbyName: UnsafeString,

    myId: number | null,

    roleList: RoleList,
    phaseTimes: PhaseTimes,
    enabledRoles: Role[],
    modifierSettings: ListMap<ModifierID, ModifierState>,

    players: ListMap<LobbyClientID, LobbyClient>,
    chatMessages: ListMap<ChatMessageIndex, ChatMessage>,
}
export type LobbyClient = {
    ready: "host" | "ready" | "notReady",
    connection: ClientConnection,
    clientType: LobbyClientType
}
export type ClientConnection = "connected" | "disconnected" | "couldReconnect";
export type GameClient = {
    clientType: GameClientType,
    connection: ClientConnection,
    host: boolean,
}
export type GameClientType = {
    type: "spectator",
    index: number
} | {
    type: "player",
    index: number,
}
export type LobbyClientType = {
    type: "spectator"
} | PlayerClientType;
export type PlayerClientType = {
    type: "player",
    name: UnsafeString,
}

type GameState = {
    stateType: "game",
    roomCode: number,
    lobbyName: UnsafeString,
    
    initialized: boolean,

    myId: number | null,

    chatMessages : ListMap<ChatMessageIndex, ChatMessage>,
    graves: ListMap<GraveIndex, Grave>,
    players: Player[],
    
    phaseState: PhaseState,
    timeLeftMs: number | null,
    dayNumber: number,

    fastForward: boolean,
    
    roleList: RoleList,
    enabledRoles: Role[],
    phaseTimes: PhaseTimes,
    modifierSettings: ListMap<ModifierID, ModifierState>,

    ticking: boolean,

    clientState: PlayerGameState | {type: "spectator"},
    host: null | {
        clients: ListMap<LobbyClientID, GameClient>
    },

    missedChatMessages: boolean
}
export default GameState;

export type PlayerGameState = {
    type: "player",

    myIndex: PlayerIndex,
    
    roleState: RoleState,

    notes: UnsafeString[],
    crossedOutOutlines: number[],
    chatFilter: ChatFilter,
    deathNote: UnsafeString,

    savedControllers: ListMapData<ControllerID, SavedController>,

    fellowInsiders: PlayerIndex[],

    sendChatGroups: ChatGroup[],
    insiderGroups: InsiderGroup[],
    
    missedWhispers: PlayerIndex[]
}

export type PlayerIndex = number;
export type LobbyClientID = number;
export type Verdict = "innocent"|"guilty"|"abstain";
export const PHASES = ["briefing", "obituary", "discussion", "nomination", "testimony", "judgement", "finalWords", "dusk", "night", "recess"] as const;
export type PhaseType = (typeof PHASES)[number];
export type PhaseState = {type: "briefing"} | {type: "recess"} | {type: "dusk"} | {type: "night"} | {type: "obituary"} | {type: "discussion"} | 
{
    type: "nomination",
    trialsLeft: number
} | {
    type: "testimony",
    playerOnTrial: PlayerIndex
    trialsLeft: number
} | {
    type: "judgement",
    playerOnTrial: PlayerIndex
    trialsLeft: number
} | {
    type: "finalWords",
    playerOnTrial: PlayerIndex
}

export type ChatGroup = "all" | "dead" | "mafia" | "cult" | "jail" | "kidnapper" | "interview" | "puppeteer";
export type InsiderGroup = (typeof INSIDER_GROUPS)[number];
export const INSIDER_GROUPS = ["mafia", "cult", "puppeteer"] as const;
export type PhaseTimes = Record<Exclude<PhaseType, "recess">, number>;
export type DefensePower = "none"|"armored"|"protected"|"invincible";

export type Tag = 
    "syndicateGun" |
    "godfatherBackup" |
    "werewolfTracked" |
    "doused" |
    "revolutionaryTarget" |
    "morticianTagged" |
    "puppeteerMarionette" |
    "frame" |
    "forfeitNominationVote" |
    "spiraling";

export type Player = {
    name: UnsafeString,
    index: number,
    numVoted: number,
    alive: boolean,
    roleLabel: Role | null,
    playerTags: Tag[]

    toString(): UnsafeString
}

// Not actually unknown, but this prevents use without sanitization
export type UnsafeString = string | (unknown & { __brand?: "UnsafeString" });

export const CONCLUSIONS = ["town", "mafia", "cult", "fiends", "politician", "niceList", "naughtyList", "draw"] as const;
export type Conclusion = (typeof CONCLUSIONS)[number];

export type WinCondition = {
    type: "gameConclusionReached"
    winIfAny: Conclusion[]
} | {
    type: "roleStateWon"
}

export function translateConclusion(conclusion: Conclusion): string {
    switch (conclusion) {
        case "politician":
            return translate("role.politician.name")
        case "draw":
            return translate("winCondition.draw")
        default:
            return translate(conclusion)
    }
}

export function translateWinCondition(winCondition: WinCondition): string {
    if (winCondition.type === "gameConclusionReached") {
        if (winCondition.winIfAny.length === 0) {
            return translate("winCondition.loser")
        } else if (winCondition.winIfAny.length === 1) {
            return translateConclusion(winCondition.winIfAny[0])
        } else if (winCondition.winIfAny.length === 4 && 
            (["mafia", "fiends", "cult", "politician"] as const).every(team => winCondition.winIfAny.includes(team))
        ) {
            return translate(`winCondition.evil`)
        } else {
            return winCondition.winIfAny.map(conclusion => translateConclusion(conclusion)).join(` ${translate('union')} `)
        }
    } else {
        return translate("winCondition.independent");
    }
}