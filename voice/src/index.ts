/**
 * Voice service: a Cloudflare Worker whose single Durable Object holds the
 * routing policy from the game server and steers the Cloudflare Realtime SFU.
 *
 * - `/control` (WebSocket, bearer secret): the game server publishes
 *   `{ version, rooms: [{ scope, members: [{ id, token, send, receive }] }] }`
 *   and receives `{ presence: { [scope]: id[] } }`.
 * - `/` (WebSocket): browsers join with a token from their game socket, push one
 *   audio track, and receive only the tracks this service pulls for them.
 * - `/` (HTTP GET): health check.
 */
import { DurableObject } from "cloudflare:workers";
import { canForwardVoice, type Policy } from "./routing.ts";
import { type SessionDescription, Sfu } from "./sfu.ts";

export interface Env {
    VOICE: DurableObjectNamespace<Voice>;
    VOICE_CONTROL_SECRET: string;
    REALTIME_APP_ID: string;
    REALTIME_APP_TOKEN: string;
    /** Comma-separated exact frontend origins. Unset allows any origin. */
    VOICE_ALLOWED_ORIGINS?: string;
    /** JSON RTCIceServer[] given to joined browsers, e.g. TURN credentials. */
    VOICE_ICE_SERVERS?: string;
}

type Peer = {
    policy: Policy;
    socket: WebSocket;
    deafened: Set<string>;
    session?: string;
    /** Speaker id to the mid of that speaker's track in this peer's session. */
    pulled: Map<number, string>;
    closed: boolean;
    queue: Promise<void>;
    reconcileQueued: boolean;
    answer?: (description: SessionDescription) => void;
};

const DEFAULT_ICE_SERVERS = [{ urls: "stun:stun.cloudflare.com:3478" }];

export default {
    fetch(request: Request, env: Env): Promise<Response> {
        // One object is the single authority for every room's routing.
        return env.VOICE.get(env.VOICE.idFromName("voice")).fetch(request);
    },
} satisfies ExportedHandler<Env>;

export class Voice extends DurableObject<Env> {
    private control?: WebSocket;
    private version = 0;
    /** Token to policy, replaced wholesale by each control snapshot. */
    private policies = new Map<string, Policy>();
    /** Scope to room client id to peer. */
    private rooms = new Map<string, Map<number, Peer>>();
    private sfu = new Sfu(this.env.REALTIME_APP_ID, this.env.REALTIME_APP_TOKEN);

    async fetch(request: Request): Promise<Response> {
        const upgrade = request.headers.get("upgrade")?.toLowerCase() === "websocket";
        if (!upgrade) return new Response(this.control ? "voice ready\n" : "waiting for game server\n");
        if (new URL(request.url).pathname === "/control") return this.acceptControl(request);
        const origins = this.env.VOICE_ALLOWED_ORIGINS?.split(",");
        if (origins && !origins.includes(request.headers.get("origin") ?? ""))
            return new Response("Forbidden origin", { status: 403 });
        return this.acceptPeer();
    }

    // ---- Game server control link ----

    private acceptControl(request: Request): Response {
        const supplied = new TextEncoder().encode(request.headers.get("authorization") ?? "");
        const expected = new TextEncoder().encode(`Bearer ${this.env.VOICE_CONTROL_SECRET}`);
        if (
            !this.env.VOICE_CONTROL_SECRET ||
            supplied.byteLength !== expected.byteLength ||
            !crypto.subtle.timingSafeEqual(supplied, expected)
        ) {
            return new Response("Unauthorized", { status: 401 });
        }
        const [client, socket] = Object.values(new WebSocketPair());
        socket.accept();
        // A new authority replaces the old one. Never merge policies from two.
        this.control?.close(1012, "Replaced");
        this.failClosed();
        this.control = socket;
        this.version = 0;
        socket.addEventListener("message", (event) => {
            if (socket !== this.control) return;
            try {
                this.applyPolicy(JSON.parse(String(event.data)));
            } catch {
                socket.close(1008, "Invalid policy");
            }
        });
        const lost = () => {
            if (socket === this.control) {
                this.control = undefined;
                this.failClosed();
            }
        };
        socket.addEventListener("close", lost);
        socket.addEventListener("error", lost);
        this.reportPresence();
        return new Response(null, { status: 101, webSocket: client });
    }

