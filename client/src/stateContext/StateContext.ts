import { createContext, Dispatch, SetStateAction, useContext, useState } from "react";
import { ClientObject } from "./state";
import { Player, PlayerGameState } from "./stateType/gameState";
import { LobbyPreviewData } from "./stateType/gameBrowserState";
import ListMap from "../ListMap";
import { WebsocketContext } from "../menu/WebsocketContext";
import { defaultPhaseTimes, LobbyClientID, PhaseTimes, PlayerIndex } from "./stateType/otherState";
import { RoleList } from "./stateType/roleListState";
import { Role } from "./stateType/roleState";
import { ModifierType } from "./stateType/modifiersState";
import { ChatMessage } from "../components/ChatMessage";
import { Grave } from "./stateType/grave";
import { PhaseState } from "./stateType/phaseState";
import { ChatFilter } from "../menu/game/gameScreenContent/ChatMenu";

export type StateContext = {
    type: "lobby" | "game" | "neither"

    lobbies: Map<number, LobbyPreviewData>,
    
    roomCode: number | null,
    lobbyName: string | null,
    
    clients: ListMap<LobbyClientID, ClientObject>,
    myId: number | null,

    roleList: RoleList,
    phaseTimes: PhaseTimes,
    enabledRoles: Role[],
    enabledModifiers: ModifierType[],
    
    chatMessages: ChatMessage[],
    graves: Grave[],
    players: Player[],
    phaseState: PhaseState,
    timeLeftMs: number | null,
    dayNumber: number,
    missedChatMessages: boolean,
    ticking: boolean,
    initialized: boolean,
    fastForward: boolean,

    clientState: PlayerGameState | {type: "spectator"},
    host: boolean,

    chatFilter: ChatFilter,
    prependWhisperFunction: (index: PlayerIndex)=>void,

    setType: Dispatch<SetStateAction<"lobby"|"game"|"neither">>,
    setLobbies: Dispatch<SetStateAction<Map<number, LobbyPreviewData>>>,
    setRoomCode: Dispatch<SetStateAction<number | null>>,
    setLobbyName: Dispatch<SetStateAction<string | null>>,
    setClients: Dispatch<SetStateAction<ListMap<LobbyClientID, ClientObject>>>,
    setMyId: Dispatch<SetStateAction<number | null>>,
    setRoleList: Dispatch<SetStateAction<RoleList>>,
    setPhaseTimes: Dispatch<SetStateAction<PhaseTimes>>,
    setEnabledRoles: Dispatch<SetStateAction<Role[]>>,
    setEnabledModifiers: Dispatch<SetStateAction<ModifierType[]>>,
    setChatMessages: Dispatch<SetStateAction<ChatMessage[]>>,
    setGraves: Dispatch<SetStateAction<Grave[]>>,
    setPlayers: Dispatch<SetStateAction<Player[]>>,
    setPhaseState: Dispatch<SetStateAction<PhaseState>>,
    setTimeLeftMs: Dispatch<SetStateAction<number | null>>,
    setDayNumber: Dispatch<SetStateAction<number>>,
    setMissedChatMessages: Dispatch<SetStateAction<boolean>>,
    setTicking: Dispatch<SetStateAction<boolean>>,
    setInitialized: Dispatch<SetStateAction<boolean>>,
    setFastForward: Dispatch<SetStateAction<boolean>>,
    setClientState: Dispatch<SetStateAction<PlayerGameState | {type: "spectator"}>>,
    setHost: Dispatch<SetStateAction<boolean>>,
    setChatFilter: Dispatch<SetStateAction<ChatFilter>>,
    setPrependWhisperFunction: Dispatch<SetStateAction<(index: PlayerIndex)=>void>>,

};


export const StateContext = createContext<StateContext | undefined>(undefined);
export function useStateContext(): StateContext {
    const websocketCtx = useContext(WebsocketContext)!;

    const [type, setType] = useState<"game"|"lobby"|"neither">("neither");

    const [lobbies, setLobbies] = useState(new Map<number, LobbyPreviewData>());
    
    const [roomCode, setRoomCode] = useState<number | null>(null);
    const [lobbyName, setLobbyName] = useState<string | null>(null);
    
    const [clients, setClients] = useState<ListMap<LobbyClientID, ClientObject>>(new ListMap());
    const [myId, setMyId] = useState<number | null>(null);

    const [roleList, setRoleList] = useState<RoleList>([]);
    const [phaseTimes, setPhaseTimes] = useState<PhaseTimes>(defaultPhaseTimes());
    const [enabledRoles, setEnabledRoles] = useState<Role[]>([]);
    const [enabledModifiers, setEnabledModifiers] = useState<ModifierType[]>([]);
    const [chatMessages, setChatMessages] = useState<ChatMessage[]>([]);
    const [graves, setGraves] = useState<Grave[]>([]);
    const [players, setPlayers] = useState<Player[]>([]);
    const [phaseState, setPhaseState] = useState<PhaseState>({type:"recess"});
    const [timeLeftMs, setTimeLeftMs] = useState<number | null>(null);
    const [dayNumber, setDayNumber] = useState<number>(0);
    const [missedChatMessages, setMissedChatMessages] = useState<boolean>(false);
    const [ticking, setTicking] = useState<boolean>(false);
    const [initialized, setInitialized] = useState<boolean>(false);
    const [fastForward, setFastForward] = useState<boolean>(false);
    const [clientState, setClientState] = useState<PlayerGameState | {type: "spectator"}>({type: "spectator"});
    const [host, setHost] = useState<boolean>(false);
    const [chatFilter, setChatFilter] = useState<ChatFilter>(null);
    const [prependWhisperFunction, setPrependWhisperFunction] = useState<(index: PlayerIndex)=>void>((index)=>{});
        
    // AudioController.clearQueue();
    // AudioController.unpauseQueue();
    
    return {
        type, setType,
        lobbies, setLobbies,
        roomCode, setRoomCode,
        lobbyName, setLobbyName,
        clients, setClients,
        myId, setMyId,
        roleList, setRoleList,
        phaseTimes, setPhaseTimes,
        enabledRoles, setEnabledRoles,
        enabledModifiers, setEnabledModifiers,
        chatMessages, setChatMessages,
        graves, setGraves,
        players, setPlayers,
        phaseState, setPhaseState,
        timeLeftMs, setTimeLeftMs,
        dayNumber, setDayNumber,
        missedChatMessages, setMissedChatMessages,
        ticking, setTicking,
        initialized, setInitialized,
        fastForward, setFastForward,
        clientState, setClientState,
        host, setHost,
        chatFilter, setChatFilter,
        prependWhisperFunction, setPrependWhisperFunction
    };
}