# Midnight Machinations tasks

Run these with [mask](https://github.com/jacobdeichert/mask) from the repository root (`cargo install mask`). `mask --help` lists every task.

Prerequisites: Node.js 22+, pnpm (`corepack enable`), and Rust with Cargo (`rustup component add clippy`). Use pnpm for all JavaScript dependencies. Each JavaScript package keeps its own pnpm lockfile; never generate npm lockfiles.

Local settings, including ports, feature flags, and voice credentials, live in `config.toml`, with overrides in the gitignored `config.local.toml`. `mask config` turns them into each service's `.env.local` and `.dev.vars` files.

## setup

> Install dependencies and generate local settings

```sh
set -e
corepack enable || true
pnpm install --frozen-lockfile
(cd client && pnpm install --frozen-lockfile)
(cd voice && pnpm install --frozen-lockfile)
(cd server/worker && pnpm install --frozen-lockfile)
(cd proto && pnpm install --frozen-lockfile)
mask config
```

## config

> Generate every service's local settings from config.toml and config.local.toml

Writes `client/.env.local`, `server/.env.local`, `deploy/server/.dev.vars`, and `deploy/voice/.dev.vars`, replacing what was there. It generates a voice control secret into `config.local.toml` the first time. Restart running services afterwards.

```sh
cargo run --quiet --manifest-path tools/gen/Cargo.toml -- vars
```

## start

> Start the client, Rust server, and (when features.voice is on) voice together. Ctrl+C stops all

Output is prefixed by service. If one service exits, the others are stopped and the commands for running each one separately are printed. Missing setup is reported, never installed automatically. Vite listens on all interfaces (so the dev container is reachable), never opens a browser, and fails on an occupied port. Needs Linux, macOS, or WSL.

**OPTIONS**
* host
    * flags: --host
    * type: string
    * desc: Address Vite listens on, such as localhost. Defaults to all interfaces

```bash
env_value() { [ -f "$1" ] && sed -n "s/^$2=//p" "$1" | tail -n 1; }
voice=$(env_value server/.env.local FEATURE_VOICE)
manual() {
  echo "Run each service in a separate terminal from the repository root:"
  echo "  mask client dev --no-open"
  echo "  mask server run"
  [ "$voice" = true ] && echo "  mask voice start"
}
missing=""
command -v pnpm >/dev/null || missing="$missing\n  pnpm is unavailable. Run: corepack enable"
command -v cargo >/dev/null || missing="$missing\n  cargo is unavailable. Install Rust with rustup."
[ -f client/node_modules/vite/bin/vite.js ] || missing="$missing\n  Client dependencies are missing. Run: mask setup"
for file in client/.env.local server/.env.local deploy/voice/.dev.vars; do
  grep -q "mask config" "$file" 2>/dev/null || missing="$missing\n  $file was not generated. Run: mask config"
done
if [ "$voice" = true ]; then
  [ -d voice/node_modules/wrangler ] || missing="$missing\n  Voice dependencies are missing. Run: mask setup"
fi
if [ -n "$missing" ]; then printf "[dev] Setup incomplete:$missing\n\n" >&2; manual >&2; exit 1; fi

prefix() { while IFS= read -r line; do printf '[%s] %s\n' "$1" "$line"; done; }
# Job control puts each service in its own process group, so stopping reaches pnpm, Cargo, and their children.
set -m
[ "$voice" = true ] && { (exec mask voice start) > >(prefix voice) 2>&1 & }
(cd server && exec cargo run) > >(prefix server) 2>&1 &
(cd client && exec pnpm dev --no-open --strictPort --host ${host:-}) > >(prefix client) 2>&1 &
pids=$(jobs -p)
stop() {
  trap - INT TERM
  echo "[dev] Stopping all services..."
  for pid in $pids; do kill -TERM -- "-$pid" 2>/dev/null; done
  for _ in $(seq 40); do
    alive=""
    for pid in $pids; do kill -0 -- "-$pid" 2>/dev/null && alive=1; done
    [ -z "$alive" ] && break
    sleep 0.1
  done
  for pid in $pids; do kill -KILL -- "-$pid" 2>/dev/null; done
  exit "$1"
}
trap 'stop 130' INT
trap 'stop 143' TERM
echo "[dev] Starting services. Ctrl+C stops all of them. Cargo may take time to compile."
wait -n
status=$?
echo "[dev] A service exited unexpectedly (code $status)." >&2
manual >&2
stop $(( status > 0 ? status : 1 ))
```

## worktree (path) (branch)

> Create a git worktree with its own client, server, and voice ports

Checks out `branch` if it exists, otherwise creates it from HEAD. Copies this checkout's `config.local.toml` (credentials and the voice secret) with the new port offset, runs `mask config` there, and installs client dependencies. Ports are the `config.toml` ports plus the offset.

**OPTIONS**
* offset
    * flags: --offset
    * type: number
    * desc: Port offset. Defaults to the number of existing worktrees, so each one gets the next free ports

```sh
set -e
offset=${offset:-$(git worktree list | wc -l | tr -d ' ')}
if git show-ref --verify --quiet "refs/heads/$branch"; then
  git worktree add "$path" "$branch"
else
  git worktree add -b "$branch" "$path"
fi
# Keep everything from this checkout's local config except its offset.
{
  [ -f config.local.toml ] && sed '/^offset *=/d; /^\[ports\]/d' config.local.toml
  printf '\n[ports]\noffset = %s\n' "$offset"
} > "$path/config.local.toml"
(cd "$path" && mask config)
(cd "$path/client" && pnpm install --frozen-lockfile)
echo
echo "Worktree ready at $(cd "$path" && pwd). Run mask start there."
```

## check

> Everything CI checks: proto lint, clippy, server tests, client build and tests, voice type-check and tests

```sh
set -e
(cd proto && pnpm exec buf lint)
(cd server && cargo clippy && cargo test)
(cd server/proto && cargo clippy)
(cd client && pnpm build && pnpm test)
(cd voice && pnpm check && pnpm test)
```

## lint

> Biome (TypeScript) and clippy (every Rust crate). Advisory: it reports and never fixes

Biome rules and clippy thresholds are copied from [aislop](https://github.com/scanaislop/aislop) (`biome.json`, `clippy.toml`, and each crate's `[lints.clippy]`). Existing code still has findings, so this is not part of `mask check`.

```sh
pnpm exec biome check . || true
for crate in server server/wasm server/proto tools/gen; do (cd "$crate" && cargo clippy); done
```

## format

> Format TypeScript with Biome (writes files)

```sh
pnpm exec biome format --write .
```

## e2e

> Playwright browser tests in e2e/. They start the client dev server themselves unless one is already running

### e2e install

> Download the Chromium browser Playwright drives (once per machine)

```sh
pnpm exec playwright install chromium
```

### e2e test

> Run the browser tests

**OPTIONS**
* ui
    * flags: --ui
    * desc: Open Playwright's UI mode instead of running headless
* headed
    * flags: --headed
    * desc: Show the browser while the tests run

```sh
pnpm exec playwright test --pass-with-no-tests ${ui:+--ui} ${headed:+--headed}
```

## deploy

> Deployment tasks live in deploy/{server,voice,client}/maskfile.md

Run them with `mask --maskfile deploy/<service>/maskfile.md <task>`. Each directory's README explains its Cloudflare and Docker setup.

```sh
for service in server voice client; do echo "== $service"; mask --maskfile "deploy/$service/maskfile.md" --help | sed -n '/SUBCOMMANDS/,$p'; done
```

## gen (generator)

> Regenerate shared files: dummy-names, broken-keyboard, scrambled, or all

`dummy-names` reads `assets/random_names/names.json` and writes `assets/resources/dummyNames.json` and the server's name CSVs. `broken-keyboard` and `scrambled` derive client language files from `en_us.json`. (`vars` is `mask config`.)

```sh
cargo run --quiet --manifest-path tools/gen/Cargo.toml -- "$generator"
```

## client

> Client tasks (TypeScript/React, Vite)

### client dev

> Dev server, default http://localhost:3000

**OPTIONS**
* no_open
    * flags: --no-open
    * desc: Don't open a browser tab
* host
    * flags: --host
    * type: string
    * desc: Address to listen on

```sh
cd client && pnpm dev ${no_open:+--no-open} ${host:+--host "$host"}
```

### client build

> Type-check (tsc) and build. Run this to check for type errors

```sh
cd client && pnpm build
```

### client test

> Run the client unit tests in client/tests

```sh
cd client && pnpm test
```

## server

> Rust game server tasks

### server run

> Native WebSocket server, default ws://localhost:9000

```sh
cd server && cargo run
```

### server worker

> The serverless game server under wrangler dev (Rust compiled to WebAssembly), on the same port

```sh
port=$(sed -n 's/^SERVER_DEV_PORT=//p' deploy/server/.dev.vars 2>/dev/null | tail -n 1)
cd server/worker && pnpm start --port "${port:-9000}"
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

## proto

> Protobuf and Twirp tasks. Definitions live in proto/ (a buf module)

Services are served as Twirp, so every endpoint accepts both protobuf and JSON (`Content-Type: application/json`). `buf generate` writes the TypeScript messages and Twirp client to `client/src/gen` (protobuf-ts). Rust types and server stubs are generated by `server/proto/build.rs` (twurst-build) whenever the crate builds, since Twirp has no buf plugin for Rust. Commit the generated TypeScript.

### proto generate

> Regenerate client/src/gen from proto/

```sh
cd proto && pnpm exec buf generate
```

### proto lint

> Lint the protos with buf

```sh
cd proto && pnpm exec buf lint
```

### proto breaking

> Check the protos for breaking changes against the main branch

```sh
cd proto && pnpm exec buf breaking --against '../.git#branch=main,subdir=proto'
```

## voice

> Voice service tasks (a Cloudflare Worker, run locally with wrangler)

### voice start

> Run the voice worker locally. Audio needs Realtime SFU credentials in config.local.toml

```sh
port=$(sed -n 's/^VOICE_DEV_PORT=//p' deploy/voice/.dev.vars 2>/dev/null | tail -n 1)
cd voice && pnpm start --port "${port:-9100}"
```

### voice test

> Type-check and run the routing tests

```sh
cd voice && pnpm check && pnpm test
```
