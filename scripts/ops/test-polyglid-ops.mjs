#!/usr/bin/env node
import assert from 'node:assert/strict';
import test from 'node:test';
import { main } from './polyglid-ops.mjs';

function invoke(args = [], { dryRun = true, spawn } = {}) {
  const stdout = [];
  const stderr = [];
  const runtime = {
    env: dryRun ? { POLYGLID_OPS_DRY_RUN: '1' } : {},
    exists: () => true,
    nodePath: process.execPath,
    spawn: spawn ?? (() => {
      throw new Error('A dry-run test unexpectedly spawned a process');
    }),
    stdout: (message) => stdout.push(message),
    stderr: (message) => stderr.push(message),
  };

  return {
    status: main(args, runtime),
    stdout: `${stdout.join('\n')}${stdout.length > 0 ? '\n' : ''}`,
    stderr: `${stderr.join('\n')}${stderr.length > 0 ? '\n' : ''}`,
  };
}

function successful(args) {
  const result = invoke(args);
  assert.equal(result.status, 0, result.stderr);
  return result;
}

function plan(result) {
  return result.stdout
    .split('\n')
    .filter((line) => line.startsWith('DRY-RUN '))
    .map((line) => JSON.parse(line.slice('DRY-RUN '.length)));
}

const workspacePlan = (subcommand, baseArgs = [], forwardedArgs = []) => [
  ['cargo', subcommand, ...baseArgs, ...forwardedArgs],
  [
    'cargo',
    subcommand,
    ...baseArgs,
    '--manifest-path',
    'sdk/Cargo.toml',
    ...forwardedArgs,
  ],
  [
    'cargo',
    subcommand,
    ...baseArgs,
    '--manifest-path',
    'tools/ai/rust/Cargo.toml',
    ...forwardedArgs,
  ],
];

const formatPlan = (forwardedArgs = []) => workspacePlan(
  'fmt',
  ['--all'],
  forwardedArgs,
).slice(0, 2);

const compilePlan = (subcommand, forwardedArgs = []) => [
  ['cargo', subcommand, '--locked', '--workspace', ...forwardedArgs],
  [
    'cargo',
    subcommand,
    '--locked',
    '--workspace',
    '--target',
    'wasm32-wasip1',
    '--manifest-path',
    'sdk/Cargo.toml',
    ...forwardedArgs,
  ],
  [
    'cargo',
    subcommand,
    '--locked',
    '--workspace',
    '--manifest-path',
    'tools/ai/rust/Cargo.toml',
    ...forwardedArgs,
  ],
];

const testPlan = (forwardedArgs = []) => [
  ['cargo', 'test', '--locked', '--workspace', ...forwardedArgs],
  [
    'cargo',
    'test',
    '--locked',
    '--workspace',
    '--manifest-path',
    'sdk/Cargo.toml',
    ...forwardedArgs,
  ],
  [
    'cargo',
    'check',
    '--locked',
    '--workspace',
    '--all-features',
    '--target',
    'wasm32-wasip1',
    '--manifest-path',
    'sdk/Cargo.toml',
  ],
  [
    'cargo',
    'test',
    '--locked',
    '--workspace',
    '--manifest-path',
    'tools/ai/rust/Cargo.toml',
    ...forwardedArgs,
  ],
];

test('help is the default and documents the complete command surface', () => {
  for (const args of [[], ['help'], ['--help'], ['-h']]) {
    const result = successful(args);
    assert.match(result.stdout, /PolyGlid operations CLI/);
    for (const command of [
      'doctor',
      'format',
      'check',
      'validate',
      'build',
      'test',
      'clean',
      'desktop',
      'server',
      'detect',
      'graph',
      'site-build',
      'mvp-smoke',
      'repo-sync',
    ]) {
      assert.match(result.stdout, new RegExp(`\\b${command}\\b`));
    }
    assert.deepEqual(plan(result), []);
  }
});

test('an unknown command exits with usage error and executes nothing', () => {
  const result = invoke(['does-not-exist']);
  assert.equal(result.status, 2);
  assert.match(result.stderr, /Unknown command: does-not-exist/);
  assert.match(result.stdout, /Usage:/);
  assert.deepEqual(plan(result), []);
});

