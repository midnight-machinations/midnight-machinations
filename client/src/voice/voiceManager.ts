import { FEATURES } from "../features";
import translate from "../game/lang";
import { ANCHOR_CONTROLLER } from "../menu/Anchor";
import { keybindReleased, matchesKeybind, matchesKeybindModifiers } from "./keybind";
import { MicrophoneProcessing } from "./microphoneProcessing";
import type { GroupPreference, VoiceSession, VoiceSettings } from "./types";

/** Sent by the voice service when it pulls speakers into this session. */
type OfferMessage = { type: "offer"; sdp: string; tracks: { mid: string; peer: number }[] };
type Playback = {
    receiver: RTCRtpReceiver;
    element: HTMLAudioElement;
    source: MediaStreamAudioSourceNode;
    gain: GainNode;
    peer: number;
    packets: number;
    energy: number;
    duration: number;
};
type PersonalPreference = { muted: boolean; volume: number };
const DEFAULT_SETTINGS: VoiceSettings = {
    mode: "openMic",
    key: "KeyV",
    deviceId: "",
    volume: 1,
    noiseSuppression: true,
    thresholdDb: -80,
};
function readStorage<T>(key: string, fallback: T): T {
    try {
        return JSON.parse(localStorage.getItem(key) ?? "null") ?? fallback;
    } catch {
        return fallback;
    }
}
function saveStorage(key: string, value: unknown) {
    try {
        localStorage.setItem(key, JSON.stringify(value));
    } catch {
        /* Voice can work without storage. */
    }
}

export class VoiceManager {
    session: VoiceSession | null = null;
    status: "disconnected" | "connecting" | "connected" | "error" = "disconnected";
    error = "";
    roomEnabled = true;
    selfMuted = true;
    microphoneLevelDb = -80;
    microphoneGateOpen = false;
    microphoneEnabled = false;
    requestingMicrophone = false;
    talking = false;
    settings: VoiceSettings;
    groups: Record<string, GroupPreference> = {};
    people: Record<number, PersonalPreference> = {};
    private listeners = new Set<() => void>();
    private socket?: WebSocket;
    /** One connection to the SFU: our microphone out, every allowed speaker in. */
    private connection?: RTCPeerConnection;
    private sender?: RTCRtpSender;
    /** Track mid to speaker, announced before each renegotiation. */
    private trackPeers = new Map<string, number>();
    private microphone?: MediaStream;
    private audio?: AudioContext;
    private playbacks = new Map<string, Playback>();
    private pending = new Map<
        number,
        { resolve: (value: unknown) => void; reject: (error: Error) => void; timeout: ReturnType<typeof setTimeout> }
    >();
    private requestId = 0;
    private generation = 0;
    private wantConnected = false;
    private retry?: ReturnType<typeof setTimeout>;
    private negotiation = Promise.resolve();
    private heldKey = false;
    private heldButton = false;
    private microphoneGeneration = 0;
    private revision = 0;
    private preferencesPending = 0;
    private preferenceRevision = 0;
    private speaking = new Set<number>();
    private speakingUntil = new Map<number, number>();
    private activityBusy = false;
    private processing?: MicrophoneProcessing;
    // Automatic retries repeat the same failure; only the first one pops up an error card.
    private lastWarning = "";
    private sentPackets?: { sender: RTCRtpSender; count: number };
    private stuckAudioChecks = 0;
    private lastAudioTime?: { audio: AudioContext; time: number };

