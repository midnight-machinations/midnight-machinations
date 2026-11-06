---
name: deep-review
description: Deep review of a pull request that also fixes what it finds. Checks out the PR in its own worktree, maps the changes, reviews each part for correctness, trust-boundary, protocol and test problems, confirms each finding with a test or a concrete trace, then commits fixes and regression tests to a separate branch and writes up a report. Use when asked to deep-review, thoroughly review, harden, or "review and fix" a PR.
---

# Deep review

A deep review finds real defects, proves them, and fixes the ones that are safe to fix. Style nits are not the goal. Each finding must be either **confirmed** (by a failing test, a reproduction, or a step-by-step trace through the code) or labeled **plausible**, with what would confirm it. Drop anything you can't back up.

## Ground rules

- Work in a separate worktree. Don't touch the user's checkout.
- Don't push, comment on the PR, or request changes without asking. Fixes go on a local branch named `review/pr-<n>`, made from the PR head.
- Fix bugs. Don't redesign. If a fix would change how a feature behaves or is designed, write it up as a question for the author. In this game, that includes fixes that change what a role can see or do, even when the change looks like an information leak: confirm with the user before fixing, because it may be intended.
- Commit each fix on its own, with its regression test, so the author can cherry-pick it or drop it.

## 1. Recon

```bash
gh pr view <n> --json title,body,author,baseRefName,headRefName,commits,files,additions,deletions,statusCheckRollup,reviews,comments
gh pr diff <n> --name-only
git fetch origin
```

- Note the base branch. If it isn't `main` (for example a `main` → `prod` release PR), CI may not have run. The workflows in `.github/workflows/checks-*.yml` only trigger on PRs into `main`, and the root `package.json` tests aren't in CI at all. Don't take a green check as evidence.
- Read the linked issues, the PR description, and any existing review comments, so you don't report what has already been raised.
- Use `git diff origin/<base>...<head>` (three dots) for the PR's own changes.

## 2. Set up

Create the worktree with `mask worktree <path> review/pr-<n>`, after making `review/pr-<n>` at the PR head (`git branch --no-track review/pr-<n> origin/<head>`). Without `--no-track`, the branch tracks the PR's head branch, and a bare `git push` would push the fixes straight to it. The worktree gets its own ports, so the app can run there.

Run every check at the PR head before reading any code, and save the output in the scratchpad:

- `server/`: `cargo build`, `cargo test`, `cargo clippy --all-targets`
- `client/`: `pnpm install --frozen-lockfile`, `pnpm build`, `pnpm test --if-present`
- root: `pnpm test`, if the root `package.json` has a test script

If something fails, check whether it also fails on the base branch, and note whether the PR caused it.

## 3. Map the change

For a large or badly split PR, read through the whole diff first and group the files into **change units**. Each unit is one feature or fix, together with every file it touches on the server, the client and in tests. Write the map to the scratchpad as a table with these columns: unit, files, what it seems to be for, and risk (high/med/low).

Rate as high-risk anything that:
- handles packets from clients (`on_client_message.rs`, `lobby/on_client_message.rs`, `controller_selection`, voice signaling)
- decides what a player is allowed to see (chat groups, spectators, possession, role reveal, `send_to`/`chat_messages` visibility)
- changes the wire protocol (`packet.rs` ↔ `client/src/game/packet.tsx`, `gameState.d.tsx`, `messageListener.tsx`)
- deletes code. List every removed feature or behavior and check that it was meant to go.
- changes build, deploy or dependency files (dockerfiles, lockfiles, `package.json`, workflows)

Review the high-risk units first.

## 4. Review each unit

For each unit, read the full functions involved, not just the diff hunks, along with their callers. Then work through these checks:

1. **Intent vs. behavior.** Does the code do what the PR or unit says? Follow it from the user's action → client → packet → server handler → state change → what gets sent back to each client.
2. **Trust boundary.** The server must not trust the client. Check whether a non-host can trigger something only the host should (skip phase, kick, settings), whether a dead player, spectator, or player in the wrong phase can do it, whether indexes, IDs and player refs are range-checked, and whether messages can be spammed without limit.
3. **Information leaks.** In a social deduction game, sending hidden information to the wrong player breaks the game. Check every new or changed field sent to clients: could it reveal a role, a night action, private chat, or the identities of spectators or players before it should?
4. **Protocol parity.** Each serde enum or struct change in Rust needs the matching TS type and handler. Check the `#[serde(rename_all…)]` / `tag` attributes against the TS field names. A mismatch fails silently at runtime and `tsc` won't catch it.
5. **State and lifecycle.** Check the state at phase changes, game end, reconnects, a host leaving, a player leaving mid-action, rejoining from the lobby, and empty or single-player games. On the client, check React effects for missing cleanup, stale closures, listeners added on every render, and keyboard handlers that still fire while typing in inputs.
6. **Edge cases.** Look at empty lists, `None`, off-by-one errors, integer overflow and underflow, `unwrap`/`expect` on data from clients (one bad packet could crash the server), and `as` casts.
7. **i18n.** Each new key needs to exist in `en_us.json`. Look for keys that were removed but are still used, and for keys that are used but never added: grep `translate("…")`. Follow the existing practice for the joke languages (`dyslexic.json`, `broken_keyboard.json`).
8. **Tests.** Check whether the tests exercise the real code or a copy of it. Watch for `.cjs` tests that reimplement the logic, or tests that only match source text with a regex. Would each test fail if the bug it guards against came back? List any high-risk behavior that has no tests.
9. **Repo conventions.** Follow `.github/copilot-instructions.md` (the checklists for roles and chat messages) and match the style of the surrounding code.
10. **Build, deploy and dependencies.** Look for removed lockfiles, a package manager switch that the Dockerfiles, CI and docs didn't all follow, `.dockerignore` rules that exclude files the build needs, and new dependencies that aren't needed.

Write down each candidate finding as you go. For each one, give the file and line, the claim, and the concrete scenario where it causes a failure.

## 5. Verify

For each candidate finding:
- If you can, write a failing test: a Rust integration test in `server/tests/` using the helpers in existing tests, or a client test. Confirm that it fails at the PR head.
- If a test isn't practical (UI or timing), trace the exact path through the code with file:line references. If it matters enough, run the app (`run` skill) and reproduce the bug.
- Mark the finding CONFIRMED or PLAUSIBLE, or drop it. Rank what's left by severity: **blocker** (crash, exploit, info leak, data loss, broken build or deploy), **major** (feature wrong in common cases), **minor** (edge case, small regression), **nit** (only report a nit if it's cheap and certain).

## 6. Fix and harden

On `review/pr-<n>`, for each confirmed blocker, major and safe minor finding:
1. Add the regression test, or keep the one from step 5.
2. Make the smallest fix that matches the surrounding code.
3. Run the checks for that area of the code, then commit: `fix(<area>): <what>` with a body that explains the bug scenario.

Then harden the PR:
- Add tests for high-risk behavior that doesn't have any, especially server-side permission and visibility checks.
- Replace tests that copy the logic or only match source text with tests that call the real code, when that can be done cheaply.

Finally, run all of the step 2 checks again and confirm they pass, then compare with the baseline. Leave PLAUSIBLE findings and design questions unfixed and put them in the report.

## 7. Report

Write the report as a page and publish it (it is meant for the PR author), then give a short summary in the terminal. Include:

- **Verdict:** whether the PR is ready to merge as is, with the fixes, or needs the author's work first.
- **Change map:** the table from step 3.
- **Findings,** ranked: severity, CONFIRMED/PLAUSIBLE, file:line, scenario, and either the commit hash of the fix or "left for author".
- **Fixes and tests added,** with commit hashes on `review/pr-<n>`, and how to apply them (`git cherry-pick` or merge the branch).
- **Questions for the author:** design choices and intent you couldn't work out.
- **Check results:** the baseline and the results after the fixes.

Then ask the user whether to push the branch, open a PR into the PR's head branch, or post the findings as review comments. Before pushing to the head branch, fetch it and check that it hasn't moved. If it has, rebase the fixes onto it and run the checks again. Always push with an explicit refspec (`git push origin review/pr-<n>:<branch>`), never with force.
