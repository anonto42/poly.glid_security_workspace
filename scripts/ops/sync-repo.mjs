#!/usr/bin/env node
import { readFileSync, existsSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';
import { execSync } from 'node:child_process';

const __dirname = dirname(fileURLToPath(import.meta.url));
const root = join(__dirname, '../..');
const filePath = join(root, 'repinfo.json');

if (!existsSync(filePath)) {
  throw new Error('repinfo.json not found in repo root');
}

const info = JSON.parse(readFileSync(filePath, 'utf-8'));
const repo = process.env.GITHUB_REPOSITORY || '';

// Description + homepage
const editArgs = [];
if (info.description) editArgs.push(`--description '${info.description.replace(/'/g, "'\\''")}'`);
if (info.homepage) editArgs.push(`--homepage '${info.homepage}'`);
if (editArgs.length) {
  execSync(`gh repo edit ${repo ? `'${repo}'` : ''} ${editArgs.join(' ')}`, { stdio: 'inherit' });
}

// Topics via API
if (info.topics?.length) {
  if (!repo) throw new Error('GITHUB_REPOSITORY not set');
  const topicsJson = JSON.stringify({ names: info.topics });
  execSync(`gh api repos/${repo}/topics -X PUT -H "Accept: application/vnd.github+json" --input -`, {
    input: topicsJson,
    stdio: ['pipe', 'inherit', 'inherit'],
  });
}

console.error('Repo metadata synced from repinfo.json');
