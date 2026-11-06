# Deploying the game server

The game logic is the Rust crate in `server/`. It runs two ways:

- **Cloudflare Workers (serverless):** `server/wasm` compiles the crate to WebAssembly, and `server/worker/index.ts` hosts it in one Durable Object that owns every room and WebSocket (`wrangler.json` here).
- **Native:** the `mafia_server` binary, packaged by `Dockerfile` here.

## Docker

```sh
mask --maskfile deploy/server/maskfile.md docker build
mask --maskfile deploy/server/maskfile.md docker run   # ws://localhost:9000
```

The image listens on port 80 (`WS_ADDRESS`). For voice, also pass `VOICE_CONTROL_SECRET`, `VOICE_CONTROL_URL` (the voice service's `/control` WebSocket), and `VOICE_PUBLIC_URL` (the browser-facing voice URL). Put TLS in front so browsers can use `wss://`.
