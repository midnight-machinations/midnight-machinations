import { ReactElement } from "react"
import React from "react"
import translate from "../game/lang"
import { PlayerIndex } from "../game/gameState.d"
import { useGameState } from "./useHooks"
import StyledText from "./StyledText"
import Select, { SelectOptionsSearch } from "./Select"

/// A dropdown menu for selecting a player.
/// canChooseNone defaults to false.
export default function PlayerOptionDropdown(props: {
    value: PlayerIndex | null,
    onChange: (player: PlayerIndex | null) => void,
    choosablePlayers?: PlayerIndex[],
    canChooseNone?: boolean
}): ReactElement {

    let players = useGameState(
        gameState => gameState.players.map(player => [player.index, player.toString()] as [PlayerIndex, string]),
        ["gamePlayers"]
    )!;


    const optionMap: SelectOptionsSearch<PlayerIndex | "none"> = new Map();

    if(props.canChooseNone === true){
        // eslint-disable-next-line react/jsx-key
        optionMap.set("none", [<StyledText noLinks={true}>{translate("none")}</StyledText>, translate("none")]);
    }

    for (const [index, name] of players) {
        if(
            props.choosablePlayers === undefined ||
            props.choosablePlayers.includes(index)
        ){
            // eslint-disable-next-line react/jsx-key
            optionMap.set(index, [<StyledText noLinks={true}>{name.toString()}</StyledText>, name]);
        }
    }

    return <Select
        value={(props.value===null ?"none":props.value) as PlayerIndex | "none"}
        optionsSearch={optionMap}
        onChange={value => props.onChange(value === "none" ? null : value)}
    />
}