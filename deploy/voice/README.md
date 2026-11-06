# Deploying voice

The voice service is a Cloudflare Worker with one Durable Object (`voice/src/index.ts`, configured by `wrangler.json` here). Audio goes through the Cloudflare Realtime SFU. Both fit the free plan: Workers Free includes SQLite-backed Durable Objects, and Realtime includes a monthly free allowance of SFU egress.

## Cloudflare

1. Create a Realtime SFU app in the dashboard (**Realtime → SFU**). Note its app ID and token.
2. Deploy and set secrets. `VOICE_CONTROL_SECRET` must be the same value the game server uses; generate it once with `openssl rand -hex 32`.

   ```sh
   mask --maskfile deploy/voice/maskfile.md deploy
   VOICE_CONTROL_SECRET=... REALTIME_APP_ID=... REALTIME_APP_TOKEN=... \
     VOICE_ALLOWED_ORIGINS=https://midnightmachinations.net mask --maskfile deploy/voice/maskfile.md secrets
   ```

3. Point the game server at it. When the game server also runs on Workers, it reaches voice through a service binding, so no control URL is needed (see `deploy/server`). A native game server needs `VOICE_CONTROL_URL=wss://<voice host>/control`.
4. Check it: `https://<voice host>/` returns `voice ready` once a game server is connected.

`VOICE_ALLOWED_ORIGINS` is a comma-separated list of exact frontend origins. `VOICE_ICE_SERVERS` is optional JSON (`RTCIceServer[]`) for TURN on restrictive networks. Those credentials go to every joined browser, so keep them limited. Cloudflare's own TURN service works here.

## Docker

`Dockerfile` runs the same Worker under `workerd` (via `wrangler dev`) for self-hosting. It still needs Realtime SFU credentials, because media goes through Cloudflare. It takes its configuration from the container environment.

```sh
mask --maskfile deploy/voice/maskfile.md docker build
mask --maskfile deploy/voice/maskfile.md docker run   # uses voice/.dev.vars
```

Put TLS in front of port 9100. Keep `/control` reachable only by the game server, or rely on its bearer secret.
