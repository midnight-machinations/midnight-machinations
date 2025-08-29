import React, { ReactElement, useContext, useEffect, useState } from "react";
import translate from "../game/lang";
import GAME_MANAGER from "..";
import { Button } from "../components/Button";
import { usePacketListener } from "../components/useHooks";
import { AnchorControllerContext } from "./Anchor";
import "./lobby/lobbyMenu.css"
import LobbyPlayerList from "./lobby/LobbyPlayerList";

export default function HostMenu(): ReactElement {
    const anchorController = useContext(AnchorControllerContext)!;

    useEffect(() => {
        GAME_MANAGER.sendHostDataRequest();
    }, [])

    const [lastRefreshed, setLastRefreshed] = useState(new Date());

    usePacketListener(type => {
        // Check on every packet since like 1 million packets can affect this
        if (!(GAME_MANAGER.state.stateType === "game" && GAME_MANAGER.state.host !== null)) {
            anchorController.clearCoverCard();
        }

        if (type === "hostData") {
            setLastRefreshed((_previous)=>new Date(Date.now()))
        }
    });

    return <div className="settings-menu-card">
        <header>
            <h1>{translate("menu.hostSettings.title")}</h1>
            {translate("menu.hostSettings.lastRefresh", lastRefreshed.toLocaleTimeString())}
        </header>
        
        <Button onClick={() => GAME_MANAGER.sendHostDataRequest()}
        >{translate("refresh")}</Button>
        
        <main className="settings-menu">
            <LobbyPlayerList />
            <div className="chat-menu-colors">
                <h2>{translate("menu.hostSettings.lobby")}</h2>
                <section>
                    <Button onClick={()=>GAME_MANAGER.sendBackToLobbyPacket()}>
                        {translate("backToLobby")}
                    </Button>
                    <Button onClick={()=>GAME_MANAGER.sendHostEndGamePacket()}>
                        {translate("menu.hostSettings.endGame")}
                    </Button>
                    <Button onClick={()=>GAME_MANAGER.sendHostSkipPhase()}>
                        {translate("menu.hostSettings.skipPhase")}
                    </Button>
                </section>
            </div>
        </main>
    </div>
}