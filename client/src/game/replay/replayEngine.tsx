import { ChatMessage, ChatMessageIndex } from "../../components/ChatMessage";
import { ControllerInput } from "../controllerInput";
import { PhaseState, PlayerIndex } from "../gameState.d";
import translate, { translateChecked } from "../lang";
import { ToClientPacket } from "../packet";
import { Role, RoleState } from "../roleState.d";
import { GameLogEntry, GameLogRecording } from "./replayLog.d";

/**
 * One position in a replay: everything in the log up to and including entry `index` has happened.
 *
 * A step is applied by handing its `packets` to the message listener, exactly as if the server had
 * just sent them. Steps with no packets (an event that changes nothing the client can see) still
 * exist so the timeline lines up one-to-one with the log.
 */
export type ReplayStep = {
    /** Index of the log entry this step replays. */
    index: number,
    /** Milliseconds since the start of the game, used to pace playback. */
    elapsedMs: number,
    /** The phase in effect once this step has been applied. */
    phase: PhaseState,
    dayNumber: number,
    /**
     * How long the clock was set to when this step's phase began, in ms, so the replay can count
     * down the way the players saw it. `null` for a phase without a timer.
     */
    phaseDurationMs: number | null,
    /** When the phase this step is in began, in ms since the start of the game. */
    phaseStartedAtMs: number,
    /** A short human readable summary, shown in the playback bar. */
    description: string,
    packets: ToClientPacket[],
}

export type Replay = {
    fileName: string,
    recording: GameLogRecording,
    /** The packets that set up the game before any of the timeline is replayed. */
    setupPackets: ToClientPacket[],
    steps: ReplayStep[],
    /** The index of the first step of each phase, in order, for skipping phase to phase. */
    phaseStartStepIndices: number[],
    /**
     * The clock the briefing started with, in ms, for the moment before any step has been applied.
     * `null` if the log has no settings to read a phase length from.
     */
    initialPhaseDurationMs: number | null,
    /** How long the recorded game lasted, in ms. */
    durationMs: number,
    playerNames: string[],
}

/**
 * Turns a game log into a replay: a list of steps, each carrying the packets that reproduce what
 * the server told clients at that moment.
 *
 * Phase, liveness, roles and nomination votes are tracked as the log is walked, because the client
 * is sent whole snapshots of those rather than deltas.
 */
