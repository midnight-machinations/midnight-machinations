# Client deployment

Run with `mask --maskfile deploy/client/maskfile.md <task>` from the repository root. Cloudflare tasks need `wrangler login` (or `CLOUDFLARE_API_TOKEN`) first.

## build (ws_address)

> Build the client for a given game server address, such as wss://midnightmachinations-server.example.workers.dev

```sh
cd "$MASKFILE_DIR/../../client" && pnpm install --frozen-lockfile && VITE_WS_ADDRESS="$ws_address" pnpm build
```

## deploy (ws_address)

> Build and upload the client as Workers static assets

```sh
set -e
mask --maskfile "$MASKFILE_DIR/maskfile.md" build "$ws_address"
cd "$MASKFILE_DIR/../../voice" && pnpm install --frozen-lockfile && pnpm exec wrangler deploy -c ../deploy/client/wrangler.json
```

## docker

> Self-hosting image

### docker build (ws_address)

```sh
cd "$MASKFILE_DIR/../.." && docker build -f deploy/client/Dockerfile --build-arg VITE_WS_ADDRESS="$ws_address" -t midnightmachinations-client .
```

### docker run

```sh
docker run --rm -p 3000:3000 midnightmachinations-client
```
