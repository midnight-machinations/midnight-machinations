import type React from "react";
import { type ReactElement, useCallback, useEffect, useMemo, useRef, useState } from "react";
import translate from "../../../game/lang";
import GAME_MANAGER from "../../../index";
import "../gameScreen.css";
import "./chatMenu.css";
import { Virtuoso } from "react-virtuoso";
import { Button } from "../../../components/Button";
import ChatGroupSelect from "../../../components/ChatGroupSelect";
import ChatElement, { encodeString } from "../../../components/ChatMessage";
import Icon from "../../../components/Icon";
import StyledText, {
    type KeywordDataMap,
    PLAYER_KEYWORD_DATA,
    PLAYER_SENDER_KEYWORD_DATA,
    ROLE_LIST_KEYWORD_DATA,
} from "../../../components/StyledText";
import { useGameState, useLobbyOrGameState, usePlayerNames, usePlayerState } from "../../../components/useHooks";
import { type ChatFilter, filterChatMessages } from "../../../game/chatFilter";
import { controllerIdToLinkWithPlayer } from "../../../game/controllerInput";
import type { PlayerClientType, PlayerIndex } from "../../../game/gameState.d";
import { HistoryPoller, HistoryQueue } from "../../../history";
import ListMap from "../../../ListMap";
import LobbyVoicePrompt from "../../../voice/LobbyVoicePrompt";
import { SelfVoiceMuteButton } from "../../../voice/SpeakingIndicator";
import { ContentMenu, ContentTab } from "../GameScreen";
import ChatFilters from "./ChatFilters";

export default function ChatMenu(): ReactElement {
    const filter = useGameState((gameState) => gameState.chatFilter, ["filterUpdate"]);

    const playerNames = usePlayerNames();
    const controllers = new ListMap(
        usePlayerState(
            (playerState) => playerState.savedControllers,
            ["yourAllowedControllers", "yourAllowedController"],
        ),
        (k1, k2) => controllerIdToLinkWithPlayer(k1) === controllerIdToLinkWithPlayer(k2),
    );

    const filterActive = filter !== undefined && filter !== null;

    return (
        <div className="chat-menu chat-menu-colors">
            <ContentTab close={ContentMenu.ChatMenu}>{translate("menu.chat.title")}</ContentTab>
            <div className={"chat-voice-header" + (filterActive ? " highlighted" : "")}>
                <ChatFilters filter={filter ?? null} />
                <div className="chat-header-voice-controls">
                    <LobbyVoicePrompt />
                    <SelfVoiceMuteButton />
                </div>
            </div>
            <ChatMessageSection filter={filter} />
            {controllers.list.map(([id, _]) => {
                if (id.type !== "chat") {
                    return null;
                }

                const sendChatController = controllers.get({ type: "sendChat", player: id.player });
                if (sendChatController === null) {
                    return null;
                }
                const groupController = controllers.get({ type: "chatGroup", player: id.player });
                if (
                    groupController?.parameters.available.type !== "chatGroup" ||
                    groupController.selection.type !== "chatGroup"
                )
                    return null;
                const sendChatGroups = groupController.parameters.available.selection.groups;

                return (
                    <div className="brand chat-menu-chat-controller" key={JSON.stringify(id)}>
                        <div key={"header: " + JSON.stringify(id)} className="chat-menu-icons">
                            {!sendChatGroups.includes("all") && (
                                <StyledText noLinks={true}>{translate("noAll.icon")}</StyledText>
                            )}
                            {sendChatGroups.map((group) => {
                                return (
                                    <StyledText key={group} noLinks={true}>
                                        {translate("chatGroup." + group + ".icon")}
                                    </StyledText>
                                );
                            })}
                            <StyledText>{encodeString(playerNames[id.player])}</StyledText>
                            <ChatGroupSelect
                                id={{ type: "chatGroup", player: id.player }}
                                groups={sendChatGroups}
                                selected={groupController.selection.selection}
                                disabled={groupController.parameters.grayedOut}
                            />
                        </div>
                        <ChatTextInput
                            key={"input: " + JSON.stringify(id)}
                            disabled={sendChatController.parameters.grayedOut}
                            controllingPlayer={id.player}
                        />
                    </div>
                );
            })}
        </div>
    );
}

