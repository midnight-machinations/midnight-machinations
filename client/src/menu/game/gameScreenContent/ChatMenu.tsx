import React, { ReactElement, useCallback, useContext, useEffect, useMemo, useRef, useState } from "react";
import translate from "../../../game/lang";
import "../gameScreen.css";
import "./chatMenu.css"
import ChatElement, { translateChatMessage } from "../../../components/ChatMessage";
import { HistoryPoller, HistoryQueue } from "../../../history";
import { Button } from "../../../components/Button";
import Icon from "../../../components/Icon";
import StyledText, { KeywordDataMap, PLAYER_KEYWORD_DATA, PLAYER_SENDER_KEYWORD_DATA } from "../../../components/StyledText";
import { Virtuoso } from 'react-virtuoso';
import GameScreenMenuTab from "../GameScreenMenuTab";
import { GameScreenMenuType } from "../GameScreenMenuContext";
import { WebsocketContext } from "../../WebsocketContext";
import { usePlayerNames, usePlayerNamesToString, usePlayerState } from "../../../stateContext/useHooks";
import { PlayerIndex } from "../../../stateContext/stateType/otherState";
import { StateContext } from "../../../stateContext/StateContext";


export default function ChatMenu(): ReactElement {
    const filter = usePlayerState()?.chatFilter;
    const sendChatGroups = usePlayerState()?.sendChatGroups;
    const playerNames = usePlayerNames()!;

    const gameState = useContext(StateContext)!;

    const filterString = useMemo(() => {
        if (filter === undefined || filter === null) {
            return "";
        } else if (filter.type === "playerNameInMessage") {
            return playerNames[filter.player];
        } else if (filter.type === "myWhispersWithPlayer") {
            return playerNames[filter.player];
        }else{
            return "";
        }
    }, [filter, playerNames]);

    return <div className="chat-menu chat-menu-colors">
        <GameScreenMenuTab close={GameScreenMenuType.ChatMenu} helpMenu={"standard/chat"}>{translate("menu.chat.title")}</GameScreenMenuTab>
        {filter === undefined || filter === null || <div className="chat-filter-zone highlighted">
            <StyledText>{translate("menu.chat.playerFilter", filterString)}</StyledText>
            <Button 
                onClick={()=> gameState.setChatFilter(null)}
                highlighted={true}
                aria-label={translate("menu.chat.clearFilter")}
            >
                <Icon>filter_alt_off</Icon>
            </Button>
        </div>}
        <ChatMessageSection filter={filter}/>
        {sendChatGroups === undefined || <>
            <div className="chat-menu-icons">
                {!sendChatGroups.includes("all") && translate("noAll.icon")}
                {sendChatGroups.map((group) => {
                    return translate("chatGroup."+group+".icon");
                })}
            </div>
            <ChatTextInput disabled={sendChatGroups.length === 0}/>
        </>}
    </div>
}

export type ChatFilter = {
    type: "playerNameInMessage",
    player: PlayerIndex
} | {
    type: "myWhispersWithPlayer",
    player: PlayerIndex,
} | null;

