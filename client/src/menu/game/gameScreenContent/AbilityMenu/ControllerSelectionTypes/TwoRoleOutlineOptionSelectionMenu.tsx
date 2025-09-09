import { ReactElement } from "react"
import React from "react"
import "./twoRoleOutlineOptionSelectionMenu.css"
import { Role } from "../../../../../game/roleState.d"
import { RoleList, translateRoleOutline } from "../../../../../game/roleListState.d"
import StyledText from "../../../../../components/StyledText"
import translate from "../../../../../game/lang"
import { useGameState, usePlayerNames } from "../../../../../components/useHooks"
import { Button } from "../../../../../components/Button"
import { AvailableTwoRoleOutlineOptionSelection, TwoRoleOutlineOptionSelection } from "../../../../../game/controllerInput"
import { UnsafeString } from "../../../../../game/gameState.d"
import { AuditorResult } from "../RoleSpecificMenus/AuditorMenu"

type AuditorButtons = ({
    type:"notUsed",
    chosen: boolean
} | {
    type:"used",
    result: AuditorResult
})[]

export default function TwoRoleOutlineOptionSelectionMenu(props: {
    selection?: TwoRoleOutlineOptionSelection,
    available?: AvailableTwoRoleOutlineOptionSelection,
    previouslyGivenResults?: [number, AuditorResult][],
    onChoose: (chosenOutlines: TwoRoleOutlineOptionSelection)=>void
}): ReactElement {
    const roleList = useGameState(
        (gameState)=>{
            return gameState.roleList;
        },
        ["roleList"]
    )!;

    const playerNames = usePlayerNames();

    const previouslyGivenResults = props.previouslyGivenResults ?? [];
    const chosenOutlines = props.selection ?? [null, null];
    const strikenOutlineIndexs = previouslyGivenResults.map(result=>result[0]);
    if(props.available !== undefined){
        for(let i = 0; i < roleList.length; i++){
            if(strikenOutlineIndexs.includes(i)){
                continue;
            }
            if(!props.available.includes(i)){
                strikenOutlineIndexs.push(i);
            }
        }
    }
    
    const buttons: AuditorButtons = [];
    for(let i = 0; i < roleList.length; i++){
        const found = previouslyGivenResults.find(result=>result[0] === i);
        if(found !== undefined){
            buttons.push({type:"used" as "used", result: found[1]});
        }else{
            buttons.push({type:"notUsed"  as "notUsed", chosen: chosenOutlines.includes(i)});
        }
    }
    
    return <div className="two-role-outline-option-input">
        <div className="grid">
            <RoleListDisplay
                roleList={roleList}
                strikenOutlineIndexs={strikenOutlineIndexs}
                playerNames={playerNames}
            />
            <ChooseButtons
                buttons={buttons}
                chosenOutlines={chosenOutlines}
                onChoose={props.onChoose}
            />
        </div>
    </div>
}
function RoleListDisplay(props: {
    roleList: RoleList,
    playerNames: UnsafeString[],
    strikenOutlineIndexs: number[],
}): ReactElement {
    return <>
        {props.roleList.map((entry, index)=>{
            return <Button 
                className="role-list-button"
                style={{ gridRow: index + 1 }} 
                key={index}
            >
                {
                    props.strikenOutlineIndexs.includes(index) ? 
                    <s><StyledText>
                        {`${index + 1}: ` + translateRoleOutline(entry, props.playerNames)}
                    </StyledText></s> : 
                    <StyledText>
                        {`${index + 1}: ` + translateRoleOutline(entry, props.playerNames)}
                    </StyledText>
                }
            </Button>
        })}
    </>
}
function ChooseButtons(props: Readonly<{
    //true means its selected, false means its not selected
    buttons: AuditorButtons,
    chosenOutlines: [number | null, number | null],
    onChoose: (chosenOutlines: [number | null, number | null])=>void
}>): ReactElement {
    return<> {
        props.buttons.map((button, index)=>{
            if(button.type === "notUsed"){
                return <Button 
                    className={"choose-button" + (button.chosen ? " highlighted" : "")}
                    key={index}
                    onClick={()=>{
                        let newChosenOutlines = [...props.chosenOutlines];

                        const foundIndex = newChosenOutlines.indexOf(index);
                        if(foundIndex !== -1){
                            newChosenOutlines[foundIndex] = null;
                        }
                        else if(newChosenOutlines[0] === null){
                            newChosenOutlines[0] = index;
                        }
                        else if(newChosenOutlines[1] === null){
                            newChosenOutlines[1] = index;
                        }
                        else{
                            newChosenOutlines[0] = index;
                        }
                        props.onChoose(newChosenOutlines as [number | null, number | null]);
                    }}
                >
                    <StyledText>
                        {translate("choose")}
                    </StyledText>
                </Button>
            }else{
                return <label
                    className="choose-button"
                    key={index}
                >
                    <StyledText>
                        {button.result.map((role)=>translate("role."+role+".name")).join(" ")}
                    </StyledText>
                </label>
            }
        })
    }</>
}