import React, { ReactElement, useContext } from "react"
import "./kiraSelectionMenu.css";
import translate, { translateChecked } from "../../../../../game/lang";
import StyledText, { KeywordDataMap } from "../../../../../components/StyledText";
import Select, { SelectOptionsSearch } from "../../../../../components/Select";
import ListMap, { ListMapData } from "../../../../../ListMap";
import { AvailableKiraSelection } from "../../../../../game/abilityInput";
import { PlayerIndex } from "../../../../../stateContext/stateType/otherState";
import { usePlayerState } from "../../../../../stateContext/useHooks";
import { StateContext } from "../../../../../stateContext/StateContext";

export const KIRA_GUESSES = [
    "none",
    "nonTown",
    "jailor", "villager",
    "detective", "lookout", "spy", "tracker", "philosopher", "psychic", "auditor", "snoop", "gossip", "tallyClerk",
    "doctor",  "bodyguard",  "cop", "bouncer", "engineer", "armorsmith", "steward",
    "vigilante",  "veteran", "marksman", "deputy", "rabblerouser",
    "escort",  "medium",  "retributionist", "reporter", "mayor",  "transporter", "porter", "polymath"
];
export type KiraGuessResult = "correct" | "notInGame" | "wrongSpot";
export type KiraSelection = ListMapData<PlayerIndex, KiraGuess>;
export type KiraResult = ListMapData<PlayerIndex, [KiraGuess, KiraGuessResult]>;

export type KiraGuess = typeof KIRA_GUESSES[number];
export function kiraGuessTranslate(kiraGuess: KiraGuess): string{
    let outString = translateChecked("role."+kiraGuess+".name");
    if(outString===null){
        outString = translate(kiraGuess);
    }
    return outString;
}



export function KiraResultDisplay(props: Readonly<{
    map: {
        type: "selection"
        map: KiraSelection
    } | {
        type: "reuslt",
        map: KiraResult
    },
    playerKeywordData?: KeywordDataMap,
    playerNames: string[],
}>): ReactElement {
    let guessesMap = new ListMap<PlayerIndex, [KiraGuess, KiraGuessResult]>();

    if(props.map.type === "reuslt"){
        for(let [player, guess] of props.map.map){
            guessesMap.insert(player, guess);
        }
    }else{
        for(let [player, guess] of props.map.map){
            guessesMap.insert(player, [guess, "notInGame"]);
        }

    }

    let out = [];

    let sortedPlayerIndexes = guessesMap.keys().sort();

    for(let playerIndex of sortedPlayerIndexes){
        let resultStyle = "";
        let resultIcon = "";
        let resultString = "";

        let guessMapValue = guessesMap.get(playerIndex);
        if(guessMapValue === null){
            continue;
        }
        let guess = guessMapValue[0];
        let result = guessMapValue[1];


        if(guess !== "none" && props.map.type !== "selection"){
            if(result === "correct"){
                resultStyle = "correct";
                resultIcon = "🟩";
                resultString = translate("kiraResult.correct");
            }else if(result === "wrongSpot"){
                resultStyle = "wrongSpot";
                resultIcon = "🟨";
                resultString = translate("kiraResult.wrongSpot");
            }else if(result === "notInGame"){
                resultStyle = "notInGame";
                resultIcon = "🟥";
                resultString = translate("kiraResult.notInGame");
            }
        }

        out.push(<div key={playerIndex} className={"kira-guess-result "+resultStyle}>
            <StyledText
                playerKeywordData={props.playerKeywordData}
            >
                {props.playerNames[playerIndex]} {kiraGuessTranslate(guess)} {resultIcon} {resultString}
            </StyledText>
        </div>)
    }

    return <>{out}</>
}

export default function KiraSelectionMenu(props: Readonly<{
    selection: KiraSelection,
    available: AvailableKiraSelection
    onChange: (selection: KiraSelection)=>void;
}>): ReactElement {

    const myIndex = usePlayerState()!.myIndex;
    const guessable = useContext(StateContext)!.players.filter((p)=>p.alive&&p.index!==myIndex).map((p)=>p.index);
    
    function sendSetKiraGuess(guesses: KiraSelection){
        props.onChange(guesses);
    }

    let currentGuessesMap = new ListMap(props.selection);    

    return <div className="large-kira-menu">
        {guessable.map((playerIndex)=>{
            return <KiraGuessPicker 
                key={playerIndex} 
                playerIndex={playerIndex} 
                guess={currentGuessesMap.get(playerIndex) ?? "none"} 
                onChange={(guess: KiraGuess) => {
                    let newGuesses = new ListMap<PlayerIndex, KiraGuess>([...props.selection]);
                    newGuesses.insert(playerIndex, guess);
                    sendSetKiraGuess(newGuesses.list);
                }}
            />
        })}
    </div>
}

function KiraGuessPicker(props: {
    playerIndex: PlayerIndex,
    guess: KiraGuess,
    onChange: (guess: KiraGuess) => void
}): ReactElement {
    const { players } = useContext(StateContext)!;

    const guessOptions: SelectOptionsSearch<KiraGuess> = new Map();
    for(let guess of KIRA_GUESSES){
        guessOptions.set(guess, [
            <StyledText noLinks={true}>{kiraGuessTranslate(guess)}</StyledText>,
            kiraGuessTranslate(guess)
        ]);
    }

    return <div className="kira-guess-picker">
        <StyledText>{players[props.playerIndex].toString()}</StyledText>
        <Select
            value={props.guess}
            onChange={(e) => props.onChange(e)}
            optionsSearch={guessOptions}
        />
    </div>
}