export function buildReplay(fileName: string, recording: GameLogRecording): Replay {
    const players = recording.setup?.players ?? recording.result.players;
    const playerNames = players.map(player => player.name as string);
    const playerCount = players.length;

    const alive: boolean[] = players.map(() => true);
    // Everyone is alive before the timeline begins; `alive` is mutated as the log is walked.
    const allAlive: boolean[] = [...alive];
    // The roles players started with. `roles` is mutated as the timeline is walked, so the setup
    // packets have to be built from a copy taken before that happens.
    const startingRoles: Role[] = players.map(player => roleStateToRole(player.role));
    const roles: Role[] = [...startingRoles];

    // Who each player has nominated. Cleared whenever the phase changes, like the server's
    // controllers are.
    let votes = new Map<PlayerIndex, PlayerIndex>();

    let chatMessageIndex: ChatMessageIndex = 0;
    let graveIndex = 0;

    let phase: PhaseState = { type: "briefing" };
    const initialPhaseDurationMs = phaseDuration(recording, phase, allAlive);
    let dayNumber = 1;
    let phaseStartedAtMs = 0;
    let phaseDurationMs: number | null = initialPhaseDurationMs;

    const steps: ReplayStep[] = [];

    const pushStep = (
        index: number,
        entry: GameLogEntry,
        description: string,
        packets: ToClientPacket[]
    ) => {
        steps.push({
            index,
            elapsedMs: entry.elapsedMs,
            phase,
            dayNumber,
            phaseDurationMs,
            phaseStartedAtMs,
            description,
            packets
        });
    };

    const beginPhase = (newPhase: PhaseState, atMs: number) => {
        phase = newPhase;
        phaseStartedAtMs = atMs;
        phaseDurationMs = phaseDuration(recording, newPhase, alive);
        votes = new Map();
    };

    recording.timeline.forEach((entry, index) => {
        if (entry.kind === "chatMessage") {
            const message: ChatMessage = {
                variant: entry.message,
                chatGroup: entry.audience.type === "group" ? entry.audience.group : null,
                replayRecipient: entry.audience.type === "player" ? entry.audience.player : undefined
            };

            const packets: ToClientPacket[] = [];

            // A phase change message carries both the phase and the day number, so it is what the
            // replay drives the header from.
            if (entry.message.type === "phaseChange") {
                dayNumber = entry.message.dayNumber;
                beginPhase(entry.message.phase, entry.elapsedMs);

                packets.push(
                    { type: "phase", phase, dayNumber },
                    { type: "phaseTimeLeft", secondsLeft: secondsOrNull(phaseDurationMs) },
                    { type: "playerVotes", votesForPlayer: [] }
                );
            }

            packets.push({ type: "addChatMessages", chatMessages: [[chatMessageIndex++, message]] });

            pushStep(index, entry, describeChatMessage(entry, playerNames), packets);
            return;
        }

        const packets: ToClientPacket[] = [];

        switch (entry.event) {
            case "OnPhaseStart": {
                // The phase change chat message above normally covers this. This is a fallback for
                // a log where that message was never sent, so the header still keeps up.
                const newPhase = entry.data?.phase as PhaseState | undefined;
                if (newPhase !== undefined && newPhase.type !== phase.type) {
                    beginPhase(newPhase, entry.elapsedMs);
                    packets.push(
                        { type: "phase", phase, dayNumber },
                        { type: "phaseTimeLeft", secondsLeft: secondsOrNull(phaseDurationMs) }
                    );
                }
                break;
            }
            case "OnGraveAdded": {
                if (entry.data?.grave !== undefined) {
                    packets.push({ type: "addGrave", grave: entry.data.grave, graveRef: graveIndex++ });
                }
                break;
            }
            case "OnAnyDeath": {
                const dead = entry.data?.deadPlayer as PlayerIndex | undefined;
                if (dead !== undefined && dead < playerCount) {
                    alive[dead] = false;
                    packets.push({ type: "playerAlive", alive: [...alive] as [boolean] });
                }
                break;
            }
            case "OnRoleSwitch": {
                const player = entry.data?.player as PlayerIndex | undefined;
                const role = entry.data?.new === undefined ? undefined : roleStateToRole(entry.data.new);
                if (player !== undefined && player < playerCount && role !== undefined) {
                    roles[player] = role;
                    packets.push({ type: "yourRoleLabels", roleLabels: roleLabels(roles) });
                }
                break;
            }
            case "OnValidatedControllerInputReceived": {
                const input = entry.data?.input as ControllerInput | undefined;
                if (input?.id.type === "nominate" && input.selection.type === "playerList") {
                    const votee = input.selection.selection[0];
                    if (votee === undefined) {
                        votes.delete(input.id.player);
                    } else {
                        votes.set(input.id.player, votee);
                    }
                    packets.push({ type: "playerVotes", votesForPlayer: voteCounts(votes) });
                }
                break;
            }
            default: break;
        }

        pushStep(index, entry, describeEvent(entry, playerNames), packets);
    });

    return {
        fileName,
        recording,
        setupPackets: buildSetupPackets(recording, playerNames, startingRoles, initialPhaseDurationMs),
        steps,
        phaseStartStepIndices: steps
            .map((step, index) => [step, index] as const)
            .filter(([step, index]) => index === 0 || steps[index - 1].phaseStartedAtMs !== step.phaseStartedAtMs)
            .map(([_, index]) => index),
        initialPhaseDurationMs,
        durationMs: steps[steps.length - 1]?.elapsedMs ?? 0,
        playerNames
    };
}

/** The packets that put the client into the game as it was the instant before the log begins. */
function buildSetupPackets(
    recording: GameLogRecording,
    playerNames: string[],
    roles: Role[],
    briefingDurationMs: number | null
): ToClientPacket[] {
    const settings = recording.setup?.settings;

    const packets: ToClientPacket[] = [
        { type: "lobbyName", name: recording.roomName },
        { type: "gamePlayers", players: playerNames },
        { type: "playerAlive", alive: playerNames.map(() => true) as [boolean] },
        { type: "yourRoleLabels", roleLabels: roleLabels(roles) },
        { type: "playerVotes", votesForPlayer: [] },
    ];

    if (settings !== undefined) {
        packets.push(
            { type: "roleList", roleList: settings.roleList },
            { type: "randomSeed", randomSeed: settings.randomSeed },
            { type: "phaseTimes", phaseTimeSettings: settings.phaseTimes },
            { type: "enabledRoles", roles: settings.enabledRoles },
            { type: "modifierSettings", modifierSettings: settings.modifiers }
        );
    }

    packets.push(
        { type: "phase", phase: { type: "briefing" }, dayNumber: 1 },
        { type: "phaseTimeLeft", secondsLeft: secondsOrNull(briefingDurationMs) }
    );

    return packets;
}

