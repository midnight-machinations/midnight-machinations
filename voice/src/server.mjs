import http from 'node:http';
import { timingSafeEqual } from 'node:crypto';
import { createWorker } from 'mediasoup';
import { WebSocketServer, WebSocket } from 'ws';

const secret = process.env.VOICE_CONTROL_SECRET;
if (!secret) throw new Error('VOICE_CONTROL_SECRET is required');
const host = process.env.VOICE_LISTEN_ADDRESS ?? '127.0.0.1';
const mediaHost = process.env.VOICE_MEDIA_ADDRESS ?? '127.0.0.1';
const announcedAddress = process.env.VOICE_ANNOUNCED_ADDRESS ?? '127.0.0.1';
const basePort = Number(process.env.VOICE_MEDIA_PORT ?? 44400);
const workers = [];
for (let index = 0; index < 3; index++) {
  const worker = await createWorker({ logLevel: 'warn' });
  worker.on('died', () => process.exit(1));
  const webRtcServer = await worker.createWebRtcServer({ listenInfos: [
    { protocol: 'udp', ip: mediaHost, announcedAddress, port: basePort + index },
    { protocol: 'tcp', ip: mediaHost, announcedAddress, port: basePort + index },
  ] });
  workers.push({ worker, webRtcServer });
}
const rooms = new Map();
let policies = new Map();
let control;
let lastHeartbeat = 0;
let version = 0;
let frozen = false;
let nextWorker = 0;
// All worker mutations and signaling requests share an ordered queue.
let pending = Promise.resolve();
function enqueue(action) {
  const result = pending.then(action);
  pending = result.catch(error => console.error('Voice operation failed:', error.message));
  return result;
}
function send(socket, data) {
  if (socket.readyState === WebSocket.OPEN) socket.send(JSON.stringify(data));
}
function healthy() { return control?.readyState === WebSocket.OPEN && Date.now() - lastHeartbeat < 3500; }
function allowed(listener, speaker) {
  if (frozen || !healthy() || listener.closed || speaker.closed || listener === speaker) return false;
  const from = policies.get(speaker.token);
  const to = policies.get(listener.token);
  return from && to && from.scope === to.scope && from.send.some(group =>
    to.receive.includes(group) && !speaker.muted.has(group) && !listener.deafened.has(group));
}
function closePeer(peer) {
  if (peer.closed) return;
  peer.closed = true;
  if (peer.room.peers.get(peer.id) === peer) peer.room.peers.delete(peer.id);
  for (const other of peer.room.peers.values()) {
    const entry = other.consumers.get(peer.id);
    if (entry) {
      entry.consumer.close();
      other.consumers.delete(peer.id);
      send(other.socket, { type: 'consumerClosed', id: entry.consumer.id });
    }
  }
  for (const transport of peer.transports.values()) transport.close();
  peer.socket.close(1000, 'Voice session ended');
}
function failClosed() {
  policies = new Map();
  for (const room of rooms.values()) {
    // Closing the router stops forwarding without waiting for browser cooperation.
    room.router.close();
    for (const peer of [...room.peers.values()]) closePeer(peer);
  }
  rooms.clear();
}
async function reconcile(room) {
  // Revoke all obsolete routes before granting any new ones.
  const pauses = [];
  const resumes = [];
  for (const listener of room.peers.values()) {
    for (const [speakerId, entry] of listener.consumers) {
      const speaker = room.peers.get(speakerId);
      if (!entry.ready || !speaker || !allowed(listener, speaker)) {
        if (!entry.consumer.paused) pauses.push(entry.consumer.pause());
      } else if (entry.consumer.paused) resumes.push(entry.consumer);
    }
  }
  await Promise.all(pauses);
  await Promise.all(resumes.filter(consumer => !consumer.closed && healthy()).map(consumer => consumer.resume()));
}
async function addConsumers(room) {
  for (const listener of room.peers.values()) {
    if (!listener.recv || !listener.capabilities) continue;
    for (const speaker of room.peers.values()) {
      if (speaker === listener || !speaker.producer || listener.consumers.has(speaker.id)) continue;
      if (!room.router.canConsume({ producerId: speaker.producer.id, rtpCapabilities: listener.capabilities })) continue;
      const consumer = await listener.recv.consume({ producerId: speaker.producer.id,
        rtpCapabilities: listener.capabilities, paused: true });
      listener.consumers.set(speaker.id, { consumer, ready: false });
      // No group, mute, permission, or producer-paused metadata goes to listeners.
      send(listener.socket, { type: 'consumer', id: consumer.id, producerId: speaker.producer.id,
        kind: consumer.kind, rtpParameters: consumer.rtpParameters, sourceId: speaker.id });
    }
  }
}
async function applyPolicy(message) {
  if (!Number.isSafeInteger(message.version) || message.version <= version || !Array.isArray(message.rooms)) return;
  version = message.version;
  const next = new Map();
  for (const room of message.rooms) for (const member of room.members) {
    next.set(member.token, { ...member, scope: room.scope });
  }
  policies = next;
  frozen = false;
  for (const [scope, room] of rooms) {
    for (const peer of [...room.peers.values()]) {
      const policy = policies.get(peer.token);
      if (!policy || policy.scope !== scope || policy.id !== peer.id) closePeer(peer);
    }
    await reconcile(room);
    if (![...policies.values()].some(policy => policy.scope === scope)) {
      room.router.close();
      rooms.delete(scope);
    }
  }
}
async function getRoom(scope) {
  if (rooms.has(scope)) return rooms.get(scope);
  const worker = workers[nextWorker++ % workers.length];
  const router = await worker.worker.createRouter({ mediaCodecs: [
    { kind: 'audio', mimeType: 'audio/opus', clockRate: 48000, channels: 2 },
  ] });
  const room = { router, webRtcServer: worker.webRtcServer, peers: new Map() };
  rooms.set(scope, room);
  return room;
}
function groupSet(value) {
  if (!Array.isArray(value) || value.length > 16 || value.some(group => typeof group !== 'string' || group.length > 32)) {
    throw new Error('Invalid group settings');
  }
  return new Set(value);
}
async function handle(socket, state, request) {
  if (!healthy()) throw new Error('Voice unavailable');
  if (request.action === 'join') {
    if (state.peer || typeof request.token !== 'string') throw new Error('Invalid session');
    const policy = policies.get(request.token);
    if (!policy) throw new Error('Invalid session');
    const room = await getRoom(policy.scope);
    const previous = room.peers.get(policy.id);
    if (previous) closePeer(previous);
    const peer = { id: policy.id, token: request.token, socket, room, closed: false,
      muted: groupSet(request.muted ?? []), deafened: groupSet(request.deafened ?? []),
      transports: new Map(), consumers: new Map() };
    state.peer = peer;
    room.peers.set(peer.id, peer);
    return { routerRtpCapabilities: room.router.rtpCapabilities, iceServers: JSON.parse(process.env.VOICE_ICE_SERVERS ?? '[]') };
  }
  const peer = state.peer;
  if (!peer || peer.closed || !policies.has(peer.token)) throw new Error('Invalid session');
  const room = peer.room;
  switch (request.action) {
    case 'preferences':
      peer.muted = groupSet(request.muted);
      peer.deafened = groupSet(request.deafened);
      await reconcile(room);
      return {};
    case 'createTransport': {
      if (!['send', 'recv'].includes(request.direction) || peer[request.direction]) throw new Error('Invalid transport');
      const transport = await room.router.createWebRtcTransport({ webRtcServer: room.webRtcServer,
        enableUdp: true, enableTcp: true, preferUdp: true, initialAvailableOutgoingBitrate: 600000 });
      peer.transports.set(transport.id, transport);
      peer[request.direction] = transport;
      transport.on('dtlsstatechange', value => { if (value === 'closed') socket.close(); });
      return { id: transport.id, iceParameters: transport.iceParameters,
        iceCandidates: transport.iceCandidates, dtlsParameters: transport.dtlsParameters };
    }
    case 'connectTransport': {
      const transport = peer.transports.get(request.transportId);
      if (!transport) throw new Error('Invalid transport');
      await transport.connect({ dtlsParameters: request.dtlsParameters });
      return {};
    }
    case 'produce': {
      if (!peer.send || peer.producer || request.kind !== 'audio') throw new Error('Invalid producer');
      // Keep one stable producer even while game permissions forbid speaking.
      // Only private, server-owned consumers determine where its audio can go.
      peer.producer = await peer.send.produce({ kind: 'audio', rtpParameters: request.rtpParameters });
      await addConsumers(room);
      return { id: peer.producer.id };
    }
    case 'receive':
      peer.capabilities = request.rtpCapabilities;
      await addConsumers(room);
      return {};
    case 'consumerReady': {
      const entry = [...peer.consumers.values()].find(entry => entry.consumer.id === request.consumerId);
      if (!entry) throw new Error('Invalid consumer');
      entry.ready = true;
      await reconcile(room);
      return {};
    }
    default: throw new Error('Unknown request');
  }
}
const publicServer = http.createServer((_, response) => {
  response.writeHead(200, { 'content-type': 'text/plain' });
  response.end(healthy() ? 'voice ready\n' : 'waiting for game server\n');
});
const publicSockets = new WebSocketServer({ server: publicServer, maxPayload: 65536 });
publicSockets.on('connection', (socket, request) => {
  const origins = process.env.VOICE_ALLOWED_ORIGINS?.split(',');
  if (origins && !origins.includes(request.headers.origin)) { socket.close(1008); return; }
  const state = {};
  let outstanding = 0;
  let windowStart = Date.now();
  let requests = 0;
  const authTimeout = setTimeout(() => { if (!state.peer) socket.close(1008); }, 10000);
  socket.on('message', raw => {
    if (Date.now() - windowStart >= 1000) { windowStart = Date.now(); requests = 0; }
    // A room can initialize 30 consumers at once; bound abuse without blocking that burst.
    if (++requests > 150 || ++outstanding > 150) { socket.close(1008); return; }
    let message;
    try { message = JSON.parse(raw.toString()); } catch { socket.close(1008); return; }
    if (!Number.isSafeInteger(message.id)) { socket.close(1008); return; }
    enqueue(async () => {
      if (socket.readyState !== WebSocket.OPEN) return;
      try { send(socket, { id: message.id, data: await handle(socket, state, message) }); }
      catch (error) {
        console.warn(`Voice ${message.action} request failed:`, error.message);
        send(socket, { id: message.id, error: 'Voice request failed. Reconnect to retry.' });
      }
    }).finally(() => { outstanding--; });
  });
  socket.on('close', () => {
    clearTimeout(authTimeout);
    enqueue(() => { if (state.peer) closePeer(state.peer); });
  });
  socket.on('error', () => socket.close());
});
const controlServer = http.createServer();
const controlSockets = new WebSocketServer({ server: controlServer, path: '/control', maxPayload: 2 * 1024 * 1024,
  verifyClient: ({ req }, done) => {
    const supplied = Buffer.from(req.headers.authorization ?? '');
    const expected = Buffer.from(`Bearer ${secret}`);
    done(supplied.length === expected.length && timingSafeEqual(supplied, expected), 401);
  },
});
controlSockets.on('connection', socket => {
  // There is one game-server authority. Never merge policies from two authorities.
  if (control?.readyState === WebSocket.OPEN) { socket.close(1008); return; }
  failClosed();
  control = socket;
  version = 0;
  lastHeartbeat = 0;
  socket.on('message', raw => {
    let message;
    try { message = JSON.parse(raw.toString()); } catch { socket.close(1008); return; }
    if (socket !== control) return;
    lastHeartbeat = Date.now();
    enqueue(async () => {
      if (socket !== control) return;
      if (Number.isSafeInteger(message.freeze)) {
        frozen = true;
        await Promise.all([...rooms.values()].map(reconcile));
        send(socket, { frozen: message.freeze });
      } else await applyPolicy(message);
    }).catch(() => socket.close());
  });
  socket.on('close', () => { if (control === socket) { control = undefined; failClosed(); } });
  socket.on('error', () => socket.close());
});
setInterval(() => { if (lastHeartbeat && !healthy()) { lastHeartbeat = 0; failClosed(); } }, 500).unref();
publicServer.listen(Number(process.env.VOICE_PORT ?? 9100), host);
controlServer.listen(Number(process.env.VOICE_CONTROL_PORT ?? 9101), process.env.VOICE_CONTROL_ADDRESS ?? '127.0.0.1');
console.log('Voice service started');
function shutdown() {
  failClosed();
  for (const { worker } of workers) worker.close();
  process.exit(0);
}
process.on('SIGINT', shutdown);
process.on('SIGTERM', shutdown);
