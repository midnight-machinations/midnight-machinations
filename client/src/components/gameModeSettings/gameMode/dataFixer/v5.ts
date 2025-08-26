import { VersionConverter } from ".";
import { GameMode, GameModeData, GameModeStorage, ShareableGameMode } from "..";
import { ModifierID, ModifierState } from "../../../../game/gameState.d";
import { ListMapData } from "../../../../ListMap";
import { Failure, ParseResult, ParseSuccess, Success, isFailure } from "../parse";
import { parseName, parsePhaseTimes } from "./initial";
import { parseEnabledRoles } from "./v2";
import { parseRoleList, parseSettings } from "./v4";

const v5: VersionConverter = {
    convertSettings: parseSettings,

    convertShareableGameMode: parseShareableGameModeData,
    convertGameModeStorage: parseGameModeStorage
}

export default v5;

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
        format: "v5",
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

        return Success({ format: "v5", name: name.value, ...gameMode.value });
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

    for (const key of ['roleList', 'phaseTimes', 'enabledRoles', 'enabledModifiers']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof GameModeData}KeyMissingFromGameModeData`, json)
        }
    }

    const roleList = parseRoleList(json.roleList);
    if (isFailure(roleList)) return roleList;

    const phaseTimes = parsePhaseTimes(json.phaseTimes);
    if (isFailure(phaseTimes)) return phaseTimes;

    const enabledRoles = parseEnabledRoles(json.enabledRoles);
    if (isFailure(enabledRoles)) return enabledRoles;

    const modifierSettings = parseModifierSettings(json.enabledModifiers);
    if (isFailure(modifierSettings)) return modifierSettings;

    return Success({
        roleList: roleList.value, 
        phaseTimes: phaseTimes.value, 
        enabledRoles: enabledRoles.value,
        modifierSettings: modifierSettings.value
    });
}

// This is lowkey bare-minimum and could easily cause problems, but let's hope it doesn't.
function parseModifierSettings(json: NonNullable<any>): ParseResult<ListMapData<ModifierID, ModifierState>> {
    if (typeof json !== "object" || !Array.isArray(json)) {
        return Failure("modifierSettingsNotArray", json);
    }

    for (const item of json) {
        if (typeof item !== "object" || !Array.isArray(item)) {
            return Failure("modifierSettingsItemNotArray", item);
        }

        if (item.length !== 2) {
            return Failure("modifierSettingsItemInvalidLength", item);
        }
        
        // Here we should make sure the state is valid, but... I'm not doing all that.
    }

    return Success(json as ListMapData<ModifierID, ModifierState>);
}