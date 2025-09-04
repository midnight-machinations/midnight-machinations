import React, { useEffect } from "react";
import { PlayerIndex } from "../../../../../game/gameState.d";
import PlayerOptionDropdown from "../../../../../components/PlayerOptionDropdown";
import { AvailableTwoPlayerOptionSelection, TwoPlayerOptionSelection } from "../../../../../game/controllerInput";

export default function TwoPlayerOptionSelectionMenu(props: Readonly<{
    availableSelection: AvailableTwoPlayerOptionSelection
    selection: TwoPlayerOptionSelection,
    onChoose: (player: TwoPlayerOptionSelection) => void
}>){
    const [selectionState, setSelectionState] = React.useState<[number | null, number | null]>([null, null]);
    
    useEffect(() => {
        if(props.selection === null){
            setSelectionState([null, null]);
        }else{
            setSelectionState(props.selection);
        }
    }, [props.selection]);

    const handleSelectionFirst = (player: PlayerIndex | null) => {
        if(player === null){
            setSelectionState([null, null]);
            props.onChoose(null);
        }else if(selectionState[1] === null){
            setSelectionState([player, null]);
        }else{
            setSelectionState([player, selectionState[1]]);
            props.onChoose([player, selectionState[1]]);
        }
    }
    
    const handleSelectionSecond = (player: PlayerIndex | null) => {
        if(player === null){
            setSelectionState([null, null]);
            props.onChoose(null);
        }else if(selectionState[0] === null){
            setSelectionState([null, player]);
        }else{
            setSelectionState([selectionState[0], player]);
            props.onChoose([selectionState[0], player]);
        }
    }

    

    const choosableFirst = props.availableSelection.availableFirstPlayers.filter(player => props.availableSelection.canChooseDuplicates || player !== selectionState[1]);
    const choosableSecond = props.availableSelection.availableSecondPlayers.filter(player => props.availableSelection.canChooseDuplicates || player !== selectionState[0]);

    return <div>
        <PlayerOptionDropdown
            value={selectionState[0]}
            onChange={handleSelectionFirst}
            choosablePlayers={choosableFirst}
            canChooseNone={props.availableSelection.canChooseNone}
        />
        <PlayerOptionDropdown
            value={selectionState[1]}
            onChange={handleSelectionSecond}
            choosablePlayers={choosableSecond}
            canChooseNone={props.availableSelection.canChooseNone}
        />
    </div>
}