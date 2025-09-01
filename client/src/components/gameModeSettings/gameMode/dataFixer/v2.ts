import { VersionConverter } from ".";
import { GameMode, GameModeData } from "..";
import { PhaseTimes } from "../../../../game/gameState.d";
import { ModifierID, MODIFIERS } from "../../../../game/modifiers";
import { RoleSet } from "../../../../game/roleListState.d";
import { Role } from "../../../../game/roleState.d";
import { Failure, ParseFailure, ParseResult, ParseSuccess, Success, isFailure } from "../parse";
import { parseName, parsePhaseTimes, parseRoleList, parseRole, InitialRoleOutline } from "./initial";

const v2: VersionConverter = {
    convertShareableGameMode: parseShareableGameModeData,
    convertGameModeStorage: parseGameModeStorage
}

export default v2;

type v3RoleOutlineOption = {
    type: "role",
    role: Role
} | {
    type: "roleSet",
    role: RoleSet
}

type v3RoleOutline = {
    type: "any"
} | {
    type: "roleOutlineOptions"
    options: v3RoleOutlineOption[]
}

type v3GameModeData = {
    roleList: v3RoleOutline[],
    phaseTimes: Omit<PhaseTimes, 'adjournment'>,
    enabledRoles: Role[],
    enabledModifiers: ModifierID[]
}
type v3ShareableGameMode = v3GameModeData & { format: "v3", name: string }
type v3GameMode = {
    name: string,
    data: Record<number, v3GameModeData>
};
type v3GameModeStorage = {
    format: "v3",
    gameModes: v3GameMode[]
};


function parseGameModeStorage(json: NonNullable<any>): ParseResult<v3GameModeStorage> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeStorageNotObject", json);
    }

    for (const key of ['format', 'gameModes']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof v3GameModeStorage}KeyMissingFromGameModeStorage`, json)
        }
    }

    const gameModeList = (json.gameModes as GameMode[]).map(parseGameMode);
    for (const gameMode of gameModeList) {
        if (isFailure(gameMode)) return gameMode;
    }

    return Success({
        format: "v3",
        gameModes: gameModeList.map(gameMode => (gameMode as ParseSuccess<v3GameMode>).value)
    })
}

function parseGameMode(json: NonNullable<any>): ParseResult<v3GameMode> {
    for (const key of ['name', 'data']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof v3GameMode}KeyMissingFromGameMode`, json)
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

function parseShareableGameModeData(json: NonNullable<any>): ParseResult<v3ShareableGameMode> {
    const gameMode = parseGameModeData(json);
    if (isFailure(gameMode)) {
        return gameMode;
    } else {
        if (!Object.keys(json).includes('name')) {
            return Failure("gameModeMissingNameKey", json);
        }

        const name = parseName(json.name);
        if (isFailure(name)) return name;

        return Success({ format: "v3", name: name.value, ...gameMode.value });
    }
}

function parseGameModeDataRecord(json: NonNullable<any>): ParseResult<Record<number, v3GameModeData>> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeDataRecordNotObject", json);
    }
    
    const parsedEntries: Record<number, v3GameModeData> = {};
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

function parseGameModeData(json: NonNullable<any>): ParseResult<v3GameModeData> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeDataNotObject", json);
    }

    for (const key of ['roleList', 'phaseTimes', 'enabledRoles', 'enabledModifiers']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key as keyof GameModeData}KeyMissingFromGameModeData`, json)
        }
    }

    const oldRoleList = parseRoleList(json.roleList);
    if (isFailure(oldRoleList)) return oldRoleList;

    const roleList = convertRoleList(oldRoleList.value);
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

function convertRoleList(roleList: InitialRoleOutline[]): ParseResult<v3RoleOutline[]> {
    const list = roleList.map(outline => {
        switch(outline.type){
            case "any":
                return Success(outline);
            case "roleOutlineOptions":
                return Success({
                    type: "roleOutlineOptions" as const,
                    options: outline.options
                        .map(option => {
                            if (option.type === "faction")
                                return {
                                    type: "roleSet" as const,
                                    roleSet: option.faction as RoleSet,
                                };
                            return option;
                        }
                    )
                })
            default:
                return Failure("outlineTypeDoesntExist", JSON.stringify(outline));
        }
    });
    
    const failure = list.find(el => isFailure(el));
    if (failure !== undefined)
        return failure as ParseFailure;

    return Success(list.map(el => (el as ParseSuccess<v3RoleOutline>).value))
}




export function parseEnabledRoles(json: NonNullable<any>): ParseResult<Role[]> {
    if (!Array.isArray(json)) {
        return Failure("enabledRolesIsNotArray", json);
    }

    const listOfRoles = json.map(parseRole);
    for (const role of listOfRoles) {
        if (isFailure(role)) return role;
    }

    return Success(listOfRoles.map(role => (role as ParseSuccess<Role>).value) as Role[]);
}

export function parseEnabledModifiers(json: NonNullable<any>): ParseResult<ModifierID[]> {
    if (!Array.isArray(json)) {
        return Failure("enabledModifiersIsNotArray", json);
    }

    const listOfModifiers = json.map(parseModifier);
    for (const modifier of listOfModifiers) {
        if (isFailure(modifier)) return modifier;
    }

    return Success(listOfModifiers.map(modifier => (modifier as ParseSuccess<ModifierID>).value) as ModifierID[]);
}

export function parseModifier(json: NonNullable<any>): ParseResult<ModifierID> {

    if (typeof json !== "string") {
        return Failure("modifierIsNotString", json);
    }
    if (!Object.values(MODIFIERS).includes(json as ModifierID)) {
        return Failure("invalidModifier", json);
    }
    return Success(json as ModifierID);
}