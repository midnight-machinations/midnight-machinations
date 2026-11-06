import StyledText from "../components/StyledText";
import RoomVoiceSetting from "../components/gameModeSettings/RoomVoiceSetting";
import React, { ReactElement, useContext, useEffect, useState } from "react";
import translate from "../game/lang";
import GAME_MANAGER from "..";
import { Button } from "../components/Button";
import { useGameState, usePacketListener } from "../components/useHooks";
import { AnchorControllerContext } from "./Anchor";
import "./lobby/lobbyMenu.css"
import "./settings.css";
import LobbyPlayerList from "./lobby/LobbyPlayerList";
import FastForwardButton from "../components/FastForwardButton";

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

    const fastForward = useGameState(state => state.host?.fastForward, ["hostData"]) ?? { type: "none" };

    return <div className="settings-menu-card host-settings-menu">
        <header>
            <h3>{translate("menu.hostSettings.title")}</h3>
            <Button onClick={() => GAME_MANAGER.sendHostDataRequest()}
            >{translate("refresh")}</Button>
        </header>
        <main className="settings-menu">
            <RoomVoiceSetting>
                <div className="host-buttons">
                    <Button onClick={()=>GAME_MANAGER.sendBackToLobbyPacket()}>
                        {translate("backToLobby")}
                    </Button>
                    <Button onClick={()=>GAME_MANAGER.sendHostEndGamePacket()}>
                        {translate("menu.hostSettings.endGame")}
                    </Button>
                    <FastForwardButton value={fastForward} onChange={setting => GAME_MANAGER.sendHostSkipPhase(setting)}/>
                </div>
            </RoomVoiceSetting>
            <LobbyPlayerList />
        </main>
        <footer>
            {translate("menu.hostSettings.lastRefresh", lastRefreshed.toLocaleTimeString())}
        </footer>
    </div>
}