    constructor() {
        const saved = readStorage<Partial<VoiceSettings>>("voiceSettings", {});
        this.settings = {
            mode:
                saved.mode === "pushToTalk" || saved.mode === "toggleMute" || saved.mode === "openMic"
                    ? saved.mode
                    : DEFAULT_SETTINGS.mode,
            key: typeof saved.key === "string" ? saved.key : DEFAULT_SETTINGS.key,
            deviceId: typeof saved.deviceId === "string" ? saved.deviceId : "",
            volume:
                typeof saved.volume === "number" && Number.isFinite(saved.volume)
                    ? Math.max(0, Math.min(2, saved.volume))
                    : 1,
            noiseSuppression:
                typeof saved.noiseSuppression === "boolean"
                    ? saved.noiseSuppression
                    : DEFAULT_SETTINGS.noiseSuppression,
            thresholdDb:
                typeof saved.thresholdDb === "number" && Number.isFinite(saved.thresholdDb)
                    ? Math.max(-80, Math.min(0, saved.thresholdDb))
                    : DEFAULT_SETTINGS.thresholdDb,
        };
        window.addEventListener("keydown", (event) => {
            if (this.heldKey && !matchesKeybindModifiers(this.settings.key, event)) {
                this.heldKey = false;
                this.updateMicrophone();
            }
            if (
                event.defaultPrevented ||
                event.isComposing ||
                !matchesKeybind(this.settings.key, event) ||
                event.repeat ||
                this.settings.mode === "openMic"
            )
                return;
            const target = event.target instanceof Element ? event.target : null;
            // Shared Select searches from a button, including in ability menus.
            if (
                target?.closest(
                    "input, textarea, select, [contenteditable]:not([contenteditable=false]), [role=combobox], [role=textbox], [role=searchbox], .custom-select, .custom-select-options-popover",
                )
            )
                return;
            event.preventDefault();
            if (this.settings.mode === "toggleMute") {
                this.toggleSelfMute();
                return;
            }
            this.heldKey = true;
            this.updateMicrophone();
        });
        window.addEventListener("keyup", (event) => {
            if (this.heldKey && keybindReleased(this.settings.key, event)) {
                this.heldKey = false;
                this.updateMicrophone();
            }
        });
        window.addEventListener("blur", () => this.releaseTalk());
        document.addEventListener("visibilitychange", () => {
            if (document.hidden) this.releaseTalk();
        });
        window.addEventListener("pagehide", () => this.leave());
        window.setInterval(() => void this.updateSpeaking(), 100);
        window.setInterval(() => void this.checkMicrophoneFlow(), 5000);
        window.setInterval(() => this.checkAudioRunning(), 1000);
    }
    subscribe = (listener: () => void) => {
        this.listeners.add(listener);
        return () => {
            this.listeners.delete(listener);
        };
    };
    getRevision = () => this.revision;
    private changed() {
        this.revision++;
        for (const listener of this.listeners) listener();
    }
    private releaseTalk() {
        this.heldKey = false;
        this.heldButton = false;
        this.updateMicrophone();
    }
    holdToTalk(held: boolean) {
        this.heldButton = held;
        this.updateMicrophone();
    }