/**
 * How much time was put on the clock when `phase` started, in ms.
 *
 * Mirrors the server's `PhaseStateMachine::get_phase_time_length`, including the speed up that
 * kicks in once three or fewer players are alive, so the replay's timer reads the way the players
 * saw it rather than however long the phase happened to last.
 */
function phaseDuration(
    recording: GameLogRecording,
    phase: PhaseState,
    alive: boolean[]
): number | null {
    if (phase.type === "recess") return null;

    const seconds = recording.setup?.settings.phaseTimes[phase.type];
    if (seconds === undefined) return null;

    const spedUp = alive.filter(isAlive => isAlive).length <= 3;
    return (spedUp ? Math.floor(seconds / 2) : seconds) * 1000;
}

function secondsOrNull(milliseconds: number | null): number | null {
    return milliseconds === null ? null : Math.floor(milliseconds / 1000);
}

function roleLabels(roles: Role[]): [PlayerIndex, Role][] {
    return roles.map((role, player) => [player, role] as [PlayerIndex, Role]);
}

function voteCounts(votes: Map<PlayerIndex, PlayerIndex>): [PlayerIndex, number][] {
    const counts = new Map<PlayerIndex, number>();
    for (const votee of votes.values()) {
        counts.set(votee, (counts.get(votee) ?? 0) + 1);
    }
    return [...counts.entries()];
}

function roleStateToRole(roleState: RoleState): Role {
    return roleState.type;
}

function playerName(playerNames: string[], player: PlayerIndex | undefined): string {
    if (player === undefined) return translate("none");
    return playerNames[player] ?? `${player}`;
}

function describeChatMessage(
    entry: GameLogEntry & { kind: "chatMessage" },
    playerNames: string[]
): string {
    switch (entry.message.type) {
        case "phaseChange":
            return `${translate("phase." + entry.message.phase.type)} ${entry.message.dayNumber}`;
        case "normal": {
            const sender = entry.message.messageSender;
            const name = sender.type === "player" || sender.type === "livingToDead"
                ? playerName(playerNames, sender.player)
                : translate("chatGroup." + sender.type + ".icon");
            return translate("menu.replay.step.said", name);
        }
        case "whisper":
            return translate(
                "menu.replay.step.whispered",
                playerName(playerNames, entry.message.fromPlayerIndex),
                playerName(playerNames, entry.message.toPlayerIndex)
            );
        case "playerDied":
            return translate("menu.replay.step.died", playerName(playerNames, entry.message.grave.player));
        case "gameOver":
            return translate("menu.replay.step.gameOver");
        default:
            if (entry.audience.type === "player") {
                return translate(
                    "menu.replay.step.privateMessage",
                    playerName(playerNames, entry.audience.player)
                );
            }
            return translate("menu.replay.step.message");
    }
}

function describeEvent(
    entry: GameLogEntry & { kind: "event" },
    playerNames: string[]
): string {
    switch (entry.event) {
        case "OnValidatedControllerInputReceived": {
            const input = entry.data?.input as ControllerInput | undefined;
            if (input === undefined) break;
            const actor = "player" in input.id ? input.id.player : undefined;
            return translate("menu.replay.step.usedAbility", playerName(playerNames, actor));
        }
        case "OnRoleSwitch": {
            const role = entry.data?.new === undefined ? undefined : roleStateToRole(entry.data.new);
            if (role === undefined) break;
            return translate(
                "menu.replay.step.becameRole",
                playerName(playerNames, entry.data?.player),
                translate("role." + role + ".name")
            );
        }
        case "OnAnyDeath":
            return translate("menu.replay.step.died", playerName(playerNames, entry.data?.deadPlayer));
        case "OnMidnight":
            return translate("menu.replay.step.midnight");
        case "OnGameEnding":
            return translate("menu.replay.step.gameOver");
        default: break;
    }
    return translateChecked("menu.replay.step." + entry.event) ?? entry.event;
}
