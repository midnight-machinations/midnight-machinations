import translate, { translateChecked } from "../game/lang";
import React, { ReactElement } from "react";
import GAME_MANAGER, { find, replaceMentions } from "..";
import StyledText, { KeywordDataMap, PLAYER_SENDER_KEYWORD_DATA } from "./StyledText";
import "./chatMessage.css"
import { ChatGroup, Conclusion, DefensePower, PhaseState, PlayerIndex, Tag, translateConclusion, translateWinCondition, Verdict, WinCondition } from "../game/gameState.d";
import { Role, RoleState } from "../game/roleState.d";
import { Grave } from "../game/graveState";
import DOMPurify from "dompurify";
import GraveComponent from "./grave";
import { RoleList, RoleOutline, translateRoleOutline } from "../game/roleListState.d";
import { CopyButton } from "./ClipboardButtons";
import { useGameState, useLobbyOrGameState, usePlayerNames, usePlayerState, useSpectator } from "./useHooks";
import { KiraResult, KiraResultDisplay } from "../menu/game/gameScreenContent/AbilityMenu/AbilitySelectionTypes/KiraSelectionMenu";
import { AuditorResult } from "../menu/game/gameScreenContent/AbilityMenu/RoleSpecificMenus/AuditorMenu";
import { ControllerID, AbilitySelection, translateControllerID, controllerIdToLink } from "../game/abilityInput";
import DetailsSummary from "./DetailsSummary";
import ListMap from "../ListMap";
import { Button } from "./Button";

const ChatElement = React.memo((
    props: {
        message: ChatMessage,
        playerNames?: string[],
        playerKeywordData?: KeywordDataMap,
        playerSenderKeywordData?: KeywordDataMap
    }, 
) => {
    const roleState = usePlayerState(
        playerState => playerState.roleState,
        ["yourRoleState"]
    );
    const forwardButton = usePlayerState(
        playerState => {
            let controller = new ListMap(playerState.savedControllers, (a,b)=>a.type===b.type)
                .get({type: "forwardMessage", player: playerState.myIndex});

            return controller!==null&&!controller.availableAbilityData.grayedOut;
        },
        ["yourPlayerIndex", "yourAllowedControllers"]
    );
    const myIndex = usePlayerState(
        playerState => playerState.myIndex,
        ["yourPlayerIndex"]
    );
    
    const roleList = useGameState(
        state => state.roleList,
        ["roleList"]
    );

    const [mouseHovering, setMouseHovering] = React.useState(false); 

    const message = props.message;
    const realPlayerNames = usePlayerNames();
    const playerNames = props.playerNames ?? realPlayerNames;
    const chatMessageStyles = require("../resources/styling/chatMessage.json");
    if(message.variant === undefined){
        console.error("ChatElement message with undefined variant:");
        console.error(message);
    }
    let style = typeof chatMessageStyles[message.variant.type] === "string" ? chatMessageStyles[message.variant.type] : "";

    let chatGroupIcon = null;
    if(message.chatGroup !== null){
        if(message.chatGroup !== "all"){
            chatGroupIcon = translateChecked("chatGroup."+message.chatGroup+".icon");
        }else{
            chatGroupIcon = "";
        }
    }else{
        chatGroupIcon = translate("noGroup.icon");
    }

    // Special chat messages that don't play by the rules
    switch (message.variant.type) {
        case "lobbyMessage":
            return <LobbyChatMessage 
                message={message as any}
                style={style}
                chatGroupIcon={chatGroupIcon!}
                playerNames={playerNames}
                playerKeywordData={props.playerKeywordData}
                playerSenderKeywordData={props.playerSenderKeywordData}
            />
        case "normal":
            return <NormalChatMessage 
                message={message as any}
                style={style}
                chatGroupIcon={chatGroupIcon!}
                playerNames={playerNames}
                roleState={roleState}
                playerKeywordData={props.playerKeywordData}
                playerSenderKeywordData={props.playerSenderKeywordData}
                mouseHovering={mouseHovering}
                setMouseHovering={setMouseHovering}
                myIndex={myIndex}
                forwardButton={forwardButton}
                roleList={roleList}
            />
        case "playerForwardedMessage":
        case "targetsMessage":
            return <div className={"chat-message-div"}>
                <span className="chat-message">
                    <StyledText className={"chat-message " + style}
                        playerKeywordData={props.playerKeywordData}
                    >
                        {(chatGroupIcon??"")} {translateChatMessage(message.variant, playerNames, roleList)}
                    </StyledText>
                    <ChatElement {...props} message={{
                        variant: message.variant.message,
                        chatGroup: message.chatGroup,
                    }}/>
                </span>
            </div>
        case "reporterReport":
            style += " block";
        break;
        case "abilityUsed":
            switch (message.variant.selection.type){
                case "kira":
                    return <div className={"chat-message-div chat-message kira-guess-results " + style}>
                        <StyledText
                            className="chat-message result"
                            playerKeywordData={props.playerKeywordData}
                        >{chatGroupIcon ?? ""} {translate("chatMessage.kiraSelection")}</StyledText>
                        <KiraResultDisplay 
                            map={{
                                type: "selection",
                                map: message.variant.selection.selection
                            }}
                            playerKeywordData={props.playerKeywordData}
                            playerNames={playerNames}
                        />
                    </div>
                case "string":
                    style += " block"
            }
        break;
        case "kiraResult":
            return <div className={"chat-message-div chat-message kira-guess-results " + style}>
                <StyledText
                    className="chat-message result"
                    playerKeywordData={props.playerKeywordData}
                >{chatGroupIcon ?? ""} {translate("chatMessage.kiraResult")}</StyledText>
                <KiraResultDisplay 
                    map={{
                        type: "reuslt",
                        map: message.variant.result.guesses
                    }}
                    playerKeywordData={props.playerKeywordData}
                    playerNames={playerNames}
                />
            </div>
        case "playerDied":
            return <PlayerDiedChatMessage
                playerKeywordData={props.playerKeywordData}
                style={style}
                chatGroupIcon={chatGroupIcon}
                playerNames={playerNames}
                message={message as any}
            />
    }

    return <div
        className={"chat-message-div " + style}
        onMouseOver={() => setMouseHovering(true)}
        onMouseOut={() => setMouseHovering(false)}
    >
        <StyledText className={"chat-message " + style} playerKeywordData={props.playerKeywordData}>
            {(chatGroupIcon??"")} {translateChatMessage(message.variant, playerNames, roleList)}
        </StyledText>
        {mouseHovering && <div
            className="chat-message-div-small-button-div"
        >
            {
                (roleState?.type === "forger" || roleState?.type === "counterfeiter")
                && <CopyButton
                    className="chat-message-div-small-button"
                    text={translateChatMessage(message.variant, playerNames, roleList)}
                />
            }
            {
                myIndex!==undefined && mouseHovering && forwardButton
                && <Button
                    className="chat-message-div-small-button material-icons-round"
                    onClick={()=>GAME_MANAGER.sendAbilityInput({
                        id: {type: "forwardMessage", player: myIndex}, 
                        selection: {type: "chatMessage", selection: props.message}
                    })}
                >forward</Button>
            }
        </div>}
        
    </div>;
});

