import GAME_MANAGER from "../..";
import AudioController from "../../menu/AudioController";
import { StateEventType } from "../gameManager.d";
import messageListener, { setApplyingReplay } from "../messageListener";
import { ToClientPacket } from "../packet";
import { Replay, ReplayStep } from "./replayEngine";

/** Playback speeds offered by the playback bar, as multiples of the speed the game was played at. */
export const REPLAY_SPEEDS = [0.5, 1, 2, 4, 8, 16] as const;

const TICK_MS = 50;

/** Notified whenever the playhead, play/pause state or speed changes. */
export type ReplayListener = () => void;

/**
 * Drives a {@link Replay}: it owns the playhead and pushes the replay's packets into
 * `GAME_MANAGER.state`, so the ordinary game screen renders the recorded game.
 *
 * Playing forwards applies steps as the playhead reaches them. Seeking backwards can't undo
 * packets, so it rebuilds the game state from scratch and reapplies everything up to the new
 * position.
 */
export class ReplayController {
    readonly replay: Replay;

    /** How many steps have been applied. `steps[stepsApplied - 1]` is the most recent one. */
    private appliedStepCount = 0;
    private playheadMs = 0;
    private speed: number = 1;
    private interval: ReturnType<typeof setInterval> | null = null;
    private listeners: ReplayListener[] = [];
    /** Every packet type this replay can send, used to refresh the whole screen after a rebuild. */
    private readonly packetTypes: Set<StateEventType>;

    constructor(replay: Replay) {
        this.replay = replay;
        this.packetTypes = new Set([
            ...replay.setupPackets,
            ...replay.steps.flatMap(step => step.packets)
        ].map(packet => packet.type));
        this.reset();
    }

    /* Reading the current position */

    get currentStepCount(): number {
        return this.appliedStepCount;
    }
    get stepCount(): number {
        return this.replay.steps.length;
    }
    get elapsedMs(): number {
        return this.playheadMs;
    }
    get durationMs(): number {
        return this.replay.durationMs;
    }
    get playbackSpeed(): number {
        return this.speed;
    }
    get isPlaying(): boolean {
        return this.interval !== null;
    }
    get isFinished(): boolean {
        return this.appliedStepCount >= this.stepCount;
    }
    /** The most recently applied step, or `null` before the replay has begun. */
    get currentStep(): ReplayStep | null {
        return this.replay.steps[this.appliedStepCount - 1] ?? null;
    }

    /* Controls */

    play() {
        if (this.interval !== null) return;

        // Restart rather than sitting at the end doing nothing.
        if (this.isFinished) this.seekToStep(0);

        this.interval = setInterval(() => this.tick(TICK_MS), TICK_MS);
        this.notify();
    }

    pause() {
        if (this.interval === null) return;
        clearInterval(this.interval);
        this.interval = null;
        this.notify();
    }

    togglePlay() {
        if (this.isPlaying) {
            this.pause();
        } else {
            this.play();
        }
    }

    setSpeed(speed: number) {
        this.speed = speed;
        this.notify();
    }

    /** Moves to just after step `count` has been applied. */
    seekToStep(count: number) {
        const clamped = Math.max(0, Math.min(this.stepCount, Math.floor(count)));

        if (clamped < this.appliedStepCount) {
            this.rebuildTo(clamped);
        } else {
            this.applyStepsUpTo(clamped);
        }

        this.playheadMs = this.replay.steps[clamped - 1]?.elapsedMs ?? 0;
        this.updateTimer();
        GAME_MANAGER.invokeStateListeners("replaySeek");
        this.notify();
    }

    /** Moves to `milliseconds` into the recorded game. */
    seekToMs(milliseconds: number) {
        const clamped = Math.max(0, Math.min(this.durationMs, milliseconds));
        const count = this.stepCountAtMs(clamped);

        if (count < this.appliedStepCount) {
            this.rebuildTo(count);
        } else {
            this.applyStepsUpTo(count);
        }

        this.playheadMs = clamped;
        this.updateTimer();
        GAME_MANAGER.invokeStateListeners("replaySeek");
        this.notify();
    }

