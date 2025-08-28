import { VersionConverter } from ".";
import { GameMode } from "..";
import { PhaseTimes } from "../../../../game/gameState.d";
import { getDefaultSettings, Settings } from "../../../../game/localStorage";
import { ModifierID } from "../../../../game/modifiers";
import { RoleList, RoleOutline, RoleOutlineOption } from "../../../../game/roleListState.d";
import { Role } from "../../../../game/roleState.d";
import { Failure, ParseResult, ParseSuccess, Success, isFailure } from "../parse";
import { parseName, parsePhaseTimes, parseRole, parseRoleSet } from "./initial";
import { parseEnabledModifiers, parseEnabledRoles } from "./v2";

const v3: VersionConverter = {
    convertSettings: parseSettings,

    convertShareableGameMode: parseShareableGameModeData,
    convertGameModeStorage: parseGameModeStorage
}

export default v3;

type v4GameModeStorage = {
    format: 'v4',
    gameModes: v4GameMode[]
};

type v4GameMode = {
    name: string,
    // A mapping from number-of-players to game mode data
    data: Record<number, v4GameModeData>
};

type v4GameModeData = {
    roleList: RoleList,
    phaseTimes: PhaseTimes,
    enabledRoles: Role[],
    enabledModifiers: ModifierID[]
}

type v4ShareableGameMode = v4GameModeData & { format: 'v4', name: string }

function parseSettings(json: NonNullable<any>): ParseResult<Settings> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("settingsNotObject", json);
    }

    for(const key of ['format', 'volume', 'fontSize', 'accessibilityFont', 'defaultName', 'language', 'roleSpecificMenus']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof Settings}KeyMissingFromSettings`, json);
        }
    }

    for(const key of ['maxMenus', 'menuOrder']) {
        if (!Object.keys(json).includes(key)) {
            json.maxMenus = getDefaultSettings().maxMenus
            json.menuOrder = getDefaultSettings().menuOrder
        }
    }

    if (json.format !== "v3") {
        return Failure("settingsFormatNotV3", json);
    }
    
    const roleSpecificMenus = parseRoleSpecificMenus(json.roleSpecificMenus);
    if (isFailure(roleSpecificMenus)) return roleSpecificMenus;

    return Success(json);
}

function parseRoleSpecificMenus(json: NonNullable<any>): ParseResult<Role[]> {
    if (!Array.isArray(json)) {
        return Failure("roleSpecificMenusNotArray", json);
    }

    const roleList = json.map(parseRole);
    for (const role of roleList) {
        if (isFailure(role)) return role;
    }

    return Success(roleList.map(success => (success as ParseSuccess<Role>).value));
}

function parseGameModeStorage(json: NonNullable<any>): ParseResult<v4GameModeStorage> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeStorageNotObject", json);
    }

    for (const key of ['format', 'gameModes']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof v4GameModeStorage}KeyMissingFromGameModeStorage`, json)
        }
    }

    const gameModeList = (json.gameModes as v4GameMode[]).map(parseGameMode);
    for (const gameMode of gameModeList) {
        if (isFailure(gameMode)) return gameMode;
    }

    return Success({
        format: "v4",
        gameModes: gameModeList.map(gameMode => (gameMode as ParseSuccess<v4GameMode>).value)
    })
}

function parseGameMode(json: NonNullable<any>): ParseResult<v4GameMode> {
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

function parseShareableGameModeData(json: NonNullable<any>): ParseResult<v4ShareableGameMode> {
    const gameMode = parseGameModeData(json);
    if (isFailure(gameMode)) {
        return gameMode;
    } else {
        if (!Object.keys(json).includes('name')) {
            return Failure("gameModeMissingNameKey", json);
        }

        const name = parseName(json.name);
        if (isFailure(name)) return name;

        return Success({ format: "v4", name: name.value, ...gameMode.value });
    }
}

function parseGameModeDataRecord(json: NonNullable<any>): ParseResult<Record<number, v4GameModeData>> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeDataRecordNotObject", json);
    }
    
    const parsedEntries: Record<number, v4GameModeData> = {};
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

