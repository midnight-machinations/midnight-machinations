# Static client served by `serve`, for self-hosting outside Cloudflare.
# Build from the repository root:
#   docker build -f deploy/client/Dockerfile --build-arg VITE_WS_ADDRESS=wss://example.com -t midnightmachinations-client .
FROM node:22-slim AS build
RUN corepack enable
WORKDIR /app
COPY client/package.json client/pnpm-lock.yaml client/pnpm-workspace.yaml ./client/
RUN --mount=type=cache,id=pnpm,target=/root/.local/share/pnpm/store \
    cd client && pnpm install --frozen-lockfile
COPY assets/public ./assets/public
COPY assets/resources ./assets/resources
COPY client ./client
ARG VITE_WS_ADDRESS=ws://localhost:80
ENV VITE_WS_ADDRESS=${VITE_WS_ADDRESS}
RUN cd client && pnpm build

FROM node:22-slim
ENV PNPM_HOME=/pnpm PATH=/pnpm:$PATH
RUN corepack enable && pnpm add --global serve
COPY --from=build /app/client/build /build
EXPOSE 3000
CMD ["serve", "-s", "/build", "-l", "3000"]
