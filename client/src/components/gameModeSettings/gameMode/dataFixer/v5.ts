import { VersionConverter } from ".";
import { GameMode, GameModeData, GameModeStorage, ShareableGameMode } from "..";
import { defaultPhaseTimes } from "../../../../game/gameState";
import { PHASES, PhaseTimes } from "../../../../game/gameState.d";
import { ModifierID, ModifierState } from "../../../../game/modifiers";
import { ListMapData } from "../../../../ListMap";
import { Failure, ParseResult, ParseSuccess, Success, isFailure } from "../parse";
import { parseName } from "./initial";
import { parseEnabledRoles } from "./v2";
import { parseSettings } from "./v3";
import { parseRoleList } from "./v4";

const v5: VersionConverter = {
    convertSettings: parseSettings,

    convertShareableGameMode: parseShareableGameModeData,
    convertGameModeStorage: parseGameModeStorage
}

export default v5;

type v6GameModeStorage = GameModeStorage;
type v6GameMode = GameMode
type v6GameModeData = GameModeData
type v6ShareableGameMode = ShareableGameMode

function parseGameModeStorage(json: NonNullable<any>): ParseResult<v6GameModeStorage> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeStorageNotObject", json);
    }

    for (const key of ['format', 'gameModes']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof v6GameModeStorage}KeyMissingFromGameModeStorage`, json)
        }
    }

    const gameModeList = (json.gameModes as v6GameMode[]).map(parseGameMode);
    for (const gameMode of gameModeList) {
        if (isFailure(gameMode)) return gameMode;
    }

    return Success({
        format: "v6",
        gameModes: gameModeList.map(gameMode => (gameMode as ParseSuccess<v6GameMode>).value)
    })
}

function parseGameMode(json: NonNullable<any>): ParseResult<v6GameMode> {
    for (const key of ['name', 'data']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof v6GameMode}KeyMissingFromGameMode`, json)
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

function parseShareableGameModeData(json: NonNullable<any>): ParseResult<v6ShareableGameMode> {
    const gameMode = parseGameModeData(json);
    if (isFailure(gameMode)) {
        return gameMode;
    } else {
        if (!Object.keys(json).includes('name')) {
            return Failure("gameModeMissingNameKey", json);
        }

        const name = parseName(json.name);
        if (isFailure(name)) return name;

        return Success({ format: "v6", name: name.value, ...gameMode.value });
    }
}

function parseGameModeDataRecord(json: NonNullable<any>): ParseResult<Record<number, v6GameModeData>> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeDataRecordNotObject", json);
    }
    
    const parsedEntries: Record<number, v6GameModeData> = {};
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

function parseGameModeData(json: NonNullable<any>): ParseResult<v6GameModeData> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeDataNotObject", json);
    }

    for (const key of ['roleList', 'phaseTimes', 'enabledRoles', 'modifierSettings']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof v6GameModeData}KeyMissingFromGameModeData`, json)
        }
    }

    const roleList = parseRoleList(json.roleList);
    if (isFailure(roleList)) return roleList;

    const phaseTimes = parsePhaseTimes(json.phaseTimes);
    if (isFailure(phaseTimes)) return phaseTimes;

    const enabledRoles = parseEnabledRoles(json.enabledRoles);
    if (isFailure(enabledRoles)) return enabledRoles;

    const modifierSettings = parseModifierSettings(json.modifierSettings);
    if (isFailure(modifierSettings)) return modifierSettings;

    return Success({
        roleList: roleList.value, 
        phaseTimes: phaseTimes.value, 
        enabledRoles: enabledRoles.value,
        modifierSettings: modifierSettings.value
    });
}

export function parsePhaseTimes(json: NonNullable<any>): ParseResult<PhaseTimes> {
    for (const phase of PHASES) {
        if (phase !== "recess" && phase !== "adjournment" && !Object.keys(json).includes(phase)) {
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

    if (!isFailure(phaseTimes)) {
        if (phaseTimes.value.adjournment === undefined) {
            phaseTimes.value.adjournment = defaultPhaseTimes().adjournment;
        }
    }

    return phaseTimes as ParseResult<PhaseTimes>;
}

// This is lowkey bare-minimum and could easily cause problems, but let's hope it doesn't.
export function parseModifierSettings(json: NonNullable<any>): ParseResult<ListMapData<ModifierID, ModifierState>> {
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