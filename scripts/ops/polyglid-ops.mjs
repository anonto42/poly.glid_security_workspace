#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const root = join(dirname(fileURLToPath(import.meta.url)), '../..');

function run(command, args, options = {}) {
  const result = spawnSync(command, args, { cwd: root, stdio: 'inherit', ...options });
  if (result.error) throw result.error;
  if (result.status !== 0) process.exit(result.status ?? 1);
}

function help() {
  console.log(`PolyGlid operations CLI

Usage: node scripts/ops/polyglid-ops.mjs <command> [arguments]

Commands:
  detect [base] [head]  Detect changed product areas as JSON
  repo-sync             Apply repinfo.json to GitHub repository metadata
  site-build            Generate the static website
  mvp-smoke             Run the real CLI-to-WASM MVP smoke test
  validate              Validate metadata, scripts, formatting, and workspace
  help                  Show this help`);
}

const [command = 'help', ...args] = process.argv.slice(2);
switch (command) {
  case 'detect':
    run('bash', ['scripts/ops/detect-changes.sh', ...args]);
    break;
  case 'repo-sync':
    run(process.execPath, ['scripts/ops/sync-repo.mjs']);
    break;
  case 'site-build':
    run('cargo', ['run', '--locked', '-p', 'polyglid-site']);
    break;
  case 'mvp-smoke':
    run('bash', ['scripts/ops/mvp-smoke.sh']);
    break;
  case 'validate':
    run(process.execPath, ['-e', "JSON.parse(require('fs').readFileSync('repinfo.json','utf8'))"]);
    run('bash', ['-n', 'scripts/ops/detect-changes.sh']);
    run('cargo', ['fmt', '--all', '--', '--check']);
    run('cargo', ['check', '--workspace']);
    break;
  case 'help':
  case '--help':
  case '-h':
    help();
    break;
  default:
    console.error(`Unknown command: ${command}\n`);
    help();
    process.exit(2);
}
