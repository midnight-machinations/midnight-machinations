import React, { useMemo } from "react"
import { ReactElement } from "react"
import translate from "../game/lang"
import StyledText, { KeywordDataMap, PLAYER_KEYWORD_DATA } from "./StyledText"
import { useGameState, usePlayerState } from "./useHooks"
import "./playerNamePlate.css"
import { encodeString } from "./ChatMessage"

export default function PlayerNamePlate(props: Readonly<{
    playerIndex: number,    //guarantee this index is valid please
}>): ReactElement {

        const phaseState = useGameState(
            (gameState) => gameState.phaseState,
            ["phase"]
        )!;
        const myRoleState = usePlayerState(
            (playerState) => playerState.roleState,
            ["yourRoleState"]
        );
        const myIndex = usePlayerState(
            (gameState) => gameState.myIndex,
            ["yourPlayerIndex"]
        )!;
        const playerRoleLabel = useGameState(
            (gameState) => gameState.players[props.playerIndex].roleLabel,
            ["gamePlayers", "yourRoleLabels"]
        );
        const playerTags = useGameState(
            (gameState) => gameState.players[props.playerIndex].playerTags,
            ["gamePlayers", "yourPlayerTags"]
        )!;
        const playerAlive = useGameState(
            (gameState) => gameState.players[props.playerIndex].alive,
            ["gamePlayers", "playerAlive"]
        )!;
        const playerNameToString = useGameState(
            (gameState) => gameState.players[props.playerIndex].toString(),
            ["gamePlayers"]
        )!;
        const playerName = useGameState(
            (gameState) => gameState.players[props.playerIndex].name,
            ["gamePlayers"]
        )!;

        const roleString = useMemo(()=>{
            if(props.playerIndex === myIndex){
                return ("("+translate("role."+myRoleState?.type+".name")+")");
            }
    
            if(playerAlive && playerRoleLabel !== null){
                return ("("+translate("role."+playerRoleLabel+".name")+")");
            }

            return "";
        }, [props.playerIndex, myIndex, myRoleState, playerAlive, playerRoleLabel]);



        const newKeywordData: KeywordDataMap = {...PLAYER_KEYWORD_DATA};
        if(myIndex === props.playerIndex){
            newKeywordData[encodeString(playerNameToString)] = [
                { style: "keyword-player-important keyword-player-number", replacement: (myIndex + 1).toString() },
                { replacement: " " },
                { style: "keyword-player-important keyword-player-sender", replacement: encodeString(playerName) }
            ];
        }




        return <div 
            className="player-name-plate"
            key={props.playerIndex}
        >
            {(() => {
                if (phaseState.type === "testimony" || phaseState.type === "judgement" || phaseState.type === "finalWords") {
                    if (phaseState.playerOnTrial === props.playerIndex) {
                        return <StyledText>{translate("trial.icon")} </StyledText>
                    }
                }
            })()}
            <StyledText playerKeywordData={newKeywordData}>{encodeString(playerNameToString)}</StyledText>
            {roleString !== null && <StyledText> {roleString}</StyledText>}
            <StyledText>{playerTags.map((tag)=>{return translate("tag."+tag)})}</StyledText>
        </div>
}