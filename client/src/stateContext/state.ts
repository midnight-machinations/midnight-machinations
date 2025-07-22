export type ClientObject = {
    clientType: ClientObjectType,
    connection: ClientConnection,
    host: boolean,
    playerIndex: number | null,
    ready: "host" | "ready" | "notReady",
};
export type ClientObjectType = {
    type: "spectator",
} | {
    type: "player",
    name: string,
};
export type ClientConnection = "connected" | "disconnected" | "couldReconnect";