function PlayerDiedChatMessage(props: Readonly<{
    playerKeywordData?: KeywordDataMap,
    style: string,
    chatGroupIcon: string | null,
    playerNames: string[],
    message: ChatMessage & { variant: { type: "playerDied" } }
}>): ReactElement {
    let graveRoleString: string;
    switch (props.message.variant.grave.information.type) {
        case "obscured":
            graveRoleString = translate("obscured");
            break;
        case "normal":
            graveRoleString = translate("role."+props.message.variant.grave.information.role+".name");
            break;
    }

    const spectator = useSpectator();

    return <div className={"chat-message-div"}>
        <DetailsSummary
            summary={
                <StyledText className={"chat-message " + props.style}
                    playerKeywordData={props.playerKeywordData}
                >
                    {(props.chatGroupIcon ?? "")} {translate("chatMessage.playerDied",
                        props.playerNames[props.message.variant.grave.player], graveRoleString
                    )}
                </StyledText>
            }
            defaultOpen={spectator}
        >
            <GraveComponent grave={props.message.variant.grave} playerNames={props.playerNames}/>
        </DetailsSummary>
    </div>;
}

function LobbyChatMessage(props: Readonly<{
    message: ChatMessage & { variant: { type: "lobbyMessage" } }
    playerNames: string[],
    style: string,
    playerKeywordData: KeywordDataMap | undefined,
    playerSenderKeywordData: KeywordDataMap | undefined
    chatGroupIcon: string
}>): ReactElement {
    let style = props.style;

    if (useContainsMention(props.message.variant, props.playerNames)) {
        style += " mention";
    }

    return <div className={"chat-message-div"}><span className={`chat-message ${style}`}>
        <StyledText
            playerKeywordData={props.playerSenderKeywordData ?? PLAYER_SENDER_KEYWORD_DATA}
        >{props.chatGroupIcon ?? ""} {props.message.variant.sender}: </StyledText>
        <StyledText
            playerKeywordData={props.playerKeywordData}
        >{translateChatMessage(props.message.variant, props.playerNames)}</StyledText>
    </span></div>;
}

