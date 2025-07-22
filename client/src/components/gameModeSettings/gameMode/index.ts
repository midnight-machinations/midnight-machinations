import { ModifierType } from "../../../stateContext/stateType/modifiersState";
import { PhaseTimes } from "../../../stateContext/stateType/otherState";
import { RoleList } from "../../../stateContext/stateType/roleListState";
import { Role } from "../../../stateContext/stateType/roleState";

export type CurrentFormat = "v4";

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
    enabledModifiers: ModifierType[]
}

export type ShareableGameMode = GameModeData & { format: CurrentFormat, name: string }