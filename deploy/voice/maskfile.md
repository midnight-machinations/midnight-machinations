# Voice deployment

Run with `mask --maskfile deploy/voice/maskfile.md <task>` from the repository root. Cloudflare tasks need `wrangler login` (or `CLOUDFLARE_API_TOKEN`) first.

## deploy

> Deploy the voice Worker to Cloudflare

```sh
cd "$MASKFILE_DIR/../../voice" && pnpm install --frozen-lockfile && pnpm exec wrangler deploy -c ../deploy/voice/wrangler.json
```

## secrets

> Upload VOICE_CONTROL_SECRET, REALTIME_APP_ID, and REALTIME_APP_TOKEN (and optional VOICE_ALLOWED_ORIGINS, VOICE_ICE_SERVERS) from the environment

```sh
set -e
cd "$MASKFILE_DIR/../../voice"
for name in VOICE_CONTROL_SECRET REALTIME_APP_ID REALTIME_APP_TOKEN VOICE_ALLOWED_ORIGINS VOICE_ICE_SERVERS; do
  value=$(printenv "$name" || true)
  if [ -n "$value" ]; then
    printf '%s' "$value" | pnpm exec wrangler secret put "$name" -c ../deploy/voice/wrangler.json
  fi
done
```

## docker

> Self-hosting image

### docker build

```sh
cd "$MASKFILE_DIR/../.." && docker build -f deploy/voice/Dockerfile -t midnightmachinations-voice .
```

### docker run

> Run the image on port 9100 with secrets from voice/.dev.vars

```sh
cd "$MASKFILE_DIR/../.." && docker run --rm -p 9100:9100 --env-file voice/.dev.vars midnightmachinations-voice
```