    private applyPolicy(message: { version?: unknown; rooms?: unknown }) {
        if (
            !Number.isSafeInteger(message.version) ||
            (message.version as number) <= this.version ||
            !Array.isArray(message.rooms)
        )
            return;
        this.version = message.version as number;
        const next = new Map<string, Policy>();
        for (const room of message.rooms as { scope: string; members: Omit<Policy, "scope">[] }[]) {
            for (const member of room.members) next.set(member.token, { ...member, scope: room.scope });
        }
        this.policies = next;
        for (const [scope, peers] of this.rooms) {
            for (const peer of [...peers.values()]) {
                const policy = next.get(peer.policy.token);
                if (!policy || policy.scope !== scope || policy.id !== peer.policy.id) this.closePeer(peer);
                else peer.policy = policy;
            }
            this.reconcileRoom(scope);
        }
    }

    /** Losing the authority revokes everything; browsers reconnect with fresh tokens. */
    private failClosed() {
        this.policies = new Map();
        for (const peers of this.rooms.values()) for (const peer of [...peers.values()]) this.closePeer(peer);
        this.rooms.clear();
    }

    private reportPresence() {
        const presence: Record<string, number[]> = {};
        for (const [scope, peers] of this.rooms) presence[scope] = [...peers.keys()];
        send(this.control, { presence });
    }

    // ---- Browser signaling ----

    private acceptPeer(): Response {
        const [client, socket] = Object.values(new WebSocketPair());
        socket.accept();
        let peer: Peer | undefined;
        let window = Date.now();
        let count = 0;
        const timeout = setTimeout(() => {
            if (!peer) socket.close(1008, "Join timeout");
        }, 10_000);
        socket.addEventListener("message", async (event) => {
            if (Date.now() - window >= 1000) {
                window = Date.now();
                count = 0;
            }
            if (++count > 50) {
                socket.close(1008, "Rate limited");
                return;
            }
            let message: Record<string, unknown>;
            try {
                message = JSON.parse(String(event.data));
            } catch {
                socket.close(1008, "Invalid message");
                return;
            }
            if (message.type === "answer") {
                peer?.answer?.({ type: "answer", sdp: String(message.sdp) });
                return;
            }
            const id = message.id;
            if (!Number.isSafeInteger(id)) {
                socket.close(1008, "Invalid message");
                return;
            }
            try {
                if (message.type === "join") {
                    if (peer) throw new Error("Already joined");
                    peer = this.join(socket, message);
                    clearTimeout(timeout);
                    send(socket, { id, data: { iceServers: this.iceServers() } });
                } else if (!peer || peer.closed) {
                    throw new Error("Not joined");
                } else if (message.type === "publish") {
                    send(socket, { id, data: await this.publish(peer, String(message.sdp), String(message.mid)) });
                } else if (message.type === "preferences") {
                    peer.deafened = groupSet(message.deafened);
                    await this.reconcile(peer);
                    send(socket, { id, data: {} });
                } else throw new Error("Unknown request");
            } catch (error) {
                console.warn(`Voice ${String(message.type)} failed:`, error instanceof Error ? error.message : error);
                send(socket, { id, error: "Voice request failed. Reconnect to retry." });
            }
        });
        const closed = () => {
            clearTimeout(timeout);
            if (peer) this.closePeer(peer);
        };
        socket.addEventListener("close", closed);
        socket.addEventListener("error", closed);
        return new Response(null, { status: 101, webSocket: client });
    }

    private join(socket: WebSocket, message: Record<string, unknown>): Peer {
        if (!this.control) throw new Error("Voice unavailable");
        const policy = typeof message.token === "string" ? this.policies.get(message.token) : undefined;
        if (!policy) throw new Error("Invalid session");
        const peers = this.rooms.get(policy.scope) ?? new Map<number, Peer>();
        this.rooms.set(policy.scope, peers);
        const previous = peers.get(policy.id);
        if (previous) this.closePeer(previous);
        const peer: Peer = {
            policy,
            socket,
            deafened: groupSet(message.deafened ?? []),
            pulled: new Map(),
            closed: false,
            queue: Promise.resolve(),
            reconcileQueued: false,
        };
        peers.set(policy.id, peer);
        this.reportPresence();
        return peer;
    }

    private async publish(peer: Peer, sdp: string, mid: string): Promise<{ sdp: string }> {
        if (peer.session) throw new Error("Already published");
        // Keep one stable track even while game permissions forbid speaking.
        // Only the tracks this service pulls decide where its audio goes.
        const session = await this.sfu.newSession();
        const result = await this.sfu.publish(session, { type: "offer", sdp }, mid);
        if (!result.sessionDescription || result.tracks?.[0]?.errorCode) throw new Error("Publish rejected");
        peer.session = session;
        // Pull others for this peer and this peer for others once the answer is applied.
        queueMicrotask(() => this.reconcileRoom(peer.policy.scope));
        return { sdp: result.sessionDescription.sdp };
    }

