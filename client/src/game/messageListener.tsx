
import { createPlayer } from "./gameState";
import { ANCHOR_CONTROLLER, chatMessageToAudio } from "./../menu/Anchor";
import GAME_MANAGER from "./../index";
import GameScreen from "./../menu/game/GameScreen";
import { ToClientPacket } from "./packet";
import { GameClient, PlayerIndex, Tag } from "./gameState.d";
import { Role } from "./roleState.d";
import translate from "./lang";
import { computePlayerKeywordData, computePlayerKeywordDataForLobby, computeRoleListKeywordData } from "../components/StyledText";
import { deleteReconnectData, loadSettingsParsed, saveReconnectData } from "./localStorage";
import { WikiArticleLink } from "../components/WikiArticleLink";
import React from "react";
import WikiArticle from "../components/WikiArticle";
import SpectatorGameScreen from "../menu/spectator/SpectatorGameScreen";
import LobbyMenu from "../menu/lobby/LobbyMenu";
import LoadingScreen from "../menu/LoadingScreen";
import AudioController from "../menu/AudioController";
import NightMessagePopup from "../components/NightMessagePopup";
import PlayMenu from "../menu/main/PlayMenu";
import StartMenu from "../menu/main/StartMenu";
import ListMap from "../ListMap";
import { controllerIdToLinkWithPlayer, sortControllerIdCompare } from "./controllerInput";
import { getNamesForPlayerPoolFromLobbyClients } from "../components/gameModeSettings/OutlineSelector";

function sendDefaultName() {
    const defaultName = loadSettingsParsed().defaultName;
    if(defaultName !== null && defaultName !== undefined && defaultName !== ""){
        GAME_MANAGER.sendSetNamePacket(defaultName as string)
    }
} 

