import ListMap from "../ListMap"
import GameState, { LobbyClient, LobbyState, PhaseTimes, Player, LobbyClientID, PlayerGameState, UnsafeString } from "./gameState.d"
import { ModifierID, ModifierState } from "./modifiers"
import { Role, RoleState } from "./roleState.d"


export function defaultPhaseTimes(): PhaseTimes {
    return {
        briefing: 45,
        obituary: 20,
        discussion: 100,
        nomination: 100,
        adjournment: 60,
        testimony: 30,
        judgement: 30,
        finalWords: 10,
        dusk: 30,
        night: 60,
    }
}

export function createLobbyState(): LobbyState {
    return {
        stateType: "lobby",
        roomCode: 0,
        lobbyName: "Mafia Lobby",

        myId: null,

        roleList: [],
        phaseTimes: defaultPhaseTimes(),
        enabledRoles: [],
        modifierSettings: new ListMap<ModifierID, ModifierState>(),

        players: new ListMap<LobbyClientID, LobbyClient>(),
        chatMessages: new ListMap(),
    }
}

export function createGameState(): GameState {
    return {
        stateType: "game",
        roomCode: 0,
        lobbyName: "",

        initialized: false,

        myId: null,

        chatMessages : new ListMap(),
        graves: new ListMap(),
        players: [],
        
        phaseState: {type:"briefing"},
        timeLeftMs: 0,
        dayNumber: 1,


        chatFilter: null,
        fastForward: {type:"none"},
        
        roleList: [],
        enabledRoles: [],
        phaseTimes: defaultPhaseTimes(),
        modifierSettings: new ListMap<ModifierID, ModifierState>(),

        ticking: true,

        clientState: createPlayerGameState(),
        host: null,

        missedChatMessages: false
    }
}

export function createPlayerGameState(): PlayerGameState {
    return {
        type: "player",

        myIndex: 0,
        
        myRole: "detective" as Role,
        roleState: { type: "detective" },
        roleStates: new ListMap<Role, RoleState>(),

        savedControllers: [],

        notes: [],
        crossedOutOutlines: [],
        deathNote: "",

        fellowInsiders: [],

        sendChatGroups: [],
        insiderGroups: [],

        missedWhispers: []
    }
}

export function createPlayer(name: UnsafeString, index: number): Player {
    return{
        name: name,
        index: index,
        numVoted: 0,
        alive: true,
        roleLabel: null,
        playerTags: [],

        toString() {
            return ""+(this.index+1)+": " + this.name;
        }
    }
}