export function ChatMessageSection(
    props: Readonly<{
        filter?: ChatFilter;
    }>,
): ReactElement {
    const players =
        useGameState(
            (gameState) => {
                return gameState.players;
            },
            ["gamePlayers"],
        ) ?? [];
    const filter = useMemo(() => props.filter ?? null, [props.filter]);
    const messages = useLobbyOrGameState((state) => state.chatMessages.entries(), ["addChatMessages"])!;
    const roleList = useLobbyOrGameState((gameState) => gameState.roleList, ["roleList"]) ?? [];

    const names = usePlayerNames();
    const plainNames = players.length > 0 ? players.map((player) => player.name) : names;
    const allMessages = filterChatMessages(filter, messages, names, roleList, plainNames).map((msg) => {
        return (
            <ChatElement
                key={msg[0]}
                messageIndex={msg[0]}
                message={msg[1]}
                playerKeywordData={(() => {
                    if (filter?.type !== "playerNameInMessage" || !players[filter.player]) return undefined;

                    const newKeywordData: KeywordDataMap = { ...PLAYER_KEYWORD_DATA };

                    newKeywordData[encodeString(players[filter.player].toString())] = [
                        {
                            style: "keyword-player-important keyword-player-number",
                            replacement: (filter.player + 1).toString(),
                        },
                        { replacement: " " },
                        {
                            style: "keyword-player-important keyword-player-sender",
                            replacement: encodeString(players[filter.player].name),
                        },
                    ];

                    return newKeywordData;
                })()}
                playerSenderKeywordData={(() => {
                    if (filter?.type !== "playerNameInMessage" || !players[filter.player]) return undefined;

                    const newKeywordData: KeywordDataMap = { ...PLAYER_SENDER_KEYWORD_DATA };

                    newKeywordData[encodeString(players[filter.player].toString())] = [
                        {
                            style: "keyword-player-important keyword-player-number",
                            replacement: (filter.player + 1).toString(),
                        },
                        { replacement: " " },
                        {
                            style: "keyword-player-important keyword-player-sender",
                            replacement: encodeString(players[filter.player].name),
                        },
                    ];

                    return newKeywordData;
                })()}
                roleListKeywordData={ROLE_LIST_KEYWORD_DATA}
            />
        );
    });

    return (
        <div className="chat-message-section">
            <Virtuoso
                alignToBottom={true}
                totalCount={allMessages.length}
                followOutput={true}
                initialTopMostItemIndex={allMessages.length === 0 ? 0 : allMessages.length - 1}
                itemContent={(index) => allMessages[index]}
                atBottomThreshold={15}
            />
        </div>
    );
}

