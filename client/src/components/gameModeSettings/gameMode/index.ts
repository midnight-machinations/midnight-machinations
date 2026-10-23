import { PhaseTimes } from "../../../game/gameState.d";
import { ModifierID, ModifierState } from "../../../game/modifiers";
import { RoleList } from "../../../game/roleListState.d";
import { Role } from "../../../game/roleState.d";
import { ListMapData } from "../../../ListMap";


// The format field can be any string - it represents the ID of the last migration applied
export type CurrentFormat = string;

export type GameModeStorage = {
    format: CurrentFormat,
    gameModes: GameMode[]
};

export type GameMode = {
    name: string,
    // A mapping from number-of-players to game mode data
    data: Record<number, GameModeData>
};

export type GameModeData = {
    roleList: RoleList,
    phaseTimes: PhaseTimes,
    enabledRoles: Role[],
    modifierSettings: ListMapData<ModifierID, ModifierState>
}

export type ShareableGameMode = GameModeData & { format: CurrentFormat, name: string }