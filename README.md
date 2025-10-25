# Mafia
Multiplayer Social Deduction game

## Getting Started
First, download and enter the [git](https://git-scm.com/) repository:
```bash
git clone https://www.github.com/mafia-rust/mafia
cd mafia
```
From here it's recommended to split terminals (If you're using VSCode), or open up a second terminal - one for client and one for server.
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

### Bot Players (Optional)
The server supports LLM-powered bot players that can join games. To enable this feature, you need to set up an OpenAI API key:

```bash
export OPENAI_API_KEY="your-api-key-here"
```

Or add it to a `.env` file in the server directory:
```
OPENAI_API_KEY=your-api-key-here
```

Once configured, hosts can add bot players from the lobby using the "Add Bot" button. Bots will use ChatGPT to make decisions during the game.

For more information about the bot system, see [server/src/game/bot/README.md](server/src/game/bot/README.md).

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
