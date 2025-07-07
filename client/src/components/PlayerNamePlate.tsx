import React, { useContext, useMemo } from "react"
import { ReactElement } from "react"
import translate from "../game/lang"
import StyledText, { KeywordDataMap, PLAYER_KEYWORD_DATA } from "./StyledText"
import "./playerNamePlate.css"
import { usePlayerState } from "../stateContext/useHooks"
import { StateContext } from "../stateContext/StateContext"

export default function PlayerNamePlate(props: Readonly<{
    playerIndex: number,    //guarantee this index is valid please
}>): ReactElement {
    const {
        players,
        phaseState
    } = useContext(StateContext)!;
    
    const {
        roleLabel: playerRoleLabel,
        alive: playerAlive,
        name: playerName,
        playerTags
    } = useMemo(()=>
        players[props.playerIndex],
        [players, props.playerIndex]
    );

    const playerNameToString = useMemo(()=>
        players[props.playerIndex].toString(),
        [players, props.playerIndex]
    );

    const {
        roleState: myRoleState,
        myIndex
    } = usePlayerState() ?? {};


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
        newKeywordData[playerNameToString] = [
            { style: "keyword-player-important keyword-player-number", replacement: (myIndex + 1).toString() },
            { replacement: " " },
            { style: "keyword-player-important keyword-player-sender", replacement: playerName }
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
        <StyledText playerKeywordData={newKeywordData}>{playerNameToString}</StyledText>
        {roleString !== null && <StyledText> {roleString}</StyledText>}
        <StyledText>{playerTags.map((tag)=>{return translate("tag."+tag)})}</StyledText>
    </div>
}