    stepForward() {
        this.pause();
        this.seekToStep(this.appliedStepCount + 1);
    }

    stepBackward() {
        this.pause();
        this.seekToStep(this.appliedStepCount - 1);
    }

    /** Jumps to the start of the next phase, or to the end of the replay if there isn't one. */
    nextPhase() {
        this.pause();
        const next = this.replay.phaseStartStepIndices.find(index => index + 1 > this.appliedStepCount);
        this.seekToStep(next === undefined ? this.stepCount : next + 1);
    }

    /**
     * Jumps to the start of the phase being watched, or to the phase before it when already at the
     * start of one - the way skipping back a track works.
     */
    previousPhase() {
        this.pause();
        const previous = [...this.replay.phaseStartStepIndices]
            .reverse()
            .find(index => index + 1 < this.appliedStepCount);
        this.seekToStep(previous === undefined ? 0 : previous + 1);
    }

    restart() {
        this.pause();
        this.seekToStep(0);
    }

    /** Stops playback and releases the interval. Must be called when the replay screen unmounts. */
    destroy() {
        this.pause();
        this.listeners = [];
    }

    /* Listeners */

    addListener(listener: ReplayListener) {
        this.listeners.push(listener);
    }
    removeListener(listener: ReplayListener) {
        const index = this.listeners.indexOf(listener);
        if (index !== -1) this.listeners.splice(index, 1);
    }
    private notify() {
        for (const listener of [...this.listeners]) listener();
    }

    /* Playback */

    private tick(deltaMs: number) {
        this.playheadMs += deltaMs * this.speed;

        this.applyStepsUpTo(this.stepCountAtMs(this.playheadMs));
        this.updateTimer();

        if (this.isFinished && this.playheadMs >= this.durationMs) {
            this.playheadMs = this.durationMs;
            this.pause();
            return;
        }

        this.notify();
    }

    /** How many steps have happened by `milliseconds` into the game. */
    private stepCountAtMs(milliseconds: number): number {
        // The log is in timestamp order, so this is the first step that hasn't happened yet.
        const next = this.replay.steps.findIndex(step => step.elapsedMs > milliseconds);
        return next === -1 ? this.stepCount : next;
    }

    private applyStepsUpTo(count: number) {
        if (count <= this.appliedStepCount) return;

        const packets = this.replay.steps
            .slice(this.appliedStepCount, count)
            .flatMap(step => step.packets);

        applyPackets(packets.length > 32 ? coalescePackets(packets) : packets);
        this.appliedStepCount = count;
    }

    /** Throws away the replayed game state and replays the first `count` steps into a fresh one. */
    private rebuildTo(count: number) {
        this.reset();
        this.applyStepsUpTo(count);
    }

    private reset() {
        AudioController.clearQueue();

        GAME_MANAGER.setSpectatorGameState();
        if (GAME_MANAGER.state.stateType === "game") {
            GAME_MANAGER.state.isReplay = true;
            GAME_MANAGER.state.lobbyName = this.replay.recording.roomName;
            // The replay drives the clock itself, so the game manager's tick must not also count down.
            GAME_MANAGER.state.ticking = false;
        }

        this.appliedStepCount = 0;
        applyPackets(this.replay.setupPackets);

        // A rebuild can undo things as well as redo them - a grave that hasn't been dug yet, a
        // player who is alive again - and the packets that come next say nothing about those. Every
        // type the replay touches has to be refreshed, not just the ones in the packets replayed.
        markForRefresh(this.packetTypes);
    }

    /** Puts the clock where it was at the current playhead, matching what the players saw. */
    private updateTimer() {
        if (GAME_MANAGER.state.stateType !== "game") return;

        const step = this.currentStep;
        const startedAtMs = step?.phaseStartedAtMs ?? 0;
        const durationMs = step === null
            ? this.replay.initialPhaseDurationMs
            : step.phaseDurationMs;

        const timeLeftMs = durationMs === null
            ? null
            : Math.max(0, durationMs - Math.max(0, this.playheadMs - startedAtMs));

        if (GAME_MANAGER.state.timeLeftMs !== timeLeftMs) {
            GAME_MANAGER.state.timeLeftMs = timeLeftMs;
            GAME_MANAGER.invokeStateListeners("tick");
        }
    }
}

