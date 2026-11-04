# Mafia
Multiplayer Social Deduction game

## Getting Started
First, download and enter the [git](https://git-scm.com/) repository:
```bash
git clone https://www.github.com/mafia-rust/mafia
cd mafia
```
## Start all development services

Use Node.js 22 or newer, Rust with Cargo, and pnpm. The devcontainer uses Node.js 22. If pnpm is already available, skip `corepack enable`. Run these setup commands once from the repository root:

```sh
corepack enable
cd client && pnpm install --frozen-lockfile
cd ../voice && pnpm install --frozen-lockfile
pnpm configure
cd ..
```

Then start the client, game server, and voice service together from the repository root:

```sh
pnpm start
```

Press **Ctrl+C** to stop all three services and their child processes. Output is labeled by service. If one fails or exits, the launcher stops the others, preserves the error output, and prints commands for running them separately. Missing dependencies or voice settings are reported with setup commands, without installing or changing configuration automatically. Rust may take time to compile on the first run.

The launcher uses Node's built-in modules, so there is no root dependency installation. Vite binds to all interfaces by default so the client is reachable outside the devcontainer. Additional arguments are passed to Vite. It disables browser auto-opening and makes Vite fail on an occupied port instead of silently choosing another port. Use `pnpm start --host localhost` to restrict Vite to local connections. Existing `.env.local` port settings still apply.

In the dev container, forward ports 3000, 9000, and 9100. WebRTC media also needs the published UDP/TCP ports and a browser-reachable announced address described in [the voice setup guide](voice/README.md#local-setup). Voice media configuration is a separate setup step.

The combined launcher is optional. Each service still runs independently in its own terminal:

```sh
# Client
cd client && pnpm dev --host --no-open
# Game server
cd server && cargo run
# Voice service
cd voice && pnpm start
```

Use **pnpm for all JavaScript dependencies**, pinned to the same version in the root, client, and voice packages. Client and voice retain separate lockfiles so voice can build independently in Docker. Cargo manages the Rust server. Start the launcher with `pnpm start`. Do not use npm to install dependencies in this repository.

To check the launcher itself, run `pnpm test` from the repository root.

## Client setup
The client uses [Vite](https://vite.dev/) as the build tool and [pnpm](https://pnpm.io/) for package management (pnpm offers better performance and more efficient disk space usage compared to npm).

First, enable pnpm via corepack:
```bash
corepack enable
```

Then enter the client directory, install dependencies, and start the dev server:
```bash
cd client
pnpm install
pnpm dev
```

Alternatively, you can use `pnpm start` which is an alias for `pnpm dev`.

To build for production:
```bash
pnpm build
```
## Server setup
### Install Rust
Follow the [tutorial](https://www.rust-lang.org/learn/get-started) on the rust website.
### VScode
If you're using VSCode, it's recommended to download the following extensions to make working on the project easier:
 - [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer) - You probably already have this. You definitely need it.
 - [Even Better Toml](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml) - Language support for .TOML files
 - [Dependi](https://marketplace.visualstudio.com/items?itemName=fill-labs.dependi) - Helps manage crate versions
 - [Error Lens](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens) - Show error messages inline
 - [GitLens](https://marketplace.visualstudio.com/items?itemName=eamodio.gitlens) - View git blame inline
 - [Spell checker](https://marketplace.visualstudio.com/items?itemName=streetsidesoftware.code-spell-checker) - Spelling corrections

It's also a good idea to install clippy (a linter):
```bash
rustup component add clippy
```
You can make it the default linter using this setting (but you don't need to):
```json
"rust-analyzer.check.command": "clippy",
```

### Starting the server
Enter the server directory and build the project using cargo.
```bash
cd server
cargo build
```
Note: If the above step fails, and you are using Linux or WSL, you may need to install OpenSSL first.

You can now start the server backend:
```bash
cargo run
```

### Production Enviornment
#### Install
We have built an install script that automatically pulls all the dependencies.
Run the following command as the root user
```bash
curl -fSsL https://raw.githubusercontent.com/mafia-rust/mafia/main/system/install.sh | sh
```

#### Update
```bash
./mafia/system/update.sh
```

## Voice chat

See [voice/README.md](voice/README.md) to configure and run the optional WebRTC voice service.
