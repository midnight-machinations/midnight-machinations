import { VersionConverter } from ".";
import { GameMode, GameModeData, GameModeStorage, ShareableGameMode } from "..";
import { PHASES, PhaseTimes } from "../../../../game/gameState.d";
import { getDefaultSettings, Settings } from "../../../../game/localStorage";
import { Failure, ParseResult, ParseSuccess, Success, isFailure } from "../parse";
import { parseName } from "./initial";
import { parseEnabledRoles } from "./v2";
// v7 expects already converted data, so no need to parse roleList - just validate
import { parseModifierSettings } from "./v5";

const v7: VersionConverter = {
    convertSettings: parseSettings,

    convertShareableGameMode: parseShareableGameModeData,
    convertGameModeStorage: parseGameModeStorage
}

export default v7;

export function parseSettings(json: NonNullable<any>): ParseResult<Settings> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("settingsNotObject", json);
    }

    for(const key of ['format', 'volume', 'fontSize', 'accessibilityFont', 'defaultName', 'language']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof Settings}KeyMissingFromSettings`, json);
        }
    }

    if (!Object.keys(json).includes("maxMenus")) {
        json.maxMenus = getDefaultSettings().maxMenus
    }

    for(const key of ['WikiMenu', 'GraveyardMenu', "PlayerListMenu", "ChatMenu", "WillMenu", "RoleSpecificMenu"]) {
        if(!Array.isArray(json.menuOrder[key])){
            json.menuOrder[key] = [key, key==="ChatMenu"]
        }
    }
    
    if(Object.keys(json).includes("roleSpecificMenus")){
        delete json.roleSpecificMenus;
    }

    return Success({ ...json, format: "v7" });
}

function parseGameModeStorage(json: NonNullable<any>): ParseResult<GameModeStorage> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeStorageNotObject", json);
    }

    for (const key of ['format', 'gameModes']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof GameModeStorage}KeyMissingFromGameModeStorage`, json)
        }
    }

    const gameModeList = (json.gameModes as GameMode[]).map(parseGameMode);
    for (const gameMode of gameModeList) {
        if (isFailure(gameMode)) return gameMode;
    }

    return Success({
        format: "v7",
        gameModes: gameModeList.map(gameMode => (gameMode as ParseSuccess<GameMode>).value)
    })
}

function parseGameMode(json: NonNullable<any>): ParseResult<GameMode> {
    for (const key of ['name', 'data']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof GameMode}KeyMissingFromGameMode`, json)
        }
    }

    const name = parseName(json.name);
    if (isFailure(name)) return name;

    const gameModeDataRecord = parseGameModeDataRecord(json.data);
    if (isFailure(gameModeDataRecord)) return gameModeDataRecord;

    return Success({
        name: name.value,
        data: gameModeDataRecord.value
    })
}

function parseShareableGameModeData(json: NonNullable<any>): ParseResult<ShareableGameMode> {
    const gameMode = parseGameModeData(json);
    if (isFailure(gameMode)) {
        return gameMode;
    } else {
        if (!Object.keys(json).includes('name')) {
            return Failure("gameModeMissingNameKey", json);
        }

        const name = parseName(json.name);
        if (isFailure(name)) return name;

        return Success({ format: "v7", name: name.value, ...gameMode.value });
    }
}

function parseGameModeDataRecord(json: NonNullable<any>): ParseResult<Record<number, GameModeData>> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeDataRecordNotObject", json);
    }
    
    const parsedEntries: Record<number, GameModeData> = {};
    for (const [key, value] of Object.entries(json)) {
        let players;
        try {
            players = parseInt(key)
        } catch {
            return Failure("gameModeDataRecordKeyNotNumber", key);
        }

        const datum = parseGameModeData(value);

        if (isFailure(datum)) {
            return datum;
        }

        if (datum.value.roleList.length !== players) {
            return Failure("gameModeDataRecordKeyDoesNotMatchRoleListLength", json);
        }
        
        parsedEntries[players] = datum.value
    }

    return Success(parsedEntries);
}

function parseGameModeData(json: NonNullable<any>): ParseResult<GameModeData> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeDataNotObject", json);
    }

    for (const key of ['roleList', 'phaseTimes', 'enabledRoles', 'modifierSettings']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof GameModeData}KeyMissingFromGameModeData`, json)
        }
    }

    // v7 expects roleList to already be in correct format from v6 conversion
    const roleList = json.roleList;

    const phaseTimes = parsePhaseTimes(json.phaseTimes);
    if (isFailure(phaseTimes)) return phaseTimes;

    const enabledRoles = parseEnabledRoles(json.enabledRoles);
    if (isFailure(enabledRoles)) return enabledRoles;

    const modifierSettings = parseModifierSettings(json.modifierSettings);
    if (isFailure(modifierSettings)) return modifierSettings;

    return Success({
        roleList: roleList, 
        phaseTimes: phaseTimes.value, 
        enabledRoles: enabledRoles.value,
        modifierSettings: modifierSettings.value
    });
}

export function parsePhaseTimes(json: NonNullable<any>): ParseResult<PhaseTimes> {
    for (const phase of PHASES) {
        if (phase !== "recess" && !Object.keys(json).includes(phase)) {
            return Failure(`${phase}KeyMissingFromPhaseTimes`, json);
        }
    }

    const phaseTimes = PHASES.reduce(
        (acc, phase) => {
            if (isFailure(acc)) return acc;

            try {
                return Success({
                    ...acc.value,
                    [phase]: Number(json[phase])
                })
            } catch {
                return Failure(`${phase}ValueOfPhaseTimesIsNotNumber`, json[phase]);
            }
        }, 
        Success({}) as ParseResult<Partial<PhaseTimes>>
    )

    return phaseTimes as ParseResult<PhaseTimes>;
}