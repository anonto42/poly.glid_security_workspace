#!/usr/bin/env node
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const scriptPath = fileURLToPath(import.meta.url);
const root = join(dirname(scriptPath), '../..');

const workspaces = [
  { name: 'root' },
  { name: 'SDK', manifest: 'sdk/Cargo.toml' },
  { name: 'AI engine', manifest: 'tools/ai/rust/Cargo.toml' },
];
const formattedWorkspaces = workspaces.slice(0, 2);

class CommandFailure extends Error {
  constructor(exitCode) {
    super(`Command failed with exit code ${exitCode}`);
    this.exitCode = exitCode;
  }
}

function defaultRuntime() {
  return {
    env: process.env,
    exists: existsSync,
    nodePath: process.execPath,
    spawn(command, args, options = {}) {
      return spawnSync(command, args, {
        cwd: root,
        stdio: 'inherit',
        ...options,
      });
    },
    stdout: (message) => console.log(message),
    stderr: (message) => console.error(message),
  };
}

function createContext(overrides = {}) {
  const runtime = { ...defaultRuntime(), ...overrides };
  return {
    ...runtime,
    dryRun: runtime.env.POLYGLID_OPS_DRY_RUN === '1',
  };
}

function run(context, command, args, options = {}) {
  if (context.dryRun) {
    // JSON keeps argument boundaries exact, even when an argument contains spaces.
    context.stdout(`DRY-RUN ${JSON.stringify([command, ...args])}`);
    return;
  }

  const result = context.spawn(command, args, options);
  if (result.error) {
    context.stderr(`Unable to run ${command}: ${result.error.message}`);
    throw new CommandFailure(1);
  }

  if (result.signal) {
    context.stderr(`${command} was terminated by ${result.signal}`);
    throw new CommandFailure(1);
  }

  if (result.status !== 0) {
    throw new CommandFailure(result.status ?? 1);
  }
}

function cargoWorkspace(context, workspace, subcommand, baseArgs, forwardedArgs = []) {
  const commandArgs = [subcommand, ...baseArgs];
  if (workspace.manifest) {
    commandArgs.push('--manifest-path', workspace.manifest);
  }
  commandArgs.push(...forwardedArgs);
  run(context, 'cargo', commandArgs);
}

function cargoWorkspaces(context, subcommand, baseArgs, forwardedArgs) {
  for (const workspace of workspaces) {
    cargoWorkspace(context, workspace, subcommand, baseArgs, forwardedArgs);
  }
}

function cargoFormattedWorkspaces(context, forwardedArgs) {
  for (const workspace of formattedWorkspaces) {
    cargoWorkspace(context, workspace, 'fmt', ['--all'], forwardedArgs);
  }
}

function cargoCompileWorkspaces(context, subcommand, forwardedArgs) {
  const baseArgs = ['--locked', '--workspace'];
  cargoWorkspace(context, workspaces[0], subcommand, baseArgs, forwardedArgs);
  cargoWorkspace(context, workspaces[1], subcommand, [
    ...baseArgs,
    '--target',
    'wasm32-wasip1',
  ], forwardedArgs);
  cargoWorkspace(context, workspaces[2], subcommand, baseArgs, forwardedArgs);
}

function cargoTestWorkspaces(context, forwardedArgs) {
  const baseArgs = ['--locked', '--workspace'];
  cargoWorkspace(context, workspaces[0], 'test', baseArgs, forwardedArgs);
  cargoWorkspace(context, workspaces[1], 'test', baseArgs, forwardedArgs);
  // Run the SDK's unit and documentation tests natively, then independently
  // prove that every SDK feature still compiles for the real guest target.
  cargoWorkspace(context, workspaces[1], 'check', [
    ...baseArgs,
    '--all-features',
    '--target',
    'wasm32-wasip1',
  ]);
  cargoWorkspace(context, workspaces[2], 'test', baseArgs, forwardedArgs);
}

function requireArgumentCount(context, command, args, maximum) {
  if (args.length <= maximum) return;

  const expectation = maximum === 0
    ? 'does not accept arguments'
    : `accepts at most ${maximum} arguments`;
  context.stderr(`The ${command} command ${expectation}.`);
  throw new CommandFailure(2);
}

function requireRepositoryFiles(context) {
  const required = [
    'Cargo.toml',
    'Cargo.lock',
    'sdk/Cargo.toml',
    'sdk/Cargo.lock',
    'tools/ai/rust/Cargo.toml',
    'tools/ai/rust/Cargo.lock',
    'repinfo.json',
  ];
  const missing = required.filter((path) => !context.exists(join(root, path)));

  if (missing.length > 0) {
    context.stderr(
      `Repository doctor found missing files:\n${missing.map((path) => `  - ${path}`).join('\n')}`,
    );
    throw new CommandFailure(1);
  }
}

function doctor(context) {
  requireRepositoryFiles(context);
  run(context, context.nodePath, ['--version']);
  run(context, 'cargo', ['--version']);
  run(context, 'rustc', ['--version']);
  run(context, 'rustup', ['--version']);
  run(context, 'git', ['--version']);
  run(context, 'bash', ['--version']);
  run(context, 'jq', ['--version']);
  run(context, 'gh', ['--version']);

  if (!context.dryRun) {
    context.stdout('PolyGlid operations environment is ready.');
  }
}

