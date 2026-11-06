import type { ReactElement } from "react";
import type { PlayerIndex } from "../game/gameState.d";
import translate from "../game/lang";
import { encodeString } from "./ChatMessage";
import Select, { type SelectOptionsSearch } from "./Select";
import StyledText from "./StyledText";
import { useGameState } from "./useHooks";

/// A dropdown menu for selecting a player.
/// canChooseNone defaults to false.
export default function PlayerOptionDropdown(props: {
    value: PlayerIndex | null;
    onChange: (player: PlayerIndex | null) => void;
    choosablePlayers?: PlayerIndex[];
    canChooseNone?: boolean;
}): ReactElement {
    const players = useGameState(
        (gameState) => gameState.players.map((player) => [player.index, player.toString()] as [PlayerIndex, string]),
        ["gamePlayers"],
    )!;

    const optionMap: SelectOptionsSearch<PlayerIndex | "none"> = new Map();

    if (props.canChooseNone === true) {
        optionMap.set("none", [
            <StyledText key="none" noLinks={true}>
                {translate("none")}
            </StyledText>,
            translate("none"),
        ]);
    }

    for (const [index, name] of players) {
        if (props.choosablePlayers === undefined || props.choosablePlayers.includes(index)) {
            optionMap.set(index, [
                <StyledText key={index} noLinks={true}>
                    {encodeString(name)}
                </StyledText>,
                name,
            ]);
        }
    }

    return (
        <Select
            value={(props.value === null ? "none" : props.value) as PlayerIndex | "none"}
            optionsSearch={optionMap}
            onChange={(value) => props.onChange(value === "none" ? null : value)}
        />
    );
}
