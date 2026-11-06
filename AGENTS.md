# AGENTS.md

Midnight Machinations (formerly Mafia) is a multiplayer social deduction game.

- `server/`: Rust game logic and native WebSocket server (Cargo). `server/wasm` builds it for WebAssembly, and `server/worker` hosts that build in a Cloudflare Durable Object
- `client/`: TypeScript/React frontend (Vite, pnpm)
- `voice/`: voice signaling Worker (TypeScript, Cloudflare Realtime SFU)
- `proto/`: protobuf definitions (a buf module) for Twirp endpoints. `mask proto generate` writes the TypeScript client to `client/src/gen`; `server/proto` generates the Rust side at build time
- `assets/`: static files shared across packages (`public/` is served by the client, `random_names/` is compiled into the server)
- `tools/gen/`: Rust generators for shared files and local settings (`mask gen <generator>`, `mask config`)
- `deploy/`: per-service wrangler configs, Dockerfiles, and deploy tasks

## Commands

All development commands are tasks in `maskfile.md`. Run `mask --help` from the repository root to list them. The usual ones:

```bash
mask setup          # install dependencies and generate local settings (mask config)
mask start          # client, server, and voice together
mask check          # cargo clippy, cargo test, and pnpm build
mask lint           # biome and clippy suggestions (advisory, never blocks)
```

After making changes, run `mask server clippy` / `mask server test` for server changes and `mask client build` for client changes, or `mask check` for both. Use pnpm, never npm, for JavaScript dependencies.

## Configuration

- `config.toml` holds local ports, feature flags (`[features]`), and voice settings. Overrides and secrets go in the gitignored `config.local.toml`. `mask config` generates `client/.env.local`, `server/.env.local`, `deploy/server/.dev.vars`, and `deploy/voice/.dev.vars` from them, so edit the TOML files, not the generated ones.
- A feature flag `name` reaches the server as `FEATURE_NAME` and the client as `VITE_FEATURE_NAME` (read through `client/src/features.ts`). Add new flags to `config.toml`.
- The committed `server/.env` and `client/.env` hold defaults when nothing was generated.

## Worktrees

To create a worktree that can run at the same time as other checkouts, use `mask worktree <path> <branch> [--offset N]`. It gives the worktree its own ports through its `config.local.toml`. Claude Code users can run the `new-worktree` skill (`.claude/skills/new-worktree/`) instead.

## Guides

`.github/copilot-instructions.md` has step-by-step checklists for adding new roles and new chat messages on both the server and the client. Follow it when doing either.
