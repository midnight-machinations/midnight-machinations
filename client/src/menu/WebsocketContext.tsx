import { createContext, ReactElement, useEffect, useMemo, useRef, useState } from "react";
import { ToClientPacket, ToServerPacket } from "../packet";
import { DoomsayerGuess } from "./game/gameScreenContent/AbilityMenu/RoleSpecificMenus/LargeDoomsayerMenu";
import { AbilityInput } from "../game/abilityInput";
import { RoleList, RoleOutline } from "../stateContext/stateType/roleListState";
import React from "react";
import { isValidPhaseTime } from "../components/gameModeSettings/PhaseTimeSelector";
import { ModifierType } from "../stateContext/stateType/modifiersState";
import { Role } from "../stateContext/stateType/roleState";
import { PhaseTimes, Verdict } from "../stateContext/stateType/otherState";
import { PhaseType } from "../stateContext/stateType/phaseState";


export function useWebsocketMessageListener(websocketContext: WebSocketContextType, listener: (packet: ToClientPacket)=>void): void{
    useEffect(()=>{
        websocketContext.addMessageListener(listener);
        return ()=>{websocketContext.removeMessageListener(listener)}
    }, []);
}



export const WebsocketContext = createContext<WebSocketContextType | undefined>(undefined);

export type WebSocketContextType = {
    webSocket: React.MutableRefObject<WebSocket | null>,
    lastMessageRecieved: ToClientPacket | null,

    open(): Promise<boolean>;
    sendPacket(packets: ToServerPacket): void;
    close(): Promise<void>;

    addMessageListener(listener: (packet: ToClientPacket) => void): void;
    removeMessageListener(listener: (packet: ToClientPacket) => void): void;
    awaitPacket<T>(listener: (packet: ToClientPacket) => T | undefined): Promise<T>;
    awaitCloseOrError(): Promise<"close" | "error">;

    sendLobbyListRequest(): void;
    sendHostPacket(): Promise<{ roomCode: number, myId: number } | null>;
    sendRejoinPacket(roomCode: number, playerId: number): Promise<boolean>;
    sendJoinPacket(roomCode: number): Promise<boolean>;
    sendKickPlayerPacket(playerId: number): void;
    sendSetPlayerHostPacket(playerId: number): void;
    sendRelinquishHostPacket(): void;
    sendSetSpectatorPacket(spectator: boolean): void;
    sendSetNamePacket(name: string): void;
    sendReadyUpPacket(ready: boolean): void;
    sendSendLobbyMessagePacket(text: string): void;
    sendSetLobbyNamePacket(name: string): void;
    sendStartGamePacket(): Promise<boolean>;
    sendBackToLobbyPacket(): void;
    sendSetPhaseTimePacket(phase: PhaseType, time: number): void;
    sendSetPhaseTimesPacket(phaseTimeSettings: PhaseTimes): void;
    sendSetRoleListPacket(roleListEntries: RoleList): void;
    sendSetRoleOutlinePacket(index: number, roleOutline: RoleOutline): void;
    sendSimplifyRoleListPacket(): void;
    
    sendJudgementPacket(judgement: Verdict): void;
    sendSaveWillPacket(will: string): void;
    sendSaveNotesPacket(notes: string[]): void;
    sendSaveCrossedOutOutlinesPacket(crossedOutOutlines: number[]): void;
    sendSaveDeathNotePacket(notes: string): void;
    sendSendChatMessagePacket(text: string, block: boolean): void;
    sendSendWhisperPacket(playerIndex: number, text: string): void;
    sendEnabledRolesPacket(roles: Role[]): void;
    sendEnabledModifiersPacket(modifiers: ModifierType[]): void;

    sendAbilityInput(input: AbilityInput): void;
    sendSetDoomsayerGuess(guesses: [
        [number, DoomsayerGuess],
        [number, DoomsayerGuess],
        [number, DoomsayerGuess]
    ]): void;
    sendSetConsortOptions(
        roleblock: boolean, 
        youWereRoleblockedMessage: boolean, 
        youSurvivedAttackMessage: boolean, 
        youWereGuardedMessage: boolean, 
        youWereTransportedMessage: boolean, 
        youWerePossessedMessage: boolean, 
        yourTargetWasJailedMessage: boolean
    ): void

    sendVoteFastForwardPhase(fastForward: boolean): void;
    sendHostDataRequest(): void;
    sendHostEndGamePacket(): void;
    sendHostSkipPhase(): void;
    sendHostSetPlayerNamePacket(player_id: number, name: string): void;
}