function parseGameModeData(json: NonNullable<any>): ParseResult<v4GameModeData> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeDataNotObject", json);
    }

    for (const key of ['roleList', 'phaseTimes', 'enabledRoles', 'enabledModifiers']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof v4GameModeData}KeyMissingFromGameModeData`, json)
        }
    }

    const roleList = parseRoleList(json.roleList);
    if (isFailure(roleList)) return roleList;

    const phaseTimes = parsePhaseTimes(json.phaseTimes);
    if (isFailure(phaseTimes)) return phaseTimes;

    const enabledRoles = parseEnabledRoles(json.enabledRoles);
    if (isFailure(enabledRoles)) return enabledRoles;

    const enabledModifiers = parseEnabledModifiers(json.enabledModifiers);
    if (isFailure(enabledModifiers)) return enabledModifiers;

    return Success({
        roleList: roleList.value, 
        phaseTimes: phaseTimes.value, 
        enabledRoles: enabledRoles.value,
        enabledModifiers: enabledModifiers.value
    });
}

function parseRoleList(json: NonNullable<any>): ParseResult<RoleOutline[]> {
    if (!Array.isArray(json)) {
        return Failure("roleListIsNotArray", json);
    }

    if (json.length === 0) {
        return Failure("roleListIsEmpty", json);
    }

    const roleList = json.map(parseRoleOutline);

    for (const outline of roleList) {
        if (isFailure(outline)) return outline;
    }

    return Success(roleList.map(success => (success as ParseSuccess<RoleOutline>).value));
}

function parseRoleOutline(json: NonNullable<any>): ParseResult<RoleOutline> {
    if (!Object.keys(json).includes('type')) {
        return Failure("roleOutlineMissingTypeKey", json);
    }

    switch (json.type) {
        case "any":
            return Success([{ roleSet: "any" }])
        case "roleOutlineOptions": {
            if (!Object.keys(json).includes('options')) {
                return Failure("roleOutlineMissingOptionsKey", json);
            }

            const options = parseRoleOutlineOptionList(json.options);
            if (isFailure(options)) return options;

            return Success(options.value);
        }
        default:
            return Failure("roleOutlineInvalidType", json);
    }
}

function parseRoleOutlineOptionList(json: NonNullable<any>): ParseResult<RoleOutlineOption[]> {
    if (!Array.isArray(json)) {
        return Failure("roleOutlineOptionListIsNotArray", json);
    }

    const outlineOptionList = json.map(parseRoleOutlineOption);
    for (const option of outlineOptionList) {
        if (isFailure(option)) return option;
    }

    return Success(outlineOptionList.map(success => (success as ParseSuccess<RoleOutlineOption>).value) as RoleOutlineOption[]);
}

function parseRoleOutlineOption(json: NonNullable<any>): ParseResult<RoleOutlineOption> {
    if (!Object.keys(json).includes('type')) {
        return Failure("roleOutlineOptionMissingTypeKey", json);
    }

    switch (json.type) {
        case "role": {
            if (!Object.keys(json).includes('role')) {
                return Failure("roleOutlineOptionMissingRoleKey", json);
            }

            const role = parseRole(json.role);
            if (isFailure(role)) return role;

            return Success({
                role: role.value
            });
        }
        case "roleSet": {
            if (!Object.keys(json).includes('roleSet')) {
                return Failure("roleOutlineOptionMissingRoleSetKey", json);
            }

            const roleSet = parseRoleSet(json.roleSet);
            if (isFailure(roleSet)) return roleSet;

            return Success ({
                roleSet: roleSet.value
            });
        }
        default:
            return Failure("roleOutlineOptionInvalidType", json);
    }
}