test('build covers every Cargo workspace and forwards arguments', () => {
  const forwarded = ['--release', '--features', 'example feature'];
  const result = successful(['build', ...forwarded]);
  assert.deepEqual(
    plan(result),
    compilePlan('build', forwarded),
  );
});

test('test covers every Cargo workspace and routes compatible arguments', () => {
  const forwarded = ['--no-default-features', '--', '--nocapture'];
  const result = successful(['test', ...forwarded]);
  assert.deepEqual(
    plan(result),
    testPlan(forwarded),
  );

  const noRun = ['--no-run'];
  const noRunPlan = plan(successful(['test', ...noRun]));
  assert.deepEqual(noRunPlan, testPlan(noRun));
  assert.equal(noRunPlan[1].filter((arg) => arg === '--no-run').length, 1);
  assert.equal(noRunPlan[2].includes('--no-run'), false);
});

test('run and routing commands preserve exact argument boundaries', () => {
  assert.deepEqual(plan(successful(['desktop', '--release', '--', 'value with spaces'])), [
    [
      'cargo',
      'run',
      '--locked',
      '-p',
      'polyglid-desktop',
      '--release',
      '--',
      'value with spaces',
    ],
  ]);
  assert.deepEqual(plan(successful(['server', '--', '--bind', '127.0.0.1:3000'])), [
    [
      'cargo',
      'run',
      '--locked',
      '-p',
      'polyglid-server',
      '--',
      '--bind',
      '127.0.0.1:3000',
    ],
  ]);
  assert.deepEqual(plan(successful(['detect', 'base ref', 'head ref'])), [
    ['bash', 'scripts/ops/detect-changes.sh', 'base ref', 'head ref'],
  ]);
});

test('legacy automation commands keep their established entry points', () => {
  assert.deepEqual(plan(successful(['detect', 'HEAD~1', 'HEAD'])), [
    ['bash', 'scripts/ops/detect-changes.sh', 'HEAD~1', 'HEAD'],
  ]);
  assert.deepEqual(plan(successful(['repo-sync'])), [
    [process.execPath, 'scripts/ops/sync-repo.mjs'],
  ]);
  assert.deepEqual(plan(successful(['site-build'])), [
    ['cargo', 'run', '--locked', '-p', 'polyglid-site'],
  ]);
  assert.deepEqual(plan(successful(['mvp-smoke'])), [
    ['bash', 'scripts/ops/mvp-smoke.sh'],
  ]);

  const validation = plan(successful(['validate']));
  assert.deepEqual(
    validation.filter((command) => (
      command.length === 2
      && command[1].endsWith('generate-graph.sh')
    )),
    [['bash', 'tools/automation/scripts/generate-graph.sh']],
  );
  assert.deepEqual(validation.slice(-5), [
    ...formatPlan(['--', '--check']),
    ...compilePlan('check'),
  ]);
});

test('workspace maintenance commands use their intentional workspace scopes', () => {
  assert.deepEqual(
    plan(successful(['format', '--', '--check'])),
    formatPlan(['--', '--check']),
  );
  assert.deepEqual(
    plan(successful(['check', '--all-targets'])),
    compilePlan('check', ['--all-targets']),
  );
  assert.deepEqual(
    plan(successful(['clean'])),
    workspacePlan('clean'),
  );
  assert.deepEqual(
    plan(successful(['graph'])),
    [['bash', 'tools/automation/scripts/generate-graph.sh']],
  );

  for (const args of [
    ['doctor', '--verbose'],
    ['graph', '--edges'],
    ['mvp-smoke', '--target', 'example.com'],
    ['repo-sync', '--dry-run'],
    ['detect', 'main', 'HEAD', 'extra'],
  ]) {
    const unsupported = invoke(args);
    assert.equal(unsupported.status, 2);
    assert.match(unsupported.stderr, /does not accept|at most 2/);
    assert.deepEqual(plan(unsupported), []);
  }
});

test('a child-process failure is returned to the caller', () => {
  const result = invoke(
    ['detect', 'HEAD', 'this-ref-does-not-exist-for-the-ops-test'],
    {
      dryRun: false,
      spawn: () => ({ error: undefined, signal: null, status: 37 }),
    },
  );
  assert.equal(result.status, 37);
});
