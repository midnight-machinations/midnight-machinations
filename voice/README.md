# Voice service

Audio-only voice chat. A Cloudflare Worker (`src/index.ts`) relays signaling and steers the [Cloudflare Realtime SFU](https://developers.cloudflare.com/realtime/sfu/). The Rust game server decides who may hear whom, so no media server needs to be hosted.

```
browser ── game socket ──▶ Rust server ── /control (secret) ──▶ voice worker ── HTTPS ──▶ Realtime SFU
   │                         (voice token)                         ▲                        ▲
   └──────────── voice socket (token) ─────────────────────────────┘                        │
   └──────────── WebRTC audio (one RTCPeerConnection) ──────────────────────────────────────┘
```

- `src/index.ts`: the Worker and its single `Voice` Durable Object. It holds the routing policy and every peer's SFU session.
- `src/routing.ts`: the forwarding rule. A speaker's single send group must be one the listener receives and hasn't deafened.
- `src/sfu.ts`: a minimal Realtime SFU API client. The app token stays in the Worker.

The wrangler config is `deploy/voice/wrangler.json`; see `deploy/voice/README.md` for deployment.

## Local setup

1. In the Cloudflare dashboard, open **Realtime → SFU** and create an app. The free plan includes it. Copy the app ID and API token.
2. From the repository root:

   ```sh
   mask setup             # installs dependencies and runs mask voice configure
   ```

   Then fill in `REALTIME_APP_ID` and `REALTIME_APP_TOKEN` in the gitignored `voice/.dev.vars`. `mask voice configure` also wrote a shared random `VOICE_CONTROL_SECRET` there and the matching `VOICE_CONTROL_SECRET`, `VOICE_CONTROL_URL`, and `VOICE_PUBLIC_URL` into `server/.env.local`.
3. `mask start` runs everything. To run voice on its own, use `mask voice start`, which runs `wrangler dev` on port 9100 (or `9100 + offset`).

`mask worktree` configures voice with the worktree's port offset. Media goes from browsers straight to Cloudflare, so no UDP ports need to be published, including in the dev container.

If any of the server's three voice variables are missing, voice stays disabled and the game runs normally.

## Protocol

Control link (`/control`, `Authorization: Bearer <VOICE_CONTROL_SECRET>`), game server → voice:

```json
{ "version": 3, "rooms": [{ "scope": "<random>", "members": [{ "id": 0, "token": "<random>", "send": ["all"], "receive": ["all", "dead"] }] }] }
```

The game server sends a snapshot whenever permissions change, and once on connect. Voice replies with `{ "presence": { "<scope>": [0, 2] } }` whenever someone joins or leaves. Only one game server is the authority at a time. A new control connection replaces the old one, and losing the link closes every voice session.

Browser socket (`/`): requests carry an `id`, and replies are `{ id, data }` or `{ id, error }`.

| Request | Reply |
| --- | --- |
| `{ type: "join", token, deafened }` | `{ iceServers }` |
| `{ type: "publish", sdp, mid }`: offer with one send-only audio transceiver | `{ sdp }` answer |
| `{ type: "preferences", deafened }` | `{}` once routes are updated |

When speakers become audible, voice sends `{ type: "offer", sdp, tracks: [{ mid, peer }] }`, and the browser replies with `{ type: "answer", sdp }`.

## Permission and privacy behavior

Rust computes the send and receive sets with the same calculations used by text chat. It applies a separate voice-only forced-silence override for Cerenovous and Blackmailer effects. Only each player's own permissions go to that player's game socket. Public roster data contains no voice availability or mute or deafen state. All room mutations go through `with_voice_update`, which recomputes permissions after every action and publishes the snapshot if it changed.

Every browser publishes exactly one track, even while listen-only or forbidden to speak, so publishing reveals nothing. Browsers can only receive tracks that the Worker pulls into their SFU session. The SFU token never leaves the Worker. On each change, the Worker revokes obsolete routes before granting new ones, per listener.

- **Revocation** force-closes the pulled track at the SFU without renegotiating. The listener gets silence and no message.
- **Granting** pulls the speaker's track, which renegotiates with the listener. The offer names only the speaker's id, with no group, mute, or permission metadata. A listener can see that a speaker became audible to them, as they can with audible speech. If the same speaker is granted again after a revocation, they appear as a new track, so a modified client could notice that their route had been revoked earlier.
- Changes take effect once the SFU applies them, usually within a few hundred milliseconds. There is no barrier that holds game actions until media is rerouted, so audio already in flight during a phase change can still arrive.

Group deafen changes mute local playback until the Worker acknowledges them. Brained and blackmailed players cannot transmit, whatever their settings. Spectators can speak in the lobby, but during a game they can only hear All.

## Client behavior

Join voice from the lobby header or the voice column in **Settings**. Joining asks for microphone permission and starts muted. If you decline, you join listen-only, and clicking your own microphone icon asks again. Input modes are open microphone (the default), push-to-talk, and toggle mute; push-to-talk and toggle mute use V by default. Jitsi RNNoise noise suppression and an optional threshold gate run locally in a 48 kHz AudioWorklet before transmission. Only the processed track is sent, and if processing fails the microphone stops rather than sending unfiltered audio. The license notice is shipped at `/licenses/rnnoise.txt`.

Click another player's microphone icon to mute them locally. Per-player volume and per-group deafen live in the voice column. Speaking indicators use audio arriving at your browser, so they don't reveal speech in private groups. Hosts can turn voice on or off for the whole room. This is a room setting, never saved in game modes, and turning it off revokes every voice credential.

## Checks

```sh
mask voice test   # tsc and routing tests
```

Real microphones, browser autoplay, restrictive networks (configure TURN through `VOICE_ICE_SERVERS`), and phase changes during speech still need a multiplayer acceptance pass.
