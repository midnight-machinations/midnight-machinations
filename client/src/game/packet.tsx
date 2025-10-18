import { PhaseType, PlayerIndex, PhaseTimes, Tag, LobbyClientID, ChatGroup, PhaseState, LobbyClient, InsiderGroup, GameClient, UnsafeString, FastForwardSetting } from "./gameState.d"
import { Grave, GraveIndex } from "./graveState"
import { ChatMessage, ChatMessageIndex } from "../components/ChatMessage"
import { RoleList, RoleOutline } from "./roleListState.d"
import { Role, RoleState } from "./roleState.d"
import { KiraGuess } from "../menu/game/gameScreenContent/AbilityMenu/ControllerSelectionTypes/KiraSelectionMenu"
import { ControllerInput, ControllerID, SavedController } from "./controllerInput"
import { ListMapData } from "../ListMap"
import { ModifierID, ModifierState } from "./modifiers"

export type LobbyPreviewData = {
    name: UnsafeString,
    inGame : boolean,
    players: [LobbyClientID, UnsafeString][]
}

export type ToClientPacket = {
    type: "pong",
} | {
    type: "hostData",
    clients: ListMapData<LobbyClientID, GameClient>
} | {
    type: "rateLimitExceeded",
} | {
    type: "forcedOutsideLobby"
} | {
    type: "forcedDisconnect"
} | {
    type: "lobbyList",
    lobbies: Record<number, LobbyPreviewData>,
} | {
    type: "acceptJoin",
    roomCode: number,
    inGame: boolean,
    playerId: number,
    spectator: boolean
} | {
    type: "rejectJoin",
    reason: string
} | 
// Lobby
{
    type: "yourId",
    playerId: LobbyClientID
} | {
    type: "lobbyClients",
    clients: ListMapData<LobbyClientID, LobbyClient>
} | {
    type: "lobbyName",
    name: UnsafeString
} | {
    type: "yourPlayerIndex",
    playerIndex: PlayerIndex
} | {
    type: "yourFellowInsiders",
    fellowInsiders: PlayerIndex[]
} | {
    type: "rejectStart",
    reason: string
} | {
    type: "playersHost",
    hosts: LobbyClientID[],
} | {
    type: "playersReady",
    ready: LobbyClientID[],
} | {
    type: "playersLostConnection",
    lostConnection: LobbyClientID[],
} | {
    type: "startGame"
} | {
    type: "gameInitializationComplete"
} | {
    type: "backToLobby",
} | {
    type: "gamePlayers",
    players: UnsafeString[]
} | {
    type: "roleList",
    roleList: RoleList,
} | {
    type: "randomSeed", 
    randomSeed: number | null,
} | {
    type: "roleOutline",
    index: number,
    roleOutline: RoleOutline
} | {
    type: "phaseTime",
    phase: Exclude<PhaseState, { type: "recess" }>, 
    time: number
} | {
    type: "phaseTimes",
    phaseTimeSettings: PhaseTimes
} | {
    type: "enabledRoles",
    roles: Role[]
} | {
    type: "modifierSettings",
    modifierSettings: {
        modifiers: ListMapData<ModifierID, ModifierState>,
    }
} |
// Game
{
    type: "phase",
    phase: PhaseState, 
    dayNumber: number, 
} | {
    type: "phaseTimeLeft",
    secondsLeft: number | null
} |{
    type: "playerAlive", 
    alive: [boolean]
} | {
    type: "playerVotes",
    votesForPlayer: ListMapData<number, number> 
} | {
    type: "yourSendChatGroups",
    sendChatGroups: ChatGroup[]
} | {
    type: "yourInsiderGroups",
    insiderGroups: InsiderGroup[]
} | {
    type: "yourAllowedControllers",
    save: ListMapData<ControllerID, SavedController>,
} | {
    type: "yourAllowedController",
    id: ControllerID, 
    controller: SavedController
} | {
    type: "yourRoleLabels",
    roleLabels: ListMapData<PlayerIndex, Role> 
} | {
    type: "yourPlayerTags",
    playerTags: ListMapData<PlayerIndex, Tag[]> 
} | {
    type: "yourNotes",
    notes: UnsafeString[]
} | {
    type: "yourCrossedOutOutlines",
    crossedOutOutlines: number[]
} | {
    type: "yourDeathNote", 
    deathNote: UnsafeString | null
} | {
    type: "yourRole",
    role: Role    
} | {
    type: "yourRoleState",
    roleState: RoleState
} | {
    type: "yourVoteFastForwardPhase",
    fastForward: FastForwardSetting
} | {
    type: "addChatMessages",
    chatMessages: [ChatMessageIndex, ChatMessage][]
} | {
    type: "nightMessages",
    chatMessages: ChatMessage[]
} | {
    type: "addGrave",
    grave: Grave,
    graveRef: GraveIndex,
} | {
    type: "gameOver",
    reason: string
} | {
    type: "yourPitchforkVote",
    player: PlayerIndex | null
}

export type ToServerPacket = {
    type: "ping",
} | {
    type: "lobbyListRequest",
} | {
    type: "hostDataRequest",
} | {
    type: "reJoin",
    roomCode: number,
    playerId: number,
} | {
    type: "join", 
    roomCode: number
} | {
    type: "host",
} | {
    type: "kick",
    playerId: number
} | {
    type: "setPlayerHost",
    playerId: number
} | {
    type: "relinquishHost",
}
// Lobby
| {
    type: "setSpectator",
    spectator: boolean
} | {
    type: "setName", 
    name: string
} | {
    type: "readyUp", 
    ready: boolean
} | {
    type: "sendLobbyMessage",
    text: string
} | {
    type: "setLobbyName", 
    name: string
} | {
    type: "startGame",
} | {
    type: "setRoleList", 
    roleList: RoleList,
} | {
    type: "setRandomSeed", 
    randomSeed: number | null,
} | {
    type: "setRoleOutline", 
    index: number,
    roleOutline: RoleOutline
} | {
    type: "simplifyRoleList"
} | {
    type: "setPhaseTime", 
    phase: PhaseType, 
    time: number
} | {
    type: "setPhaseTimes", 
    phaseTimeSettings: PhaseTimes
} | {
    type: "setEnabledRoles", 
    roles: Role[], 
} | {
    type: "setModifierSettings",
    modifierSettings: {
        modifiers: ListMapData<ModifierID, ModifierState>,
    }
} | {
    type: "enableModifier",
    modifier: ModifierID
} | {
    type: "disableModifier",
    modifier: ModifierID
} |
// Game
{
    type: "saveNotes", 
    notes: string[]
} | {
    type: "saveCrossedOutOutlines",
    crossedOutOutlines: number[]
} | {
    type: "saveDeathNote", 
    deathNote: string | null
} | {
    type: "leave",
} | {
    type: "controllerInput",
    controllerInput: ControllerInput
} | {
    type: "setKiraGuess",
    guesses: [PlayerIndex, KiraGuess][]
} | {
    type: "setConsortOptions",
    roleblock: boolean,
    
    youWereRoleblockedMessage: boolean,
    youSurvivedAttackMessage: boolean,
    youWereGuardedMessage: boolean,
    youWereTransportedMessage: boolean,
    youWerePossessedMessage: boolean,
    youWereWardblockedMessage: boolean
} | {
    type: "voteFastForwardPhase",
    fastForward: FastForwardSetting
} | {
    type: "hostForceBackToLobby"
} | {
    type: "hostForceEndGame",
} | {
    type: "hostForceSkipPhase",
} | {
    type: "hostForceSetPlayerName",
    id: number,
    name: string
}