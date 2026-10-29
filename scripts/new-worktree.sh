#!/usr/bin/env sh
# Creates a git worktree with its own client/server ports so it can run alongside other checkouts.
# Usage: scripts/new-worktree.sh <path> <branch> [port-offset]
set -e

if [ $# -lt 2 ]; then
    echo "Usage: $0 <path> <branch> [port-offset]" >&2
    exit 1
fi

path=$1
branch=$2
# Default offset: number of existing worktrees, so each new one gets the next free pair of ports
offset=${3:-$(git worktree list | wc -l)}

client_port=$((3000 + offset))
server_port=$((9000 + offset))

if git show-ref --verify --quiet "refs/heads/$branch"; then
    git worktree add "$path" "$branch"
else
    git worktree add -b "$branch" "$path"
fi

printf "VITE_WS_ADDRESS=ws://localhost:%s\nCLIENT_PORT=%s\n" "$server_port" "$client_port" > "$path/client/.env.local"
echo "WS_ADDRESS=0.0.0.0:$server_port" > "$path/server/.env.local"

# Voice gets its own ports from the same offset; this also adds the voice settings to server/.env.local
if [ -f "$path/voice/configure-local.mjs" ]; then
    (cd "$path/voice" && node configure-local.mjs "$offset")
fi

(cd "$path/client" && pnpm install --frozen-lockfile)

# Print absolute paths so the commands below work from any directory
path=$(cd "$path" && pwd)

echo
echo "Worktree ready at $path"
echo "  server: ws://localhost:$server_port (cd $path/server && cargo run)"
echo "  client: http://localhost:$client_port (cd $path/client && pnpm dev, add --no-open to skip opening a browser)"
if [ -f "$path/voice/configure-local.mjs" ]; then
    echo "  voice:  ws://localhost:$((9100 + offset * 2)) (cd $path/voice && npm ci && npm start)"
fi