    private closePeer(peer: Peer) {
        if (peer.closed) return;
        peer.closed = true;
        peer.answer = undefined;
        const peers = this.rooms.get(peer.policy.scope);
        if (peers?.get(peer.policy.id) === peer) peers.delete(peer.policy.id);
        if (peers?.size === 0) this.rooms.delete(peer.policy.scope);
        try {
            peer.socket.close(1000, "Voice session ended");
        } catch {
            /* Already closed. */
        }
        // Listeners stop pulling the departed speaker.
        this.reconcileRoom(peer.policy.scope);
        this.reportPresence();
    }

    // ---- Routing ----

    private reconcileRoom(scope: string) {
        for (const peer of this.rooms.get(scope)?.values() ?? []) void this.reconcile(peer);
    }

    /**
     * Brings one listener's pulled tracks in line with the current policy.
     * Runs serially per listener; revocations happen before any new grants.
     * Queued runs collapse into one, which reads the policy when it starts.
     */
    private reconcile(peer: Peer): Promise<void> {
        if (peer.reconcileQueued) return peer.queue;
        peer.reconcileQueued = true;
        peer.queue = peer.queue
            .then(async () => {
                peer.reconcileQueued = false;
                if (peer.closed || !peer.session) return;
                const speakers = this.rooms.get(peer.policy.scope) ?? new Map<number, Peer>();
                const allowed = (speaker: Peer | undefined) =>
                    speaker !== undefined &&
                    !speaker.closed &&
                    speaker.session !== undefined &&
                    this.control !== undefined &&
                    canForwardVoice(speaker.policy, peer.policy, peer.deafened);
                const revoke = [...peer.pulled].filter(([id]) => !allowed(speakers.get(id)));
                if (revoke.length) {
                    for (const [id] of revoke) peer.pulled.delete(id);
                    await this.sfu.close(
                        peer.session,
                        revoke.map(([, mid]) => mid),
                    );
                }
                const grant = [...speakers.values()].filter(
                    (speaker) => speaker !== peer && !peer.pulled.has(speaker.policy.id) && allowed(speaker),
                );
                if (!grant.length || peer.closed) return;
                const result = await this.sfu.pull(
                    peer.session,
                    grant.map((speaker) => speaker.session as string),
                );
                const tracks: { mid: string; peer: number }[] = [];
                for (const track of result.tracks ?? []) {
                    const speaker = grant.find((speaker) => speaker.session === track.sessionId);
                    if (!speaker || !track.mid || track.errorCode) continue;
                    peer.pulled.set(speaker.policy.id, track.mid);
                    tracks.push({ mid: track.mid, peer: speaker.policy.id });
                }
                if (result.requiresImmediateRenegotiation && result.sessionDescription) {
                    const answer = new Promise<SessionDescription>((resolve, reject) => {
                        const timer = setTimeout(() => reject(new Error("Answer timeout")), 10_000);
                        peer.answer = (description) => {
                            clearTimeout(timer);
                            peer.answer = undefined;
                            resolve(description);
                        };
                    });
                    // Only the speaker id goes to listeners: no group, mute, or permission metadata.
                    send(peer.socket, { type: "offer", sdp: result.sessionDescription.sdp, tracks });
                    await this.sfu.renegotiate(peer.session, await answer);
                }
                // The policy may have changed while waiting on the SFU or browser.
                if ([...peer.pulled.keys()].some((id) => !allowed(speakers.get(id)))) void this.reconcile(peer);
            })
            .catch((error) => {
                console.warn("Voice routing failed:", error instanceof Error ? error.message : error);
                this.closePeer(peer);
            });
        return peer.queue;
    }

    private iceServers(): unknown[] {
        try {
            return this.env.VOICE_ICE_SERVERS ? JSON.parse(this.env.VOICE_ICE_SERVERS) : DEFAULT_ICE_SERVERS;
        } catch {
            return DEFAULT_ICE_SERVERS;
        }
    }
}

function send(socket: WebSocket | undefined, data: unknown) {
    try {
        if (socket?.readyState === WebSocket.OPEN) socket.send(JSON.stringify(data));
    } catch {
        /* Closing. */
    }
}

function groupSet(value: unknown): Set<string> {
    if (
        !Array.isArray(value) ||
        value.length > 16 ||
        value.some((group) => typeof group !== "string" || group.length > 32)
    ) {
        throw new Error("Invalid group settings");
    }
    return new Set(value);
}
