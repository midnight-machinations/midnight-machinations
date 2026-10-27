---
name: new-worktree
description: Create a new git worktree of this repo with its own client/server ports, so it can be developed and run alongside other checkouts. Use when asked to set up, create, or start work in a new worktree or parallel checkout.
---

# New worktree

Use `scripts/new-worktree.sh` from the repo root. Don't create the worktree with plain `git worktree add`, because the new checkout would then use the same ports as the main one.

## Steps

1. Decide on a branch name. If the user didn't give one, ask, or derive a short kebab-case name from the task.
2. Choose a path. By default, put it next to the main checkout: `../midnight-machinations-<branch>`, with any `/` in the branch name replaced by `-`.
3. Run the script from the main checkout's root:
   ```bash
   scripts/new-worktree.sh <path> <branch> [port-offset]
   ```
   - If `<branch>` exists, the script checks it out. Otherwise it creates it from the current HEAD.
   - Leave out `port-offset` unless the user asks for specific ports or the default clashes with something. The default is the number of existing worktrees, so the first extra worktree gets client 3001 and server 9001.
   - It writes `client/.env.local` (`VITE_WS_ADDRESS`, `CLIENT_PORT`) and `server/.env.local` (`WS_ADDRESS`), then runs `pnpm install`. This can take a minute.
4. Tell the user the worktree path and the client and server ports the script printed.

## Notes

- Both `.env.local` files are gitignored and take precedence over the committed `.env` files. To change a worktree's ports later, edit those files. Keep the client's `VITE_WS_ADDRESS` port the same as the server's `WS_ADDRESS` port.
- Each worktree has its own `server/target`, so the first `cargo build` in it starts from scratch.
- To remove a worktree: `git worktree remove <path>`, then `git branch -d <branch>` if the branch isn't needed anymore. Ask before deleting a branch that has unmerged commits.
