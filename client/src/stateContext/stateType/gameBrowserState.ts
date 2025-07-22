import { LobbyClientID } from "./otherState"

export type Disconnected = {
    type: "disconnected"
}

export type GameBrowserState = {
    type: "gameBrowser",

    selectedRoomCode: string | null,
    lobbies: Map<number, LobbyPreviewData>,
}

export type LobbyPreviewData = {
    name: string,
    inGame : boolean,
    players: [LobbyClientID, string][]
}

export function createGameBrowserState(): GameBrowserState{
    return {
        type: "gameBrowser",
        selectedRoomCode: null,
        lobbies: new Map<number, LobbyPreviewData>(),
    }
}