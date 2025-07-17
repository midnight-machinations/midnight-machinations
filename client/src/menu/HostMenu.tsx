import React, { ReactElement, useContext, useEffect, useState } from "react";
import translate from "../game/lang";
import { Button } from "../components/Button";
import "./lobby/lobbyMenu.css"
import LobbyPlayerList from "./lobby/LobbyPlayerList";
import { AppContext } from "./AppContext";
import { WebsocketContext } from "./WebsocketContext";
import { StateContext } from "../stateContext/StateContext";

export default function HostMenu(): ReactElement {
    const anchorContext = useContext(AppContext)!;
    const {
        sendHostDataRequest, sendBackToLobbyPacket, sendHostEndGamePacket, sendHostSkipPhase,
        lastMessageRecieved
    } = useContext(WebsocketContext)!;

    const stateCtx = useContext(StateContext);

    useEffect(() => {
        sendHostDataRequest();
    }, [sendHostDataRequest])

    const [lastRefreshed, setLastRefreshed] = useState(new Date());

    useEffect(()=>{
        // Check on every packet since like 1 million packets can affect this
        if (stateCtx === undefined || stateCtx.type !== "game" || stateCtx.host === null) {
            anchorContext.clearCoverCard();
        }

        if (lastMessageRecieved?.type === "hostData") {
            setLastRefreshed(new Date(Date.now()))
        }
    }, [lastMessageRecieved])

    return <div className="settings-menu-card">
        <header>
            <h1>{translate("menu.hostSettings.title")}</h1>
            {translate("menu.hostSettings.lastRefresh", lastRefreshed.toLocaleTimeString())}
        </header>
        
        <Button onClick={() => sendHostDataRequest()}
        >{translate("refresh")}</Button>
        
        <main className="settings-menu">
            <LobbyPlayerList />
            <div className="chat-menu-colors">
                <h2>{translate("menu.hostSettings.lobby")}</h2>
                <section>
                    <Button onClick={()=>sendBackToLobbyPacket()}>
                        {translate("backToLobby")}
                    </Button>
                    <Button onClick={()=>sendHostEndGamePacket()}>
                        {translate("menu.hostSettings.endGame")}
                    </Button>
                    <Button onClick={()=>sendHostSkipPhase()}>
                        {translate("menu.hostSettings.skipPhase")}
                    </Button>
                </section>
            </div>
        </main>
    </div>
}