export function ChatTextInput(
    props: Readonly<{
        disabled?: boolean;
        whispering?: PlayerIndex | null;
        controllingPlayer?: PlayerIndex;
        hidden?: boolean;
    }>,
): ReactElement {
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

    const gamePlayers = useGameState((gameState) => gameState.players, ["gamePlayers"]);
    const stateType = useLobbyOrGameState(
        (state) => state.stateType,
        ["acceptJoin", "gameInitializationComplete", "startGame", "backToLobby"],
    )!;
    const playerStrings = useLobbyOrGameState((state) => {
        if (state.stateType === "game") {
            return state.players.map((player) => player.toString());
        } else if (state.stateType === "lobby") {
            return Array.from(state.players.values())
                .filter((player) => player.clientType.type === "player")
                .map((player) => (player.clientType as PlayerClientType).name);
        }
    })!;

    const whisperingPlayerName = useMemo(() => {
        return whispering !== null ? playerStrings[whispering] : null;
    }, [playerStrings, whispering]);

    const prependWhisper = useCallback(
        (index: PlayerIndex) => {
            if (gamePlayers !== undefined && index < gamePlayers.length && index !== props.controllingPlayer) {
                setWhispering(index);
                setDrawAttentionSeconds(1.5);
                ref.current?.focus();
            }
        },
        [gamePlayers, props.controllingPlayer],
    );

    useEffect(() => {
        if (drawAttentionSeconds === 0) {
            return;
        } else if (drawAttentionSeconds < 0) {
            setDrawAttentionSeconds(0);
        } else {
            setTimeout(() => {
                setDrawAttentionSeconds(drawAttentionSeconds - 0.5);
            }, 500);
        }
    }, [drawAttentionSeconds]);

    useEffect(() => {
        GAME_MANAGER.setPrependWhisperFunction(prependWhisper);
        return () => GAME_MANAGER.setPrependWhisperFunction(() => {});
    }, [prependWhisper]);

    const history: HistoryQueue<string> = useMemo(() => new HistoryQueue(40), []);
    const historyPoller: HistoryPoller<string> = useMemo(() => new HistoryPoller(), []);

    const sendChatField = useCallback(() => {
        const text = chatBoxText.trim();
        setWhispering(null);
        setChatBoxText("");
        if (text === "") return;
        history.push(text);
        historyPoller.reset();
        if (stateType === "game") {
            if (whispering !== null) {
                GAME_MANAGER.sendSendWhisperPacket(whispering, text, props.controllingPlayer);
            } else {
                GAME_MANAGER.sendSendChatMessagePacket(text, false, props.controllingPlayer);
            }
        } else if (stateType === "lobby") {
            GAME_MANAGER.sendSendLobbyMessagePacket(text);
        }
    }, [chatBoxText, history, historyPoller, stateType, whispering, props.controllingPlayer]);

    const handleInputChange = useCallback(
        (event: React.ChangeEvent<HTMLTextAreaElement>) => {
            const text = event.target.value;
            const whisperCommandMatch = RegExp(/\/w(\d+) /).exec(text);
            if (whispering === null && whisperCommandMatch !== null) {
                const index = parseInt(whisperCommandMatch[1]) - 1;
                if (
                    gamePlayers !== undefined &&
                    index < gamePlayers.length &&
                    index >= 0 &&
                    index !== props.controllingPlayer
                ) {
                    setWhispering(index);
                    setChatBoxText(text.slice(whisperCommandMatch[0].length));
                } else {
                    setWhispering(null);
                    setChatBoxText(text);
                }
            } else {
                setChatBoxText(text);
            }
        },
        [gamePlayers, props.controllingPlayer, whispering],
    );

    const handleInputKeyDown = useCallback(
        (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
            //if press enter while holding shift
            if (event.key === "Enter" && event.shiftKey) {
                event.preventDefault();
                setChatBoxText(chatBoxText + "\n");
            } else if (event.key === "Enter") {
                event.preventDefault();
                sendChatField();
            } else if (event.key === "ArrowUp") {
                event.preventDefault();
                const text = historyPoller.poll(history);
                if (text !== undefined) setChatBoxText(text);
            } else if (event.key === "ArrowDown") {
                event.preventDefault();
                const text = historyPoller.pollPrevious(history);
                setChatBoxText(text ?? "");
            } else if (event.key === "Escape") {
                event.preventDefault();
                setWhispering(null);
            }
        },
        [sendChatField, history, historyPoller, chatBoxText],
    );

    const myIndex = usePlayerState((playerState, _) => playerState.myIndex, ["yourPlayerIndex"]);
    const sendingPlayer = props.controllingPlayer ?? myIndex ?? null;
    const sendingPlayerName = useMemo(() => {
        return sendingPlayer !== null ? playerStrings[sendingPlayer] : null;
    }, [playerStrings, sendingPlayer]);

    if (sendingPlayer !== null && sendingPlayer === whispering) {
        return (
            <div className="chat-send-section">
                <StyledText className="chat-message special">{translate("cannotWhisperYourself")}</StyledText>
            </div>
        );
    }

    return (
        <>
            {whisperingPlayerName !== null && (
                <div className="chat-whisper-notification" hidden={props.hidden}>
                    {sendingPlayerName !== null ? (
                        <StyledText className="discreet">
                            {translate(
                                "playerIsWhisperingToPlayer",
                                encodeString(sendingPlayerName),
                                encodeString(whisperingPlayerName),
                            )}
                        </StyledText>
                    ) : null}
                    {props.whispering === undefined ? (
                        <Button highlighted={true} onClick={() => setWhispering(null)}>
                            {translate("cancelWhisper")}
                        </Button>
                    ) : null}
                </div>
            )}
            <div className="chat-send-section" hidden={props.hidden}>
                <textarea
                    className={(drawAttentionSeconds * 2) % 2 === 1 ? "highlighted" : undefined}
                    ref={ref}
                    value={chatBoxText}
                    onChange={handleInputChange}
                    onKeyDown={handleInputKeyDown}
                    placeholder={translate("menu.chat.placeHolder")}
                />
                <Button
                    className="flush"
                    disabled={props.disabled}
                    onClick={sendChatField}
                    aria-label={translate("menu.chat.button.send")}
                >
                    <Icon>send</Icon>
                </Button>
            </div>
        </>
    );
}
