import React, { ReactElement, useContext } from "react";
import translate from "../../../game/lang";
import "./playerListMenu.css"
import "./../gameScreen.css"
import StyledText from "../../../components/StyledText";
import Icon from "../../../components/Icon";
import { Button } from "../../../components/Button";
import PlayerNamePlate from "../../../components/PlayerNamePlate";
import ChatMessage, { translateChatMessage } from "../../../components/ChatMessage";
import GraveComponent, { translateGraveRole } from "../../../components/grave";
import { ChatMessageSection, ChatTextInput } from "./ChatMenu";
import GameScreenMenuTab from "../GameScreenMenuTab";
import { GameScreenMenuType } from "../GameScreenMenuContext";
import { usePlayerNames, usePlayerState } from "../../../stateContext/useHooks";
import { PlayerIndex } from "../../../stateContext/stateType/otherState";
import { StateContext } from "../../../stateContext/StateContext";

export default function PlayerListMenu(): ReactElement {
    const stateCtx = useContext(StateContext)!;
    const players = stateCtx.players;
    const graves = stateCtx.graves;


    return <div className="player-list-menu player-list-menu-colors">
        <GameScreenMenuTab close={GameScreenMenuType.PlayerListMenu} helpMenu={"standard/playerList"}>{translate("menu.playerList.title")}</GameScreenMenuTab>

        <div className="player-list">
            {players
                .filter(player => player.alive)
                .map(player => <div key={player.index} className="player-card-holder"><PlayerCard playerIndex={player.index}/></div>)
            }

            {graves.length === 0 || 
                <>
                    <div className="dead-players-separator">
                        <StyledText>{translate("grave.icon")} {translate("graveyard")}</StyledText>
                    </div>
                    {graves.map((grave, index) => <div key={grave.player} className="player-card-holder"><PlayerCard graveIndex={index} playerIndex={grave.player}/></div>)}
                </>
            }

            {players
                .filter(
                    player => !player.alive && 
                    graves.find(grave => grave.player === player.index) === undefined
                ).length === 0 || 
                <>
                    <div className="dead-players-separator">
                        <StyledText>{translate("grave.icon")} {translate("graveyard")}</StyledText>
                    </div>
                    {players
                        .filter(player => !player.alive)
                        .map(player => <div key={player.index} className="player-card-holder"><PlayerCard playerIndex={player.index}/></div>)
                    }
                </>
            }
        </div>
    </div>
}