export default function WebSocketContextProvider(props: Readonly<{ children: React.ReactNode }>): ReactElement {
    // TODO: This is kind of dumb, let's do something else
    const messagesQueue = useRef<ToClientPacket[]>([]);
    const [lastMessageRecieved, setLastMessageRecieved] = useState<ToClientPacket | null>(null);
    const webSocket = useRef<WebSocket | null>(null);

    const messageListeners = useRef<((packet: ToClientPacket)=>void)[]>([]);

    useEffect(() => {
        const interval = setInterval(() => {
            if (messagesQueue.current.length !== 0) {
                const last = messagesQueue.current.shift()!;
                setLastMessageRecieved(last)
            }
        }, 0);
        return () => clearInterval(interval);
    })

    const websocketContext: WebSocketContextType = useMemo(() => ({
        webSocket,
        lastMessageRecieved,

        open: async () => {
            if(websocketContext.webSocket.current?.OPEN === 1){
                await websocketContext.close();
            }

            let address = process.env.REACT_APP_WS_ADDRESS;
            if(!address){
                throw new Error("Missing env var REACT_APP_WS_ADDRESS, make sure you defined it in .env");
            }
            try {
                websocketContext.webSocket.current = new WebSocket(address);
            } catch {
                return Promise.resolve(false);
            }

            let completePromise: (value: boolean) => void;
            const promise = Promise.race([
                new Promise<boolean>((resolver) => {
                    completePromise = resolver;
                }),
                new Promise<boolean>((resolver) => {
                    setTimeout(() => {
                        resolver(false)
                    }, 3000)
                })
            ]);

            websocketContext.webSocket.current.onopen = (event: Event)=>{
                completePromise(true);
                console.log("Connected to server.");
            };
            websocketContext.webSocket.current.onclose = (event: CloseEvent)=>{
                console.log("Disconnected from server.");
                completePromise(false);
                if (websocketContext.webSocket.current === null) return; // We closed it ourselves
                websocketContext.webSocket.current = null;
            };
            websocketContext.webSocket.current.onmessage = (event: MessageEvent<string>)=>{
                const parsed = JSON.parse(event.data) as ToClientPacket;
                for(let listener of messageListeners.current){
                    listener(parsed);
                }
                messagesQueue.current.push(parsed);
            };
            websocketContext.webSocket.current.onerror = (event: Event) => {
                websocketContext.close();
                completePromise(false);
            };
            
            return promise;
        },
        sendPacket: (packet: ToServerPacket)=>{
            if (websocketContext.webSocket.current === null) {
                console.error("Attempted to send packet to null websocket!");
            } else if (websocketContext.webSocket.current.OPEN !== 1) {
                console.error("Attempted to send packet to not open websocket!");
            } else {
                websocketContext.webSocket.current.send(JSON.stringify(packet));
            }
        },
        close: async () => {
            if(websocketContext.webSocket.current === null) return;

            let completePromise: (value: void) => void;
            const promise = new Promise<void>((resolve) => {
                completePromise = resolve;
            })

            websocketContext.webSocket.current.addEventListener("close", () => {
                completePromise();
            });
            
            websocketContext.webSocket.current.close();
            websocketContext.webSocket.current = null;

            return promise;
        },

        addMessageListener(listener) {
            messageListeners.current.push(listener);
        },
        removeMessageListener(listener) {
            messageListeners.current = messageListeners.current.filter((l)=>l!==listener);
        },
        awaitPacket(packetListener) {
            let completePromise: (result: any) => void;
            const promise = new Promise<any>(resolve => completePromise = resolve)

            const websocketListener = (packet: MessageEvent<string>) => {
                const result = packetListener(JSON.parse(packet.data));
                if (result !== undefined) {
                    completePromise(result);
                    websocketContext.webSocket.current!.removeEventListener("message", websocketListener);
                }
            };
            websocketContext.webSocket.current!.addEventListener("message", websocketListener);

            return promise;
        },
        awaitCloseOrError() {
            let completePromise: (result: "close" | "error") => void;
            const promise = new Promise<"close" | "error">(resolve => completePromise = resolve)

            const websocketListener = (ev: Event | CloseEvent) => {
                completePromise(ev.type as "close" | "error");
                websocketContext.webSocket.current?.removeEventListener("close", websocketListener);
                websocketContext.webSocket.current?.removeEventListener("error", websocketListener);
            };
            websocketContext.webSocket.current?.addEventListener("close", websocketListener);
            websocketContext.webSocket.current?.addEventListener("error", websocketListener);

            return promise;
        },


        sendLobbyListRequest() {
            websocketContext.sendPacket({ type: "lobbyListRequest" });
        },
        sendHostPacket() {
            const promise = websocketContext.awaitPacket(packet => {
                switch (packet.type) {
                    case "acceptJoin":
                        return { roomCode: packet.roomCode, myId: packet.playerId }
                    case "rejectJoin":
                        return null
                }
            })

            websocketContext.sendPacket({ type: "host" });

            return promise
        },
        sendRejoinPacket(roomCode: number, playerId: number) {
            const promise = websocketContext.awaitPacket(packet => {
                switch (packet.type) {
                    case "acceptJoin":
                        return true;
                    case "rejectJoin":
                        return false;
                }
            });

            websocketContext.sendPacket({
                type: "reJoin",
                roomCode,
                playerId
            });

            return promise;
        },
        sendJoinPacket(roomCode: number) {
            const promise = websocketContext.awaitPacket(packet => {
                switch (packet.type) {
                    case "acceptJoin":
                        return true;
                    case "rejectJoin":
                        return false;
                }
            });

            websocketContext.sendPacket({
                type: "join",
                roomCode
            });

            return promise;
        },
        sendKickPlayerPacket(playerId: number) {
            websocketContext.sendPacket({
                type: "kick",
                playerId: playerId
            });
        },
        sendSetPlayerHostPacket(playerId: number) {
            websocketContext.sendPacket({
                type: "setPlayerHost",
                playerId: playerId
            });
        },
        sendRelinquishHostPacket() {
            websocketContext.sendPacket({
                type: "relinquishHost",
            });
        },
    
        sendSetSpectatorPacket(spectator) {
            websocketContext.sendPacket({
                type: "setSpectator",
                spectator: spectator
            });
        },
    
        sendSetNamePacket(name) {
            websocketContext.sendPacket({
                type: "setName",
                name: name
            });
        },
    
        sendReadyUpPacket(ready) {
            websocketContext.sendPacket({
                type: "readyUp",
                ready: ready
            });
        },
        sendSendLobbyMessagePacket(text) {
            websocketContext.sendPacket({
                type: "sendLobbyMessage",
                text: text
            });
        },
    
        sendSetLobbyNamePacket(name) {
            websocketContext.sendPacket({
                type: "setLobbyName",
                name: name
            });
        },
        sendStartGamePacket() {
            const promise = websocketContext.awaitPacket(packet => {
                switch (packet.type) {
                    case "startGame":
                        return true;
                    case "rejectStart":
                        return false;
                }
            })

            websocketContext.sendPacket({
                type: "startGame"
            });

            return promise;
        },
        sendBackToLobbyPacket() {
            websocketContext.sendPacket({
                type: "hostForceBackToLobby"
            });
        },
        sendSetPhaseTimePacket(phase: PhaseType, time: number) {
            if (isValidPhaseTime(time)) {
                websocketContext.sendPacket({
                    type: "setPhaseTime",
                    phase: phase,
                    time: time
                });
            }
        },
        sendSetPhaseTimesPacket(phaseTimeSettings: PhaseTimes) {
            websocketContext.sendPacket({
                type: "setPhaseTimes",
                phaseTimeSettings
            });
        },
        sendSetRoleListPacket(roleListEntries: RoleOutline[]) {
            websocketContext.sendPacket({
                type: "setRoleList",
                roleList: roleListEntries
            });
        },
        sendSetRoleOutlinePacket(index: number, roleOutline: RoleOutline) {
            websocketContext.sendPacket({
                type: "setRoleOutline",
                index,
                roleOutline
            });
        },
        sendSimplifyRoleListPacket() {
            websocketContext.sendPacket({
                type: "simplifyRoleList"
            });
        },
    
        sendJudgementPacket(judgement: Verdict) {
            websocketContext.sendPacket({
                type: "judgement",
                verdict: judgement
            });
        },
    
        sendSaveWillPacket(will) {
            websocketContext.sendPacket({
                type: "saveWill",
                will: will
            });
        },
        sendSaveNotesPacket(notes) {
            websocketContext.sendPacket({
                type: "saveNotes",
                notes: notes
            });
        },
        sendSaveCrossedOutOutlinesPacket(crossedOutOutlines) {
            websocketContext.sendPacket({
                type: "saveCrossedOutOutlines",
                crossedOutOutlines: crossedOutOutlines
            });
        },
        sendSaveDeathNotePacket(notes) {
            websocketContext.sendPacket({
                type: "saveDeathNote",
                deathNote: notes.trim().length === 0 ? null : notes
            });
        },
        sendSendChatMessagePacket(text, block) {
            websocketContext.sendPacket({
                type: "sendChatMessage",
                text: text,
                block: block
            });
        },
        sendSendWhisperPacket(playerIndex, text) {
            websocketContext.sendPacket({
                type: "sendWhisper",
                playerIndex: playerIndex,
                text: text
            });
        },
        sendEnabledRolesPacket(roles) {
            websocketContext.sendPacket({
                type: "setEnabledRoles",
                roles: roles
            });
        },
        sendEnabledModifiersPacket(modifiers) {
            websocketContext.sendPacket({
                type: "setEnabledModifiers",
                modifiers: modifiers
            });
        },
    
        sendAbilityInput(input) {
            websocketContext.sendPacket({
                type: "abilityInput",
                abilityInput: input
            });
        },
        sendSetDoomsayerGuess(guesses) {
            websocketContext.sendPacket({
                type: "setDoomsayerGuess",
                guesses: guesses
            });
        },
        sendSetConsortOptions(
            roleblock: boolean,
            youWereRoleblockedMessage: boolean,
            youSurvivedAttackMessage: boolean,
            youWereGuardedMessage: boolean,
            youWereTransportedMessage: boolean,
            youWerePossessedMessage: boolean,
            youWereWardblockedMessage: boolean
        ): void {
            websocketContext.sendPacket({
                type: "setConsortOptions",
                roleblock: roleblock,
    
                youWereRoleblockedMessage: youWereRoleblockedMessage ?? false,
                youSurvivedAttackMessage: youSurvivedAttackMessage ?? false,
                youWereGuardedMessage: youWereGuardedMessage ?? false,
                youWereTransportedMessage: youWereTransportedMessage ?? false,
                youWerePossessedMessage: youWerePossessedMessage ?? false,
                youWereWardblockedMessage: youWereWardblockedMessage ?? false
            });
        },
    
        sendVoteFastForwardPhase(fastForward: boolean) {
            websocketContext.sendPacket({
                type: "voteFastForwardPhase",
                fastForward: fastForward
            });
        },
    
        sendHostDataRequest() {
            websocketContext.sendPacket({
                type: "hostDataRequest"
            })
        },
        sendHostEndGamePacket() {
            websocketContext.sendPacket({
                type: "hostForceEndGame"
            })
        },
        sendHostSkipPhase() {
            websocketContext.sendPacket({
                type: "hostForceSkipPhase"
            })
        },
        sendHostSetPlayerNamePacket(playerId, name) {
            websocketContext.sendPacket({
                type: "hostForceSetPlayerName",
                id: playerId,
                name
            })
        }
    }), [lastMessageRecieved]);

    useEffect(()=>{
        return ()=>{
            webSocket.current?.close();
            webSocket.current = null;
        }
        // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [])

    return <WebsocketContext.Provider value={websocketContext}>
        {props.children}
    </WebsocketContext.Provider>
}