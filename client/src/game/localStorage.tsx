import DEFAULT_GAME_MODES from "../resources/defaultGameModes.json";
import { CurrentFormat, GameModeStorage } from "../components/gameModeSettings/gameMode";
import { Language } from "./lang";
import parseFromJson from "../components/gameModeSettings/gameMode/dataFixer";
import { ContentMenu } from "../menu/game/GameScreen";
import { ParseResult, Success } from "../components/gameModeSettings/gameMode/parse";
import { UnsafeString } from "./gameState.d";
import { ListMapData } from "../ListMap";


export function saveReconnectData(roomCode: number, playerId: number) {
    localStorage.setItem(
        "reconnectData",
        JSON.stringify({
            "roomCode": roomCode,
            "playerId": playerId,
            "lastSaveTime": Date.now()
        })
    );
}
export function deleteReconnectData() {
    localStorage.removeItem("reconnectData");
}
export function loadReconnectData(): {
    roomCode: number,
    playerId: number,
    lastSaveTime: number,
} | null {
    let dataJSON = localStorage.getItem("reconnectData");
    
    if (dataJSON) {
        let reconnectData = JSON.parse(dataJSON);
    
        // Make sure it isn't expired
        const HOUR_IN_SECONDS = 3_600_000;
        if (reconnectData.lastSaveTime < Date.now() - HOUR_IN_SECONDS) {
            deleteReconnectData();
            return null
        }

        return reconnectData;
    }

    return null;
}



export type Settings = {
    format: CurrentFormat;
    volume: number;
    fontSize: number;
    accessibilityFont: boolean;
    defaultName: UnsafeString | null;
    language: Language;
    maxMenus: number;
    menuOrder: ListMapData<ContentMenu, boolean>
};

export type RoleSpecificMenuType = "playerList" | "standalone";



export function loadSettingsParsed(): Settings {
    const result = parseFromJson("Settings", loadSettings());
    if(result.type === "failure") {
        return getDefaultSettings();
    }else{
        return result.value;
    }
}
export function getDefaultSettings(): Readonly<Settings> {
    const mobile = window.innerWidth < 600;

    let menuOrder: undefined | ListMapData<ContentMenu, boolean> = undefined;
    if(mobile) {
        menuOrder = [
            [ContentMenu.ChatMenu, true],
            [ContentMenu.WikiMenu, false], 
            [ContentMenu.GraveyardMenu, false], 
            [ContentMenu.PlayerListMenu, false],
            [ContentMenu.WillMenu, false], 
            [ContentMenu.RoleSpecificMenu, false]
        ]
    }else{
        menuOrder = [
            [ContentMenu.WikiMenu, false], 
            [ContentMenu.GraveyardMenu, false], 
            [ContentMenu.PlayerListMenu, true], 
            [ContentMenu.ChatMenu, true], 
            [ContentMenu.WillMenu, false], 
            [ContentMenu.RoleSpecificMenu, true]
        ]
    }

    return {
        format: "v6",
        volume: 0.5,
        fontSize: 1,
        accessibilityFont: false,
        language: "en_us",
        defaultName: null,
        maxMenus: mobile ? 1 : 6,
        menuOrder: menuOrder,
    }
}
export function loadSettings(): unknown {
    const data = localStorage.getItem("settings");
    if (data !== null) {
        try {
            return JSON.parse(data);
        } catch {
            return null;
        }
    }
    return getDefaultSettings();
}
export function saveSettings(newSettings: Partial<Settings>) {
    const currentSettings = parseFromJson("Settings", loadSettings());

    console.log(currentSettings);


    if(currentSettings.type === "failure") {
        localStorage.setItem("settings", JSON.stringify({
            ...getDefaultSettings(),
            ...newSettings,
        }));
    }else{
        localStorage.setItem("settings", JSON.stringify({
            ...currentSettings.value,
            ...newSettings,
        }));
    }
}

let cachedGameModes: ParseResult<GameModeStorage> | null = null;

export function loadGameModesParsed(): ParseResult<GameModeStorage> {

    if(cachedGameModes !== null) return cachedGameModes;

    cachedGameModes = parseFromJson("GameModeStorage", loadGameModes());
    return cachedGameModes;
}
export function defaultGameModes(): unknown {
    // Typescript is a Division One tweaker
    return DEFAULT_GAME_MODES;
}
export function saveGameModes(gameModes: GameModeStorage) {
    cachedGameModes = Success(gameModes);
    localStorage.setItem("savedGameModes", JSON.stringify(gameModes));
}
export function loadGameModes(): unknown {
    const data = localStorage.getItem("savedGameModes");
    if (data !== null) {
        try {
            return JSON.parse(data);
        } catch {
            return null;
        }
    }
    return defaultGameModes();
}
export function deleteGameModes() {
    localStorage.removeItem("savedGameModes");
}

