import { ListMapData } from "../../ListMap"
import {
    ChatGroup,
    ClientConnection,
    InsiderGroup,
    PlayerIndex,
    Verdict
} from "./otherState";
import { Role, RoleState } from "./roleState";
import { ChatFilter } from "../../menu/game/gameScreenContent/ChatMenu";
import { ControllerID, SavedController } from "../../game/abilityInput";
import { Tag } from "./tagState";

export type PlayerGameState = {
    type: "player",

    myIndex: PlayerIndex,
    
    roleState: RoleState,

    will: string,
    notes: string[],
    crossedOutOutlines: number[],
    chatFilter: ChatFilter,
    deathNote: string,
    judgement: Verdict,

    savedControllers: ListMapData<ControllerID, SavedController>,

    fellowInsiders: PlayerIndex[],

    sendChatGroups: ChatGroup[],
    insiderGroups: InsiderGroup[],
    
    missedWhispers: PlayerIndex[],

    updateChatFilter: (filter: PlayerIndex | null)=>void
}
export function createPlayerGameState(): PlayerGameState {
    return {
        type: "player",

        myIndex: 0,
        
        roleState: { type: "detective" },

        savedControllers: [],

        will: "",
        notes: [],
        crossedOutOutlines: [],
        chatFilter: null,
        deathNote: "",
        judgement: "abstain",

        fellowInsiders: [],

        sendChatGroups: [],
        insiderGroups: [],

        missedWhispers: [],

        updateChatFilter(filter) {
            
        },
    }
}


export type Player = {
    name: string,
    index: number,
    numVoted: number,
    alive: boolean,
    roleLabel: Role | null,
    playerTags: Tag[]

    toString(): string
}
export function createPlayer(name: string, index: number): Player {
    return{
        name: name,
        index: index,
        numVoted: 0,
        alive: true,
        roleLabel: null,
        playerTags: [],

        toString() {
            return ""+(this.index+1)+": " + this.name;
        }
    }
}

export type GameClient = {
    clientType: GameClientType,
    connection: ClientConnection,
    host: boolean,
}
export type GameClientType = {
    type: "spectator",
    index: number
} | {
    type: "player",
    index: number,
}