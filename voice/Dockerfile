FROM node:22-bookworm-slim AS build
RUN apt-get update && apt-get install -y --no-install-recommends python3 python3-pip make g++ && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml ./
RUN corepack enable && pnpm install --prod --frozen-lockfile

FROM node:22-bookworm-slim
WORKDIR /app
COPY --from=build /app/node_modules ./node_modules
COPY package.json ./
COPY src ./src
USER node
EXPOSE 9100 44400-44402/udp 44400-44402/tcp
CMD ["node", "src/server.mjs"]
