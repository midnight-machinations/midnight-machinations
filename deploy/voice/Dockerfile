# Voice worker run under workerd (wrangler dev), for self-hosting outside Cloudflare.
# Media still goes through the Cloudflare Realtime SFU, so pass REALTIME_APP_ID and
# REALTIME_APP_TOKEN along with VOICE_CONTROL_SECRET (and optionally
# VOICE_ALLOWED_ORIGINS, VOICE_ICE_SERVERS) as container environment variables.
# Build from the repository root: docker build -f deploy/voice/Dockerfile -t midnightmachinations-voice .
FROM node:22-bookworm-slim
RUN corepack enable
WORKDIR /app/voice
COPY voice/package.json voice/pnpm-lock.yaml voice/pnpm-workspace.yaml ./
RUN pnpm install --frozen-lockfile
COPY voice ./
COPY deploy/voice/wrangler.json ../deploy/voice/wrangler.json
ENV WRANGLER_SEND_METRICS=false
EXPOSE 9100
USER node
CMD ["sh", "-c", "env | grep -E '^(VOICE_|REALTIME_)' > /tmp/voice.env; exec pnpm exec wrangler dev -c ../deploy/voice/wrangler.json --env-file /tmp/voice.env --ip 0.0.0.0 --port 9100 --show-interactive-dev-session=false"]
