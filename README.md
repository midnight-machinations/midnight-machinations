# Midnight Machinations
Multiplayer social deduction game.

## Getting started

```bash
git clone https://www.github.com/midnight-machinations/midnight-machinations
cd midnight-machinations
```

Install Node.js 22+, [Rust](https://www.rust-lang.org/learn/get-started), and [mask](https://github.com/jacobdeichert/mask) (`cargo install mask`). Then, from the repository root:

```bash
mask setup   # install dependencies, configure local voice
mask start   # start client, server, and voice together
```

Every development task lives in [`maskfile.md`](maskfile.md); `mask --help` lists them. Each service can also be run on its own (`mask client dev`, `mask server run`, `mask voice start`).

In the dev container, forward ports 3000, 9000, and 9100. Voice media also needs the ports described in [the voice guide](voice/README.md#local-setup).

Note: if the server build fails on Linux or WSL, you may need to install OpenSSL first.

## Editor setup

For VS Code these extensions help:
 - [Rust Analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
 - [Even Better Toml](https://marketplace.visualstudio.com/items?itemName=tamasfe.even-better-toml)
 - [Dependi](https://marketplace.visualstudio.com/items?itemName=fill-labs.dependi)
 - [Error Lens](https://marketplace.visualstudio.com/items?itemName=usernamehw.errorlens)
 - [GitLens](https://marketplace.visualstudio.com/items?itemName=eamodio.gitlens)
 - [Spell checker](https://marketplace.visualstudio.com/items?itemName=streetsidesoftware.code-spell-checker)

You can make clippy rust-analyzer's default linter with `"rust-analyzer.check.command": "clippy"`.

## Voice chat

See [voice/README.md](voice/README.md).
