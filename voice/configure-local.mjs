import { randomBytes } from 'node:crypto';
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { networkInterfaces } from 'node:os';
import { isIP } from 'node:net';

// Optional offset lets separate checkouts run their own media services.
const args = process.argv.slice(2);
const addressIndex = args.indexOf('--announced-address');
const suppliedAddress = addressIndex >= 0 ? args[addressIndex + 1] : process.env.VOICE_ANNOUNCED_ADDRESS;
if (addressIndex >= 0) args.splice(addressIndex, 2);
if (suppliedAddress !== undefined && isIP(suppliedAddress) !== 4) throw new Error('--announced-address requires an IPv4 address');
if (args.length > 1) throw new Error('Usage: npm run configure -- [offset] [--announced-address IPv4]');
const offset = Number(args[0] ?? 0);
if (!Number.isInteger(offset) || offset < 0 || offset > 1000) throw new Error('Expected a port offset between 0 and 1000');
// Firefox rejects loopback media candidates. On Linux the host can also reach
// its dev container directly at the container's non-loopback interface address.
const interfaceAddress = Object.values(networkInterfaces()).flat().find(address =>
  address?.family === 'IPv4' && !address.internal && !address.address.startsWith('169.254.'))?.address;
const announcedAddress = suppliedAddress ?? interfaceAddress;
if (!announcedAddress) throw new Error('No media interface found. Supply --announced-address with a browser-reachable IPv4 address.');
const voiceFile = fileURLToPath(new URL('.env.local', import.meta.url));
const serverFile = fileURLToPath(new URL('../server/.env.local', import.meta.url));
function read(file) { return existsSync(file) ? readFileSync(file, 'utf8') : ''; }
const previous = read(voiceFile);
const secret = previous.match(/^VOICE_CONTROL_SECRET=([a-f0-9]{64})$/m)?.[1] ?? randomBytes(32).toString('hex');
function merge(file, values) {
  let contents = read(file);
  for (const [key, value] of Object.entries(values)) {
    const pattern = new RegExp(`^${key}=.*$`, 'm');
    if (pattern.test(contents)) contents = contents.replace(pattern, `${key}=${value}`);
    else contents = contents.trimEnd() + `\n${key}=${value}\n`;
  }
  writeFileSync(file, contents.trimStart(), { mode: 0o600 });
}
merge(voiceFile, {
  VOICE_CONTROL_SECRET: secret,
  VOICE_PORT: 9100 + offset * 2,
  VOICE_CONTROL_PORT: 9101 + offset * 2,
  VOICE_MEDIA_PORT: 44400 + offset * 3,
  VOICE_LISTEN_ADDRESS: '127.0.0.1',
  VOICE_CONTROL_ADDRESS: '127.0.0.1',
  // Docker's published ports target the container interface, not its loopback.
  VOICE_MEDIA_ADDRESS: '0.0.0.0',
  VOICE_ANNOUNCED_ADDRESS: announcedAddress,
});
merge(serverFile, {
  VOICE_CONTROL_SECRET: secret,
  VOICE_CONTROL_URL: `ws://127.0.0.1:${9101 + offset * 2}/control`,
  VOICE_PUBLIC_URL: `ws://localhost:${9100 + offset * 2}`,
});
console.log('Configured voice/.env.local and server/.env.local for local voice. Existing unrelated settings were preserved.');
console.log(`Media address announced to browsers: ${announcedAddress}`);
