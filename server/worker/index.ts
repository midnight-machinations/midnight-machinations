/**
 * Serverless host for the Rust game server (`../wasm`), deployed with
 * `deploy/server/wrangler.json`.
 *
 * One Durable Object owns every room, like the single native process: it
 * accepts game WebSockets, feeds them to the WebAssembly listener, ticks it
 * once a second while anyone is connected, and relays voice routing snapshots
 * to the voice Worker over a service binding.
 */
import { DurableObject } from "cloudflare:workers";
import wasm from "../wasm/target/wasm32-unknown-unknown/release/mafia_wasm.wasm";

export interface Env {
    GAME: DurableObjectNamespace<Game>;
    /** Service binding to the voice Worker. Voice needs it or VOICE_CONTROL_URL, plus the two settings below. */
    VOICE?: Fetcher;
    /**
     * Dial the voice control link over the network instead of the binding, such as
     * ws://127.0.0.1:9100/control in local dev, where the two Workers run in separate processes.
     */
    VOICE_CONTROL_URL?: string;
    VOICE_CONTROL_SECRET?: string;
    /** Browser-facing voice URL, such as wss://midnight-machinations-voice.<account>.workers.dev */
    VOICE_PUBLIC_URL?: string;
    /** Feature flag from config.toml; "false" turns voice off. */
    FEATURE_VOICE?: string;
}

type Exports = {
    memory: WebAssembly.Memory;
    alloc(len: number): number;
    dealloc(ptr: number, len: number): void;
    output_ptr(): number;
    init(ptr: number, len: number): void;
    connect(id: number): number;
    message(id: number, ptr: number, len: number): number;
    disconnect(id: number): number;
    tick(elapsedMs: number): number;
    voice_presence(ptr: number, len: number): number;
    voice_resend(): number;
};

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export default {
    fetch(request: Request, env: Env): Promise<Response> {
        return env.GAME.get(env.GAME.idFromName("game")).fetch(request);
    },
} satisfies ExportedHandler<Env>;

export class Game extends DurableObject<Env> {
    private game!: Exports;
    private sockets = new Map<number, WebSocket>();
    private nextId = 1;
    private ticker?: ReturnType<typeof setInterval>;
    private lastTick = 0;
    private voice?: WebSocket;
    private voiceConnecting = false;
    private voiceVersion = 0;
    private latestVoiceSnapshot?: string;

    constructor(ctx: DurableObjectState, env: Env) {
        super(ctx, env);
        this.start();
    }

    async fetch(request: Request): Promise<Response> {
        if (request.headers.get("upgrade")?.toLowerCase() !== "websocket") {
            return new Response("Midnight Machinations game server\n");
        }
        const [client, socket] = Object.values(new WebSocketPair());
        socket.accept();
        const id = this.nextId++;
        this.sockets.set(id, socket);
        this.run(() => this.game.connect(id));
        socket.addEventListener("message", (event) => {
            if (typeof event.data === "string")
                this.run(() => this.withString(event.data as string, (ptr, len) => this.game.message(id, ptr, len)));
        });
        const closed = () => {
            if (!this.sockets.delete(id)) return;
            this.run(() => this.game.disconnect(id));
            if (this.sockets.size === 0) this.idle();
        };
        socket.addEventListener("close", closed);
        socket.addEventListener("error", closed);
        this.startTicking();
        return new Response(null, { status: 101, webSocket: client });
    }

    private start() {
        const instance = new WebAssembly.Instance(wasm, {
            host: {
                now_ms: () => Date.now(),
                log: (ptr: number, len: number) => console.log(this.read(ptr, len)),
                random: (ptr: number, len: number) => {
                    // getRandomValues fills at most 65536 bytes per call.
                    for (let offset = 0; offset < len; offset += 65536) {
                        crypto.getRandomValues(
                            new Uint8Array(this.game.memory.buffer, ptr + offset, Math.min(65536, len - offset)),
                        );
                    }
                },
            },
        });
        this.game = instance.exports as unknown as Exports;
        const voiceOn =
            this.env.FEATURE_VOICE !== "false" &&
            (this.env.VOICE || this.env.VOICE_CONTROL_URL) &&
            this.env.VOICE_CONTROL_SECRET &&
            this.env.VOICE_PUBLIC_URL;
        const voiceUrl = voiceOn ? (this.env.VOICE_PUBLIC_URL ?? "") : "";
        this.withString(voiceUrl, (ptr, len) => this.game.init(ptr, len));
    }