export default function messageListener(packet: ToClientPacket){
    console.log(JSON.stringify(packet, null, 2));

    switch(packet.type) {
        case "pong":
            if (GAME_MANAGER.state.stateType !== "disconnected") {
                GAME_MANAGER.server.sendPacket({
                    type: "ping"
                });
            }
        break;
        case "rateLimitExceeded":
            ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rateLimitExceeded"), body: "" });
        break;
        case "forcedOutsideLobby":
            GAME_MANAGER.setOutsideLobbyState();
            ANCHOR_CONTROLLER?.setContent(<PlayMenu/>);
        break;
        case "forcedDisconnect":
            GAME_MANAGER.setDisconnectedState();
            ANCHOR_CONTROLLER?.setContent(<StartMenu/>);
        break
        case "lobbyList":
            if(GAME_MANAGER.state.stateType === "outsideLobby"){
                GAME_MANAGER.state.lobbies = new Map();

                for(let [lobbyId, lobbyData] of Object.entries(packet.lobbies))
                    GAME_MANAGER.state.lobbies.set(Number.parseInt(lobbyId), lobbyData);
            }
        break;
        case "acceptJoin":
            if(packet.inGame && packet.spectator){
                GAME_MANAGER.setSpectatorGameState();
                ANCHOR_CONTROLLER?.setContent(<LoadingScreen type="join" />)
            }else if(packet.inGame && !packet.spectator){
                GAME_MANAGER.setGameState();
                ANCHOR_CONTROLLER?.setContent(<LoadingScreen type="join" />)
            }else{
                GAME_MANAGER.setLobbyState();
                ANCHOR_CONTROLLER?.setContent(<LobbyMenu/>);
            }
            

            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game"){
                GAME_MANAGER.state.roomCode = packet.roomCode;
                GAME_MANAGER.state.myId = packet.playerId;
            }

            saveReconnectData(packet.roomCode, packet.playerId);
            sendDefaultName();
            ANCHOR_CONTROLLER?.clearCoverCard();
        break;
        case "rejectJoin":
            switch(packet.reason) {
                case "roomDoesntExist":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.roomDoesntExist") });
                    // If the room doesn't exist, don't suggest the user to reconnect to it.
                    deleteReconnectData();
                    ANCHOR_CONTROLLER?.clearCoverCard();
                break;
                case "gameAlreadyStarted":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.gameAlreadyStarted") });
                break;
                case "roomFull":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.roomFull") });
                break;
                case "serverBusy":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.serverBusy") });
                break;
                case "playerTaken":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.playerTaken") });
                break;
                case "playerDoesntExist":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.playerDoesntExist") });
                break;
                default:
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectJoin"), body: `${packet.type} message response not implemented: ${packet.reason}` });
                    console.error(`${packet.type} message response not implemented: ${packet.reason}`);
                    console.error(packet);
                break;
            }
            deleteReconnectData();
            
        break;
        case "rejectStart":
            switch(packet.reason) {
                case "gameEndsInstantly":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.gameEndsInstantly") });
                break;
                case "roleListTooSmall":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.roleListTooSmall") });
                break;
                case "roleListCannotCreateRoles":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.roleListCannotCreateRoles") });
                break;
                case "zeroTimeGame":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.zeroTimeGame") });
                break;
                case "tooManyCLients":
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.tooManyClients") });
                break;
                default:
                    ANCHOR_CONTROLLER?.pushErrorCard({ title: translate("notification.rejectStart"), body: "" });
                    console.error(`${packet.type} message response not implemented: ${packet.reason}`);
                    console.error(packet);
                break;
            }
        break;
        case "playersHost":
            if(GAME_MANAGER.state.stateType === "lobby"){
                for(let [playerId, player] of GAME_MANAGER.state.players.entries()){
                    if (packet.hosts.includes(playerId)) {
                        player.ready = "host";
                    } else {
                        player.ready = player.ready === "host" ? "ready" : player.ready
                    }
                }
                GAME_MANAGER.state.players = new ListMap(GAME_MANAGER.state.players.entries());
            }else if(GAME_MANAGER.state.stateType === "game"){
                if (packet.hosts.includes(GAME_MANAGER.state.myId ?? -1)) {
                    if (GAME_MANAGER.state.host === null) {
                        GAME_MANAGER.state.host = {
                            clients: new ListMap()
                        }
                    }

                    for (const [id, client] of GAME_MANAGER.state.host.clients.entries()) {
                        client.host = packet.hosts.includes(id);
                    }
                } else {
                    GAME_MANAGER.state.host = null
                }
            }
        break;
        case "playersReady":
            if(GAME_MANAGER.state.stateType === "lobby"){
                for(let [playerId, player] of GAME_MANAGER.state.players.entries()){
                    if (packet.ready.includes(playerId)) {
                        player.ready = "ready";
                    } else {
                        player.ready = player.ready === "host" ? "host" : "notReady"
                    }
                }
                GAME_MANAGER.state.players = new ListMap(GAME_MANAGER.state.players.entries());
            }
        break;
        case "playersLostConnection":
            if(GAME_MANAGER.state.stateType === "lobby"){
                for(let [playerId, player] of GAME_MANAGER.state.players.entries()){
                    if(packet.lostConnection.includes(playerId))
                        player.connection = "couldReconnect";
                }
                GAME_MANAGER.state.players = new ListMap(GAME_MANAGER.state.players.entries());
            }
        break;
        /*
        In Lobby/Game 
        */
        case "yourId":
            if(GAME_MANAGER.state.stateType === "lobby")
                GAME_MANAGER.state.myId = packet.playerId;
        break;
        case "yourPlayerIndex":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.myIndex = packet.playerIndex;

            //TODO jack Im sorry
            AudioController.clearQueue();
        break;
        case "yourFellowInsiders":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.fellowInsiders = packet.fellowInsiders;
        break;
        case "lobbyClients":
            if(GAME_MANAGER.state.stateType === "lobby"){
                const oldMySpectator = GAME_MANAGER.state.players.get(GAME_MANAGER.state.myId!)?.clientType.type === "spectator";

                GAME_MANAGER.state.players = new ListMap();
                for(const [clientId, lobbyClient] of packet.clients){
                    GAME_MANAGER.state.players.insert(clientId, lobbyClient);
                }
                const newMySpectator = GAME_MANAGER.state.players.get(GAME_MANAGER.state.myId!)?.clientType.type === "spectator";

                
                if (oldMySpectator && !newMySpectator){
                    sendDefaultName();
                }

                // Recompute keyword data, since player names are keywords.
                computePlayerKeywordDataForLobby(
                    Array.from(GAME_MANAGER.state.players.values())
                        .filter(client => client.clientType.type === "player")
                        .map(client => (client.clientType as { type: "player", name: string }).name)
                );
                // Recompute keyword data, since role list entries are keywords.
                const names = getNamesForPlayerPoolFromLobbyClients(GAME_MANAGER.state.players)
                computeRoleListKeywordData(names, GAME_MANAGER.state.roleList);
            }
        break;
        case "hostData":
            if (GAME_MANAGER.state.stateType === "game") {
                GAME_MANAGER.state.host = {
                    clients: new ListMap<number, GameClient>(packet.clients)
                }
            } 
        break;
        case "lobbyName":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game"){
                GAME_MANAGER.state.lobbyName = packet.name;
            }
        break;
        case "startGame": 
            if (GAME_MANAGER.state.stateType === "lobby") {
                const isSpectator = GAME_MANAGER.state.players.get(GAME_MANAGER.state.myId!)?.clientType.type === "spectator";
                if(isSpectator){
                    GAME_MANAGER.setSpectatorGameState();
                    ANCHOR_CONTROLLER?.setContent(<LoadingScreen type="join" />)
                }else{
                    GAME_MANAGER.setGameState();
                    ANCHOR_CONTROLLER?.setContent(<LoadingScreen type="join" />)
                }
    
                AudioController.queueFile("audio/start_game.mp3");
            }
            break;
        case "gameInitializationComplete":
            if (GAME_MANAGER.state.stateType === "game") {
                const isSpectator = GAME_MANAGER.state.clientState.type === "spectator";
                GAME_MANAGER.state.initialized = true;
                if(isSpectator){
                    ANCHOR_CONTROLLER?.setContent(<SpectatorGameScreen/>);
                }else{
                    ANCHOR_CONTROLLER?.setContent(<GameScreen/>);
                }
            }
            break;
        case "backToLobby":
            GAME_MANAGER.setLobbyState();
            ANCHOR_CONTROLLER?.setContent(<LobbyMenu/>);
        break;
        case "gamePlayers":
            if(GAME_MANAGER.state.stateType === "game"){
                //only update the playerlist with the new one if there are any differences
                let playersChanged = false;
                if(GAME_MANAGER.state.players.length !== packet.players.length)
                    playersChanged = true;
                else{
                    for(let i = 0; i < packet.players.length; i++){
                        if(GAME_MANAGER.state.players[i].name !== packet.players[i]){
                            playersChanged = true;
                            break;
                        }
                    }
                }
                if(playersChanged){
                    GAME_MANAGER.state.players = [];
                    for(let i = 0; i < packet.players.length; i++){
                        GAME_MANAGER.state.players.push(createPlayer(packet.players[i], i));
                    }
                }

                // Recompute keyword data, since player names are keywords.
                computePlayerKeywordData(GAME_MANAGER.state.players);
                // Recompute keyword data, since role list entries are keywords.
                computeRoleListKeywordData(GAME_MANAGER.state.players.map(p=>p.toString()), GAME_MANAGER.state.roleList);
            }
        break;
        case "roleList":
            //list of role list entriy
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game") {
                GAME_MANAGER.state.roleList = packet.roleList;

                // Recompute keyword data, since role list entries are keywords.
                if(GAME_MANAGER.state.stateType === "game") {
                    computeRoleListKeywordData(GAME_MANAGER.state.players.map(p=>p.toString()), GAME_MANAGER.state.roleList);
                } else {
                    const names = getNamesForPlayerPoolFromLobbyClients(GAME_MANAGER.state.players)
                    computeRoleListKeywordData(names, GAME_MANAGER.state.roleList);
                }
            }
        break;
        case "roleOutline":
            //role list entriy
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game") {
                GAME_MANAGER.state.roleList = structuredClone(GAME_MANAGER.state.roleList);
                GAME_MANAGER.state.roleList[packet.index] = packet.roleOutline;
                GAME_MANAGER.state.roleList = [...GAME_MANAGER.state.roleList];

                // Recompute keyword data, since role list entries are keywords.
                if(GAME_MANAGER.state.stateType === "game") {
                    computeRoleListKeywordData(GAME_MANAGER.state.players.map(p=>p.toString()), GAME_MANAGER.state.roleList);
                } else {
                    const names = getNamesForPlayerPoolFromLobbyClients(GAME_MANAGER.state.players)
                    computeRoleListKeywordData(names, GAME_MANAGER.state.roleList);
                }
            }
        break;
        case "phaseTime":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game") {
                GAME_MANAGER.state.phaseTimes[packet.phase.type] = packet.time;
                GAME_MANAGER.state.phaseTimes = {...GAME_MANAGER.state.phaseTimes};
            }
        break;
        case "phaseTimes":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.phaseTimes = packet.phaseTimeSettings;
        break;
        case "enabledRoles":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.enabledRoles = packet.roles;
        break;
        case "enabledModifiers":
            if(GAME_MANAGER.state.stateType === "lobby" || GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.enabledModifiers = packet.modifiers;
        break;
        case "phase":
            if(GAME_MANAGER.state.stateType === "game"){
                GAME_MANAGER.state.phaseState = packet.phase;
                GAME_MANAGER.state.dayNumber = packet.dayNumber;
        
                if(packet.phase.type === "briefing" && GAME_MANAGER.state.clientState.type === "player"){
                    const role = GAME_MANAGER.state.clientState.roleState?.type;
                    if(role !== undefined){
                        ANCHOR_CONTROLLER?.setCoverCard(<WikiArticle article={"role/"+role as WikiArticleLink}/>);
                    }
                }
            }
        break;
        case "phaseTimeLeft":
            if(GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.timeLeftMs = packet.secondsLeft!==null?(packet.secondsLeft * 1000):null;
        break;
        case "playerAlive":
            if(GAME_MANAGER.state.stateType === "game"){
                for(let i = 0; i < GAME_MANAGER.state.players.length && i < packet.alive.length; i++){
                    GAME_MANAGER.state.players[i].alive = packet.alive[i];
                }
                GAME_MANAGER.state.players = [...GAME_MANAGER.state.players];
            }
        break;
        case "playerVotes":
            if(GAME_MANAGER.state.stateType === "game"){

                let listMapVotes = new ListMap<PlayerIndex, number>(packet.votesForPlayer);

                for(let i = 0; i < GAME_MANAGER.state.players.length; i++){
                    let numVoted = listMapVotes.get(i)??0;
                    GAME_MANAGER.state.players[i].numVoted = numVoted;
                }
                GAME_MANAGER.state.players = [...GAME_MANAGER.state.players];
            }
        break;
        case "yourSendChatGroups":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.sendChatGroups = [...packet.sendChatGroups];
            }
        break;
        case "yourInsiderGroups":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.insiderGroups = [...packet.insiderGroups];
            }
        break;
        case "yourAllowedControllers":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.savedControllers = 
                    packet.save.sort((a, b) => sortControllerIdCompare(a[0],b[0]));
            }
        break;
        case "yourAllowedController":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                let savedControllers = new ListMap(GAME_MANAGER.state.clientState.savedControllers,
                    (k1,k2)=>controllerIdToLinkWithPlayer(k1)===controllerIdToLinkWithPlayer(k2)
                );
                if(packet.controller===null){
                    savedControllers.delete(packet.id);
                }else{
                    savedControllers.insert(packet.id, packet.controller);
                }
                GAME_MANAGER.state.clientState.savedControllers = [...savedControllers.entries().sort((a, b) => sortControllerIdCompare(a[0],b[0]))]
            }
        break;
        case "yourRoleLabels":
            if(GAME_MANAGER.state.stateType === "game"){
                for (const player of GAME_MANAGER.state.players) {
                    player.roleLabel = null;
                }
                for (const [key, value] of packet.roleLabels) { 
                    if(
                        GAME_MANAGER.state.players !== undefined && 
                        GAME_MANAGER.state.players[key] !== undefined
                    )
                        GAME_MANAGER.state.players[key].roleLabel = value as Role;
                }
                GAME_MANAGER.state.players = [...GAME_MANAGER.state.players];
            }
        break;
        case "yourPlayerTags":
            if(GAME_MANAGER.state.stateType === "game"){
                for(let i = 0; i < GAME_MANAGER.state.players.length; i++){
                    GAME_MANAGER.state.players[i].playerTags = [];
                }

                for(const [key, value] of packet.playerTags){
                    if(
                        GAME_MANAGER.state.players !== undefined && 
                        GAME_MANAGER.state.players[key] !== undefined
                    )
                        GAME_MANAGER.state.players[key].playerTags = value as Tag[];
                }
                GAME_MANAGER.state.players = [...GAME_MANAGER.state.players];
            }
        break;
        case "yourNotes":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.notes = packet.notes;
            }
        break;
        case "yourCrossedOutOutlines":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.crossedOutOutlines = packet.crossedOutOutlines;
        break;
        case "yourDeathNote":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player")
                GAME_MANAGER.state.clientState.deathNote = packet.deathNote ?? "";
        break;
        case "yourRoleState":
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player"){
                GAME_MANAGER.state.clientState.roleState = packet.roleState;
            }
        break;
        case "yourVoteFastForwardPhase":
            if(GAME_MANAGER.state.stateType === "game")
                GAME_MANAGER.state.fastForward = packet.fastForward;
        break;
        case "addChatMessages":
            if(GAME_MANAGER.state.stateType === "game" || GAME_MANAGER.state.stateType === "lobby"){
                GAME_MANAGER.state.chatMessages = new ListMap(
                    GAME_MANAGER.state.chatMessages.entries().concat(packet.chatMessages)
                );

                // Chat notification icon state
                if(GAME_MANAGER.state.stateType === "game" && packet.chatMessages.length !== 0){
                    GAME_MANAGER.state.missedChatMessages = true;
                    
                    // eslint-disable-next-line
                    for(let [_index, chatMessage] of packet.chatMessages){
                        if(
                            chatMessage.variant.type === "whisper" &&
                            GAME_MANAGER.state.clientState.type === "player" &&
                            chatMessage.variant.toPlayerIndex === GAME_MANAGER.state.clientState.myIndex
                        ){
                            GAME_MANAGER.state.clientState.missedWhispers.push(chatMessage.variant.fromPlayerIndex);
                        }
                    }
                }

                if (GAME_MANAGER.state.stateType !== "game" || GAME_MANAGER.state.initialized === true) {
                    // eslint-disable-next-line
                    for(let [_index, chatMessage] of packet.chatMessages){
                        let audioSrc = chatMessageToAudio(chatMessage);
                        if(audioSrc)
                            AudioController.queueFile(audioSrc);
                    }
                }
            }
        break;
        case "nightMessages":
            if(GAME_MANAGER.state.stateType === "game" || GAME_MANAGER.state.stateType === "lobby"){

                if(ANCHOR_CONTROLLER?.getCoverCard()===null && packet.chatMessages.length!==0){
                    ANCHOR_CONTROLLER?.setCoverCard(<NightMessagePopup messages={packet.chatMessages}/>)
                }
            }
        break;
        case "addGrave":
            if(GAME_MANAGER.state.stateType === "game"){
                GAME_MANAGER.state.graves.insert(packet.graveRef, packet.grave);
                GAME_MANAGER.state.graves = new ListMap(
                    [...GAME_MANAGER.state.graves
                        .entries()
                        .sort(([a,_a],[b,_b])=>a-b)
                    ]
                );
            }
        break;
        case "gameOver":
            if(GAME_MANAGER.state.stateType === "game"){
                GAME_MANAGER.state.ticking = false;
                switch(packet.reason) {
                    case "reachedMaxDay":
                    case "draw":
                        console.log("Game ended! (naturally)");
                    break;
                    default:
                        // alert("Game ended for an unknown reason!");
                        console.error(`${packet.type} message response not implemented: ${packet.reason}`);
                        console.error(packet);
                    break;
                }
            }
        break;
        default:
            console.error(`incoming message response not implemented: ${(packet as any)?.type}`);
            console.error(packet);
        break;
    }

    GAME_MANAGER.invokeStateListeners(packet.type)
}