function NormalChatMessage(props: Readonly<{
    message: ChatMessage & { variant: { type: "normal" } }
    style: string,
    chatGroupIcon: string,
    playerNames: string[],
    roleState: RoleState | undefined,
    playerKeywordData: KeywordDataMap | undefined,
    playerSenderKeywordData: KeywordDataMap | undefined,
    mouseHovering: boolean,
    setMouseHovering: (hovering: boolean) => void,
    myIndex: PlayerIndex | undefined,
    forwardButton: boolean | undefined,
    roleList: RoleList | undefined
}>): ReactElement {
    let style = props.style;
    let chatGroupIcon = props.chatGroupIcon;

    if(props.message.variant.messageSender.type !== "player" && props.message.variant.messageSender.type !== "livingToDead"){
        style += " discreet";
    } else if (props.message.chatGroup === "dead") {
        style += " dead player";
    } else {
        style += " player"
    }
    
    if (props.message.variant.messageSender.type === "livingToDead") {
        chatGroupIcon += translate("messageSender.livingToDead.icon")
    }

    let messageSender = "";
    if (props.message.variant.messageSender.type === "player" || props.message.variant.messageSender.type === "livingToDead") {
        messageSender = props.playerNames[props.message.variant.messageSender.player];
    }else if(props.message.variant.messageSender.type === "jailor" || props.message.variant.messageSender.type === "reporter"){
        messageSender = translate("role."+props.message.variant.messageSender.type+".name");
    }
    
    if (useContainsMention(props.message.variant, props.playerNames)) {
        style += " mention";
    }


    if (props.message.variant.block) {
        style += " block";
    }


    return <div
        className={"chat-message-div"}
        onMouseOver={() => props.setMouseHovering(true)}
        onMouseOut={() => props.setMouseHovering(false)}
    >
        <span className={`chat-message ${style}`}>
            <StyledText
                playerKeywordData={props.playerSenderKeywordData ?? PLAYER_SENDER_KEYWORD_DATA}
            >
                {chatGroupIcon ?? ""} {messageSender}: </StyledText>
            <StyledText
                playerKeywordData={props.playerKeywordData}
            >
                {translateChatMessage(props.message.variant, props.playerNames, undefined)}
            </StyledText>
        </span>
        {props.mouseHovering && <div
            className="chat-message-div-small-button-div"
        >
            {
                (props.roleState?.type === "forger" || props.roleState?.type === "counterfeiter")
                && <CopyButton
                    className="chat-message-div-small-button"
                    text={translateChatMessage(props.message.variant, props.playerNames, props.roleList)}
                />
            }
            {
                props.myIndex!==undefined && props.mouseHovering && props.forwardButton
                && <Button
                    className="chat-message-div-small-button material-icons-round"
                    onClick={()=>GAME_MANAGER.sendAbilityInput({
                        id: {type: "forwardMessage", player: props.myIndex?props.myIndex:0}, 
                        selection: {type: "chatMessage", selection: props.message}
                    })}
                >forward</Button>
            }
        </div>}
    </div>;
}

function useContainsMention(message: ChatMessageVariant & { text: string }, playerNames: string[]): boolean {
    const myNumber = usePlayerState(
        gameState => gameState.myIndex,
        ["yourPlayerIndex"]
    );

    const myName = useLobbyOrGameState(
        state => {
            if (state.stateType === "game" && state.clientState.type === "player")
                return state.players[state.clientState.myIndex].name
            else if (state.stateType === "lobby" && state.myId) {
                const me = state.players.get(state.myId)
                if (me?.clientType.type === "player") {
                    return me.clientType.name
                }
            } else {
                return undefined;
            }
        },
        ["lobbyClients", "yourId", "yourPlayerIndex", "gamePlayers"]
    );

    if (myName === undefined) {
        return false;
    }
    return (
        find(myName).test(sanitizePlayerMessage(replaceMentions(message.text, playerNames))) ||
        (
            myNumber !== undefined && 
            find("" + (myNumber + 1)).test(sanitizePlayerMessage(replaceMentions(message.text, playerNames)))
        )
    )
}

export default ChatElement;

function playerListToString(playerList: PlayerIndex[], playerNames: string[]): string {
    if (playerList.length === 0) {
        return translate("nobody");
    }
    return playerList.map((playerIndex) => {
        return playerNames[playerIndex];
    }).join(", ");
}

function roleListToString(roleList: Role[]): string {
    if (roleList === null || roleList.length === 0) {
        return translate("none");
    }
    return roleList.map((role) => {
        return translate("role."+role+".name")
    }).join(", ");
}

export function sanitizePlayerMessage(text: string): string {
    return DOMPurify.sanitize(text, { 
        ALLOWED_TAGS: []
    });
}