    setRoomEnabled(enabled: boolean) {
        this.roomEnabled = enabled;
        if (!enabled) this.clearSession();
        this.changed();
    }
    toggleSelfMute() {
        this.selfMuted = !this.selfMuted;
        this.updateMicrophone();
        this.changed();
    }
    async togglePlayerMute(id: number) {
        if (id === this.session?.selfId) {
            if (!this.microphoneEnabled) {
                if (this.status !== "connected") {
                    await this.join();
                    if (this.microphoneEnabled) {
                        this.selfMuted = false;
                        this.updateMicrophone();
                        this.changed();
                    }
                } else await this.enableMicrophone({ unmute: true });
            } else this.toggleSelfMute();
        } else this.setPerson(id, { muted: !this.person(id).muted });
    }
    playerMuted(id: number) {
        return id === this.session?.selfId ? this.selfMuted || !this.microphoneEnabled : this.person(id).muted;
    }
    setSession(session: VoiceSession | null) {
        const previous = this.session;
        const newSession =
            session?.token !== previous?.token || session?.scope !== previous?.scope || session?.url !== previous?.url;
        this.session = session;
        if (!session) {
            this.leave();
            return;
        }
        if (session.scope !== previous?.scope) {
            this.groups = readStorage(`voiceGroups:${session.scope}`, {});
            this.people = readStorage(`voicePeople:${session.scope}`, {});
        }
        if (newSession) {
            this.disconnect();
            if (this.wantConnected) void this.connect();
        }
        this.updateMicrophone();
        this.changed();
    }
    async join() {
        if (
            !FEATURES.voice ||
            !this.roomEnabled ||
            !this.session ||
            this.status === "connecting" ||
            this.status === "connected"
        )
            return;
        this.wantConnected = true;
        this.selfMuted = true;
        this.lastWarning = "";
        this.stuckAudioChecks = 0;
        this.status = "connecting";
        this.changed();
        // Start during the click so playback works under browser autoplay restrictions.
        // Don't wait for it: Firefox never settles resume() while a Bluetooth headset
        // is switching to its microphone profile.
        this.audio ??= new AudioContext({ sampleRate: 48000 });
        const generation = this.generation;
        this.audio.resume().catch((error) => this.playbackBlocked(error, generation));
        // Ask for the microphone before connecting: Firefox only offers WebRTC the
        // addresses it needs once the page has microphone access. Declining joins
        // listen-only; the player's own mic button asks again.
        if (!this.microphoneEnabled) await this.enableMicrophone({ quietDecline: true });
        if (this.wantConnected) await this.connect();
    }
    leave() {
        this.wantConnected = false;
        this.lastWarning = "";
        this.disconnect();
        this.disableMicrophone();
        this.status = "disconnected";
        this.error = "";
        if (!this.session) {
            void this.audio?.close();
            this.audio = undefined;
        }
        this.changed();
    }
    clearSession() {
        this.setSession(null);
    }
    private disconnect() {
        this.generation++;
        clearTimeout(this.retry);
        this.retry = undefined;
        this.releaseTalk();
        if (this.socket) {
            this.socket.onclose = null;
            this.socket.close();
            this.socket = undefined;
        }
        for (const request of this.pending.values()) {
            clearTimeout(request.timeout);
            request.reject(new Error("Voice disconnected"));
        }
        this.pending.clear();
        this.preferencesPending = 0;
        for (const id of [...this.playbacks.keys()]) this.closePlayback(id);
        this.connection?.close();
        this.connection = undefined;
        this.sender = undefined;
        this.trackPeers.clear();
        this.negotiation = Promise.resolve();
        this.speakingUntil.clear();
        this.speaking.clear();
        this.status = "disconnected";
        this.updateMicrophone();
    }
    private preferences() {
        return {
            deafened: Object.entries(this.groups)
                .filter(([, value]) => value.deafened)
                .map(([group]) => group),
        };
    }
    private async connect() {
        const session = this.session;
        if (!session || !this.wantConnected) return;
        this.disconnect();
        const generation = this.generation;
        const current = () => generation === this.generation && this.wantConnected;
        this.status = "connecting";
        this.preferencesPending = 1;
        this.error = "";
        this.changed();
        try {
            const socket = new WebSocket(session.url);
            this.socket = socket;
            await new Promise<void>((resolve, reject) => {
                const timeout = setTimeout(() => reject(new Error("Voice connection timeout")), 10000);
                socket.onopen = () => {
                    clearTimeout(timeout);
                    resolve();
                };
                socket.onerror = () => {
                    clearTimeout(timeout);
                    reject(new Error("Voice connection failed"));
                };
                socket.onclose = () => {
                    clearTimeout(timeout);
                    reject(new Error("Voice connection closed"));
                };
            });
            if (!current()) return;
            socket.onmessage = (event) => {
                if (!current()) return;
                let message;
                try {
                    message = JSON.parse(event.data);
                } catch {
                    return;
                }
                if (typeof message.id === "number") {
                    const request = this.pending.get(message.id);
                    if (request) {
                        clearTimeout(request.timeout);
                        this.pending.delete(message.id);
                        if (message.error) request.reject(new Error(message.error));
                        else request.resolve(message.data);
                    }
                } else if (message.type === "offer") {
                    this.negotiation = this.negotiation
                        .then(async () => {
                            if (current()) await this.answer(message);
                        })
                        .catch((error) => {
                            if (current()) this.connectionFailed(error);
                        });
                }
            };
            socket.onclose = (event) => {
                if (current())
                    this.connectionFailed(new Error(`Voice signaling closed (${event.code}): ${event.reason}`));
            };
            const joined = await this.request<{ iceServers: RTCIceServer[] }>("join", {
                token: session.token,
                ...this.preferences(),
            });
            if (!current()) return;
            const connection = new RTCPeerConnection({ iceServers: joined.iceServers, bundlePolicy: "max-bundle" });
            this.connection = connection;
            connection.onconnectionstatechange = () => {
                if (connection.connectionState === "failed" && connection === this.connection) {
                    this.connectionFailed(new Error("Voice connection failed"));
                }
            };
            connection.ontrack = (event) => this.play(event.transceiver);
            // Publish one stable track even without a microphone, so other clients get
            // no signal when it is enabled later. A missing track just sends nothing.
            const transceiver = connection.addTransceiver(this.processing?.track ?? "audio", { direction: "sendonly" });
            this.sender = transceiver.sender;
            await connection.setLocalDescription(await connection.createOffer());
            const published = await this.request<{ sdp: string }>("publish", {
                sdp: connection.localDescription?.sdp,
                mid: transceiver.mid,
            });
            if (!current()) return;
            await connection.setRemoteDescription({ type: "answer", sdp: published.sdp });
            if (!current()) return;
            // Preferences can change while transport setup is awaiting the server.
            // Keep playback and the microphone gated until the latest choices apply.
            let appliedRevision: number;
            do {
                appliedRevision = this.preferenceRevision;
                await this.request("preferences", this.preferences());
                if (!current()) return;
            } while (appliedRevision !== this.preferenceRevision);
            this.preferencesPending = 0;
            this.updateVolumes();
            this.status = "connected";
            this.updateMicrophone();
            this.changed();
        } catch (error) {
            if (current()) this.connectionFailed(error);
        }
    }
    private connectionFailed(error?: unknown) {
        console.warn("Voice connection failed", error);
        this.disconnect();
        this.status = "error";
        this.setError("voice.error.connection", error);
        this.changed();
        if (this.wantConnected && this.session) this.retry = setTimeout(() => void this.connect(), 2500);
    }
    /** Shows a warning in the voice column, and in the error card the first time it occurs. */
    private setError(key: string, cause?: unknown) {
        this.error = key;
        if (key === this.lastWarning) return;
        this.lastWarning = key;
        const detail = cause instanceof Error ? cause.message : "";
        ANCHOR_CONTROLLER?.pushErrorCard({
            title: translate("voice.error.title"),
            body: detail ? `${translate(key)} (${detail})` : translate(key),
        });
    }
    private request<T = Record<string, never>>(action: string, data: object = {}): Promise<T> {
        const socket = this.socket;
        if (!socket || socket.readyState !== WebSocket.OPEN) return Promise.reject(new Error("Voice disconnected"));
        const id = ++this.requestId;
        return new Promise<T>((resolve, reject) => {
            const timeout = setTimeout(() => {
                this.pending.delete(id);
                reject(new Error("Voice request timeout"));
            }, 10000);
            this.pending.set(id, { resolve: (value) => resolve(value as T), reject, timeout });
            socket.send(JSON.stringify({ id, action, ...data }));
        });
    }
    /** Applies the service's offer for newly pulled speakers and answers it. */
    private async answer(message: OfferMessage) {
        const connection = this.connection;
        if (!connection) return;
        for (const track of message.tracks) this.trackPeers.set(track.mid, track.peer);
        await connection.setRemoteDescription({ type: "offer", sdp: message.sdp });
        await connection.setLocalDescription(await connection.createAnswer());
        this.socket?.send(JSON.stringify({ type: "answer", sdp: connection.localDescription?.sdp }));
    }
    private play(transceiver: RTCRtpTransceiver) {
        const audio = this.audio;
        const mid = transceiver.mid;
        const peer = mid === null ? undefined : this.trackPeers.get(mid);
        if (!audio || mid === null || peer === undefined) return;
        // A speaker pulled again replaces their previous, now silent, track.
        for (const [id, playback] of this.playbacks) if (playback.peer === peer) this.closePlayback(id);
        const generation = this.generation;
        // A media element drives remote decoding in browsers that leave a
        // WebRTC stream silent when connected only to a Web Audio source node.
        const element = document.createElement("audio");
        element.autoplay = true;
        element.hidden = true;
        element.srcObject = new MediaStream([transceiver.receiver.track]);
        element.volume = 0;
        document.body.append(element);
        // The element drives decoding while Web Audio plays the stream with
        // gain, allowing master volume above the element's 100% limit.
        const source = audio.createMediaStreamSource(element.srcObject);
        const gain = audio.createGain();
        source.connect(gain).connect(audio.destination);
        this.playbacks.set(mid, {
            receiver: transceiver.receiver,
            element,
            source,
            gain,
            peer,
            packets: 0,
            energy: 0,
            duration: 0,
        });
        this.updateVolumes();
        // Autoplay failures are playback problems, not broken WebRTC sessions.
        void element.play().catch((error) => this.playbackBlocked(error, generation));
    }
    private playbackBlocked(error: unknown, generation: number) {
        if (generation !== this.generation) return;
        console.warn("Voice playback blocked", error);
        this.setError("voice.error.playback", error);
        this.changed();
    }
    async resumePlayback() {
        const generation = this.generation;
        try {
            await this.audio?.resume();
            await Promise.all([...this.playbacks.values()].map((playback) => playback.element.play()));
            this.error = "";
            this.lastWarning = "";
            this.changed();
        } catch (error) {
            this.playbackBlocked(error, generation);
        }
    }
    private closePlayback(id: string) {
        const playback = this.playbacks.get(id);
        if (playback) {
            playback.element.pause();
            playback.element.srcObject = null;
            playback.element.remove();
            playback.source.disconnect();
            playback.gain.disconnect();
            this.playbacks.delete(id);
            this.speakingUntil.delete(playback.peer);
        }
    }
    /** `unmute` turns off self-mute once access is granted. `quietDecline` shows no warning if the player declines access. */
    async enableMicrophone(options: { unmute?: boolean; quietDecline?: boolean } = {}) {
        if (!this.session || this.requestingMicrophone) return;
        this.requestingMicrophone = true;
        const generation = ++this.microphoneGeneration;
        this.error = "";
        this.changed();
        let stream: MediaStream | undefined;
        let processing: MicrophoneProcessing | undefined;
        let processingStarted = false;
        try {
            stream = await navigator.mediaDevices.getUserMedia({
                audio: {
                    deviceId: this.settings.deviceId ? { exact: this.settings.deviceId } : undefined,
                    echoCancellation: true,
                    noiseSuppression: false,
                    autoGainControl: true,
                    channelCount: 1,
                },
                video: false,
            });
            if (generation !== this.microphoneGeneration || !this.session) {
                stream.getTracks().forEach((track) => track.stop());
                return;
            }
            this.audio ??= new AudioContext({ sampleRate: 48000 });
            processingStarted = true;
            processing = await MicrophoneProcessing.create(
                this.audio,
                stream,
                this.settings,
                (db, gateOpen) => {
                    if (generation !== this.microphoneGeneration) return;
                    this.microphoneLevelDb = db;
                    this.microphoneGateOpen = gateOpen;
                    this.changed();
                },
                () => {
                    if (generation !== this.microphoneGeneration) return;
                    this.disableMicrophone();
                    this.setError("voice.error.processing");
                },
            );
            if (generation !== this.microphoneGeneration || !this.session) {
                processing.close();
                stream.getTracks().forEach((track) => track.stop());
                return;
            }
            this.processing?.close();
            this.microphone?.getTracks().forEach((track) => track.stop());
            this.microphone = stream;
            this.processing = processing;
            // Settings may change while the worklet is loading.
            processing.configure(this.settings);
            this.microphoneEnabled = true;
            if (options.unmute) this.selfMuted = false;
            const activeStream = stream;
            for (const track of stream.getAudioTracks()) {
                track.onended = () => {
                    if (this.microphone === activeStream) this.disableMicrophone();
                };
            }
            // A failed swap is a connection problem, not a microphone one. Keep the
            // microphone so the reconnect in connect() publishes it again.
            const connection = this.generation;
            try {
                await this.sender?.replaceTrack(processing.track);
            } catch (error) {
                if (connection === this.generation) this.connectionFailed(error);
            }
            this.updateMicrophone();
        } catch (error) {
            processing?.close();
            stream?.getTracks().forEach((track) => track.stop());
            if (generation === this.microphoneGeneration) {
                this.disableMicrophone();
                const declined = error instanceof DOMException && error.name === "NotAllowedError";
                if (!(declined && options.quietDecline))
                    this.setError(processingStarted ? "voice.error.processing" : "voice.error.microphone", error);
            }
        } finally {
            if (generation === this.microphoneGeneration) this.requestingMicrophone = false;
            this.changed();
        }
    }
    disableMicrophone() {
        this.microphoneGeneration++;
        this.requestingMicrophone = false;
        this.microphoneEnabled = false;
        if (this.processing) this.processing.track.enabled = false;
        // Keep the sender stable so other clients receive no mute notification.
        // Release the hardware; a fresh track can replace this stopped track later.
        this.microphone?.getTracks().forEach((track) => track.stop());
        this.microphone = undefined;
        this.processing?.close();
        this.processing = undefined;
        this.microphoneLevelDb = -80;
        this.microphoneGateOpen = false;
        if (this.session) this.speakingUntil.delete(this.session.selfId);
        this.releaseTalk();
        this.changed();
    }
    private updateMicrophone() {
        const canSpeak = this.session && !this.session.forcedSilenced && this.session.sendGroups.length > 0;
        const talking = Boolean(
            this.roomEnabled &&
                !this.selfMuted &&
                this.status === "connected" &&
                this.preferencesPending === 0 &&
                this.microphoneEnabled &&
                canSpeak &&
                (this.settings.mode !== "pushToTalk" || this.heldKey || this.heldButton),
        );
        if (this.processing) this.processing.track.enabled = talking;
        if (this.talking !== talking) {
            this.talking = talking;
            this.changed();
        }
    }
    group(group: string): GroupPreference {
        return { deafened: this.groups[group]?.deafened === true };
    }
    setGroup(group: string, change: Partial<GroupPreference>) {
        this.preferenceRevision++;
        this.groups = { ...this.groups, [group]: { ...this.group(group), ...change } };
        if (this.session) saveStorage(`voiceGroups:${this.session.scope}`, this.groups);
        if (this.status === "connected") {
            const generation = this.generation;
            this.preferencesPending++;
            this.updateMicrophone();
            this.updateVolumes();
            void this.request("preferences", this.preferences())
                .then(() => {
                    if (generation !== this.generation) return;
                    this.preferencesPending--;
                    this.updateMicrophone();
                    this.updateVolumes();
                    this.changed();
                })
                .catch(() => {
                    if (generation === this.generation) this.connectionFailed();
                });
        }
        this.updateMicrophone();
        this.changed();
    }
    person(id: number): PersonalPreference {
        return this.people[id] ?? { muted: false, volume: 1 };
    }
    setPerson(id: number, change: Partial<PersonalPreference>) {
        this.people = { ...this.people, [id]: { ...this.person(id), ...change } };
        if (this.session) saveStorage(`voicePeople:${this.session.scope}`, this.people);
        this.updateVolumes();
        this.changed();
    }
    setSettings(change: Partial<VoiceSettings>) {
        const deviceChanged = change.deviceId !== undefined && change.deviceId !== this.settings.deviceId;
        if (change.mode === "toggleMute" && this.settings.mode !== "toggleMute") this.selfMuted = true;
        this.settings = { ...this.settings, ...change };
        this.processing?.configure(this.settings);
        saveStorage("voiceSettings", this.settings);
        this.releaseTalk();
        this.updateVolumes();
        this.changed();
        if (deviceChanged && this.microphoneEnabled) void this.enableMicrophone();
    }
    private updateVolumes() {
        for (const playback of this.playbacks.values()) {
            const preference = this.person(playback.peer);
            playback.element.muted = preference.muted || this.preferencesPending > 0;
            playback.gain.gain.value = playback.element.muted
                ? 0
                : Math.max(0, Math.min(1, preference.volume)) * this.settings.volume;
        }
    }
    isSpeaking(id: number | undefined) {
        return id !== undefined && this.speaking.has(id);
    }
    /**
     * A working microphone always sends packets, even while push-to-talk or mute
     * disables its track, because the browser then sends silence. No new packets
     * means the audio device has stalled, such as a Bluetooth headset that failed
     * to switch to its microphone profile.
     */
    private async checkMicrophoneFlow() {
        const sender = this.sender;
        if (!sender?.track || this.status !== "connected" || !this.microphoneEnabled) {
            this.sentPackets = undefined;
            return;
        }
        // A stuck audio context stalls the microphone too; keep its more specific warning.
        if (this.error === "voice.error.stuckAudio") return;
        let count = 0;
        try {
            for (const stat of (await sender.getStats()).values()) {
                if (stat.type === "outbound-rtp") count += stat.packetsSent ?? 0;
            }
        } catch {
            return;
        }
        if (sender !== this.sender) return;
        const stalled = this.sentPackets?.sender === sender && this.sentPackets.count === count;
        this.sentPackets = { sender, count };
        if (stalled) {
            this.setError("voice.error.silentMicrophone");
            this.changed();
        } else if (this.error === "voice.error.silentMicrophone") {
            this.error = "";
            this.changed();
        }
    }
    /**
     * Audio streams that a browser opens while a Bluetooth headset switches to its
     * microphone profile can stay stuck for good. Seen with Firefox on PipeWire,
     * where opening the microphone triggers the switch: the AudioContext's clock
     * stops even though its state may still say running, so remote voices are
     * silent and the microphone sends nothing. The page can't recover the stream,
     * and reopening the microphone switches the headset again. Reloading the page
     * once the headset has switched works, so tell the player to do that.
     */
    private checkAudioRunning() {
        const audio = this.audio;
        if (!audio || !this.wantConnected || audio.state === "closed") {
            this.stuckAudioChecks = 0;
            return;
        }
        const time = audio.currentTime;
        const advanced = this.lastAudioTime?.audio !== audio || time > this.lastAudioTime.time;
        this.lastAudioTime = { audio, time };
        if (audio.state === "running" && advanced) {
            this.stuckAudioChecks = 0;
            if (this.error === "voice.error.stuckAudio") {
                this.error = "";
                this.changed();
            }
            return;
        }
        if (++this.stuckAudioChecks === 3) {
            console.warn(`Voice audio context is stuck (state ${audio.state}, time ${time})`);
            this.setError("voice.error.stuckAudio");
            this.changed();
        }
    }
    private async updateSpeaking() {
        if (this.activityBusy || this.status !== "connected" || !this.session) return;
        this.activityBusy = true;
        const generation = this.generation;
        const now = performance.now();
        try {
            if (this.talking && this.processing) {
                const data = new Float32Array(this.processing.analyser.fftSize);
                this.processing.analyser.getFloatTimeDomainData(data);
                const rms = Math.sqrt(data.reduce((sum, sample) => sum + sample * sample, 0) / data.length);
                if (rms > 0.015) this.speakingUntil.set(this.session.selfId, now + 250);
            } else this.speakingUntil.delete(this.session.selfId);
            await Promise.all(
                [...this.playbacks].map(async ([id, playback]) => {
                    if (playback.element.muted || playback.gain.gain.value === 0 || playback.element.paused) {
                        this.speakingUntil.delete(playback.peer);
                        return;
                    }
                    try {
                        const stats = await playback.receiver.getStats();
                        if (generation !== this.generation || !this.playbacks.has(id)) return;
                        for (const stat of stats.values()) {
                            if (stat.type !== "inbound-rtp") continue;
                            const duration = stat.totalSamplesDuration ?? 0;
                            const energy = stat.totalAudioEnergy ?? 0;
                            const rms =
                                duration > playback.duration
                                    ? Math.sqrt(Math.max(0, energy - playback.energy) / (duration - playback.duration))
                                    : 0;
                            if (
                                stat.packetsReceived > playback.packets &&
                                Math.max(stat.audioLevel ?? 0, rms) > 0.015
                            ) {
                                this.speakingUntil.set(playback.peer, now + 250);
                            }
                            playback.packets = stat.packetsReceived;
                            playback.energy = energy;
                            playback.duration = duration;
                        }
                    } catch {
                        /* A track may close while statistics are being read. */
                    }
                }),
            );
            if (generation !== this.generation) return;
            const next = new Set([...this.speakingUntil].filter(([, until]) => until > now).map(([id]) => id));
            if (next.size !== this.speaking.size || [...next].some((id) => !this.speaking.has(id))) {
                this.speaking = next;
                this.changed();
            }
        } finally {
            this.activityBusy = false;
        }
    }
}

export const VOICE = new VoiceManager();
