# AGENTS.md

Midnight Machinations (formerly Mafia) is a multiplayer social deduction game.

- `server/`: Rust WebSocket server and game logic (Cargo)
- `client/`: TypeScript/React frontend (Vite, pnpm)
- `generate/`: Python code generation utilities

## Commands

Client (from `client/`):
```bash
corepack enable
pnpm install --frozen-lockfile
pnpm dev          # dev server, default http://localhost:3000; add --no-open (not --open=false) to skip opening a browser
pnpm build        # type-checks (tsc) and builds; run this to check for type errors
```

Server (from `server/`):
```bash
cargo run         # WebSocket server, default ws://localhost:9000
cargo test
cargo clippy      # CI runs clippy; keep it clean
```

After making changes, run `cargo clippy` / `cargo test` for server changes and `pnpm build` for client changes.

## Configuration

- `server/.env` sets `WS_ADDRESS` (the address the server binds to). `client/.env` sets `VITE_WS_ADDRESS` (the address the client connects to). Both files are committed.
- To override them for one checkout, use the gitignored `server/.env.local` and `client/.env.local`. `client/.env.local` can also set `CLIENT_PORT` for the Vite dev server.

## Worktrees

To create a worktree that can run at the same time as other checkouts, use `scripts/new-worktree.sh <path> <branch> [port-offset]`. It gives the worktree its own ports through the `.env.local` files. Claude Code users can run the `new-worktree` skill (`.claude/skills/new-worktree/`) instead.

## Guides

`.github/copilot-instructions.md` has step-by-step checklists for adding new roles and new chat messages on both the server and the client. Follow it when doing either.