export function translateChatMessage(
    message: ChatMessageVariant,
    playerNames: string[],
    roleList?: RoleOutline[]
): string {
    switch (message.type) {
        case "lobbyMessage":
            return sanitizePlayerMessage(replaceMentions(message.text, playerNames));
        case "normal":
            return (message.block===true?"\n":"")+sanitizePlayerMessage(replaceMentions(message.text, playerNames));
        case "whisper":
            return translate("chatMessage.whisper", 
                playerNames[message.fromPlayerIndex],
                playerNames[message.toPlayerIndex],
                sanitizePlayerMessage(replaceMentions(message.text, playerNames))
            );
        case "broadcastWhisper":
            return translate("chatMessage.broadcastWhisper",
                playerNames[message.whisperer],
                playerNames[message.whisperee],
            );
        case "roleAssignment":
            return translate("chatMessage.roleAssignment", 
                translate("role."+message.role+".name")
            );
        case "playersRoleRevealed":
            return translate("chatMessage.playersRoleRevealed",
                playerNames[message.player],
                translate("role."+message.role+".name")
            );
        case "playersRoleConcealed":
            return translate("chatMessage.playersRoleConcealed",
                playerNames[message.player]
            );
        case "tagAdded":
            return translate("chatMessage.tagAdded",
                playerNames[message.player],
                translate("tag."+message.tag+".name"),
                translate("tag."+message.tag)
            );
        case "tagRemoved":
            return translate("chatMessage.tagRemoved",
                playerNames[message.player],
                translate("tag."+message.tag+".name"),
                translate("tag."+message.tag)
            );
        case "playerWonOrLost":
            if(message.won){
                return translate("chatMessage.playerWon",
                    playerNames[message.player], translate("role."+message.role+".name")
                );
            }else{
                return translate("chatMessage.playerLost",
                    playerNames[message.player], translate("role."+message.role+".name")
                );
            }
        case "playerQuit":
            return translate(`chatMessage.playerQuit${message.gameOver ? ".gameOver" : ""}`,
                playerNames[message.playerIndex]
            );
        case "youDied":
            return translate("chatMessage.youDied");
        case "phaseChange":
            switch (message.phase.type) {
                case "nomination":
                    if (message.phase.trialsLeft === 1) {
                        return translate("chatMessage.phaseChange.nomination.lastTrial",
                            translate("phase."+message.phase.type),
                            message.dayNumber,
                        );
                    } else {
                        return translate("chatMessage.phaseChange.nomination",
                            translate("phase."+message.phase.type),
                            message.dayNumber,
                            message.phase.trialsLeft
                        );
                    }
                case "testimony":
                case "judgement":
                case "finalWords":
                    return translate("chatMessage.phaseChange.trial",
                        translate("phase."+message.phase.type),
                        message.dayNumber,
                        playerNames[message.phase.playerOnTrial]
                    );
                case "recess":
                    return translate("chatMessage.phaseChange.recess");
                default:
                    return translate("chatMessage.phaseChange",
                        translate("phase."+message.phase.type),
                        message.dayNumber
                    );
            }
            
        case "trialInformation":
            return translate("chatMessage.trialInformation",
                message.requiredVotes,
                message.trialsLeft
            );
        case "voted":
            if (message.votee !== null) {
                return translate("chatMessage.voted",
                    playerNames[message.voter],
                    playerNames[message.votee],
                );
            } else {
                return translate("chatMessage.voted.cleared",
                    playerNames[message.voter],
                );
            }
        case "playerNominated":
            return translate("chatMessage.playerNominated",
                playerNames[message.playerIndex],
                playerListToString(message.playersVoted, playerNames)
            );
        case "judgementVerdict":
            return translate("chatMessage.judgementVerdict",
                playerNames[message.voterPlayerIndex],
                translate("verdict."+message.verdict.toLowerCase())
            );
        case "trialVerdict":{
            let hang;
            // Damn
            if (GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.enabledModifiers.includes("twoThirdsMajority")) {
                hang = message.innocent <= 2 * message.guilty
            } else {
                hang = message.innocent < message.guilty
            }
            return translate("chatMessage.trialVerdict",
                playerNames[message.playerOnTrial],
                hang?translate("verdict.guilty"):translate("verdict.innocent"),
                message.innocent,
                message.guilty
            );
        }
        case "abilityUsed":

            let out;

            switch (message.selection.type) {
                case "unit":
                    out = translate("chatMessage.abilityUsed.selection.unit");
                    break;
                case "boolean":{
                    let text = null;
                    if(message.selection.selection===true){
                        text = translateChecked("controllerId."+controllerIdToLink(message.abilityId).replace(/\//g, ".") + ".boolean.true");
                        if(text===null)
                            text = " "+translate("on");
                        else
                            text = " "+text;
                    }else{
                        text = translateChecked("controllerId."+controllerIdToLink(message.abilityId).replace(/\//g, ".") + ".boolean.false");
                        if(text===null)
                            text = " "+translate("off");
                        else
                            text = " "+text;
                    }
                    out = translate("chatMessage.abilityUsed.selection.boolean", text);
                    break;
                }
                case "playerList":
                    out = translate("chatMessage.abilityUsed.selection.playerList",
                        playerListToString(message.selection.selection, playerNames)
                    );
                    break;
                case "twoPlayerOption":
                    out = translate("chatMessage.abilityUsed.selection.twoPlayerOption",
                        playerListToString(message.selection.selection===null?[]:message.selection.selection, playerNames)
                    );
                    break;
                case "roleList":
                    out = translate("chatMessage.abilityUsed.selection.roleList",
                        roleListToString(message.selection.selection)
                    );
                    break;
                case "twoRoleOption":
                    out = translate("chatMessage.abilityUsed.selection.twoRoleOption",
                        message.selection.selection[0]===null?translate("none"):translate("role."+message.selection.selection[0]+".name"),
                        message.selection.selection[1]===null?translate("none"):translate("role."+message.selection.selection[1]+".name"),
                    );
                    break;
                case "twoRoleOutlineOption":                    
                    let first = message.selection.selection[0] === null ? 
                        translate("none") :
                        roleList === undefined ?
                            message.selection.selection[0].toString() :
                            translateRoleOutline(roleList[message.selection.selection[0]]);

                    let second = message.selection.selection[1] === null ? 
                        translate("none") :
                        roleList === undefined ?
                            message.selection.selection[1].toString() :
                            translateRoleOutline(roleList[message.selection.selection[1]]);

                    

                    out = translate("chatMessage.abilityUsed.selection.twoRoleOutlineOption", first, second);
                    break;
                case "string":
                    out = translate("chatMessage.abilityUsed.selection.string", sanitizePlayerMessage(replaceMentions(message.selection.selection, playerNames)));
                    break;
                case "integer":
                    let text = translateChecked("controllerId."+controllerIdToLink(message.abilityId).replace(/\//g, ".") + ".integer." + message.selection.selection);
                    
                    if(text === null){
                        text = message.selection.selection.toString()
                    }

                    out = translate("chatMessage.abilityUsed.selection.integer", text);
                    break;
                default:
                    out = "";
            }
            
            let abilityIdString = translateControllerID(message.abilityId);
                
            return translate("chatMessage.abilityUsed", playerNames[message.player], abilityIdString, out);
        case "mayorRevealed":
            return translate("chatMessage.mayorRevealed",
                playerNames[message.playerIndex],
            );
        case "martyrRevealed":
            return translate("chatMessage.martyrRevealed",
                playerNames[message.martyr],
            );
        case "reporterReport":
            return translate("chatMessage.reporterReport",
                sanitizePlayerMessage(replaceMentions(message.report, playerNames))
            );
        case "playerIsBeingInterviewed":
            return translate("chatMessage.playerIsBeingInterviewed",
                playerNames[message.playerIndex],
            );
        case "jailedTarget":
            return translate("chatMessage.jailedTarget",
                playerNames[message.playerIndex],
            );
        case "jailedSomeone":
            return translate("chatMessage.jailedSomeone",
                playerNames[message.playerIndex]
            );
        case "wardenPlayersImprisoned":
            return translate("chatMessage.wardenPlayersImprisoned",
                playerListToString(message.players, playerNames)
            )
        case "deputyKilled":
            return translate("chatMessage.deputyKilled",
                playerNames[message.shotIndex]
            );
        case "puppeteerPlayerIsNowMarionette":
            return translate("chatMessage.puppeteerPlayerIsNowMarionette",
                playerNames[message.player]
            );
        case "recruiterPlayerIsNowRecruit":
            return translate("chatMessage.recruiterPlayerIsNowRecruit",
                playerNames[message.player]
            );
        case "godfatherBackup":
            if (message.backup !== null) {
                return translate("chatMessage.godfatherBackup", playerNames[message.backup]);
            } else {
                return translate("chatMessage.godfatherBackup.nobody");
            }
        /* NIGHT */
        case "godfatherBackupKilled":
            return translate("chatMessage.godfatherBackupKilled", playerNames[message.backup]);
        case "sheriffResult":
            return translate("chatMessage.sheriffResult." + (message.suspicious ? "suspicious" : "innocent"));
        case "snoopResult":
            return translate("chatMessage.snoopResult." + (message.townie ? "townie" : "inconclusive"));
        case "polymathSnoopResult":
            return translate(message.inno ? "chatMessage.sheriffResult.innocent" : "chatMessage.snoopResult.inconclusive");
        case "gossipResult":
            return translate("chatMessage.gossipResult." + (message.enemies ? "enemies" : "none"));
        case "tallyClerkResult":
            return translate("chatMessage.tallyClerkResult", message.evilCount);
        case "lookoutResult":
            return translate("chatMessage.lookoutResult", playerListToString(message.players, playerNames));
        case "spyMafiaVisit":
            return translate("chatMessage.spyMafiaVisit", playerListToString(message.players, playerNames));
        case "spyBug":
            return translate("chatMessage.spyBug."+message.bug);
        case "trackerResult":
            return translate("chatMessage.trackerResult", playerListToString(message.players, playerNames));
        case "seerResult":
            return translate("chatMessage.seerResult." + (message.enemies ? "enemies" : "friends"));
        case "psychicEvil":
            return translate(
                "chatMessage.psychicEvil",
                playerNames[message.first],
                playerNames[message.second]
            );
        case "psychicGood":
            return translate(
                "chatMessage.psychicGood",
                playerNames[message.player]
            );
        case "auditorResult":
            return translate("chatMessage.auditorResult",
                message.outlineIndex+1,
                translateRoleOutline(message.roleOutline),
                message.result.map((role)=>translate("role."+role+".name")).join(", ")
            );
        case "engineerVisitorsRole":
            return translate("chatMessage.engineerVisitorsRole", translate("role."+message.role+".name"));
        case "trapState":
            return translate("chatMessage.trapState."+message.state.type);
        case "trapStateEndOfNight":
            return translate("chatMessage.trapStateEndOfNight."+message.state.type);
        case "playerRoleAndAlibi":
            return translate("chatMessage.playerRoleAndAlibi",
                playerNames[message.player],
                translate("role."+message.role+".name"),
                sanitizePlayerMessage(replaceMentions(message.will, playerNames))
            );
        case "informantResult":
            return translate("chatMessage.informantResult",
                playerNames[message.player],
                translate("role."+message.role+".name"),
                translate("chatMessage.informantResult.visited", playerListToString(message.visited, playerNames)),
                translate("chatMessage.informantResult.visitedBy", playerListToString(message.visitedBy, playerNames))
            );
        case "scarecrowResult":
            return translate("chatMessage.scarecrowResult",
                playerListToString(message.players, playerNames)
            );
        case "ambusherCaught":
            return translate("chatMessage.ambusherCaught",
                playerNames[message.ambusher]
            );
        case "mercenaryHits":
            return translate("chatMessage.mercenaryHits", roleListToString(message.roles));
        case "mercenaryResult":
            return translate("chatMessage.mercenaryResult."+(message.hit?"hit":"notHit"));
        case "mediumHauntStarted":
            return translate("chatMessage.mediumHauntStarted", playerNames[message.medium], playerNames[message.player]);
        case "youWerePossessed":
            return translate("chatMessage.youWerePossessed" + (message.immune ? ".immune" : ""));
        case "targetHasRole":
            return translate("chatMessage.targetHasRole", translate("role."+message.role+".name"));
        case "targetHasWinCondition":
            return translate("chatMessage.targetHasWinCondition", translateWinCondition(message.winCondition));
        case "werewolfTrackingResult":
            return translate("chatMessage.werewolfTrackingResult", 
                playerNames[message.trackedPlayer],
                playerListToString(message.players, playerNames)
            );
        case "wildcardConvertFailed":
            return translate("chatMessage.wildcardConvertFailed", translate("role."+message.role+".name"));
        case "chronokaiserSpeedUp":
            return translate("chatMessage.chronokaiserSpeedUp", message.percent);
        case "addedToNiceList":
            return translate("chatMessage.addedToNiceList");
        case "nextSantaAbility":
            return translate(`chatMessage.nextSantaAbility.${message.ability}`);
        case "nextKrampusAbility":
            return translate(`chatMessage.nextKrampusAbility.${message.ability}`);
        case "addedToNaughtyList":
            return translate("chatMessage.addedToNaughtyList");
        case "santaAddedPlayerToNaughtyList":
            return translate("chatMessage.santaAddedPlayerToNaughtyList", playerNames[message.player]);
        case "gameOver": {
            const conclusionString = 
                translateChecked(`chatMessage.gameOver.conclusion.${message.synopsis.conclusion}`)
                ?? translate(`chatMessage.gameOver.conclusion.unknown`, translateConclusion(message.synopsis.conclusion))
            
            return conclusionString + '\n'
                + message.synopsis.playerSynopses.map((synopsis, index) => 
                    translate(`chatMessage.gameOver.player.won.${synopsis.won}`, playerNames![index])
                        + ` (${
                            synopsis.crumbs.map(crumb => translate("chatMessage.gameOver.player.crumb",
                                translateWinCondition(crumb.winCondition), 
                                translate(`role.${crumb.role}.name`)
                            )).join(" → ")
                        })`
                ).join('\n');
        }
        case "playerForwardedMessage":
            return translate(`chatMessage.playerForwardedMessage`, playerNames[message.forwarder]);
        case "fragileVestBreak":
            console.log(playerNames);
            return translate(
                `chatMessage.fragileVestBreak`,
                translate("defense."+message.defense),
                playerNames[message.playerWithVest]
            );
        case "mercenaryYouAreAHit":
        case "deputyShotYou":
        case "mediumExists":
        case "youGuardedSomeone":
        case "youWereGuarded":
        case "revolutionaryWon":
        case "jesterWon":
        case "wardblocked":
        case "roleBlocked":
        case "yourConvertFailed":
        case "cultConvertsNext":
        case "cultKillsNext":
        case "someoneSurvivedYourAttack":
        case "transported":
        case "targetIsPossessionImmune":
        case "youSurvivedAttack":
        case "youArePoisoned":
        case "doomsayerFailed":
        case "doomsayerWon":
        case "silenced":
        case "martyrFailed":
        case "martyrWon":
        case "targetsMessage":
        case "psychicFailed":
        case "phaseFastForwarded":
        case "invalidWhisper":
        case "politicianCountdownStarted":
        case "youAttackedSomeone":
        case "youWereAttacked":
        case "werewolfTracked":
            return translate("chatMessage."+message.type);
        case "playerDied":
        case "kiraResult":
        default:
            console.error("Unknown message type " + (message as any).type + ":");
            console.error(message);
            return "FIXME: " + translate("chatMessage." + message);
    }
}
export type ChatMessage = {
    variant: ChatMessageVariant
    chatGroup: ChatGroup | null
}
export type ChatMessageVariant = {
    type: "lobbyMessage",
    sender: string,
    text: string
} | {
    type: "normal", 
    messageSender: MessageSender,
    text: string,
    block: boolean
} | {
    type: "whisper", 
    fromPlayerIndex: PlayerIndex, 
    toPlayerIndex: PlayerIndex, 
    text: string
} | {
    type: "broadcastWhisper", 
    whisperer: PlayerIndex, 
    whisperee: PlayerIndex 
} | 
// System
{
    type: "roleAssignment", 
    role: Role
} | {
    type: "playerDied", 
    grave: Grave
} | {
    type: "playersRoleRevealed",
    role: Role,
    player: PlayerIndex
} | {
    type: "playersRoleConcealed",
    player: PlayerIndex
} | {
    type: "tagAdded",
    player: PlayerIndex,
    tag: Tag
} | {
    type: "tagRemoved",
    player: PlayerIndex,
    tag: Tag
} | {
    type: "gameOver"
    synopsis: {
        playerSynopses: {
            crumbs: {
                night: number | null,
                role: Role,
                winCondition: WinCondition
            }[],
            won: boolean
        }[],
        conclusion: Conclusion
    }
} | {
    type: "playerWonOrLost",
    player: PlayerIndex,
    won: boolean,
    role: Role
} | {
    type: "playerQuit",
    playerIndex: PlayerIndex
    gameOver: boolean,
} | {
    type: "phaseChange", 
    phase: PhaseState,
    dayNumber: number
} | 
// Trial
{
    type: "trialInformation", 
    requiredVotes: number, 
    trialsLeft: number
} | {
    type: "voted", 
    voter: PlayerIndex, 
    votee: PlayerIndex | null 
} | {
    type: "playerNominated", 
    playerIndex: PlayerIndex,
    playersVoted: PlayerIndex[]
} | {
    type: "judgementVerdict", 
    voterPlayerIndex: PlayerIndex, 
    verdict: Verdict
} | {
    type: "trialVerdict", 
    playerOnTrial: PlayerIndex, 
    innocent: number, 
    guilty: number
} | 
// Misc.
{
    type: "abilityUsed", 
    player: PlayerIndex,
    abilityId: ControllerID,
    selection: AbilitySelection
    
} | {
    type: "phaseFastForwarded"
} |
// Role-specific
{
    type: "mayorRevealed", 
    playerIndex: PlayerIndex
} | {
    type: "invalidWhisper"
} | {
    type: "politicianCountdownStarted"
} | {
    type: "reporterReport",
    report: string
} | {
    type: "playerIsBeingInterviewed",
    playerIndex: PlayerIndex
} | {
    type: "jailedTarget"
    playerIndex: PlayerIndex
} | {
    type: "jailedSomeone",
    playerIndex: PlayerIndex
} | {
    type: "wardenPlayersImprisoned",
    players: PlayerIndex[]
} | {
    type: "yourConvertFailed"
} | {
    type: "cultConvertsNext"
} | {
    type: "cultKillsNext"
} | {
    type: "mediumHauntStarted",
    medium: PlayerIndex,
    player: PlayerIndex
} | {
    type: "mediumExists"
} | {
    type: "deputyKilled",
    shotIndex: PlayerIndex
} | {
    type: "deputyShotYou"
} | {
    type: "puppeteerPlayerIsNowMarionette",
    player: PlayerIndex
} | {
    type: "recruiterPlayerIsNowRecruit",
    player: PlayerIndex
} | {
    type: "roleBlocked"
} | {
    type: "someoneSurvivedYourAttack"
} | {
    type: "youSurvivedAttack"
} | {
    type: "youWereAttacked"
} | {
    type: "youAttackedSomeone"
} | {
    type: "youArePoisoned"
} |
/* Role-specific */
{
    type: "wardblocked"
} | {
    type: "sheriffResult", 
    suspicious: boolean
} | {
    type: "snoopResult", 
    townie: boolean
} | {
    type: "polymathSnoopResult", 
    inno: boolean
} | {
    type: "gossipResult",
    enemies: boolean
} | {
    type: "tallyClerkResult",
    evilCount: number
} | {
    type: "lookoutResult", 
    players: PlayerIndex[]
} | {
    type: "spyMafiaVisit", 
    players: PlayerIndex[]
} | {
    type: "spyBug", 
    bug: "silenced" | "roleblocked" | "protected" | "transported" | "possessed"
} | {
    type: "trackerResult",
    players: PlayerIndex[]
} | {
    type: "seerResult",
    enemies: boolean
} | {
    type: "psychicGood",
    player: PlayerIndex
} | {
    type: "psychicEvil",
    first: PlayerIndex,
    second: PlayerIndex
} | {
    type: "psychicFailed"
} | {
    type: "auditorResult",
    outlineIndex: number,
    roleOutline: RoleOutline,
    result: AuditorResult,
} | {
    type: "engineerVisitorsRole",
    role: Role
} | {
    type: "trapState",
    state: {
        type: "dismantled" | "ready" | "set"
    }
} | {
    type: "trapStateEndOfNight",
    state: {
        type: "dismantled" | "ready" | "set"
    }
} | {
    type: "fragileVestBreak",
    playerWithVest: PlayerIndex,
    defense: DefensePower
} | {
    type: "youGuardedSomeone"
} | {
    type: "youWereGuarded"
} | {
    type: "youDied"
} | {
    type: "transported"
} | {
    type: "godfatherBackup",
    backup: PlayerIndex | null
} | {
    type: "godfatherBackupKilled",
    backup: PlayerIndex
} | {
    type: "silenced"
} | {
    type: "playerRoleAndAlibi",
    player: PlayerIndex,
    role: Role,
    will: string
} | {
    type: "informantResult", 
    player: PlayerIndex
    role: Role,
    visitedBy: PlayerIndex[],
    visited: PlayerIndex[]
} | {
    type: "scarecrowResult",
    players: PlayerIndex[]
} | {
    type: "ambusherCaught",
    ambusher: PlayerIndex
} | {
    type: "targetIsPossessionImmune"
} | {
    type: "youWerePossessed",
    immune: boolean
} | {
    type: "targetHasRole",
    role: Role
} | {
    type: "targetHasWinCondition",
    winCondition: WinCondition
} | {
    type: "targetsMessage",
    message: ChatMessageVariant
} | {
    type: "playerForwardedMessage",
    forwarder: PlayerIndex,
    message: ChatMessageVariant
} | {
    type: "werewolfTrackingResult",
    trackedPlayer: PlayerIndex
    players: PlayerIndex[]
} | {
    type: "jesterWon"
} | {
    type: "wildcardConvertFailed",
    role: Role
} | {
    type: "revolutionaryWon"
} | {
    type: "chronokaiserSpeedUp"
    percent: number
} | {
    type: "doomsayerFailed"
} | {
    type: "doomsayerWon"
} | {
    type: "mercenaryHits",
    roles: Role[]
} | {
    type: "mercenaryResult",
    hit: boolean
} | {
    type: "mercenaryYouAreAHit"
} | {
    type: "kiraResult",
    result: {
        guesses: KiraResult
    }
} | {
    type: "martyrFailed"
} | {
    type: "martyrWon"
} | {
    type: "martyrRevealed",
    martyr: PlayerIndex
} | {
    type: "addedToNiceList"
} | {
    type: "nextSantaAbility"
    ability: "nice" | "naughty"
} | {
    type: "nextKrampusAbility",
    ability: "doNothing" | "kill"
} | {
    type: "addedToNaughtyList"
} | {
    type: "santaAddedPlayerToNaughtyList",
    player: PlayerIndex
} | {
    type: "werewolfTracked"
}

export type MessageSender = {
    type: "player", 
    player: PlayerIndex
} | {
    type: "livingToDead",
    player: PlayerIndex,
} | {
    type: "jailor" | "reporter"
}
