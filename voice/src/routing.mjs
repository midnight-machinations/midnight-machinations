// The game server grants exactly one selected send group. Never broadcast a
// producer into multiple groups, even if a malformed policy is received.
export function canForwardVoice(from, to, deafened) {
  if (!from || !to || from.scope !== to.scope || from.send.length !== 1) return false;
  const group = from.send[0];
  return to.receive.includes(group) && !deafened.has(group);
}