/**
 * Hands packets to the message listener as if the server had sent them.
 *
 * Marked as a replay for the duration so the listener doesn't log hundreds of synthesized packets
 * to the console on every seek.
 */
function applyPackets(packets: ToClientPacket[]) {
    setApplyingReplay(true);
    try {
        for (const packet of packets) {
            messageListener(packet);
        }
    } finally {
        setApplyingReplay(false);
    }

    scheduleRefresh(packets);
}

/**
 * Event types whose listeners still need to be told about a batch of applied packets.
 *
 * A live game trickles packets in one at a time, but a replay applies a whole batch of them in one
 * go - hundreds of them when seeking. Components subscribe through `usePacketListener`, which hands
 * its listener over across React's render cycle, so a batch applied while a component is between
 * renders can be missed and leave that component showing a stale value even though
 * `GAME_MANAGER.state` is already correct. Notifying the listeners again once React has settled
 * fixes them up: they re-read the state rather than being handed a value, so a second notification
 * is harmless.
 */
const PENDING_REFRESH = new Set<StateEventType>();
let refreshScheduled = false;

function scheduleRefresh(packets: ToClientPacket[]) {
    markForRefresh(packets.map(packet => packet.type));
}

function markForRefresh(types: Iterable<StateEventType>) {
    for (const type of types) {
        PENDING_REFRESH.add(type);
    }

    if (refreshScheduled || PENDING_REFRESH.size === 0) return;
    refreshScheduled = true;

    setTimeout(() => {
        refreshScheduled = false;
        const types = [...PENDING_REFRESH];
        PENDING_REFRESH.clear();
        for (const type of types) {
            GAME_MANAGER.invokeStateListeners(type);
        }
    }, 0);
}

/**
 * Collapses a long run of packets into an equivalent, much shorter one.
 *
 * Every packet type the replay emits except `addGrave` is a whole snapshot rather than a delta, so
 * only the last of each matters, and all the chat messages can be added in one go. This keeps a
 * seek to the end of a long game from being quadratic in the number of chat messages.
 */
function coalescePackets(packets: ToClientPacket[]): ToClientPacket[] {
    const SNAPSHOT_TYPES: ReadonlySet<string> = new Set([
        "phase", "phaseTimeLeft", "playerAlive", "playerVotes", "yourRoleLabels",
        "lobbyName", "gamePlayers", "roleList", "randomSeed", "phaseTimes",
        "enabledRoles", "modifierSettings"
    ]);

    const chatMessages: (ToClientPacket & { type: "addChatMessages" })["chatMessages"] = [];
    const seenSnapshotTypes = new Set<string>();
    const kept: ToClientPacket[] = [];

    // Backwards, so the packet kept for each snapshot type is the last one.
    for (let i = packets.length - 1; i >= 0; i--) {
        const packet = packets[i];

        if (packet.type === "addChatMessages") {
            chatMessages.unshift(...packet.chatMessages);
        } else if (SNAPSHOT_TYPES.has(packet.type)) {
            if (seenSnapshotTypes.has(packet.type)) continue;
            seenSnapshotTypes.add(packet.type);
            kept.unshift(packet);
        } else {
            kept.unshift(packet);
        }
    }

    if (chatMessages.length !== 0) {
        kept.push({ type: "addChatMessages", chatMessages });
    }

    return kept;
}

const HOLDER: { controller: ReplayController | null } = { controller: null };

/** The replay currently being watched, if any. */
export function getReplayController(): ReplayController | null {
    return HOLDER.controller;
}

export function setReplayController(controller: ReplayController | null) {
    if (HOLDER.controller === controller) return;
    HOLDER.controller?.destroy();
    HOLDER.controller = controller;
}
