import { VersionConverter } from ".";
import { PHASES, PhaseTimes } from "../../../../game/gameState.d";
import { Settings } from "../../../../game/localStorage";
import { BASE_ROLE_SETS, RoleSet, getAllRoles } from "../../../../game/roleListState.d";
import { Role } from "../../../../game/roleState.d";
import { Failure, ParseResult, ParseSuccess, Success, isFailure } from "../parse";

const initial: VersionConverter = {
    matchSettings: (json: NonNullable<any>) => typeof json === "object" && !Array.isArray(json) && json.format === undefined,
    convertSettings: convertSettings,

    matchGameModeStorage: (json: NonNullable<any>) => typeof json === "object" && !Array.isArray(json) && json.format === undefined,
    matchShareableGameMode: (json: NonNullable<any>) => typeof json === "object" && !Array.isArray(json) && json.format === undefined,

    convertGameModeStorage,
    convertShareableGameMode
}

type InitialGameMode = { name: string, roleList: InitialRoleOutline[], phaseTimes: Omit<PhaseTimes, 'adjournment'>, disabledRoles: Role[] }
type InitialGameModeStorage = Record<string, InitialGameMode>;

const FACTIONS = ["mafia", "town", "neutral", "cult", "fiends"]
export type InitialRoleOutlineOption = { type: "role", role: Role } | { type: "roleSet", roleSet: RoleSet } | { type: "faction", faction: typeof FACTIONS[number] }
export type InitialRoleOutline = { type: "any" } | { type: "roleOutlineOptions", options: InitialRoleOutlineOption[] }



