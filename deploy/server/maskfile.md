# Server deployment

Run with `mask --maskfile deploy/server/maskfile.md <task>` from the repository root. Cloudflare tasks need `wrangler login` (or `CLOUDFLARE_API_TOKEN`) and the `wasm32-unknown-unknown` Rust target (`rustup target add wasm32-unknown-unknown`).

## deploy

> Build the game server to WebAssembly and deploy it as a Cloudflare Worker

```sh
cd "$MASKFILE_DIR/../../server/worker" && pnpm install --frozen-lockfile && pnpm exec wrangler deploy -c ../../deploy/server/wrangler.json
```

## secrets

> Upload VOICE_CONTROL_SECRET and VOICE_PUBLIC_URL (and optional FEATURE_VOICE) from the environment

```sh
set -e
cd "$MASKFILE_DIR/../../server/worker"
for name in VOICE_CONTROL_SECRET VOICE_PUBLIC_URL FEATURE_VOICE; do
  value=$(printenv "$name" || true)
  if [ -n "$value" ]; then
    printf '%s' "$value" | pnpm exec wrangler secret put "$name" -c ../../deploy/server/wrangler.json
  fi
done
```

## docker

> Native server image, for self-hosting

### docker build

```sh
cd "$MASKFILE_DIR/../.." && docker build -f deploy/server/Dockerfile -t midnightmachinations-server .
```

### docker run

> Serve on port 9000 with the settings mask config generated in server/.env.local

```sh
cd "$MASKFILE_DIR/../.."
env_file=""
[ -f server/.env.local ] && env_file="--env-file server/.env.local"
docker run --rm -p 9000:80 $env_file -e WS_ADDRESS=0.0.0.0:80 midnightmachinations-server
```