function PlayerCard(props: Readonly<{
    graveIndex?: number,
    playerIndex: number
}>): ReactElement{
    const [alibiOpen, setAlibiOpen] = React.useState(false);
    const [graveOpen, setGraveOpen] = React.useState(false);
    const [whisperChatOpen, setWhisperChatOpen] = React.useState(false);

    const stateCtx = useContext(StateContext)!;
    const playerState = usePlayerState();

    const playerAlive = stateCtx.players[props.playerIndex].alive;
    const numVoted = stateCtx.players[props.playerIndex].numVoted
    const phaseState = stateCtx.phaseState;

    let isPlayerSelf = false;
    let chatFilter = undefined;
    let sendChatGroups = undefined;
    let whisperNotification = false;
    if(playerState !== undefined){
        isPlayerSelf = playerState.myIndex === props.playerIndex;
        chatFilter = playerState?.chatFilter;
        sendChatGroups = playerState?.sendChatGroups;
        
        whisperNotification = playerState.missedWhispers.some(player => player === props.playerIndex) &&
            !isPlayerSelf &&
            !whisperChatOpen;
    }

    const playerNames = usePlayerNames()!;

    type NonAnonymousBlockMessage = {
        variant: {
            type: "normal", 
            messageSender: {
                type: "player", 
                player: PlayerIndex
            } | {
                type: "livingToDead",
                player: PlayerIndex,
            },
            text: string,
            block: true
        }
        chatGroup: "all"
    }

    const mostRecentBlockMessage: undefined | NonAnonymousBlockMessage = findLast(
        stateCtx.chatMessages,
        message => message.chatGroup === "all" && 
            message.variant.type === "normal" &&
            message.variant.block &&
            (message.variant.messageSender.type === "player" || message.variant.messageSender.type === "livingToDead") &&
            message.variant.messageSender.player === props.playerIndex
    ) as undefined | NonAnonymousBlockMessage;
    

    const whispersDisabled = stateCtx.enabledModifiers.includes("noWhispers");

    const grave = props.graveIndex === undefined?undefined:stateCtx.graves[props.graveIndex];

    const spectator = stateCtx.clientState.type === "spectator";

    return <><div 
        className={`player-card`}
        key={props.playerIndex}
    >
        <PlayerNamePlate playerIndex={props.playerIndex}/>
        
        {mostRecentBlockMessage !== undefined ? 
            <Button onClick={()=>setAlibiOpen(!alibiOpen)}>
                <StyledText noLinks={true}>
                    {
                        translateChatMessage(mostRecentBlockMessage.variant, playerNames)
                            .split("\n")[1]
                            .trim()
                            .substring(0,30)
                            .trim()
                    }
                </StyledText>
            </Button>
        : null}
        {grave !== undefined ? 
            <Button onClick={()=>setGraveOpen(!graveOpen)}>
                <StyledText noLinks={true}>
                    {translateGraveRole(grave.information)} {translate(grave.diedPhase+".icon")}{grave.dayNumber.toString()}
                </StyledText>
            </Button>
        : null}
        
        {
            phaseState.type === "nomination" && playerAlive && 
            <StyledText>{translate("menu.playerList.player.votes", numVoted)}</StyledText>
        }
        {spectator ||
            <Button 
                disabled={isPlayerSelf || whispersDisabled}
                onClick={()=>{
                    // GAME_MANAGER.prependWhisper(props.playerIndex); return true;
                    setWhisperChatOpen(!whisperChatOpen);
                    if(playerState !== undefined){
                        playerState.missedWhispers = playerState.missedWhispers.filter(player => player !== props.playerIndex);
                    }
                }}
                pressedChildren={() => <Icon>done</Icon>}
            >
                {whisperChatOpen===true?<Icon>close</Icon>:<Icon>chat</Icon>}
                {whisperNotification===true && <div className="chat-notification highlighted">!</div>}
            </Button>
        }
        {spectator || (() => {
            const filter = props.playerIndex;
            const isFilterSet = chatFilter?.type === "playerNameInMessage" && (chatFilter.player === filter);
            
            return <Button 
                className={"filter"} 
                highlighted={isFilterSet}
                onClick={() => {
                    stateCtx.setChatFilter(isFilterSet ? null : {
                        type: "playerNameInMessage",
                        player: filter,
                    });
                    return true;
                }}
                pressedChildren={result => <Icon>{result ? "done" : "warning"}</Icon>}
                aria-label={translate("menu.playerList.button.filter")}
            >
                <Icon>filter_alt</Icon>
            </Button>
        })()}
    </div>
    {alibiOpen && mostRecentBlockMessage !== undefined ? <div onClick={()=>setAlibiOpen(false)}>
        <ChatMessage message={mostRecentBlockMessage}/>
    </div> : null}
    {graveOpen && grave !== undefined ? <div onClick={()=>setGraveOpen(false)}>
        <GraveComponent grave={grave}/>
    </div> : null}
    {(whisperChatOpen && !isPlayerSelf) && <div className="chat-menu-colors player-list-chat-section">
        <div className="player-list-chat-message-section">
            <ChatMessageSection filter={{
                type: "myWhispersWithPlayer",
                player: props.playerIndex
            }}/>
        </div>
        {sendChatGroups === undefined || <ChatTextInput
            disabled={sendChatGroups.length === 0}
            whispering={props.playerIndex}
        />}
    </div>}
    </>
}

function findLast<T>(array: T[], predicate: (e: T, i: number, array: T[])=>boolean): T | undefined {
    for(let i = array.length - 1; i >= 0; i--) 
        if(predicate( array[i], i, array )) return array[i];
    return undefined;
};