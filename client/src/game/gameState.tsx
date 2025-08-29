import ListMap from "../ListMap"
import GameState, { LobbyClient, LobbyState, PhaseTimes, Player, LobbyClientID, PlayerGameState, UnsafeString } from "./gameState.d"
import { ModifierID, ModifierState } from "./modifiers"


export function defaultPhaseTimes(): PhaseTimes {
    return {
        briefing: 45,
        obituary: 60,
        discussion: 120,
        nomination: 120,
        testimony: 30,
        judgement: 60,
        finalWords: 30,
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
        
        roleState: { type: "detective" },

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


