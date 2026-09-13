import { ChatMessageVariant } from "../../components/ChatMessage";
import { ListMapData } from "../../ListMap";
import { Conclusion, ChatGroup, InsiderGroup, PhaseState, PhaseTimes, PlayerIndex, UnsafeString, WinCondition } from "../gameState.d";
import { Grave } from "../graveState";
import { ModifierID, ModifierState } from "../modifiers";
import { RoleList } from "../roleListState.d";
import { Role, RoleState } from "../roleState.d";

/**
 * The shape of one of the server's game logs (`logs/<room>_<time>.json`).
 *
 * This mirrors `server/src/game/game_log.rs` - when the server's log format changes, this changes
 * with it. Everything here comes off disk rather than from a live game, so treat player names and
 * other text as {@link UnsafeString}.
 */
export type GameLogRecording = {
    roomName: UnsafeString,
    /** RFC 3339, as written by chrono. */
    startedAt: string,
    endedAt: string,
    setup: GameLogSetup | null,
    timeline: GameLogEntry[],
    result: GameLogResult,
}

export type GameLogSetup = {
    settings: GameLogSettings,
    players: GameLogPlayerSnapshot[],
}

export type GameLogSettings = {
    randomSeed: number | null,
    roleList: RoleList,
    phaseTimes: PhaseTimes,
    enabledRoles: Role[],
    modifiers: {
        modifiers: ListMapData<ModifierID, ModifierState>,
    },
}

export type GameLogPlayerSnapshot = {
    player: PlayerIndex,
    name: UnsafeString,
    alive: boolean,
    role: RoleState,
    insiderGroups: InsiderGroup[],
    winCondition: WinCondition,
}

export type GameLogResult = {
    conclusion: Conclusion,
    players: GameLogPlayerSnapshot[],
}

/** Who a logged chat message was sent to. */
export type GameLogChatAudience = {
    type: "group",
    group: ChatGroup,
} | {
    type: "player",
    player: PlayerIndex,
}

export type GameLogEntry = {
    timestamp: string,
    /** Milliseconds since the game started. This is what the replay plays back against. */
    elapsedMs: number,
} & ({
    kind: "event",
    /** The name of the Rust event struct, e.g. `"OnPhaseStart"`. */
    event: string,
    data: any,
} | {
    kind: "chatMessage",
    audience: GameLogChatAudience,
    message: ChatMessageVariant,
})

/** The `data` of the logged events the replay reconstructs state from. */
export type GameLogEventData = {
    OnPhaseStart: { phase: PhaseState },
    OnGraveAdded: { grave: Grave },
    OnAnyDeath: { deadPlayer: PlayerIndex },
    OnRoleSwitch: { player: PlayerIndex, old: RoleState, new: RoleState },
    OnAddInsider: { player: PlayerIndex, group: InsiderGroup },
    OnRemoveInsider: { player: PlayerIndex, group: InsiderGroup },
    OnGameEnding: { conclusion: Conclusion },
}
