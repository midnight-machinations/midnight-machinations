# Deploying the game server

The game logic is the Rust crate in `server/`, which runs two ways:

- **Cloudflare Workers (serverless):** `server/wasm` compiles the crate to WebAssembly. `server/worker/index.ts` hosts it in one Durable Object that owns every room and WebSocket, configured by `wrangler.json` here. No Rust game code changes between the two builds; only the socket, timer, clock, logging, and randomness plumbing is swapped (`server/src/platform.rs`).
- **Native:** the `mafia_server` binary, packaged by `Dockerfile` here.

## Cloudflare (free plan)

```sh
rustup target add wasm32-unknown-unknown
mask --maskfile deploy/voice/maskfile.md deploy     # voice first: the server binds to it
mask --maskfile deploy/server/maskfile.md deploy
VOICE_CONTROL_SECRET=... VOICE_PUBLIC_URL=wss://midnight-machinations-voice.<account>.workers.dev \
  mask --maskfile deploy/server/maskfile.md secrets
mask --maskfile deploy/client/maskfile.md deploy wss://midnight-machinations-server.<account>.workers.dev
```

The deploy runs `cargo build --release` in `server/wasm`. The module is about 1.2 MB, or 340 KB gzipped, well under the 3 MB Worker limit on the free plan. The server reaches voice through the `VOICE` service binding, so its control link is never public. Leave `VOICE_PUBLIC_URL` unset, or set `FEATURE_VOICE=false`, to run without voice.

How it fits the free plan:

- **One Durable Object for every room.** This mirrors the single native process, so room codes and rejoin work as before. It uses the SQLite-backed Durable Object class, which the free plan allows. Game state lives in memory, so a deploy or eviction ends running games, like a server restart.
- **It only runs while people are connected.** The once-a-second game tick and the voice control link stop when the last socket closes, so both objects can go idle. Durable Object duration is billed only while active, and the free allowance covers roughly 14 hours a day of active play across the game and voice objects together.
- **WebSocket messages, not requests.** Incoming WebSocket messages count toward the daily request allowance at 20 messages per request.
- **Panics.** A Rust panic traps the WebAssembly instance. The host then closes every socket and starts a fresh instance, as the native server restarts.

### Local

```sh
mask setup             # includes server/worker dependencies and mask config
mask voice start       # optional; another terminal
mask server worker     # the serverless server on the server port
```

Separate `wrangler dev` processes can't upgrade a WebSocket over a service binding, so `mask config` gives the local Worker a `VOICE_CONTROL_URL` and it dials voice directly. In production, leave that unset so the binding is used.

## Docker

```sh
mask --maskfile deploy/server/maskfile.md docker build
mask --maskfile deploy/server/maskfile.md docker run   # ws://localhost:9000
```

The image listens on port 80 (`WS_ADDRESS`). For voice, also pass `VOICE_CONTROL_SECRET`, `VOICE_CONTROL_URL` (the voice service's `/control` WebSocket), and `VOICE_PUBLIC_URL` (the browser-facing voice URL). Put TLS in front so browsers can use `wss://`.