export function ChatMessageSection(props: Readonly<{
    filter?: ChatFilter,
}>): ReactElement {
    const gameState = useContext(StateContext)!;

    const players = gameState!.players;
    const filter = useMemo(() => props.filter ?? null, [props.filter]);

    const myPlayerIndex = usePlayerState()?.myIndex;


    const allMessages = gameState.chatMessages
        .filter((msg)=>{
            if(filter === null || filter === undefined)
                return true;

            switch(filter.type){
                case "playerNameInMessage":
                    let msgTxt = "";
                    //special case messages, where translate chat message doesnt work properly, or it should be let through anyway
                    switch (msg.variant.type) {
                        //translateChatMessage errors for playerDied type.
                        case "playerDied":
                        case "phaseChange":
                            return true
                        case "normal":
                            switch(msg.variant.messageSender.type) {
                                case "player":
                                case "livingToDead":
                                    if(msg.variant.messageSender.player === filter.player)
                                        return true;
                                    break;
                            }
                            break;
                        case "targetsMessage":
                            msgTxt = translateChatMessage(msg.variant.message, players.map(player => player.toString()));
                            break;
                    }

                    msgTxt += translateChatMessage(msg.variant, players.map(player => player.toString()));
                    
                    return msgTxt.includes(players[filter.player].toString());
                case "myWhispersWithPlayer":
                    switch (msg.variant.type) {
                        //translateChatMessage errors for playerDied type.
                        case "phaseChange":
                            return true
                        case "whisper":
                            if(
                                (msg.variant.fromPlayerIndex === filter.player && msg.variant.toPlayerIndex === myPlayerIndex) ||
                                (msg.variant.toPlayerIndex === filter.player && msg.variant.fromPlayerIndex === myPlayerIndex)
                            )
                                return true;
                            else
                                return false;
                        default:
                            return false;
                    }
            }
            return true;
        }).filter((msg, index, array)=>{
            //if there is a filter, remove repeat phaseChange message
            if(filter === null){return true}
            if(msg.variant.type !== "phaseChange"){return true}
            if(index+1===array.length){return true}
            if(array[index+1].variant.type !== "phaseChange"){return true}
            return false;
        }).map((msg, index) => {
            return <ChatElement
                key={index}
                message={msg}
                playerKeywordData={(() => {
                    if (filter===null) {return undefined}
                    if (filter.type === "myWhispersWithPlayer") {return undefined}

                    const newKeywordData: KeywordDataMap = {...PLAYER_KEYWORD_DATA};

                    newKeywordData[players[filter.player].toString()] = [
                        { style: "keyword-player-important keyword-player-number", replacement: (filter.player + 1).toString() },
                        { replacement: " " },
                        { style: "keyword-player-important keyword-player-sender", replacement: players[filter.player].name }
                    ];
                    
                    return newKeywordData;
                })()}
                playerSenderKeywordData={(() => {
                    if (filter===null) {return undefined}
                    if (filter.type === "myWhispersWithPlayer") {return undefined}

                    const newKeywordData: KeywordDataMap = {...PLAYER_SENDER_KEYWORD_DATA};

                    newKeywordData[players[filter.player].toString()] = [
                        { style: "keyword-player-important keyword-player-number", replacement: (filter.player + 1).toString() },
                        { replacement: " " },
                        { style: "keyword-player-important keyword-player-sender", replacement: players[filter.player].name }
                    ];
                    
                    return newKeywordData;
                })()}
            />;
        })

    return <div className="chat-message-section"><Virtuoso
        alignToBottom={true}
        totalCount={allMessages.length}
        followOutput={true}
        initialTopMostItemIndex={allMessages.length===0 ? 0 : allMessages.length-1}
        itemContent={(index) => allMessages[index]}
        atBottomThreshold={15}
    /></div>;
}

