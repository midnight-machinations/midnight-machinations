import React from "react";
import { ARTICLES, WikiArticleLink } from "./components/WikiArticleLink";
import { AnchorController } from "./menu/Anchor";
import StandaloneWiki from "./menu/main/StandaloneWiki";
import { deleteReconnectData, loadReconnectData } from "./game/localStorage";
import GAME_MANAGER from ".";
import StartMenu from "./menu/main/StartMenu";
import GameModesEditor from "./components/gameModeSettings/GameModesEditor";
import parseFromJson from "./components/gameModeSettings/gameMode/migrations";
import { isFailure } from "./components/gameModeSettings/gameMode/parse";

function uriAsFileURI(path: string): string {
    if (path.endsWith('/')) {
        return path.substring(0, path.length - 1);
    } else {
        return path;
    }
}

async function routeWiki(anchorController: AnchorController, page: string) {
    const wikiPage = uriAsFileURI(page);

    if (wikiPage === "") {
        anchorController.setContent(<StandaloneWiki />)
    } else if (ARTICLES.includes(wikiPage.substring(1) as any)) {
        anchorController.setContent(<StandaloneWiki initialWikiPage={wikiPage.substring(1) as WikiArticleLink}/>)
    } else {
        return await route404(anchorController, `/wiki${page}`);
    }
}

async function routeLobby(anchorController: AnchorController, roomCode: string) {
    const reconnectData = loadReconnectData();

    if (!await GAME_MANAGER.setOutsideLobbyState()) {
        anchorController.setContent(<StartMenu/>);
        return;
    }
    
    window.history.replaceState({}, "", '/');

    let success: boolean;
    try {
        const code = parseInt(roomCode, 18)
        if (reconnectData) {
            success = await GAME_MANAGER.sendRejoinPacket(code, reconnectData.playerId);
            

            if(!success) {
                deleteReconnectData();
                success = await GAME_MANAGER.sendJoinPacket(code);
            }
        }else{
            success = await GAME_MANAGER.sendJoinPacket(code);
        }
    } catch {
        success = false;
    }
    
    if (!success) {
        await GAME_MANAGER.setDisconnectedState();
        anchorController.clearCoverCard();
        anchorController.setContent(<StartMenu/>)
    }
}

async function routeGameMode(anchorController: AnchorController, gameModeString: string) {
    window.history.replaceState({}, "", "/");
    
    let gameMode: any;
    try {
        gameMode = JSON.parse(gameModeString);
    } catch {
        return await route404(anchorController, `/gameMode/?mode=${gameModeString}`);
    }

    const verifiedGameMode = parseFromJson("ShareableGameMode", gameMode);

    if (isFailure(verifiedGameMode)) {
        console.log(verifiedGameMode.reason);
        console.log(verifiedGameMode.snippet);
        return await route404(anchorController, `/gameMode/?mode=${gameModeString}`);
    } else {
        anchorController.setContent(<StartMenu/>)
        anchorController.setCoverCard(<GameModesEditor initialGameMode={verifiedGameMode.value}/>)
    }
}

async function route404(anchorController: AnchorController, path: string) {
    anchorController.setContent(
        <div style={{ height: "100vh", display: "flex", alignItems: "center", justifyContent: "center", gap: "1rem" }}>
            <h1>404</h1>
            <p>The requested path ({path}) could not be found</p>
        </div>
    )
}

async function routeMainButFirstTryUsingReconnectData(anchorController: AnchorController) {
    window.history.replaceState({}, "", "/");

    const reconnectData = loadReconnectData();
    
    if (!reconnectData) {
        anchorController.setContent(<StartMenu/>)
        return;
    }

    if (!await GAME_MANAGER.setOutsideLobbyState()) {
        anchorController.setContent(<StartMenu/>);
        return;
    }

    if (!await GAME_MANAGER.sendRejoinPacket(reconnectData.roomCode, reconnectData.playerId)) {
        anchorController.setContent(<StartMenu/>);
        deleteReconnectData();
        return;
    }

    // This is where we *should* handle joining the lobby, but it's handled in messageListener... grumble grumble
}

export default async function route(anchorController: AnchorController, url: Location) {

    if (url.pathname.startsWith("/wiki")) {
        return await routeWiki(anchorController, url.pathname.substring(5));
    } else if (url.pathname.startsWith("/connect")) {
        const roomCode = new URLSearchParams(url.search).get("code");
        if (roomCode !== null) {
            return await routeLobby(anchorController, roomCode);
        }
    } else if (url.pathname.startsWith("/gameMode")) {
        const gameMode = new URLSearchParams(url.search).get("mode");
        if (gameMode !== null) {
            return await routeGameMode(anchorController, gameMode);
        }
    }

    

    if (url.pathname && url.pathname !== '/') {
        return await route404(anchorController, url.pathname);
    }
    
    return await routeMainButFirstTryUsingReconnectData(anchorController);
}