# Midnight Machinations tasks

Run these with [mask](https://github.com/jacobdeichert/mask) from the repository root (`cargo install mask`). `mask --help` lists every task.

Prerequisites: Node.js 22+, pnpm (`corepack enable`), and Rust with Cargo (`rustup component add clippy`). Use pnpm for all JavaScript dependencies. Each JavaScript package keeps its own pnpm lockfile; never generate npm lockfiles.

## setup

> Install client and voice dependencies and configure local voice

```sh
corepack enable || true
(cd client && pnpm install --frozen-lockfile)
(cd voice && pnpm install --frozen-lockfile && pnpm configure)
```

## start

> Start the client, Rust server, and voice together. Ctrl+C stops all three

```sh
node scripts/start-dev.mjs "$@"
```

## worktree (path) (branch)

> Create a git worktree with its own client, server, and voice ports

**OPTIONS**
* offset
    * flags: --offset
    * type: number
    * desc: Port offset. Defaults to the number of existing worktrees

```sh
scripts/new-worktree.sh "$path" "$branch" ${offset}
```

## check

> Everything CI checks: clippy, server tests, client build

```sh
set -e
(cd server && cargo clippy && cargo test)
(cd client && pnpm build)
```

## client

> Client tasks (TypeScript/React, Vite)

### client dev

> Dev server, default http://localhost:3000. Pass --no-open (not --open=false) to skip opening a browser

```sh
cd client && pnpm dev "$@"
```

### client build

> Type-check (tsc) and build. Run this to check for type errors

```sh
cd client && pnpm build
```

## server

> Rust game server tasks

### server run

> WebSocket server, default ws://localhost:9000

```sh
cd server && cargo run
```

### server test

```sh
cd server && cargo test
```

### server clippy

> CI runs clippy; keep it clean

```sh
cd server && cargo clippy
```

## voice

> Voice service tasks

### voice start

```sh
cd voice && pnpm start
```

### voice configure

> Write local voice secrets and ports into voice/.env.local and server/.env.local

**OPTIONS**
* offset
    * flags: --offset
    * type: number
    * desc: Port offset for simultaneous checkouts
* announced_address
    * flags: --announced-address
    * type: string
    * desc: Browser-reachable IPv4 address for media

```sh
cd voice && node configure-local.mjs ${offset} ${announced_address:+--announced-address "$announced_address"}
```