function convertSettings(json: NonNullable<any>): ParseResult<Settings> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("settingsNotObject", json);
    }

    for(const key of ['volume', 'defaultName', 'language', 'roleSpecificMenus']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key}KeyMissingFromSettings`, json);
        }
    }
    
    const roleSpecificMenus = parseRoleSpecificMenus(json.roleSpecificMenus);
    if (isFailure(roleSpecificMenus)) return roleSpecificMenus;

    return Success({
        format: "v3",
        ...json
    });
}

function parseRoleSpecificMenus(json: NonNullable<any>): ParseResult<Role[]> {
    return Success([]);
}

function convertGameModeStorage(json: NonNullable<any>): ParseResult<any> {
    const storage = parseGameModeStorage(json);

    if (isFailure(storage)) return storage;

    const gameModes = [];

    for (const gameMode of Object.values(storage.value)) {
        let name = gameMode.name;
        const indexOfNumber = name.search(/\d*$/);
        if (indexOfNumber > 0 && name.charAt(indexOfNumber - 1) === ' ') {
            name = name.substring(0, indexOfNumber - 1);
        }

        const existingEntry = gameModes.find(entry => entry.name === name);

        if (existingEntry !== undefined) {
            existingEntry.data[gameMode.roleList.length] = {
                roleList: gameMode.roleList,
                phaseTimes: gameMode.phaseTimes,
                disabledRoles: gameMode.disabledRoles
            }
        } else {
            gameModes.push({
                name: name,
                data: {
                    [gameMode.roleList.length]: {
                        roleList: gameMode.roleList,
                        phaseTimes: gameMode.phaseTimes,
                        disabledRoles: gameMode.disabledRoles
                    }
                }
            })
        }
    }

    return Success({
        format: "v0",
        gameModes
    });
}

export function convertShareableGameMode(json: NonNullable<any>): ParseResult<any> {
    const gameMode = parseGameMode(json);

    if (isFailure(gameMode)) return gameMode;

    return Success({
        format: "v0",
        ...gameMode.value
    });
}

export default initial;

function parseGameModeStorage(json: NonNullable<any>): ParseResult<InitialGameModeStorage> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeStorageNotObject", json);
    }
    
    const parsedEntries: Record<string, any> = {};
    for (const [key, value] of Object.entries(json)) {
        const gameMode = parseGameMode(value);

        if (isFailure(gameMode)) {
            return gameMode;
        }
        
        parsedEntries[key] = gameMode.value
    }

    return Success(parsedEntries);
}

function parseGameMode(json: NonNullable<any>): ParseResult<InitialGameMode> {
    if (typeof json !== "object" || Array.isArray(json)) {
        return Failure("gameModeNotObject", json);
    }

    for (const key of ['name', 'roleList', 'phaseTimes', 'disabledRoles']) {
        if (!Object.keys(json).includes(key)) {
            return Failure(`${key}KeyMissingFromGameMode`, json)
        }
    }

    const name = parseName(json.name);
    if (isFailure(name)) return name;

    const roleList = parseRoleList(json.roleList);
    if (isFailure(roleList)) return roleList;

    const phaseTimes = parsePhaseTimes(json.phaseTimes);
    if (isFailure(phaseTimes)) return phaseTimes;

    const disabledRoles = parseDisabledRoles(json.disabledRoles);
    if (isFailure(disabledRoles)) return disabledRoles;

    return Success({ 
        name: name.value, 
        roleList: roleList.value, 
        phaseTimes: phaseTimes.value, 
        disabledRoles: disabledRoles.value 
    });
}

export function parseName(json: NonNullable<any>): ParseResult<string> {
    if (typeof json !== "string") {
        return Failure("gameModeNameIsNotString", json)
    } else {
        if (json.includes('\n') || json.includes('\r')) {
            return Failure("gameModeNameContainsNewline", json)
        }
        return Success(json);
    }
}

export function parseRoleList(json: NonNullable<any>): ParseResult<InitialRoleOutline[]> {
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

    return Success(roleList.map(success => (success as ParseSuccess<InitialRoleOutline>).value));
}

function parseRoleOutline(json: NonNullable<any>): ParseResult<InitialRoleOutline> {
    if (!Object.keys(json).includes('type')) {
        return Failure("roleOutlineMissingTypeKey", json);
    }

    switch (json.type) {
        case "any":
            return Success({ type: "any" })
        case "roleOutlineOptions":
            if (!Object.keys(json).includes('options')) {
                return Failure("roleOutlineMissingOptionsKey", json);
            }

            const options = parseRoleOutlineOptionList(json.options);
            if (isFailure(options)) return options;

            return Success({
                type: "roleOutlineOptions",
                options: options.value
            });
        default:
            return Failure("roleOutlineInvalidType", json);
    }
}

function parseRoleOutlineOptionList(json: NonNullable<any>): ParseResult<InitialRoleOutlineOption[]> {
    if (!Array.isArray(json)) {
        return Failure("roleOutlineOptionListIsNotArray", json);
    }

    const outlineOptionList = json.map(parseRoleOutlineOption);
    for (const option of outlineOptionList) {
        if (isFailure(option)) return option;
    }

    return Success(outlineOptionList.map(success => (success as ParseSuccess<InitialRoleOutlineOption>).value) as InitialRoleOutlineOption[]);
}

function parseRoleOutlineOption(json: NonNullable<any>): ParseResult<InitialRoleOutlineOption> {
    if (!Object.keys(json).includes('type')) {
        return Failure("roleOutlineOptionMissingTypeKey", json);
    }

    switch (json.type) {
        case "role":
            if (!Object.keys(json).includes('role')) {
                return Failure("roleOutlineOptionMissingRoleKey", json);
            }

            const role = parseRole(json.role);
            if (isFailure(role)) return role;

            return Success({
                type: "role",
                role: role.value
            });
        case "roleSet":
            if (!Object.keys(json).includes('roleSet')) {
                return Failure("roleOutlineOptionMissingRoleSetKey", json);
            }

            const roleSet = parseRoleSet(json.roleSet);
            if (isFailure(roleSet)) return roleSet;

            return Success ({
                type: "roleSet",
                roleSet: roleSet.value
            });
        case "faction":
            if (!Object.keys(json).includes('faction')) {
                return Failure("roleOutlineOptionMissingFactionKey", json);
            }

            const faction = parseFaction(json.faction);
            if (isFailure(faction)) return faction;

            return Success({
                type: "faction",
                faction: faction.value
            });
        default:
            return Failure("roleOutlineOptionInvalidType", json);
    }
}

export function parsePhaseTimes(json: NonNullable<any>): ParseResult<Omit<PhaseTimes, "adjournment">> {
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

    return phaseTimes as ParseResult<PhaseTimes>;
}

export function parseDisabledRoles(json: NonNullable<any>): ParseResult<Role[]> {
    if (!Array.isArray(json)) {
        return Failure("disabledRolesIsNotArray", json);
    }

    const listOfRoles = json.map(parseRole);
    for (const role of listOfRoles) {
        if (isFailure(role)) return role;
    }

    return Success(listOfRoles.map(role => (role as ParseSuccess<Role>).value) as Role[]);
}

export function parseRole(json: NonNullable<any>): ParseResult<Role> {
    if (typeof json !== "string") {
        return Failure("roleIsNotString", json);
    }
    if (!getAllRoles().includes(json as Role)) {
        return Failure("invalidRole", json);
    }
    return Success(json as Role);
}

export function parseRoleSet(json: NonNullable<any>): ParseResult<RoleSet> {
    if (typeof json !== "string") {
        return Failure("roleSetIsNotString", json);
    }
    if (!BASE_ROLE_SETS.includes(json as any)) {
        return Failure("invalidRoleSet", json);
    }
    return Success(json as any);
}

function parseFaction(json: NonNullable<any>): ParseResult<typeof FACTIONS[number]> {
    if (typeof json !== "string") {
        return Failure("factionIsNotString", json);
    }
    if (!FACTIONS.includes(json as any)) {
        return Failure("invalidFaction", json)
    }
    return Success(json as any);
}