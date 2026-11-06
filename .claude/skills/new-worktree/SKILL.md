---
name: new-worktree
description: Create a new git worktree of this repo with its own client/server ports, so it can be developed and run alongside other checkouts. Use when asked to set up, create, or start work in a new worktree or parallel checkout.
---

# New worktree

Use the `mask worktree` task (in `maskfile.md`) from the repo root. Don't create the worktree with plain `git worktree add`, because the new checkout would then use the same ports as the main one.

## Steps

1. Decide on a branch name. If the user didn't give one, ask, or derive a short kebab-case name from the task.
2. Choose a path. By default, put it next to the main checkout: `../midnight-machinations-<branch>`, with any `/` in the branch name replaced by `-`.
3. Run the script from the main checkout's root:
   ```bash
   mask worktree <path> <branch> [--offset N]
   ```
   - If `<branch>` exists, the task checks it out. Otherwise it creates it from the current HEAD.
   - Leave out `--offset` unless the user asks for specific ports or the default clashes with something. The default is the number of existing worktrees, so the first extra worktree gets client 3001 and server 9001.
   - It copies the main checkout's `config.local.toml` (voice credentials and secret) with the new `[ports] offset`, runs `mask config` there, and then runs `pnpm install`. This can take a minute.
   - Ports are the `config.toml` ports plus the offset: client `3000 + offset`, server `9000 + offset`, voice `9100 + offset`.
4. Tell the user the worktree path and the client, server, and voice ports the task printed.

## Running the worktree

Run the tasks from the worktree root. Each task changes into its service directory, which is where that service's `.env.local` is read from:

```bash
cd <path> && mask server run          # serves on the worktree's server port
cd <path> && mask client dev --no-open  # serves on the worktree's client port
cd <path> && mask voice start         # serves on the worktree's voice ports (run mask setup first)
```


- Start the server too, not only the client. The worktree's client connects only to its own server port, so it won't reach a server running from another checkout. If the page can't connect, check that this worktree's server is running and that it logged `Started listening on 0.0.0.0:<server port>`.
- Don't run `cargo run` from the worktree root (for example with `--manifest-path`). From there, the server doesn't read `server/.env.local`.
- `pnpm dev` opens a browser tab by default. To stop that, use `--no-open`. Don't use `--open=false`, because Vite reads the value as a path and opens `http://localhost:<port>/false`.

## Notes

- The generated `.env.local` and `.dev.vars` files are gitignored. To change a worktree's ports or flags later, edit its `config.local.toml` and run `mask config` again.
- Each worktree has its own `server/target`, so the first `cargo build` in it starts from scratch.
- To remove a worktree: `git worktree remove <path>`, then `git branch -d <branch>` if the branch isn't needed anymore. Ask before deleting a branch that has unmerged commits.