export function ChatTextInput(props: Readonly<{
    disabled?: boolean,
    whispering?: PlayerIndex | null,
    inLobby?: boolean
}>): ReactElement {
    const gameState = useContext(StateContext)!;
    const {sendSendWhisperPacket, sendSendChatMessagePacket, sendSendLobbyMessagePacket} = useContext(WebsocketContext)!;
    
    const [chatBoxText, setChatBoxText] = useState<string>("");
    const [drawAttentionSeconds, setDrawAttentionSeconds] = useState<number>(0);
    const ref = useRef<HTMLTextAreaElement>(null);
    const [whisperingState, setWhispering] = useState<PlayerIndex | null>(null);

    const whispering = useMemo(() => {
        if (props.whispering === undefined) {
            return whisperingState;
        } else {
            return props.whispering;
        }
    }, [props.whispering, whisperingState]);

    const gamePlayers = gameState.players;
    const myIndex = usePlayerState()?.myIndex;

    const playerStrings = usePlayerNamesToString()!;

    const whisperingPlayer = useMemo(() => {
        return whispering!==null ? playerStrings[whispering] : null
    }, [playerStrings, whispering])
    
    const prependWhisper = useCallback((index: PlayerIndex) => {
        if (gamePlayers !== undefined && index < gamePlayers.length && index !== myIndex) {
            setWhispering(index);
            setDrawAttentionSeconds(1.5);
            ref.current?.focus()
        }
    }, [gamePlayers, myIndex]);

    useEffect(() => {
        if (drawAttentionSeconds === 0) {
            return;
        } else if (drawAttentionSeconds < 0) {
            setDrawAttentionSeconds(0);
        } else {
            setTimeout(() => {
                setDrawAttentionSeconds(drawAttentionSeconds - 0.5);
            }, 500)
        }
    }, [drawAttentionSeconds])

    useEffect(() => {
        gameState.setPrependWhisperFunction(prependWhisper);
        return () => gameState.setPrependWhisperFunction(() => {});
    }, [prependWhisper, gameState]);


    const history: HistoryQueue<string> = useMemo(() => new HistoryQueue(40), []);
    const historyPoller: HistoryPoller<string> = useMemo(() => new HistoryPoller(), []);


    const sendChatField = useCallback(() => {
        let text = chatBoxText.trim();
        setWhispering(null);
        setChatBoxText("");
        if (text === "") return;
        history.push(text);
        historyPoller.reset();
        if (props.inLobby === false || props.inLobby === undefined) {
            if (whispering !== null) {
                sendSendWhisperPacket(whispering, text);
            } else {
                sendSendChatMessagePacket(text, false);
            }
        } else {
            sendSendLobbyMessagePacket(text);
        }
    }, [chatBoxText, history, historyPoller, props.inLobby, whispering]);

    const handleInputChange = useCallback((event: React.ChangeEvent<HTMLTextAreaElement>) => {
        const text = event.target.value;
        const whisperCommandMatch = RegExp(/\/w(\d+) /).exec(text);
        if (whispering === null && whisperCommandMatch !== null) {
            const index = parseInt(whisperCommandMatch[1]) - 1;
            if (gamePlayers !== undefined && index < gamePlayers.length && index >= 0 && index !== myIndex) {
                setWhispering(index);
                setChatBoxText(text.slice(whisperCommandMatch[0].length));
            } else {
                setWhispering(null);
                setChatBoxText(text);
            }
        } else {
            setChatBoxText(text);
        }
    }, [gamePlayers, myIndex, whispering]);

    const handleInputKeyDown = useCallback((event: React.KeyboardEvent<HTMLTextAreaElement>) => {
        
        //if press enter while holding shift
        if(event.key === "Enter" && event.shiftKey){
            event.preventDefault();
            setChatBoxText(chatBoxText+"\n");
        } else if (event.key === "Enter") {
            event.preventDefault();
            sendChatField();
        } else if (event.key === "ArrowUp") {
            event.preventDefault();
            const text = historyPoller.poll(history);
            if (text !== undefined) 
                setChatBoxText(text);
        } else if (event.key === "ArrowDown") {
            event.preventDefault();
            const text = historyPoller.pollPrevious(history);
            setChatBoxText(text ?? "");
        } else if (event.key === "Escape") {
            event.preventDefault();
            setWhispering(null);
        }
    }, [sendChatField, history, historyPoller, chatBoxText]);

    return <>
        {whisperingPlayer !== null && <div className="chat-whisper-notification">
            <StyledText className="discreet">{translate("youAreWhispering", whisperingPlayer)}</StyledText>
            {props.whispering === undefined ? <Button
                highlighted={true}
                onClick={() => setWhispering(null)}
            >
                {translate("cancelWhisper")}
            </Button>:null}
        </div>}
        <div className="chat-send-section">
            <textarea
                className={drawAttentionSeconds * 2 % 2 === 1 ? "highlighted" : undefined}
                ref={ref}
                value={chatBoxText}
                onChange={handleInputChange}
                onKeyDown={handleInputKeyDown}
            />
            <Button 
                disabled={props.disabled}
                onClick={sendChatField}
                aria-label={translate("menu.chat.button.send")}
            >
                <Icon>send</Icon>
            </Button>
        </div>
    </>
}
