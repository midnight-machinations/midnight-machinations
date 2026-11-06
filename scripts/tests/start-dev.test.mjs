import assert from 'node:assert/strict';
import { spawnSync } from 'node:child_process';
import { EventEmitter } from 'node:events';
import { mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { PassThrough } from 'node:stream';
import { setTimeout } from 'node:timers/promises';
import { test } from 'node:test';
import { manualCommands, runServices, serviceCommands } from '../start-dev.mjs';

function fixture(t) {
  const directory = mkdtempSync(join(tmpdir(), 'midnight-dev-'));
  t.after(() => rmSync(directory, { recursive: true, force: true }));
  const output = new PassThrough();
  let text = '';
  output.on('data', data => { text += data; });
  return { directory, output, errorOutput: output, signals: new EventEmitter(), graceMs: 250, text: () => text };
}

function service(name, code) {
  return { name, cwd: '.', command: process.execPath, args: ['-e', code] };
}

async function until(predicate) {
  for (let attempt = 0; attempt < 200; attempt++) {
    if (predicate()) return;
    await setTimeout(20);
  }
  assert.fail('Timed out waiting for service output');
}

function alive(pid) {
  try {
    process.kill(pid, 0);
    // Container init may leave exited orphan processes as zombies.
    try { if (/^State:\s+Z/m.test(readFileSync(`/proc/${pid}/status`, 'utf8'))) return false; } catch {}
    return true;
  } catch (error) {
    if (error.code === 'ESRCH') return false;
    throw error;
  }
}

test('Vite arguments go only to the client and manual commands preserve shell quoting', () => {
  const services = serviceCommands(['--host', '0.0.0.0', '--port', '3005']);
  assert.deepEqual(services.find(s => s.name === 'client').args, ['dev', '--no-open', '--strictPort', '--host', '0.0.0.0', '--port', '3005']);
  assert.deepEqual(services.find(s => s.name === 'voice').args, ['start']);
  assert.deepEqual(services.find(s => s.name === 'server').args, ['run']);
  assert.match(manualCommands(['--host']), /pnpm dev .*'--host'/);
  assert.ok(manualCommands(["a'b"]).includes("'a'\\''b'"));
});

test('Vite exposes the container by default and accepts an explicit host override', () => {
  const client = args => serviceCommands(args).find(s => s.name === 'client').args;
  assert.deepEqual(client([]), ['dev', '--no-open', '--strictPort', '--host']);
  assert.deepEqual(client(['--port', '3005']), ['dev', '--no-open', '--strictPort', '--host', '--port', '3005']);
  assert.deepEqual(client(['--host', 'localhost']), ['dev', '--no-open', '--strictPort', '--host', 'localhost']);
  assert.deepEqual(client(['--host=localhost']), ['dev', '--no-open', '--strictPort', '--host=localhost']);
  assert.match(manualCommands(), /'--host'/);
});

test('a service failure reports its stderr and stops other services', async t => {
  const f = fixture(t);
  const result = runServices([
    service('running', 'console.log("PID=" + process.pid); setInterval(() => {}, 1000)'),
    service('failed', 'console.error("port already in use"); setTimeout(() => process.exit(7), 100)'),
  ], f);
  assert.equal(await result, 7);
  assert.match(f.text(), /\[failed\] port already in use/);
  assert.match(f.text(), /\[failed\] Unexpected exit \(code 7\)/);
  const pid = Number(f.text().match(/PID=(\d+)/)[1]);
  await until(() => !alive(pid));
});

test('missing executable stops the other services with a useful error', async t => {
  const f = fixture(t);
  const result = await runServices([
    service('running', 'setInterval(() => {}, 1000)'),
    { name: 'missing', cwd: '.', command: join(f.directory, 'not-installed'), args: [] },
  ], f);
  assert.equal(result, 1);
  assert.match(f.text(), /\[missing\] Failed to start: .*ENOENT/);
});

test('Ctrl+C stops entire process groups, including descendants that ignore SIGTERM', async t => {
  const f = fixture(t);
  const childScript = join(f.directory, 'child.cjs');
  writeFileSync(childScript, 'process.on("SIGTERM", () => {}); console.log("DESCENDANT=" + process.pid); setInterval(() => {}, 1000);');
  const result = runServices([
    service('tree', `require('node:child_process').spawn(process.execPath, [${JSON.stringify(childScript)}], {stdio:'inherit'}); console.log('PARENT=' + process.pid); setInterval(() => {}, 1000)`),
    service('other', 'console.log("OTHER=" + process.pid); setInterval(() => {}, 1000)'),
  ], f);
  await until(() => /DESCENDANT=/.test(f.text()) && /OTHER=/.test(f.text()));
  f.signals.emit('SIGINT');
  assert.equal(await result, 130);
  for (const [, pid] of f.text().matchAll(/(?:PARENT|DESCENDANT|OTHER)=(\d+)/g)) {
    await until(() => !alive(Number(pid)));
  }
  assert.equal(f.signals.listenerCount('SIGINT'), 0);
  assert.equal(f.signals.listenerCount('SIGTERM'), 0);
});

test('a clean but unexpected service exit still stops the stack as a failure', async t => {
  const f = fixture(t);
  assert.equal(await runServices([service('exited', 'process.exit(0)')], f), 1);
  assert.match(f.text(), /Unexpected exit \(code 0\)/);
});

test('SIGTERM stops the stack too', async t => {
  const f = fixture(t);
  const result = runServices([service('running', 'console.log("ready"); setInterval(() => {}, 1000)')], f);
  await until(() => /\[running\] ready/.test(f.text()));
  f.signals.emit('SIGTERM');
  assert.equal(await result, 143);
});

test('missing setup reports fixes and separate commands without starting services', t => {
  const f = fixture(t);
  mkdirSync(join(f.directory, 'scripts'));
  const script = join(f.directory, 'scripts/start-dev.mjs');
  writeFileSync(script, readFileSync(new URL('../start-dev.mjs', import.meta.url)));
  const result = spawnSync(process.execPath, [script, '--host'], {
    env: { PATH: '/not-installed' }, encoding: 'utf8',
  });
  assert.equal(result.status, 1);
  assert.match(result.stderr, /pnpm is unavailable/);
  assert.match(result.stderr, /cd client && pnpm install --frozen-lockfile/);
  assert.match(result.stderr, /cd voice && pnpm install --frozen-lockfile/);
  assert.match(result.stderr, /cd voice && pnpm configure/);
  assert.match(result.stderr, /cd server && cargo run/);
  assert.match(result.stderr, /pnpm dev .*'--host'/);
  assert.doesNotMatch(result.stdout, /Starting/);
});
