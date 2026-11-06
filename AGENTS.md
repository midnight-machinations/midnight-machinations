# AGENTS.md

Midnight Machinations (formerly Mafia) is a multiplayer social deduction game.

- `server/`: Rust WebSocket server and game logic (Cargo)
- `client/`: TypeScript/React frontend (Vite, pnpm)
- `voice/`: WebRTC voice service
- `generate/`: Python code generation utilities

## Commands

All development commands are tasks in `maskfile.md`. Run `mask --help` from the repository root to list them. The usual ones:

```bash
mask setup          # install dependencies and configure local voice
mask start          # client, server, and voice together
mask check          # cargo clippy, cargo test, and pnpm build
```

After making changes, run `mask server clippy` / `mask server test` for server changes and `mask client build` for client changes, or `mask check` for both. Use pnpm, never npm, for JavaScript dependencies.

## Configuration

- `server/.env` sets `WS_ADDRESS` (the address the server binds to). `client/.env` sets `VITE_WS_ADDRESS` (the address the client connects to). Both files are committed.
- To override them for one checkout, use the gitignored `server/.env.local` and `client/.env.local`. `client/.env.local` can also set `CLIENT_PORT` for the Vite dev server.

## Worktrees

To create a worktree that can run at the same time as other checkouts, use `mask worktree <path> <branch> [--offset N]`. It gives the worktree its own ports through the `.env.local` files. Claude Code users can run the `new-worktree` skill (`.claude/skills/new-worktree/`) instead.

## Guides

`.github/copilot-instructions.md` has step-by-step checklists for adding new roles and new chat messages on both the server and the client. Follow it when doing either.
