import { ChatMessage } from "./components/ChatMessage"
import { RoleList, RoleOutline } from "./stateContext/stateType/roleListState"
import { DoomsayerGuess } from "./menu/game/gameScreenContent/AbilityMenu/RoleSpecificMenus/LargeDoomsayerMenu"
import { KiraGuess } from "./menu/game/gameScreenContent/AbilityMenu/AbilitySelectionTypes/KiraSelectionMenu"
import { AbilityInput, ControllerID, SavedController } from "./game/abilityInput"
import { ListMapData } from "./ListMap"
import { LobbyPreviewData } from "./stateContext/stateType/gameBrowserState"
import { ChatGroup, InsiderGroup, LobbyClientID, PhaseTimes, PlayerIndex, Verdict } from "./stateContext/stateType/otherState"
import { GameClient } from "./stateContext/stateType/gameState"
import { PhaseState, PhaseType } from "./stateContext/stateType/phaseState"
import { ModifierType } from "./stateContext/stateType/modifiersState"
import { Role, RoleState } from "./stateContext/stateType/roleState"
import { Grave } from "./stateContext/stateType/grave"
import { Tag } from "./stateContext/stateType/tagState"
import { ClientObject } from "./stateContext/state"

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
    type: "clientObjects",
    clients: ListMapData<LobbyClientID, ClientObject>,
} | {
    type: "lobbyName",
    name: string
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
    players: string[]
} | {
    type: "roleList",
    roleList: RoleList,
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
    type: "enabledModifiers",
    modifiers: ModifierType[]
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
    type: "yourRoleLabels",
    roleLabels: ListMapData<PlayerIndex, Role> 
} | {
    type: "yourPlayerTags",
    playerTags: ListMapData<PlayerIndex, Tag[]> 
} | {
    type: "yourWill",
    will: string
} | {
    type: "yourNotes",
    notes: string[]
} | {
    type: "yourCrossedOutOutlines",
    crossedOutOutlines: number[]
} | {
    type: "yourDeathNote", 
    deathNote: string | null
} | {
    type: "yourRoleState",
    roleState: RoleState
} | {
    type: "yourJudgement",
    verdict: Verdict
} | {
    type: "yourVoteFastForwardPhase",
    fastForward: boolean
} | {
    type: "addChatMessages",
    chatMessages: ChatMessage[]
} | {
    type: "nightMessages",
    chatMessages: ChatMessage[]
} | {
    type: "addGrave",
    grave: Grave
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
    type: "setEnabledModifiers",
    modifiers: ModifierType[]
} |
// Game
{
    type: "judgement", 
    verdict: Verdict
} | {
    type: "sendChatMessage", 
    text: string,
    block: boolean,
} | {
    type: "sendWhisper", 
    playerIndex: PlayerIndex, 
    text: string
} | {
    type: "saveWill", 
    will: string
} | {
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
    type: "abilityInput",
    abilityInput: AbilityInput
} | {
    type: "setKiraGuess",
    guesses: [PlayerIndex, KiraGuess][]
} | {
    type: "setDoomsayerGuess",
    guesses: [
        [number, DoomsayerGuess],
        [number, DoomsayerGuess],
        [number, DoomsayerGuess]
    ]
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
    fastForward: boolean
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