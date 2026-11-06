# Midnight Machinations tasks

Run these with [mask](https://github.com/jacobdeichert/mask) from the repository root (`cargo install mask`). `mask --help` lists every task.

Prerequisites: Node.js 22+, pnpm (`corepack enable`), and Rust with Cargo (`rustup component add clippy`). Use pnpm for all JavaScript dependencies. Each JavaScript package keeps its own pnpm lockfile; never generate npm lockfiles.

## setup

> Install client and voice dependencies and configure local voice

```sh
corepack enable || true
(cd client && pnpm install --frozen-lockfile)
(cd voice && pnpm install --frozen-lockfile)
mask voice configure
```

## start

> Start the client, Rust server, and voice together. Ctrl+C stops all three

Output is prefixed by service. If one service exits, the others are stopped and the commands for running each one separately are printed. Missing setup is reported, never installed automatically. Vite listens on all interfaces (so the dev container is reachable), never opens a browser, and fails on an occupied port. Existing `.env.local` port settings still apply. Needs Linux, macOS, or WSL.

**OPTIONS**
* host
    * flags: --host
    * type: string
    * desc: Address Vite listens on, such as localhost. Defaults to all interfaces

```bash
manual() {
  echo "Run each service in a separate terminal from the repository root:"
  echo "  mask client dev --no-open"
  echo "  mask server run"
  echo "  mask voice start"
}
env_value() { [ -f "$1" ] && sed -n "s/^$2=//p" "$1" | tail -n 1; }
missing=""
command -v pnpm >/dev/null || missing="$missing\n  pnpm is unavailable. Run: corepack enable"
command -v cargo >/dev/null || missing="$missing\n  cargo is unavailable. Install Rust with rustup."
[ -f client/node_modules/vite/bin/vite.js ] || missing="$missing\n  Client dependencies are missing. Run: mask setup"
[ -d voice/node_modules/wrangler ] || missing="$missing\n  Voice dependencies are missing. Run: mask setup"
secret=$(env_value voice/.dev.vars VOICE_CONTROL_SECRET)
server_secret=$(env_value server/.env.local VOICE_CONTROL_SECRET)
if [ -z "$secret" ] || [ -z "$server_secret" ] || [ -z "$(env_value server/.env.local VOICE_PUBLIC_URL)" ]; then
  missing="$missing\n  Local voice settings are missing. Run: mask voice configure"
elif [ "$secret" != "$server_secret" ]; then
  missing="$missing\n  The voice and server control secrets do not match. Run: mask voice configure"
fi
if [ -n "$missing" ]; then printf "[dev] Setup incomplete:$missing\n\n" >&2; manual >&2; exit 1; fi

prefix() { while IFS= read -r line; do printf '[%s] %s\n' "$1" "$line"; done; }
# Job control puts each service in its own process group, so stopping reaches pnpm, Cargo, and their children.
set -m
(cd voice && exec pnpm start) > >(prefix voice) 2>&1 &
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
echo "[dev] Starting client, server, and voice. Ctrl+C stops all three. Cargo may take time to compile."
wait -n
status=$?
echo "[dev] A service exited unexpectedly (code $status)." >&2
manual >&2
stop $(( status > 0 ? status : 1 ))
```

## worktree (path) (branch)

> Create a git worktree with its own client, server, and voice ports

Checks out `branch` if it exists, otherwise creates it from HEAD. Writes the worktree's `client/.env.local` and `server/.env.local`, configures voice with the same offset, and installs client dependencies. The client gets port `3000 + offset`, the server `9000 + offset`, and voice `9100 + offset`.

**OPTIONS**
* offset
    * flags: --offset
    * type: number
    * desc: Port offset. Defaults to the number of existing worktrees, so each one gets the next free ports

```sh
set -e
offset=${offset:-$(git worktree list | wc -l | tr -d ' ')}
client_port=$((3000 + offset))
server_port=$((9000 + offset))
if git show-ref --verify --quiet "refs/heads/$branch"; then
  git worktree add "$path" "$branch"
else
  git worktree add -b "$branch" "$path"
fi
printf "VITE_WS_ADDRESS=ws://localhost:%s\nCLIENT_PORT=%s\n" "$server_port" "$client_port" > "$path/client/.env.local"
echo "WS_ADDRESS=0.0.0.0:$server_port" > "$path/server/.env.local"
# Also adds the voice settings to server/.env.local.
(cd "$path" && mask voice configure --offset "$offset")
(cd "$path/client" && pnpm install --frozen-lockfile)
path=$(cd "$path" && pwd)
echo
echo "Worktree ready at $path"
echo "  server: ws://localhost:$server_port (cd $path && mask server run)"
echo "  client: http://localhost:$client_port (cd $path && mask client dev --no-open)"
echo "  voice:  ws://localhost:$((9100 + offset)) (cd $path && mask voice start)"
```

## check

> Everything CI checks: clippy, server tests, client build

```sh
set -e
(cd server && cargo clippy && cargo test)
(cd client && pnpm build)
```

## deploy

> Deployment tasks live in deploy/{server,voice,client}/maskfile.md

Run them with `mask --maskfile deploy/<service>/maskfile.md <task>`. Each directory's README explains its Cloudflare and Docker setup.

```sh
for service in server voice client; do echo "== $service"; mask --maskfile "deploy/$service/maskfile.md" --help | sed -n '/SUBCOMMANDS/,$p'; done
```

## gen (generator)

> Regenerate shared files: dummy-names, broken-keyboard, scrambled, or all

`dummy-names` reads `assets/random_names/names.json` and writes the client's `dummyNames.json` and the server's name CSVs. `broken-keyboard` and `scrambled` derive client language files from `en_us.json`.

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

> Voice service tasks (a Cloudflare Worker, run locally with wrangler)

### voice start

> Run the voice worker locally. Needs Realtime SFU credentials in voice/.dev.vars

```sh
port=$(sed -n 's/^VOICE_DEV_PORT=//p' voice/.dev.vars 2>/dev/null | tail -n 1)
cd voice && pnpm start --port "${port:-9100}"
```

### voice configure

> Write a shared control secret and ports into voice/.dev.vars and server/.env.local

Keeps an existing secret and unrelated settings. Add your Cloudflare Realtime SFU app ID and token to `voice/.dev.vars` afterwards (see voice/README.md). Restart the server after running this.

**OPTIONS**
* offset
    * flags: --offset
    * type: number
    * desc: Port offset for simultaneous checkouts. Voice listens on 9100 + offset

```sh
set -e
port=$((9100 + ${offset:-0}))
set_var() { # file key value
  touch "$1"
  grep -v "^$2=" "$1" > "$1.tmp" || true
  echo "$2=$3" >> "$1.tmp"
  mv "$1.tmp" "$1"
}
secret=$(sed -n 's/^VOICE_CONTROL_SECRET=//p' voice/.dev.vars 2>/dev/null | tail -n 1)
[ -n "$secret" ] || secret=$(node -e "console.log(require('crypto').randomBytes(32).toString('hex'))")
set_var voice/.dev.vars VOICE_CONTROL_SECRET "$secret"
set_var voice/.dev.vars VOICE_DEV_PORT "$port"
grep -q '^REALTIME_APP_ID=' voice/.dev.vars || printf 'REALTIME_APP_ID=\nREALTIME_APP_TOKEN=\n' >> voice/.dev.vars
set_var server/.env.local VOICE_CONTROL_SECRET "$secret"
set_var server/.env.local VOICE_CONTROL_URL "ws://127.0.0.1:$port/control"
set_var server/.env.local VOICE_PUBLIC_URL "ws://localhost:$port"
echo "Configured voice/.dev.vars and server/.env.local for voice on port $port."
grep -q '^REALTIME_APP_ID=.' voice/.dev.vars || echo "Add REALTIME_APP_ID and REALTIME_APP_TOKEN to voice/.dev.vars (see voice/README.md)."
```

### voice test

> Type-check and run the routing tests

```sh
cd voice && pnpm check && pnpm test
```
