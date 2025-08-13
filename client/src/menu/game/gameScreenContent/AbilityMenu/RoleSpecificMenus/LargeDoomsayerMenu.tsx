import React from "react"
import "./largeDoomsayerMenu.css"
import translate, { translateChecked } from "../../../../../game/lang";
import GameState from "../../../../../game/gameState.d";
import { StateEventType } from "../../../../../game/gameManager.d";
import GAME_MANAGER from "../../../../..";
import { encodeString } from "../../../../../components/ChatMessage";


export type Doomsayer = {
    type: "doomsayer",
    guesses: [
        [number, DoomsayerGuess],
        [number, DoomsayerGuess],
        [number, DoomsayerGuess]
    ]
}

export const DOOMSAYER_GUESSES = [
    "nonTown",
    "jailor", "villager",  
    "doctor",  "bodyguard",  "cop", "bouncer", "engineer", "armorsmith", "steward",
    "vigilante",  "veteran", "marksman", "deputy", "rabblerouser",
    "escort",  "medium",  "retributionist", "reporter", "mayor",  "transporter", "porter", "polymath"
];

export type DoomsayerGuess = typeof DOOMSAYER_GUESSES[number];
export function doomsayerGuessTranslate(doomsayerGuess: DoomsayerGuess): string{
    let outString = translateChecked("role."+doomsayerGuess+".name");
    if(outString===null){
        outString = translate(doomsayerGuess);
    }
    return outString;
}

type LargeDoomsayerMenuProps = {
}
type LargeDoomsayerMenuState = {
    gameState: GameState
    localDoomsayerGuesses: [
        [number, DoomsayerGuess],
        [number, DoomsayerGuess],
        [number, DoomsayerGuess]
    ]
}
export default class LargeDoomsayerMenu extends React.Component<LargeDoomsayerMenuProps, LargeDoomsayerMenuState> {
    listener: (type?: StateEventType) => void;
    constructor(props: LargeDoomsayerMenuState) {
        super(props);

        let defaultGuess: [
            [number, DoomsayerGuess],
            [number, DoomsayerGuess],
            [number, DoomsayerGuess]
        ];
        if(
            GAME_MANAGER.state.stateType === "game" &&
            GAME_MANAGER.state.clientState.type === "player" &&
            GAME_MANAGER.state.clientState.roleState?.type === "doomsayer"
        ){
            defaultGuess = GAME_MANAGER.state.clientState.roleState.guesses;
        }else{
            defaultGuess = [
                [0, "neutral"],
                [0, "neutral"],
                [0, "neutral"]
            ];
        }
        
        if(GAME_MANAGER.state.stateType === "game")
            this.state = {
                gameState : GAME_MANAGER.state,
                localDoomsayerGuesses: defaultGuess
            };
        this.listener = (type)=>{
            if(GAME_MANAGER.state.stateType === "game")
                this.setState({
                    gameState: GAME_MANAGER.state
                });
            if(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.clientState.type === "player" && type==="yourRoleState" && GAME_MANAGER.state.clientState.roleState?.type === "doomsayer"){
                this.setState({
                    localDoomsayerGuesses: GAME_MANAGER.state.clientState.roleState.guesses
                });
            }
        };  
    }
    componentDidMount() {
        GAME_MANAGER.addStateListener(this.listener);
    }
    componentWillUnmount() {
        GAME_MANAGER.removeStateListener(this.listener);
    }

    sendAndSetGuesses(guess: [
        [number, DoomsayerGuess],
        [number, DoomsayerGuess],
        [number, DoomsayerGuess]
    ]){
        this.setState({
            localDoomsayerGuesses: guess
        });
        GAME_MANAGER.sendSetDoomsayerGuess(guess);
    }

    renderGuessPicker(index: number){
        //Cant guess a player 2x
        //cant guess yourself
        //cant guess a dead player (keep in mind a player could possibly appear dead when their not, so maybe keep it so you can guess dead players?)

        let playerOptions: JSX.Element[] = [];
        for(let i = 0; i < this.state.gameState.players.length; i++){
            playerOptions.push(<option key={i} value={i}>{encodeString(this.state.gameState.players[i].toString())}</option>);
        }
        let doomsayerGuessOptions: JSX.Element[] = [];
        for(let i = 0; i < DOOMSAYER_GUESSES.length; i++){
            doomsayerGuessOptions.push(<option key={i} value={DOOMSAYER_GUESSES[i]}>{doomsayerGuessTranslate(DOOMSAYER_GUESSES[i])}</option>);
        }
        return <div>
            <select
                value={this.state.localDoomsayerGuesses[index][0]}
                onChange={(e)=>{
                    let newGuess = this.state.localDoomsayerGuesses;
                    newGuess[index][0] = parseInt(e.target.options[e.target.selectedIndex].value);
                    this.sendAndSetGuesses(newGuess);
                }}
            >{playerOptions}</select>
            <select
                value={this.state.localDoomsayerGuesses[index][1]}
                onChange={(e)=>{
                    let newGuess = this.state.localDoomsayerGuesses;
                    newGuess[index][1] = e.target.options[e.target.selectedIndex].value as DoomsayerGuess;
                    this.sendAndSetGuesses(newGuess);
                }}
            >{doomsayerGuessOptions}</select>
        </div>
    }
    render(){
        return <div className="large-doomsayer-menu">
            {this.renderGuessPicker(0)}
            {this.renderGuessPicker(1)}
            {this.renderGuessPicker(2)}
        </div>
    }
}