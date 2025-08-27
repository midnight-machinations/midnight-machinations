import React from "react";
import { PlayerIndex } from "../../../../../game/gameState.d";
import PlayerOptionDropdown from "../../../../../components/PlayerOptionDropdown";
import { AvailablePlayerListSelection, PlayerListSelection, } from "../../../../../game/controllerInput";
import { Button } from "../../../../../components/Button";
import Icon from "../../../../../components/Icon";

export default function PlayerListSelectionMenu(props: Readonly<{
    availableSelection: AvailablePlayerListSelection
    selection: PlayerListSelection,
    onChoose: (player: PlayerListSelection) => void
}>){

    const handleSelection = (player: PlayerIndex | null, index: number) => {
        let newSelection: PlayerListSelection = props.selection.slice();

        if(index >= newSelection.length && player !== null){
            newSelection.push(player);
        }else{
            if(player === null){
                newSelection = newSelection.slice(0,index).concat(newSelection.slice(index+1));
            }else{
                newSelection[index] = player;
            }
        }
        
        props.onChoose(newSelection);
    }

    const newChoosablePlayers = props.availableSelection.availablePlayers.filter((p)=>
        props.availableSelection.canChooseDuplicates || !props.selection.includes(p)
    ) as PlayerIndex[];

    return <div className="generic-list-controller-menu">
        {
            props.selection.map((p,i)=><PlayerOptionDropdown
                key={i}
                value={p}
                onChange={(p)=>handleSelection(p,i)}
                choosablePlayers={props.availableSelection.availablePlayers.filter((p)=>
                    props.availableSelection.canChooseDuplicates || !props.selection.includes(p) || p === props.selection[i]
                ) as PlayerIndex[]}
                canChooseNone={true}
            />)
        }
        {
            ((props.availableSelection.maxPlayers??Infinity) > props.selection.length) && newChoosablePlayers.length !== 0 ? 
            <PlayerOptionDropdown
                value={null}
                onChange={(p)=>handleSelection(p,props.selection.length)}
                choosablePlayers={newChoosablePlayers}
                canChooseNone={true}
            /> : null
        }
        <div>
            {
                ((props.availableSelection.maxPlayers??Infinity) >= props.availableSelection.availablePlayers.length) && newChoosablePlayers.length !== 0
                ?
                    <Button
                        onClick={()=>props.onChoose(props.availableSelection.availablePlayers)}
                    >
                        <Icon>select_all</Icon>
                    </Button>
                :null
            }
            {
                ((props.availableSelection.maxPlayers??Infinity) > 1) &&
                props.availableSelection.availablePlayers.length > 1 &&
                props.selection.length > 0
                ?
                    <Button
                        onClick={()=>props.onChoose([])}
                    >
                        <Icon>deselect</Icon>
                    </Button>
                :null
            }
        </div>
    </div>
}