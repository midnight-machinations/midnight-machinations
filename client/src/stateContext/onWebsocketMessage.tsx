import AudioController from "../menu/AudioController";
import { computePlayerKeywordData, computePlayerKeywordDataForLobby } from "../components/StyledText";
import { WikiArticleLink } from "../wiki/WikiArticleLink";
import { defaultAlibi } from "../menu/game/gameScreenContent/WillMenu";
import ListMap from "../ListMap";
import { WebSocketContextType } from "../menu/WebsocketContext";
import { ToClientPacket } from "../packet";
import translate from "../game/lang";
import { AppContextType } from "../menu/AppContext";
import { Tag } from "./stateType/tagState";
import { PlayerIndex } from "./stateType/otherState";
import { Role } from "./stateType/roleState";
import { chatMessageToAudio, sendDefaultName } from "../menu/App";
import { StateContext } from "./StateContext";
import { deleteReconnectData, saveReconnectData } from "../game/localStorage";
import { createPlayer, createPlayerGameState } from "./stateType/gameState";
import { sortControllerIdCompare } from "../game/abilityInput";
import NightMessagePopup from "../components/NightMessagePopup"
import WikiArticle from "../wiki/WikiArticle";
import React from "react";


export default function onWebsocketMessage(
    packet: ToClientPacket,
    appCtx: AppContextType,
    stateCtx: StateContext,
    websocketCtx: WebSocketContextType,
){
    console.log(JSON.stringify(packet, null, 2));

    switch(packet.type) {
        case "pong":
            websocketCtx.sendPacket({
                type: "ping"
            });
        break;
        case "rateLimitExceeded":
            appCtx.pushErrorCard({ title: translate("notification.rateLimitExceeded"), body: "" });
        break;
        case "forcedOutsideLobby":
            appCtx.setContent({type:"gameBrowser"});
        break;
        case "forcedDisconnect":
            websocketCtx.close();
            appCtx.setContent({type:"main"});
        break;
        case "lobbyList":
            stateCtx.lobbies = new Map();

            for(let [lobbyId, lobbyData] of Object.entries(packet.lobbies))
                stateCtx.lobbies.set(Number.parseInt(lobbyId), lobbyData);
        break;
        case "acceptJoin":
            stateCtx.setMyId(packet.playerId);
            stateCtx.setRoomCode(packet.roomCode);
            stateCtx.setClientState(
                (packet.spectator?{type: "spectator"}:createPlayerGameState())
            );

            if(packet.inGame){
                appCtx.setContent({type:"gameScreen", spectator: packet.spectator});
            }else{
                appCtx.setContent({type:"lobbyScreen"});
            }

            saveReconnectData(packet.roomCode, packet.playerId);
            sendDefaultName(websocketCtx);
            appCtx.clearCoverCard();
        break;
        case "rejectJoin":
            switch(packet.reason) {
                case "roomDoesntExist":
                    appCtx.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.roomDoesntExist") });
                    // If the room doesn't exist, don't suggest the user to reconnect to it.
                    deleteReconnectData();
                    appCtx.clearCoverCard();
                break;
                case "gameAlreadyStarted":
                    appCtx.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.gameAlreadyStarted") });
                break;
                case "roomFull":
                    appCtx.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.roomFull") });
                break;
                case "serverBusy":
                    appCtx.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.serverBusy") });
                break;
                case "playerTaken":
                    appCtx.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.playerTaken") });
                break;
                case "playerDoesntExist":
                    appCtx.pushErrorCard({ title: translate("notification.rejectJoin"), body: translate("notification.rejectJoin.playerDoesntExist") });
                break;
                default:
                    appCtx.pushErrorCard({ title: translate("notification.rejectJoin"), body: `${packet.type} message response not implemented: ${packet.reason}` });
                    console.error(`${packet.type} message response not implemented: ${packet.reason}`);
                    console.error(packet);
                break;
            }
            deleteReconnectData();
            
        break;
        case "rejectStart":
            switch(packet.reason) {
                case "gameEndsInstantly":
                    appCtx.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.gameEndsInstantly") });
                break;
                case "roleListTooSmall":
                    appCtx.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.roleListTooSmall") });
                break;
                case "roleListCannotCreateRoles":
                    appCtx.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.roleListCannotCreateRoles") });
                break;
                case "zeroTimeGame":
                    appCtx.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.zeroTimeGame") });
                break;
                case "tooManyCLients":
                    appCtx.pushErrorCard({ title: translate("notification.rejectStart"), body: translate("notification.rejectStart.tooManyClients") });
                break;
                default:
                    appCtx.pushErrorCard({ title: translate("notification.rejectStart"), body: "" });
                    console.error(`${packet.type} message response not implemented: ${packet.reason}`);
                    console.error(packet);
                break;
            }
        break;
        case "playersHost":
            for(let [playerId, player] of stateCtx.clients.entries()){
                if (packet.hosts.includes(playerId)) {
                    player.ready = "host";
                } else {
                    player.ready = player.ready === "host" ? "ready" : player.ready
                }
            }
            stateCtx.setClients(stateCtx.clients);
            stateCtx.setHost(packet.hosts.includes(stateCtx.myId??-1));
        break;
        case "playersReady":
            for(let [playerId, player] of stateCtx.clients.entries()){
                if (packet.ready.includes(playerId)) {
                    player.ready = "ready";
                } else {
                    player.ready = player.ready === "host" ? "host" : "notReady"
                }
            }
            stateCtx.setClients(stateCtx.clients);
        break;
        case "playersLostConnection":
            for(let [playerId, client] of stateCtx.clients.entries()){
                if(packet.lostConnection.includes(playerId))
                    client.connection = "couldReconnect";
            }
            stateCtx.setClients(stateCtx.clients);
        break;
        /*
        In Lobby/Game 
        */
        case "yourId":
            stateCtx.setMyId(stateCtx.myId);
        break;
        case "yourPlayerIndex":
            if(stateCtx.clientState.type === "player"){
                stateCtx.clientState.myIndex = packet.playerIndex;
                stateCtx.setClientState(stateCtx.clientState);
            }

            //TODO jack Im sorry
            AudioController.clearQueue();
        break;
        case "yourFellowInsiders":
            if(stateCtx.clientState.type === "player"){
                stateCtx.clientState.fellowInsiders = packet.fellowInsiders;
                stateCtx.setClientState(stateCtx.clientState);
            }
        break;
        case "clientObjects":
            const oldMySpectator = stateCtx.clients.get(stateCtx.myId!)?.clientType.type === "spectator";

            stateCtx.clients = new ListMap();
            for(const [clientId, lobbyClient] of packet.clients){
                stateCtx.clients.insert(clientId, lobbyClient);
            }
            const newMySpectator = stateCtx.clients.get(stateCtx.myId!)?.clientType.type === "spectator";
            
            if (oldMySpectator && !newMySpectator){
                sendDefaultName(websocketCtx);
            }
            
            stateCtx.clientState = (newMySpectator?{type: "spectator"}:createPlayerGameState());
            stateCtx.setClientState(stateCtx.clientState);

            // Recompute keyword data, since player names are keywords.
            computePlayerKeywordDataForLobby(
                Array.from(stateCtx.clients.values())
                    .filter(client => client.clientType.type === "player")
                    .map(client => (client.clientType as { type: "player", name: string }).name)
            );
            
            stateCtx.setClients(stateCtx.clients);
        break;
        // case "hostData":
        //     stateCtx.setClients(
        //         new ListMap<number, GameClient>(packet.clients)
        //     );
        // break;
        case "lobbyName":
            stateCtx.setLobbyName(packet.name);
        break;
        case "startGame":
            appCtx.setContent({type:"loading"});
            
            // const spectator = stateCtx.clients.get(stateCtx.myId!)?.clientType.type === "spectator";
            // stateCtx.setClientState(
            //     (spectator?{type: "spectator"}:createPlayerGameState())
            // );

            AudioController.queueFile("audio/start_game.mp3");
            break;
        case "gameInitializationComplete":
            stateCtx.setInitialized(true);
            appCtx.setContent({
                type:"gameScreen",
                spectator: stateCtx.clientState.type === "spectator"
            });
            break;
        case "backToLobby":
            appCtx.setContent({type:"lobbyScreen"});
        break;
        case "gamePlayers":
            //only update the playerlist with the new one if there are any differences
            let playersNamesChanged = false;
            if(stateCtx.players.length !== packet.players.length)
                playersNamesChanged = true;
            else{
                for(let i = 0; i < packet.players.length; i++){
                    if(stateCtx.players[i].name !== packet.players[i]){
                        playersNamesChanged = true;
                        break;
                    }
                }
            }
            if(playersNamesChanged){
                stateCtx.players = [];
                for(let i = 0; i < packet.players.length; i++){
                    stateCtx.players.push(createPlayer(packet.players[i], i));
                }
            }

            // Recompute keyword data, since player names are keywords.
            computePlayerKeywordData(stateCtx.players);
            stateCtx.setPlayers(stateCtx.players);
        break;
        case "roleList":
            //list of role list entriy
            stateCtx.setRoleList(packet.roleList);
        break;
        case "roleOutline":
            //role list entriy
            stateCtx.roleList = structuredClone(stateCtx.roleList);
            stateCtx.roleList[packet.index] = packet.roleOutline;
            stateCtx.roleList = [...stateCtx.roleList];
            stateCtx.setRoleList(stateCtx.roleList);
        break;
        case "phaseTime":
            stateCtx.phaseTimes[packet.phase.type] = packet.time;
            stateCtx.phaseTimes = {...stateCtx.phaseTimes};
            stateCtx.setPhaseTimes(stateCtx.phaseTimes);
        break;
        case "phaseTimes":
            stateCtx.setPhaseTimes(packet.phaseTimeSettings);
        break;
        case "enabledRoles":
            stateCtx.setEnabledRoles(packet.roles);
        break;
        case "enabledModifiers":
            stateCtx.setEnabledModifiers(packet.modifiers);
        break;
        case "phase":
            stateCtx.setPhaseState(packet.phase);
            stateCtx.setDayNumber(packet.dayNumber);
    
            if(packet.phase.type === "briefing" && stateCtx.clientState.type === "player"){
                const role = stateCtx.clientState.roleState?.type;
                if(role !== undefined){
                    appCtx.setCoverCard(<WikiArticle article={"role/"+role as WikiArticleLink}/>);
                }
            }
        break;
        case "phaseTimeLeft":
            stateCtx.setTimeLeftMs(packet.secondsLeft!==null?(packet.secondsLeft * 1000):null);
        break;
        case "playerAlive":
            for(let i = 0; i < stateCtx.players.length && i < packet.alive.length; i++){
                stateCtx.players[i].alive = packet.alive[i];
            }
            stateCtx.setPlayers(stateCtx.players);
        break;
        case "playerVotes":
            let listMapVotes = new ListMap<PlayerIndex, number>(packet.votesForPlayer);

            for(let i = 0; i < stateCtx.players.length; i++){
                stateCtx.players[i].numVoted = 0;
                
                let numVoted = listMapVotes.get(i);
                if(numVoted !== null){
                    stateCtx.players[i].numVoted = numVoted;
                }
            }
            stateCtx.setPlayers(stateCtx.players);
        break;
        case "yourSendChatGroups":
            if(stateCtx.clientState.type === "player"){
                stateCtx.clientState.sendChatGroups = [...packet.sendChatGroups];
                stateCtx.setClientState(stateCtx.clientState);
            }
        break;
        case "yourInsiderGroups":
            if(stateCtx.clientState.type === "player"){
                stateCtx.clientState.insiderGroups = [...packet.insiderGroups];
                stateCtx.setClientState(stateCtx.clientState);
            }
        break;
        case "yourAllowedControllers":
            if(stateCtx.clientState.type === "player"){
                stateCtx.clientState.savedControllers = 
                    packet.save.sort((a, b) => sortControllerIdCompare(a[0],b[0]));
                stateCtx.setClientState(stateCtx.clientState);
                }
        break;
        case "yourRoleLabels":
            for (const player of stateCtx.players) {
                player.roleLabel = null;
            }
            for (const [key, value] of packet.roleLabels) { 
                if(
                    stateCtx.players !== undefined && 
                    stateCtx.players[key] !== undefined
                )
                    stateCtx.players[key].roleLabel = value as Role;
            }
            stateCtx.setPlayers(stateCtx.players);
            
        break;
        case "yourPlayerTags":
            for(let i = 0; i < stateCtx.players.length; i++){
                stateCtx.players[i].playerTags = [];
            }

            for(const [key, value] of packet.playerTags){
                if(
                    stateCtx.players !== undefined && 
                    stateCtx.players[key] !== undefined
                )
                    stateCtx.players[key].playerTags = value as Tag[];
            }
            stateCtx.setPlayers(stateCtx.players);
            
        break;
        case "yourWill":
            if(stateCtx.clientState.type === "player"){
                stateCtx.clientState.will = packet.will;
                
                if(stateCtx.clientState.will === ""){
                    websocketCtx.sendSaveWillPacket(defaultAlibi());
                }

                stateCtx.setClientState(stateCtx.clientState);
            }
        break;
        case "yourNotes":
            if(stateCtx.clientState.type === "player"){
                stateCtx.clientState.notes = packet.notes;
                stateCtx.setClientState(stateCtx.clientState);
                // old default notes
                // if(stateCtx.clientState.notes.length === 0){
                //     const myIndex = stateCtx.clientState.myIndex;
                //     const myRoleKey = `role.${stateCtx.clientState.roleState.type}.name`;

                //     stateCtx.sendSaveNotesPacket([
                //         "Claims\n" + 
                //         stateCtx.players
                //             .map(player => 
                //                 `@${player.index + 1} - ${player.index === myIndex ? translate(myRoleKey) : ''}\n`
                //             )
                //             .join('')
                //     ]);
                // }
            }
        break;
        case "yourCrossedOutOutlines":
            if(stateCtx.clientState.type === "player")
                stateCtx.clientState.crossedOutOutlines = packet.crossedOutOutlines;
                stateCtx.setClientState(stateCtx.clientState);
        break;
        case "yourDeathNote":
            if(stateCtx.clientState.type === "player")
                stateCtx.clientState.deathNote = packet.deathNote ?? "";
            stateCtx.setClientState(stateCtx.clientState);
        break;
        case "yourRoleState":
            if(stateCtx.clientState.type === "player"){
                stateCtx.clientState.roleState = packet.roleState;
                stateCtx.setClientState(stateCtx.clientState);
            }
        break;
        case "yourJudgement":
            if(stateCtx.clientState.type === "player"){
                stateCtx.clientState.judgement = packet.verdict;
                stateCtx.setClientState(stateCtx.clientState);
            }
        break;
        case "yourVoteFastForwardPhase":
            stateCtx.setFastForward(packet.fastForward);
        break;
        case "addChatMessages":
            stateCtx.setChatMessages(stateCtx.chatMessages.concat(packet.chatMessages));

            // Chat notification icon state
            if(packet.chatMessages.length !== 0){
                stateCtx.setMissedChatMessages(true);
                
                for(let chatMessage of packet.chatMessages){
                    if(
                        chatMessage.variant.type === "whisper" &&
                        stateCtx.clientState.type === "player" &&
                        chatMessage.variant.toPlayerIndex === stateCtx.clientState.myIndex
                    ){
                        stateCtx.clientState.missedWhispers.push(chatMessage.variant.fromPlayerIndex);
                    }
                }
            }

            if (stateCtx.initialized === true) {
                for(let chatMessage of packet.chatMessages){
                    let audioSrc = chatMessageToAudio(chatMessage);
                    if(audioSrc)
                        AudioController.queueFile(audioSrc);
                }
            }
            
            stateCtx.setClientState(stateCtx.clientState);
        break;
        case "nightMessages":
            if(appCtx.getCoverCard()===null && packet.chatMessages.length!==0){
                appCtx.setCoverCard(<NightMessagePopup messages={packet.chatMessages}/>)
            }
        break;
        case "addGrave":
            stateCtx.setGraves([...stateCtx.graves, packet.grave]);
        break;
        case "gameOver":
            stateCtx.setTicking(false);
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
        break;
        default:
            console.error(`incoming message response not implemented: ${(packet as any)?.type}`);
            console.error(packet);
        break;
    }

    /*BEFORE YOU DELETE THIS LINE, REMEMBER THAT STATECTX STUFF NEEDS SET STATE HERE?!?!??*/
    // stateCtx.invokeStateListeners(packet.type);
}
