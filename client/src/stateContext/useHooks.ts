import { useContext } from "react";
import { PlayerGameState } from "./stateType/gameState";
import { StateContext } from "./StateContext";



export function usePlayerState(): PlayerGameState | undefined {
    const stateCtx = useContext(StateContext);
    if(stateCtx === undefined){return undefined};
    const { clientState } = stateCtx

    if (clientState.type === "player") {
        return clientState
    } else {
        return undefined
    }
}
export function usePlayerNames(): string[] | undefined {
    const stateCtx = useContext(StateContext);
    if(stateCtx===undefined){return undefined}
    if(stateCtx.clients.length() !== 0){
        return stateCtx.clients.values()
            .filter((c)=>c.clientType.type==="player")
            //thanks typescript very cool
            .map((c)=>c.clientType.type==="player"?c.clientType.name:undefined) as string[]
    }
    if(stateCtx.players.length !== 0){
        return stateCtx.players.map((p)=>p.name)
    }
}
export function usePlayerNamesToString(): string[] | undefined {
    const stateCtx = useContext(StateContext);
    if(stateCtx===undefined){return undefined}
    if(stateCtx.clients.length() !== 0){
        return stateCtx.clients.values()
            .filter((c)=>c.clientType.type==="player")
            //thanks typescript very cool
            .map((c)=>c.clientType.type==="player"?c.clientType.name:undefined) as string[]
    }
    if(stateCtx.players.length !== 0){
        return stateCtx.players.map((p)=>p.toString())
    }
}