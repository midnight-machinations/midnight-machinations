import { ReactElement } from "react";
import { RoleState } from "../../../../../game/roleState.d";
import CheckBox from "../../../../../components/CheckBox";
import ChatElement from "../../../../../components/ChatMessage";
import StyledText from "../../../../../components/StyledText";
import React from "react";
import translate from "../../../../../game/lang";
import GAME_MANAGER from "../../../../..";
import "./largeHypnotistMenu.css"

export type Hypnotist = {
    roleblock: boolean,
    
    youWereRoleblockedMessage: boolean,
    youSurvivedAttackMessage: boolean,
    youWereGuardedMessage: boolean,
    youWereTransportedMessage: boolean,
    youWerePossessedMessage: boolean,
    youWereWardblockedMessage: boolean
}

export default function HypnotistMenu(props: {roleState: RoleState & {type: "hypnotist"}}): ReactElement{
    
    function handleRoleblockToggle(){
        GAME_MANAGER.sendSetConsortOptions(
            !props.roleState.roleblock, 
            props.roleState.youWereRoleblockedMessage, 
            props.roleState.youSurvivedAttackMessage, 
            props.roleState.youWereGuardedMessage, 
            props.roleState.youWereTransportedMessage, 
            props.roleState.youWerePossessedMessage, 
            props.roleState.youWereWardblockedMessage
        );
    }
    function handleYouWereRoleblockedMessageToggle(){
        GAME_MANAGER.sendSetConsortOptions(
            props.roleState.roleblock, 
            !props.roleState.youWereRoleblockedMessage, 
            props.roleState.youSurvivedAttackMessage, 
            props.roleState.youWereGuardedMessage, 
            props.roleState.youWereTransportedMessage, 
            props.roleState.youWerePossessedMessage, 
            props.roleState.youWereWardblockedMessage
        );
    }
    function handleYouSurvivedAttackMessageToggle(){
        GAME_MANAGER.sendSetConsortOptions(
            props.roleState.roleblock, 
            props.roleState.youWereRoleblockedMessage, 
            !props.roleState.youSurvivedAttackMessage, 
            props.roleState.youWereGuardedMessage, 
            props.roleState.youWereTransportedMessage, 
            props.roleState.youWerePossessedMessage, 
            props.roleState.youWereWardblockedMessage
        );
    }
    function handleYouWereGuardedMessageToggle(){
        GAME_MANAGER.sendSetConsortOptions(
            props.roleState.roleblock, 
            props.roleState.youWereRoleblockedMessage, 
            props.roleState.youSurvivedAttackMessage, 
            !props.roleState.youWereGuardedMessage, 
            props.roleState.youWereTransportedMessage, 
            props.roleState.youWerePossessedMessage, 
            props.roleState.youWereWardblockedMessage
        );
    }
    function handleYouWereTransportedMessageToggle(){
        GAME_MANAGER.sendSetConsortOptions(
            props.roleState.roleblock, 
            props.roleState.youWereRoleblockedMessage, 
            props.roleState.youSurvivedAttackMessage, 
            props.roleState.youWereGuardedMessage, 
            !props.roleState.youWereTransportedMessage, 
            props.roleState.youWerePossessedMessage, 
            props.roleState.youWereWardblockedMessage
        );
    }
    function handleYouWerePossessedMessageToggle(){
        GAME_MANAGER.sendSetConsortOptions(
            props.roleState.roleblock, 
            props.roleState.youWereRoleblockedMessage, 
            props.roleState.youSurvivedAttackMessage, 
            props.roleState.youWereGuardedMessage, 
            props.roleState.youWereTransportedMessage, 
            !props.roleState.youWerePossessedMessage, 
            props.roleState.youWereWardblockedMessage
        );
    }
    function handleYourTargetWasJailedMessageToggle(){
        GAME_MANAGER.sendSetConsortOptions(
            props.roleState.roleblock, 
            props.roleState.youWereRoleblockedMessage, 
            props.roleState.youSurvivedAttackMessage, 
            props.roleState.youWereGuardedMessage, 
            props.roleState.youWereTransportedMessage, 
            props.roleState.youWerePossessedMessage, 
            !props.roleState.youWereWardblockedMessage
        );
    }

    return <div className="large-hypnotist-menu">
        <div>
            <StyledText>
                {translate("wiki.article.standard.roleblock.title")}
            </StyledText>
            <CheckBox checked={props.roleState.roleblock} onChange={(checked)=>{handleRoleblockToggle()}}/>
        </div>
        <div>
            <ChatElement message={{
                variant: {
                    type: "roleBlocked",
                },
                chatGroup:null
            }}/>
            <CheckBox checked={props.roleState.youWereRoleblockedMessage} onChange={(checked)=>handleYouWereRoleblockedMessageToggle()}/>
        </div>
        <div>
            <ChatElement message={{
                variant: {
                    type: "youSurvivedAttack",
                },
                chatGroup:null
            }}/>
            <CheckBox checked={props.roleState.youSurvivedAttackMessage} onChange={(checked)=>handleYouSurvivedAttackMessageToggle()}/>
        </div>
        <div>
            <ChatElement message={{
                variant: {
                    type: "youWereGuarded",
                },
                chatGroup:null
            }}/>
            <CheckBox checked={props.roleState.youWereGuardedMessage} onChange={(checked)=>handleYouWereGuardedMessageToggle()}/>
        </div>
        <div>
            <ChatElement message={{
                variant: {
                    type: "transported",
                },
                chatGroup:null
            }}/>
            <CheckBox checked={props.roleState.youWereTransportedMessage} onChange={(checked)=>handleYouWereTransportedMessageToggle()}/>
        </div>
        <div>
            <ChatElement message={{
                variant: {
                    type: "youWerePossessed",
                    immune: false,
                },
                chatGroup:null
            }}/>
            <ChatElement message={{
                variant: {
                    type: "youWerePossessed",
                    immune: true,
                },
                chatGroup:null
            }}/>
            <CheckBox checked={props.roleState.youWerePossessedMessage} onChange={(checked)=>handleYouWerePossessedMessageToggle()}/>
        </div>
        <div>
            <ChatElement message={{
                variant: {
                    type: "wardblocked",
                },
                chatGroup:null
            }}/>
            <CheckBox checked={props.roleState.youWereWardblockedMessage} onChange={(checked)=>handleYourTargetWasJailedMessageToggle()}/>
        </div>            
    </div>
}