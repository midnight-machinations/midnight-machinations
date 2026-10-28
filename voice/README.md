# Voice service

Audio-only WebRTC through mediasoup, controlled by the Rust game server. Requires Node.js 22 or newer. The service starts three media workers and assigns each game a router. This is intended for a small deployment of up to three games with 30 players each. Capacity and browser interoperability have not been load-tested.

## Local setup

The dev container includes Python with pip and the C/C++ build tools needed when mediasoup's prebuilt worker cannot run. After changing the dev-container configuration, use **Dev Containers: Rebuild Container** in VS Code. The first source build during `npm ci` can take several minutes.

To repair an already-running Ubuntu dev container without rebuilding its image:

```sh
sudo apt-get update
sudo apt-get install -y build-essential python3-pip python3-venv
```

From `voice/`:

```sh
npm ci
npm run configure
npm start
```

`configure` creates a random shared control secret in gitignored `voice/.env.local` and `server/.env.local`. It preserves unrelated server settings. Restart the Rust server after configuring it. Run the server and client normally from their own directories:

```sh
# server/
cargo run
# client/
pnpm dev --no-open
```

The dev container publishes media ports 44400–44402 over UDP and TCP. VS Code forwards the web and signaling ports separately. Media binds to `0.0.0.0`, and `configure` announces a non-loopback interface address. On Linux a browser on the Docker host can reach the container directly at that address. Firefox rejects loopback media candidates, so `127.0.0.1` must not be used as the announced media address. The game page and signaling URL can still use localhost.

For Docker Desktop or a remote Docker host, explicitly announce the Docker host's IPv4 address that your browser can reach:

```sh
npm run configure -- --announced-address 192.168.1.100
# An offset can be combined with the explicit address:
npm run configure -- 1 --announced-address 192.168.1.100
```

Re-run `npm run configure` after changing container networks. A container rebuild is required to apply changed published-port mappings. With a port offset, update those mappings to match.

Open two browser profiles or devices, join a lobby, click **Join voice** in the lobby header, or open the top-right hamburger menu and choose **Settings**. Voice controls have their own column there. Listening does not request microphone permission. Click your microphone icon to enable the microphone. If the browser blocks audio autoplay, use **Enable audio playback** in the voice column; this does not reconnect the voice session. The default input mode is push-to-talk with V. Choose push-to-talk, open microphone, or toggle mute in the voice column of Settings. Toggle mute starts muted and switches with its configured key or your own mic button. Microphone selection, key bindings, and voice volume are in that same column. The hold-to-talk button supports touch input.

The lobby and game player lists show a persistent mic button for each player while you are connected to voice. Its icon changes between idle, speaking, and locally muted. Click another player's mic to mute or unmute their voice locally; click your own mic to mute or unmute your microphone. Remote speaking indicators use audio arriving at your browser and respect your playback mute and volume settings. They do not reveal speech in private groups or anyone else's permission or mute state.

Each group occupies one row with independent outgoing mute and incoming deafen icon buttons. Individual player mute and volume controls affect only your playback. Group settings survive reconnects within the same lobby or game. A fresh game resets them. Deafening one group still allows a speaker to be heard through another enabled group. Brained and blackmailed players cannot transmit, regardless of their settings. Spectators can speak in the lobby, but can only hear All during a game.

For simultaneous checkouts, use `npm run configure -- 1` with a different offset per checkout. This configures voice ports independently of the existing game/client worktree ports.

Hosts can enable or disable voice for the whole room in lobby settings or in-game host settings. The toggle is a room setting, is absent from the game mode editor, and is never included in saved client game modes. The server checks host authority, preserves the setting across lobby/game transitions, revokes media credentials and routes when disabled, and updates all clients. Enabling voice again lets players join it explicitly; it does not turn on their microphones.

## Deployment

1. Build the service with `docker build -t midnight-voice voice/`, or install its locked dependencies with Node.js 22+ and `npm ci`.
2. Set the same nonempty `VOICE_CONTROL_SECRET` for Rust and the service. Keep it out of the client build and source control.
3. Set Rust's `VOICE_CONTROL_URL` to the private service address, such as `ws://voice:9101/control`. Set `VOICE_PUBLIC_URL` to the browser-facing `wss://` signaling URL.
4. Set service `VOICE_LISTEN_ADDRESS=0.0.0.0`, `VOICE_MEDIA_ADDRESS=0.0.0.0`, and `VOICE_ANNOUNCED_ADDRESS` to its reachable public IP. When using containers, set `VOICE_CONTROL_ADDRESS=0.0.0.0` on the private container network. Never publish port 9101 to the internet.
5. Proxy the public signaling port 9100 with WebSocket upgrades and TLS. Serve the game client over HTTPS and its game socket over WSS. Microphone access works on localhost during local development.
6. Forward ports 44400–44402 over both UDP and TCP directly to the service. These media ports do not use the HTTP reverse proxy. Set `VOICE_MEDIA_PORT` to change the first port.
7. Set `VOICE_ALLOWED_ORIGINS` to a comma-separated list of exact frontend origins. Configure `VOICE_ICE_SERVERS` with browser-compatible ICE server JSON if TURN is needed for restrictive networks. These credentials are shared with authenticated browsers, so use limited credentials and rotate them as appropriate. No TURN server is bundled.

If any of Rust's three voice environment variables are absent, voice stays disabled and the game continues normally. A configured but unavailable service produces a reconnect message in the voice column.

## Permission and privacy behavior

Rust computes private send and receive sets from the same calculations used by text chat. It applies a separate voice-only forced-silence override for Cerenovous and Blackmailer effects. Only each player's own permissions go to that player's game socket. Public roster data contains no voice availability or mute/deafen state.

The listener wraps all mutating actions in `with_voice_update`. It recomputes permissions after every action, including mid-phase controller/role effects and silence changes; it does not depend on phase-change notifications. Future timer, admin, or external event handlers must use the same wrapper. Before game mutations, connects, disconnects, and ticks, Rust asks the SFU to pause forwarding and waits for acknowledgement. It then sends the new permission snapshot to resume authorized routes. This prevents an old permission snapshot from remaining active during a phase or role change. The control connection also sends one-second heartbeats. The SFU applies route revocations before grants. A listener receives one copy of a speaker's stream when at least one group allows it. Client requests cannot create or resume unauthorized routes. Group preference changes temporarily stop local transmission and playback until the service acknowledges them.

Consumers remain established while routing permissions change. The SFU pauses forwarding privately and never broadcasts producer mute, permission, group membership, or subscription-denied events. Muting a microphone does not send a mute status to other players. Network traffic and audible silence can still support guesses; the application does not report the reason.

Loss of the control socket closes media routers immediately. If a forwarding acknowledgement fails, a game operation may briefly wait for the old media lease to expire before proceeding. A stalled control connection expires after approximately 3.5–4 seconds. Reconnecting game sockets receive new random voice credentials and replace old sessions. Room and game changes also rotate credentials. Routing changes take effect when the media service applies the control snapshot; audio already in transit or buffered cannot be recalled.

## Checks

```sh
npm run check
# client/
pnpm build
# server/
cargo clippy
```

Automated tests and load testing are deferred. Real microphones, browser autoplay behavior, restrictive networks, and phase changes during speech still need a multiplayer acceptance pass.