    /** Calls into the game and delivers its output. A trap means the Rust code panicked. */
    private run(call: () => number) {
        let len: number;
        try {
            len = call();
        } catch (error) {
            if (!(error instanceof WebAssembly.RuntimeError)) throw error;
            this.crashed(error);
            return;
        }
        if (len === 0) return;
        const output = this.read(this.game.output_ptr(), len);
        for (const line of output.split("\n")) {
            const space = line.indexOf(" ");
            if (space < 0) continue;
            const target = line.slice(0, space);
            const payload = line.slice(space + 1);
            if (target === "voice") this.sendVoice(payload);
            else {
                try {
                    this.sockets.get(Number(target))?.send(payload);
                } catch {
                    /* Socket is closing. */
                }
            }
        }
    }

    /** Like the native server after a panic: drop everyone and start over. */
    private crashed(error: Error) {
        console.error("The game server panicked; restarting.", error);
        const sockets = [...this.sockets.values()];
        this.sockets.clear();
        for (const socket of sockets) {
            try {
                socket.close(1011, "Server restarted");
            } catch {
                /* Already closed. */
            }
        }
        this.idle();
        this.start();
    }

    private startTicking() {
        if (this.ticker) return;
        this.lastTick = Date.now();
        this.ticker = setInterval(() => {
            const now = Date.now();
            const elapsed = now - this.lastTick;
            this.lastTick = now;
            this.run(() => this.game.tick(elapsed));
        }, 1000);
    }

    /** With nobody connected, stop the clock and the voice link so both objects can go idle. */
    private idle() {
        if (this.ticker) clearInterval(this.ticker);
        this.ticker = undefined;
        const voice = this.voice;
        this.voice = undefined;
        voice?.close(1000, "Idle");
    }

    // ---- Voice control link ----

    private sendVoice(snapshot: string) {
        this.latestVoiceSnapshot = snapshot;
        if (this.voice) this.voice.send(`{"version":${++this.voiceVersion},"rooms":${snapshot}}`);
        else if (snapshot !== "[]" && this.sockets.size > 0) void this.connectVoice();
    }

    private async connectVoice() {
        if (this.voiceConnecting || !(this.env.VOICE || this.env.VOICE_CONTROL_URL)) return;
        this.voiceConnecting = true;
        try {
            const url = this.env.VOICE_CONTROL_URL?.replace(/^ws/, "http");
            const request = new Request(url ?? "https://voice/control", {
                headers: { upgrade: "websocket", authorization: `Bearer ${this.env.VOICE_CONTROL_SECRET}` },
            });
            const response = url || !this.env.VOICE ? await fetch(request) : await this.env.VOICE.fetch(request);
            const socket = response.webSocket;
            if (!socket) throw new Error(`Voice control link refused (${response.status})`);
            socket.accept();
            if (this.sockets.size === 0) {
                socket.close(1000, "Idle");
                return;
            }
            this.voice = socket;
            this.voiceVersion = 0;
            socket.addEventListener("message", (event) => {
                const message = JSON.parse(String(event.data)) as { presence?: unknown };
                if (message.presence) {
                    this.run(() =>
                        this.withString(JSON.stringify(message.presence), (ptr, len) =>
                            this.game.voice_presence(ptr, len),
                        ),
                    );
                }
            });
            const lost = () => {
                if (this.voice !== socket) return;
                this.voice = undefined;
                this.run(() => this.withString("{}", (ptr, len) => this.game.voice_presence(ptr, len)));
                if (this.sockets.size > 0) setTimeout(() => void this.connectVoice(), 2000);
            };
            socket.addEventListener("close", lost);
            socket.addEventListener("error", lost);
            // The voice service starts empty: send the current routes first.
            this.run(() => this.game.voice_resend());
        } catch (error) {
            console.warn("Voice control link failed", error);
            if (this.sockets.size > 0 && this.latestVoiceSnapshot !== "[]")
                setTimeout(() => void this.connectVoice(), 2000);
        } finally {
            this.voiceConnecting = false;
        }
    }

    // ---- Linear memory ----

    private read(ptr: number, len: number): string {
        return decoder.decode(new Uint8Array(this.game.memory.buffer, ptr, len));
    }

    private withString<T>(text: string, use: (ptr: number, len: number) => T): T {
        const bytes = encoder.encode(text);
        const len = bytes.length;
        const ptr = this.game.alloc(len);
        new Uint8Array(this.game.memory.buffer, ptr, len).set(bytes);
        try {
            return use(ptr, len);
        } finally {
            this.game.dealloc(ptr, len);
        }
    }
}
