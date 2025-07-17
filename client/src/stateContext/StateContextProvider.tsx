import { StateContext, useStateContext } from "./StateContext";
import { useWebsocketMessageListener, WebsocketContext } from "../menu/WebsocketContext";
import { AppContext } from "../menu/AppContext";
import React, { ReactElement, useContext } from "react";
import onWebsocketMessage from "./onWebsocketMessage";
import { ToClientPacket } from "../packet";

export default function StateContextProvider(props: Readonly<{
    children: React.ReactNode
}>): ReactElement{
    const appCtx = useContext(AppContext)!;
    const websocketContext = useContext(WebsocketContext)!;
    const stateCtx = useStateContext();

    const onMessage = (packet: ToClientPacket)=>{
        onWebsocketMessage(packet, appCtx, stateCtx, websocketContext);
    };

    useWebsocketMessageListener(websocketContext, onMessage);

    return <StateContext.Provider value={stateCtx}>
        {props.children}
    </StateContext.Provider>
}