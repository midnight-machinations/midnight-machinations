import React from "react";
import { PlayerIndex } from "../../../../../game/gameState.d";
import PlayerOptionDropdown from "../../../../../components/PlayerOptionDropdown";
import { AvailablePlayerListSelection, PlayerListSelection, } from "../../../../../game/controllerInput";

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

    return <div>
        {
            props.selection.map((p,i)=><PlayerOptionDropdown
                key={i}
                value={p}
                onChange={(p)=>handleSelection(p,i)}
                choosablePlayers={props.availableSelection.availablePlayers.filter((p)=>
                    props.availableSelection.canChooseDuplicates || !props.selection.includes(p) || p === props.selection[i]
                ) as number[]}
                canChooseNone={true}
            />)
        }
        {(props.availableSelection.maxPlayers??Infinity) > props.selection.length ? <PlayerOptionDropdown
            value={null}
            onChange={(p)=>handleSelection(p,props.selection.length)}
            choosablePlayers={props.availableSelection.availablePlayers.filter((p)=>
                props.availableSelection.canChooseDuplicates || !props.selection.includes(p)
            ) as number[]}
            canChooseNone={true}
        /> : null}
    </div>
}