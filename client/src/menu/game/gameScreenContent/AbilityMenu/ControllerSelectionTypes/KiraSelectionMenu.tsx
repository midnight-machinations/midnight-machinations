import { type ReactElement, useEffect, useState } from "react";
import "./kiraSelectionMenu.css";
import GAME_MANAGER from "../../../../..";
import { encodeString } from "../../../../../components/ChatMessage";
import Select, { type SelectOptionsSearch } from "../../../../../components/Select";
import StyledText, { type KeywordDataMap } from "../../../../../components/StyledText";
import { useGameState, usePlayerState } from "../../../../../components/useHooks";
import type { AvailableKiraSelection } from "../../../../../game/controllerInput";
import type { PlayerIndex, UnsafeString } from "../../../../../game/gameState.d";
import translate, { translateChecked } from "../../../../../game/lang";
import { getRolesFromRoleSet } from "../../../../../game/roleListState.d";
import type { Role } from "../../../../../game/roleState.d";
import ListMap, { type ListMapData } from "../../../../../ListMap";

export type KiraGuessResult = "correct" | "notInGame" | "wrongSpot";
export type KiraSelection = ListMapData<PlayerIndex, KiraGuess>;
export type KiraResult = ListMapData<PlayerIndex, [KiraGuess, KiraGuessResult]>;

export type KiraGuess = "none" | "nonTown" | Role;
export function kiraGuessTranslate(kiraGuess: KiraGuess): string {
    let outString = translateChecked(`role.${kiraGuess}.name`);
    if (outString === null) {
        outString = translate(kiraGuess);
    }
    return outString;
}

export function KiraResultDisplay(
    props: Readonly<{
        map:
            | {
                  type: "selection";
                  map: KiraSelection;
              }
            | {
                  type: "reuslt";
                  map: KiraResult;
              };
        playerKeywordData?: KeywordDataMap;
        playerNames: UnsafeString[];
        noLinks?: boolean;
    }>,
): ReactElement {
    const guessesMap = new ListMap<PlayerIndex, [KiraGuess, KiraGuessResult]>();

    if (props.map.type === "reuslt") {
        for (const [player, guess] of props.map.map) {
            guessesMap.insert(player, guess);
        }
    } else {
        for (const [player, guess] of props.map.map) {
            guessesMap.insert(player, [guess, "notInGame"]);
        }
    }

    const out = [];

    const sortedPlayerIndexes = guessesMap.keys().sort();

    for (const playerIndex of sortedPlayerIndexes) {
        let resultStyle = "";
        let resultIcon = "";
        let resultString = "";

        const guessMapValue = guessesMap.get(playerIndex);
        if (guessMapValue === null) {
            continue;
        }
        const guess = guessMapValue[0];
        const result = guessMapValue[1];

        if (guess !== "none" && props.map.type !== "selection") {
            if (result === "correct") {
                resultStyle = "correct";
                resultIcon = "🟩";
                resultString = translate("kiraResult.correct");
            } else if (result === "wrongSpot") {
                resultStyle = "wrongSpot";
                resultIcon = "🟨";
                resultString = translate("kiraResult.wrongSpot");
            } else if (result === "notInGame") {
                resultStyle = "notInGame";
                resultIcon = "🟥";
                resultString = translate("kiraResult.notInGame");
            }
        }

        out.push(
            <div key={playerIndex} className={`kira-guess-result ${resultStyle}`}>
                <StyledText playerKeywordData={props.playerKeywordData} noLinks={props.noLinks}>
                    {encodeString(props.playerNames[playerIndex])} {kiraGuessTranslate(guess)} {resultIcon}{" "}
                    {resultString}
                </StyledText>
            </div>,
        );
    }

    return <>{out}</>;
}

export default function KiraSelectionMenu(
    props: Readonly<{
        selection: KiraSelection;
        available: AvailableKiraSelection;
        onChange: (selection: KiraSelection) => void;
    }>,
): ReactElement {
    const myIndex = usePlayerState((playerState) => playerState.myIndex)!;

    const guessable = useGameState((gameState) =>
        gameState.players.filter((p) => p.alive && p.index !== myIndex).map((p) => p.index),
    )!;

    function sendSetKiraGuess(guesses: KiraSelection) {
        props.onChange(guesses);
    }

    const currentGuessesMap = new ListMap(props.selection);

    return (
        <div className="large-kira-menu">
            {guessable.map((playerIndex) => {
                return (
                    <KiraGuessPicker
                        key={playerIndex}
                        playerIndex={playerIndex}
                        guess={currentGuessesMap.get(playerIndex) ?? "none"}
                        availableGuesses={["none", "nonTown", ...getRolesFromRoleSet("town")]}
                        onChange={(guess: KiraGuess) => {
                            const newGuesses = new ListMap<PlayerIndex, KiraGuess>([...props.selection]);
                            newGuesses.insert(playerIndex, guess);
                            sendSetKiraGuess(newGuesses.list);
                        }}
                    />
                );
            })}
        </div>
    );
}

function KiraGuessPicker(props: {
    playerIndex: PlayerIndex;
    guess: KiraGuess;
    availableGuesses: KiraGuess[];
    onChange: (guess: KiraGuess) => void;
}): ReactElement {
    const [players, setPlayers] = useState(() => {
        if (GAME_MANAGER.state.stateType === "game") return GAME_MANAGER.state.players;
        return [];
    });

    useEffect(() => {
        const listener = () => {
            if (GAME_MANAGER.state.stateType === "game") setPlayers(GAME_MANAGER.state.players);
        };

        listener();
        GAME_MANAGER.addStateListener(listener);
        return () => GAME_MANAGER.removeStateListener(listener);
    }, []);

    const guessOptions: SelectOptionsSearch<KiraGuess> = new Map();
    for (const guess of props.availableGuesses) {
        guessOptions.set(guess, [
            <StyledText key={guess} noLinks={true}>
                {kiraGuessTranslate(guess)}
            </StyledText>,
            kiraGuessTranslate(guess),
        ]);
    }

    return (
        <div className="kira-guess-picker">
            <StyledText>{encodeString(players[props.playerIndex].toString())}</StyledText>
            <Select value={props.guess} onChange={(e) => props.onChange(e)} optionsSearch={guessOptions} />
        </div>
    );
}
