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

const opt = (val) => val != null && val !== '';

// Build gh repo edit args for all supported fields
const editArgs = [];
const addArg = (flag, val) => {
  if (opt(val)) editArgs.push(`${flag} '${String(val).replace(/'/g, "'\\''")}'`);
};

addArg('--description', info.description);
addArg('--homepage', info.homepage);
addArg('--default-branch', info.default_branch);
addArg('--visibility', info.visibility);

if (info.has_issues != null) editArgs.push(info.has_issues ? '--enable-issues' : '--disable-issues');
if (info.has_wiki != null) editArgs.push(info.has_wiki ? '--enable-wiki' : '--disable-wiki');
if (info.has_projects != null) editArgs.push(info.has_projects ? '--enable-projects' : '--disable-projects');
if (info.allow_squash_merge != null) editArgs.push(info.allow_squash_merge ? '--allow-squash-merge' : '--disallow-squash-merge');
if (info.allow_merge_commit != null) editArgs.push(info.allow_merge_commit ? '--allow-merge-commit' : '--disallow-merge-commit');
if (info.allow_rebase_merge != null) editArgs.push(info.allow_rebase_merge ? '--allow-rebase-merge' : '--disallow-rebase-merge');
if (info.delete_branch_on_merge != null) editArgs.push(info.delete_branch_on_merge ? '--delete-branch-on-merge' : '--keep-branch-on-merge');
if (info.allow_update_branch != null) editArgs.push(info.allow_update_branch ? '--allow-update-branch' : '--disallow-update-branch');

if (editArgs.length) {
  const repoArg = repo ? `'${repo}'` : '';
  execSync(`gh repo edit ${repoArg} ${editArgs.join(' ')}`, { stdio: 'inherit' });
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
