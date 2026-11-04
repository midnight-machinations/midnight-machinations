import { spawn, spawnSync } from 'node:child_process';
import { existsSync, readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { createInterface } from 'node:readline';
import { fileURLToPath } from 'node:url';
import { parseEnv } from 'node:util';

const root = fileURLToPath(new URL('../', import.meta.url));

function viteOptions(args) {
  const hasHost = args.some(arg => arg === '--host' || arg.startsWith('--host='));
  return ['--no-open', '--strictPort', ...(hasHost ? [] : ['--host']), ...args];
}

export function serviceCommands(args = []) {
  return [
    { name: 'voice', cwd: 'voice', command: 'pnpm', args: ['start'] },
    { name: 'server', cwd: 'server', command: 'cargo', args: ['run'] },
    // Expose Vite outside the container and avoid opening a browser inside it.
    { name: 'client', cwd: 'client', command: 'pnpm', args: ['dev', ...viteOptions(args)] },
  ];
}

export function manualCommands(args = []) {
  const quote = value => `'${value.replaceAll("'", "'\\''")}'`;
  return [
    'Run each service in a separate terminal from the repository root:',
    `  cd client && pnpm dev ${viteOptions(args).map(quote).join(' ')}`,
    '  cd server && cargo run',
    '  cd voice && pnpm start',
  ].join('\n');
}

function environment(directory, files) {
  const values = {};
  for (const file of files) {
    const path = resolve(root, directory, file);
    if (existsSync(path)) Object.assign(values, parseEnv(readFileSync(path, 'utf8')));
  }
  return { ...values, ...process.env };
}

function checkSetup() {
  const errors = [];
  if (process.platform === 'win32') {
    errors.push('The combined launcher needs Linux, macOS, or WSL to stop entire service process groups.');
  }
  for (const command of ['pnpm', 'cargo']) {
    const result = spawnSync(command, ['--version'], { cwd: root, stdio: 'ignore' });
    if (result.error || result.status !== 0) {
      errors.push(`${command} is unavailable. ${command === 'pnpm' ? 'Run corepack enable.' : 'Install Rust with rustup.'}`);
    }
  }
  if (!existsSync(resolve(root, 'client/node_modules/vite/bin/vite.js'))) {
    errors.push('Client dependencies are missing. Run: cd client && pnpm install --frozen-lockfile');
  }
  if (!existsSync(resolve(root, 'voice/node_modules/mediasoup')) || !existsSync(resolve(root, 'voice/node_modules/ws'))) {
    errors.push('Voice dependencies are missing. Run: cd voice && pnpm install --frozen-lockfile');
  }
  const voice = environment('voice', ['.env.local']);
  const server = environment('server', ['.env', '.env.local']);
  if (!voice.VOICE_CONTROL_SECRET || !server.VOICE_CONTROL_SECRET || !server.VOICE_CONTROL_URL || !server.VOICE_PUBLIC_URL) {
    errors.push('Local voice settings are missing. Run: cd voice && pnpm configure');
  } else if (voice.VOICE_CONTROL_SECRET !== server.VOICE_CONTROL_SECRET) {
    errors.push('The voice and server control secrets do not match. Run: cd voice && pnpm configure');
  }
  return errors;
}

// Detached POSIX groups let shutdown reach pnpm, Cargo, and their descendants,
// including mediasoup workers. Never signal unrelated services already running.
export async function runServices(services, {
  directory = root,
  output = process.stdout,
  errorOutput = process.stderr,
  signals = process,
  graceMs = 4000,
} = {}) {
  const children = [];
  let stopping = false;
  let exitCode = 0;
  let finish;
  const finished = new Promise(resolve => { finish = resolve; });

  function signalGroups(signal) {
    for (const child of children) {
      if (!child.pid) continue;
      try { process.kill(-child.pid, signal); }
      catch (error) { if (error.code !== 'ESRCH') errorOutput.write(`Cannot stop process group ${child.pid}: ${error.message}\n`); }
    }
  }
  function groupsRemain() {
    return children.some(child => {
      if (!child.pid) return false;
      try { process.kill(-child.pid, 0); return true; }
      catch (error) { return error.code !== 'ESRCH'; }
    });
  }
  function stop(code) {
    if (stopping) return;
    stopping = true;
    exitCode = code;
    output.write('[dev] Stopping all services...\n');
    signalGroups('SIGTERM');
    const deadline = Date.now() + graceMs;
    const timer = setInterval(() => {
      if (groupsRemain() && Date.now() < deadline) return;
      signalGroups('SIGKILL');
      clearInterval(timer);
      finish(exitCode);
    }, 50);
  }
  const interrupt = () => stop(130);
  const terminate = () => stop(143);
  signals.on('SIGINT', interrupt);
  signals.on('SIGTERM', terminate);
  for (const service of services) {
    if (stopping) break;
    output.write(`[${service.name}] Starting ${service.command} ${service.args.join(' ')}\n`);
    const child = spawn(service.command, service.args, {
      cwd: resolve(directory, service.cwd),
      detached: true,
      stdio: ['ignore', 'pipe', 'pipe'],
    });
    children.push(child);
    for (const [stream, destination] of [[child.stdout, output], [child.stderr, errorOutput]]) {
      const lines = createInterface({ input: stream });
      lines.on('line', line => destination.write(`[${service.name}] ${line}\n`));
    }
    child.once('error', error => {
      errorOutput.write(`[${service.name}] Failed to start: ${error.message}\n`);
      stop(1);
    });
    child.once('exit', (code, signal) => {
      if (stopping) return;
      errorOutput.write(`[${service.name}] Unexpected exit (${signal ?? `code ${code}`}).\n`);
      stop(code && code > 0 ? code : 1);
    });
  }
  if (!services.length) finish(0);
  const result = await finished;
  signals.removeListener('SIGINT', interrupt);
  signals.removeListener('SIGTERM', terminate);
  return result;
}

async function main() {
  const args = process.argv.slice(2);
  if (args.includes('--help') || args.includes('-h')) {
    console.log('Usage: pnpm start [Vite options]\nStarts client, server, and voice. Ctrl+C stops all three.\nVite is exposed outside the container by default. Use --host localhost to restrict it.\nDependencies and voice configuration must already be installed.\n');
    console.log(manualCommands());
    return;
  }
  try {
    const errors = checkSetup();
    if (errors.length) throw new Error(errors.join('\n'));
    console.log('[dev] Starting client, server, and voice. Ctrl+C stops all three.');
    console.log('[dev] Service output follows. Cargo may take time to compile.');
    const code = await runServices(serviceCommands(args));
    if (code !== 0 && code !== 130 && code !== 143) console.error(`\n${manualCommands(args)}`);
    process.exitCode = code;
  } catch (error) {
    console.error(`[dev] ${error.message}\n\n${manualCommands(args)}`);
    process.exitCode = 1;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) await main();