function validate(context, args) {
  run(context, context.nodePath, [
    '-e',
    "for (const file of ['package.json', 'repinfo.json']) JSON.parse(require('fs').readFileSync(file, 'utf8'))",
  ]);
  run(context, 'bash', ['-n', 'scripts/ops/detect-changes.sh']);
  run(context, 'bash', ['-n', 'scripts/ops/test-detect-changes.sh']);
  run(context, 'bash', ['-n', 'scripts/ops/mvp-smoke.sh']);
  run(context, 'bash', ['-n', 'tools/automation/scripts/validate-workspace.sh']);
  run(context, 'bash', ['-n', 'tools/automation/scripts/generate-graph.sh']);
  run(context, context.nodePath, ['--check', 'scripts/ops/polyglid-ops.mjs']);
  run(context, context.nodePath, ['--check', 'scripts/ops/sync-repo.mjs']);
  run(context, context.nodePath, ['--check', 'scripts/ops/test-polyglid-ops.mjs']);
  run(context, context.nodePath, ['--test', 'scripts/ops/test-polyglid-ops.mjs']);
  run(context, 'bash', ['scripts/ops/test-detect-changes.sh']);
  run(context, 'bash', ['tools/automation/scripts/validate-workspace.sh', '--quiet']);
  run(context, 'bash', ['tools/automation/scripts/generate-graph.sh']);
  cargoFormattedWorkspaces(context, ['--', '--check']);
  cargoCompileWorkspaces(context, 'check', args);
}

function help(context) {
  context.stdout(`PolyGlid operations CLI

Usage: node scripts/ops/polyglid-ops.mjs <command> [arguments]

Commands:
  doctor                Check repository files and required development tools
  format [args]         Format the maintained root and SDK workspaces
  check [args]          Check all three Cargo workspaces
  validate [args]       Validate metadata, operations scripts, and workspaces
  build [args]          Build all three Cargo workspaces
  test [args]           Test hosts and validate the WASM-targeted SDK
  clean [args]          Clean all three Cargo workspaces
  desktop [args]        Run the Dioxus desktop client
  server [args]         Run the optional backend server
  detect [base] [head]  Detect changed product areas as JSON
  graph                 Generate one DOT graph for all Cargo workspaces
  site-build [args]     Generate the static website
  mvp-smoke             Run the real CLI-to-WASM MVP smoke test
  repo-sync             Explicitly apply repinfo.json to GitHub metadata
  help                  Show this help

Arguments accepted by a command are forwarded without shell interpretation.
Set POLYGLID_OPS_DRY_RUN=1 to print the exact command plan without executing it.`);
}

function dispatch(context, argv) {
  const [command = 'help', ...args] = argv;

  switch (command) {
    case 'doctor':
      requireArgumentCount(context, command, args, 0);
      doctor(context);
      break;
    case 'format':
      cargoFormattedWorkspaces(context, args);
      break;
    case 'check':
      cargoCompileWorkspaces(context, 'check', args);
      break;
    case 'validate':
      validate(context, args);
      break;
    case 'build':
      cargoCompileWorkspaces(context, 'build', args);
      break;
    case 'test':
      cargoTestWorkspaces(context, args);
      break;
    case 'clean':
      cargoWorkspaces(context, 'clean', [], args);
      break;
    case 'desktop':
      run(context, 'cargo', ['run', '--locked', '-p', 'polyglid-desktop', ...args]);
      break;
    case 'server':
      run(context, 'cargo', ['run', '--locked', '-p', 'polyglid-server', ...args]);
      break;
    case 'detect':
      requireArgumentCount(context, command, args, 2);
      run(context, 'bash', ['scripts/ops/detect-changes.sh', ...args]);
      break;
    case 'graph':
      requireArgumentCount(context, command, args, 0);
      run(context, 'bash', ['tools/automation/scripts/generate-graph.sh']);
      break;
    case 'site-build':
      run(context, 'cargo', ['run', '--locked', '-p', 'polyglid-site', ...args]);
      break;
    case 'mvp-smoke':
      requireArgumentCount(context, command, args, 0);
      run(context, 'bash', ['scripts/ops/mvp-smoke.sh']);
      break;
    case 'repo-sync':
      requireArgumentCount(context, command, args, 0);
      run(context, context.nodePath, ['scripts/ops/sync-repo.mjs']);
      break;
    case 'help':
    case '--help':
    case '-h':
      help(context);
      break;
    default:
      context.stderr(`Unknown command: ${command}\n`);
      help(context);
      throw new CommandFailure(2);
  }
}

export function main(argv = process.argv.slice(2), runtime = {}) {
  const context = createContext(runtime);
  try {
    dispatch(context, argv);
    return 0;
  } catch (error) {
    if (error instanceof CommandFailure) {
      return error.exitCode;
    }
    throw error;
  }
}

if (process.argv[1] && resolve(process.argv[1]) === scriptPath) {
  process.exitCode = main();
}
