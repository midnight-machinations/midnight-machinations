import { ANCHOR_CONTROLLER } from "./../menu/Anchor";
import StartMenu from "./../menu/main/StartMenu";
import GAME_MANAGER from "./../index";
import messageListener from "./messageListener";
import React from "react";
import { PhaseType, PhaseTimes, Verdict, PlayerIndex } from "./gameState.d";
import { GameManager, Server, StateListener } from "./gameManager.d";
import { LobbyPreviewData, ToClientPacket, ToServerPacket } from "./packet";
import { RoleOutline } from "./roleListState.d";
import translate from "./lang";
import PlayMenu from "../menu/main/PlayMenu";
import { createGameState, createLobbyState } from "./gameState";
import { deleteReconnectData } from "./localStorage";
import AudioController from "../menu/AudioController";
import ListMap from "../ListMap";
import { defaultAlibi } from "../menu/game/gameScreenContent/WillMenu";

export function createGameManager(): GameManager {

    console.log("Game manager created.");
    
    let gameManager: GameManager = {
        async setDisconnectedState(): Promise<void> {
            AudioController.clearQueue();
            AudioController.pauseQueue();

            if (GAME_MANAGER.server.ws) {
                let completePromise: () => void;
                const promise = new Promise<void>((resolver) => {
                    completePromise = resolver;
                });

                GAME_MANAGER.server.ws?.addEventListener("close", () => completePromise());
                GAME_MANAGER.server.close();

                GAME_MANAGER.state = {
                    stateType: "disconnected"
                };
                return promise;
            } else {
                GAME_MANAGER.state = {
                    stateType: "disconnected"
                };
                return Promise.resolve();
            }
        },
        setLobbyState() {
            
            let gameState = null
            if (GAME_MANAGER.state.stateType === "game") {
                gameState = {...GAME_MANAGER.state};
            }

            GAME_MANAGER.state = createLobbyState();

            if(gameState!=null){
                GAME_MANAGER.state.roomCode = gameState.roomCode;
                GAME_MANAGER.state.lobbyName = gameState.lobbyName;
                GAME_MANAGER.state.roleList = gameState.roleList;
                GAME_MANAGER.state.phaseTimes = gameState.phaseTimes;
                GAME_MANAGER.state.enabledRoles = gameState.enabledRoles;
            }
        },
        setGameState() {

            let lobbyState = null;
            if (GAME_MANAGER.state.stateType === "lobby") {
                lobbyState = {...GAME_MANAGER.state};
            }


            AudioController.clearQueue();
            AudioController.unpauseQueue();
            GAME_MANAGER.state = createGameState();
            if (lobbyState !== null && GAME_MANAGER.state.stateType === "game") {
                GAME_MANAGER.state.roomCode = lobbyState.roomCode;
                GAME_MANAGER.state.lobbyName = lobbyState.lobbyName;
                GAME_MANAGER.state.roleList = lobbyState.roleList;
                GAME_MANAGER.state.phaseTimes = lobbyState.phaseTimes;
                GAME_MANAGER.state.enabledRoles = lobbyState.enabledRoles;
                if (lobbyState.players.get(lobbyState.myId!)?.ready === "host") {
                    GAME_MANAGER.state.host = {
                        clients: new ListMap()
                    };
                }
                GAME_MANAGER.state.myId = lobbyState.myId
            }
        },
        setSpectatorGameState() {
            this.setGameState();
            if(GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.clientState = {
                    type: "spectator"
                };
        },
        async setOutsideLobbyState() {
            AudioController.clearQueue();
            AudioController.pauseQueue();
            
            if (!GAME_MANAGER.server.ws?.OPEN && !await GAME_MANAGER.server.open()) {
                await this.setDisconnectedState();
                return false;
            }

            GAME_MANAGER.state = {
                stateType: "outsideLobby",
                selectedRoomCode: null,
                lobbies: new Map<number, LobbyPreviewData>()
            };

            return true;
        },

        state: {
            stateType: "disconnected"
        },

        updateChatFilter(filter: PlayerIndex | null) {
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.chatFilter = filter===null?null:{
                    type: "playerNameInMessage",
                    player: filter
                };
                GAME_MANAGER.invokeStateListeners("filterUpdate");
            }
        },


        server: createServer(),

        listeners: [],

        addStateListener(listener) {
            gameManager.listeners.push(listener);
        },
        removeStateListener(listener) {
            let index = gameManager.listeners.indexOf(listener);
            if (index !== -1)
                gameManager.listeners.splice(index, 1);
        },
        invokeStateListeners(type) {
            for (let i = 0; i < gameManager.listeners.length; i++) {
                if (typeof (gameManager.listeners[i]) === "function") {
                    gameManager.listeners[i](type);
                }
            }
        },

        setPrependWhisperFunction: (f) => {
            gameManager.prependWhisper = f;
        },
        prependWhisper: (index) => {},
        
        wikiArticleCallbacks: [],
        addSetWikiArticleCallback: (callback) => {
            gameManager.wikiArticleCallbacks.push(callback);
        },
        removeSetWikiArticleCallback: (callback) => {
            gameManager.wikiArticleCallbacks.splice(gameManager.wikiArticleCallbacks.indexOf(callback), 1)
        },
        setWikiArticle: (article) => {
            for (const callback of gameManager.wikiArticleCallbacks) {
                callback(article);
            }
        },


        leaveGame() {
            if (this.state.stateType !== "disconnected") {
                this.server.sendPacket({ type: "leave" });
            }
            deleteReconnectData();
            this.setDisconnectedState();
            ANCHOR_CONTROLLER?.setContent(<PlayMenu/>);
        },

        sendLobbyListRequest() {
            this.server.sendPacket({ type: "lobbyListRequest" });
        },
        sendHostPacket() {
            let completePromise: (success: boolean) => void;
            const promise = new Promise<boolean>((resolver) => {
                completePromise = resolver;
            });
            let onJoined: StateListener = (type) => {
                if (type === "acceptJoin") {
                    completePromise(true);
                    GAME_MANAGER.removeStateListener(onJoined);
                } else if (type === "rejectJoin") {
                    completePromise(false);
                    GAME_MANAGER.removeStateListener(onJoined);
                }
            };
            GAME_MANAGER.addStateListener(onJoined);
            this.server.sendPacket({ type: "host" });

            return promise;
        },
        sendRejoinPacket(roomCode: number, playerId: number) {
            let completePromise: (success: boolean) => void;
            const promise = new Promise<boolean>((resolver) => {
                completePromise = resolver;
            });
            const onJoined: StateListener = (type) => {
                if (type === "acceptJoin") {
                    completePromise(true);
                    GAME_MANAGER.removeStateListener(onJoined);
                } else if (type === "rejectJoin") {
                    completePromise(false);
                    GAME_MANAGER.removeStateListener(onJoined);
                } else if (type === "connectionClosed") {
                    completePromise(false);
                    GAME_MANAGER.removeStateListener(onJoined);
                }
            };
            GAME_MANAGER.addStateListener(onJoined);

            this.server.sendPacket({
                type: "reJoin",
                roomCode,
                playerId
            });


            return promise;
        },
        sendJoinPacket(roomCode: number) {
            let completePromise: (success: boolean) => void;
            const promise = new Promise<boolean>((resolver) => {
                completePromise = resolver;
            });
            const onJoined: StateListener = (type) => {
                if (type === "acceptJoin") {
                    completePromise(true);
                    GAME_MANAGER.removeStateListener(onJoined);
                } else if (type === "rejectJoin") {
                    completePromise(false);
                    GAME_MANAGER.removeStateListener(onJoined);
                } else if (type === "connectionClosed") {
                    completePromise(false);
                    GAME_MANAGER.removeStateListener(onJoined);
                }
            };
            GAME_MANAGER.addStateListener(onJoined);

            this.server.sendPacket({
                type: "join",
                roomCode
            });

            return promise;
        },
        sendKickPlayerPacket(playerId: number) {
            this.server.sendPacket({
                type: "kick",
                playerId: playerId
            });
        },
        sendSetPlayerHostPacket(playerId: number) {
            this.server.sendPacket({
                type: "setPlayerHost",
                playerId: playerId
            });
        },
        sendRelinquishHostPacket() {
            this.server.sendPacket({
                type: "relinquishHost",
            });
        },

        sendSetSpectatorPacket(spectator) {
            this.server.sendPacket({
                type: "setSpectator",
                spectator: spectator
            });
        },

        sendSetNamePacket(name) {
            this.server.sendPacket({
                type: "setName",
                name: name
            });
        },

        sendReadyUpPacket(ready) {
            this.server.sendPacket({
                type: "readyUp",
                ready: ready
            });
        },
        sendSendLobbyMessagePacket(text) {
            this.server.sendPacket({
                type: "sendLobbyMessage",
                text: text
            });
        },

        sendSetLobbyNamePacket(name) {
            this.server.sendPacket({
                type: "setLobbyName",
                name: name
            });
        },
        sendStartGamePacket() {
            let completePromise: (success: boolean) => void;
            let promise = new Promise<boolean>((resolver) => {
                completePromise = resolver;
            });
            let onJoined: StateListener = (type) => {
                if (type === "startGame") {
                    completePromise(true);
                    GAME_MANAGER.removeStateListener(onJoined);
                } else if (type === "rejectStart") {
                    completePromise(false);
                    GAME_MANAGER.removeStateListener(onJoined);
                }
            };
            GAME_MANAGER.addStateListener(onJoined);

            this.server.sendPacket({
                type: "startGame"
            });

            return promise;
        },
        sendBackToLobbyPacket() {
            this.server.sendPacket({
                type: "hostForceBackToLobby"
            });
        },
        sendSetPhaseTimePacket(phase: PhaseType, time: number) {
            if (isValidPhaseTime(time)) {
                this.server.sendPacket({
                    type: "setPhaseTime",
                    phase: phase,
                    time: time
                });
            }
        },
        sendSetPhaseTimesPacket(phaseTimeSettings: PhaseTimes) {
            this.server.sendPacket({
                type: "setPhaseTimes",
                phaseTimeSettings
            });
        },
        sendSetRoleListPacket(roleListEntries: RoleOutline[]) {
            this.server.sendPacket({
                type: "setRoleList",
                roleList: roleListEntries
            });
        },
        sendSetRoleOutlinePacket(index: number, roleOutline: RoleOutline) {
            this.server.sendPacket({
                type: "setRoleOutline",
                index,
                roleOutline
            });
        },
        sendSimplifyRoleListPacket() {
            this.server.sendPacket({
                type: "simplifyRoleList"
            });
        },

        sendJudgementPacket(judgement: Verdict) {
            this.server.sendPacket({
                type: "judgement",
                verdict: judgement
            });
        },

        sendSaveWillPacket(will) {
            if(will === ""){
                will = defaultAlibi();
            }

            let player = undefined;
            // if(player===undefined){
                if(this.state.stateType==="game" && this.state.clientState.type === "player"){
                    player = this.state.clientState.myIndex;
                }
            // }
            if(player===undefined){return}

            this.sendAbilityInput({
                id: {
                    type: "alibi",
                    player: player
                }, 
                selection: {
                    type: "string",
                    selection: will
                }
            });
        },
        sendSaveNotesPacket(notes) {
            this.server.sendPacket({
                type: "saveNotes",
                notes: notes
            });
        },
        sendSaveCrossedOutOutlinesPacket(crossedOutOutlines) {
            this.server.sendPacket({
                type: "saveCrossedOutOutlines",
                crossedOutOutlines: crossedOutOutlines
            });
        },
        sendSaveDeathNotePacket(notes) {
            this.server.sendPacket({
                type: "saveDeathNote",
                deathNote: notes.trim().length === 0 ? null : notes
            });
        },
        sendSendChatMessagePacket(text, block) {
            let player = undefined;
            // if(player===undefined){
                if(this.state.stateType==="game" && this.state.clientState.type === "player"){
                    player = this.state.clientState.myIndex;
                }
            // }
            if(player===undefined){return}

            this.sendAbilityInput({
                id: {
                    type: "chatIsBlock",
                    player: player
                }, 
                selection: {
                    type: "boolean",
                    selection: block
                }
            });

            this.sendAbilityInput({
                id: {
                    type: "chat",
                    player: player
                }, 
                selection: {
                    type: "string",
                    selection: text
                }
            });

            
            this.sendAbilityInput({
                id: {
                    type: "sendChat",
                    player: player
                }, 
                selection: {
                    type: "unit",
                    selection: null
                }
            });
        },
        sendSendWhisperPacket(whisperToPlayer, text) {
            let player = undefined;
            // if(player===undefined){
                if(this.state.stateType==="game" && this.state.clientState.type === "player"){
                    player = this.state.clientState.myIndex;
                }
            // }
            if(player===undefined){return}

            this.sendAbilityInput({
                id: {
                    type: "whisperToPlayer",
                    player: player
                }, 
                selection: {
                    type: "playerList",
                    selection: [whisperToPlayer]
                }
            });

            this.sendAbilityInput({
                id: {
                    type: "whisper",
                    player: player
                }, 
                selection: {
                    type: "string",
                    selection: text
                }
            });

            
            this.sendAbilityInput({
                id: {
                    type: "sendWhisper",
                    player: player
                }, 
                selection: {
                    type: "unit",
                    selection: null
                }
            });
        },
        sendEnabledRolesPacket(roles) {
            this.server.sendPacket({
                type: "setEnabledRoles",
                roles: roles
            });
        },
        sendEnabledModifiersPacket(modifiers) {
            this.server.sendPacket({
                type: "setEnabledModifiers",
                modifiers: modifiers
            });
        },

        sendAbilityInput(input) {
            this.server.sendPacket({
                type: "abilityInput",
                abilityInput: input
            });
        },
        sendSetDoomsayerGuess(guesses) {
            this.server.sendPacket({
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
            this.server.sendPacket({
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
            this.server.sendPacket({
                type: "voteFastForwardPhase",
                fastForward: fastForward
            });
        },

        sendHostDataRequest() {
            this.server.sendPacket({
                type: "hostDataRequest"
            })
        },
        sendHostEndGamePacket() {
            this.server.sendPacket({
                type: "hostForceEndGame"
            })
        },
        sendHostSkipPhase() {
            this.server.sendPacket({
                type: "hostForceSkipPhase"
            })
        },
        sendHostSetPlayerNamePacket(playerId, name) {
            this.server.sendPacket({
                type: "hostForceSetPlayerName",
                id: playerId,
                name
            })
        },

        messageListener(serverMessage) {
            messageListener(serverMessage);
        },

        tick(timePassedMs) {
            if (gameManager.state.stateType !== "game") {return}
            if (!gameManager.state.ticking) return;
            if(gameManager.state.timeLeftMs === null) {return}

            const newTimeLeft = gameManager.state.timeLeftMs - timePassedMs;
            if (Math.floor(newTimeLeft / 1000) < Math.floor(gameManager.state.timeLeftMs / 1000)) {
                gameManager.invokeStateListeners("tick");
            }
            gameManager.state.timeLeftMs = newTimeLeft;
            if (gameManager.state.timeLeftMs < 0) {
                gameManager.state.timeLeftMs = 0;
            }
        },
    }
    return gameManager;
}
function createServer(){

    let Server: Server = {
        ws: null,

        open : () => {
            let address = process.env.REACT_APP_WS_ADDRESS;
            if(!address){
                throw new Error("Missing env var REACT_APP_WS_ADDRES, make sure you defined it in .env");
            }
            try {
                Server.ws = new WebSocket(address);
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

            Server.ws.onopen = (event: Event)=>{
                completePromise(true);
                console.log("Connected to server.");
            };
            Server.ws.onclose = (event: CloseEvent)=>{
                console.log("Disconnected from server.");
                completePromise(false);
                GAME_MANAGER.invokeStateListeners("connectionClosed");
                if (Server.ws === null) return; // We closed it ourselves
                Server.ws = null;

                ANCHOR_CONTROLLER?.pushErrorCard({
                    title: translate("notification.connectionFailed"), 
                    body: ""
                });
                ANCHOR_CONTROLLER?.setContent(<StartMenu/>);
            };
            Server.ws.onmessage = (event: MessageEvent<string>)=>{
                GAME_MANAGER.messageListener(
                    JSON.parse(event.data) as ToClientPacket
                );
            };
            Server.ws.onerror = (event: Event) => {
                Server.close();
                completePromise(false);
                ANCHOR_CONTROLLER?.pushErrorCard({
                    title: translate("notification.connectionFailed"), 
                    body: translate("notification.serverNotFound")
                });
            };
            
            return promise;
        },

        sendPacket : (packet: ToServerPacket)=>{
            if (Server.ws === null) {
                console.error("Attempted to send packet to null websocket!");
            } else {
                Server.ws.send(JSON.stringify(packet));
            }
        },

        close : ()=>{
            if(Server.ws === null) return;
            
            Server.ws.close();
            Server.ws = null;
        }
        
    }
    return Server;
}

export function isValidPhaseTime(time: number) {
    return Number.isSafeInteger(time) && time <= 1000 && 0 <= time;
}

export type { GameManager, Server } from "./gameManager.d";
