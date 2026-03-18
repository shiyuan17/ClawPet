#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE_MODULES = path.join(ROOT, 'node_modules');
const SOURCE_OPENCLAW = path.join(NODE_MODULES, 'openclaw');
const OUTPUT_ROOT = path.join(ROOT, 'src-tauri', 'resources', 'openclaw');
const OUTPUT_NODE_MODULES = path.join(OUTPUT_ROOT, 'node_modules');

const SKIP_PACKAGES = new Set(['typescript', '@playwright/test']);
const ROOT_OVERRIDE_PACKAGES = [
  'agent-base',
  'proxy-agent',
  'http-proxy-agent',
  'https-proxy-agent',
  'socks-proxy-agent',
  'pac-proxy-agent',
  'get-uri',
  'pac-resolver',
];

function cleanDir(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
}

function packageJsonPath(pkgDir) {
  return path.join(pkgDir, 'package.json');
}

function readPackageJson(pkgDir) {
  const pkgPath = packageJsonPath(pkgDir);
  if (!fs.existsSync(pkgPath)) return null;
  try {
    return JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
  } catch {
    return null;
  }
}

function resolvePackageDir(pkgName, parentPkgDir) {
  let cursor = parentPkgDir;
  const stopAt = path.resolve(ROOT);
  while (true) {
    const candidate = path.join(cursor, 'node_modules', ...pkgName.split('/'));
    if (fs.existsSync(candidate)) {
      return candidate;
    }

    const next = path.dirname(cursor);
    if (next === cursor) {
      break;
    }
    cursor = next;
    if (cursor.length < stopAt.length) {
      break;
    }
  }

  const fallback = path.join(NODE_MODULES, ...pkgName.split('/'));
  if (fs.existsSync(fallback)) {
    return fallback;
  }

  return null;
}

function mapSourcePathToOutput(sourcePath) {
  const normalizedSource = path.resolve(sourcePath);
  const normalizedOpenclaw = path.resolve(SOURCE_OPENCLAW);
  const normalizedNodeModules = path.resolve(NODE_MODULES);

  if (
    normalizedSource === normalizedOpenclaw ||
    normalizedSource.startsWith(`${normalizedOpenclaw}${path.sep}`)
  ) {
    return path.join(OUTPUT_ROOT, path.relative(normalizedOpenclaw, normalizedSource));
  }
  if (
    normalizedSource === normalizedNodeModules ||
    normalizedSource.startsWith(`${normalizedNodeModules}${path.sep}`)
  ) {
    return path.join(OUTPUT_NODE_MODULES, path.relative(normalizedNodeModules, normalizedSource));
  }
  throw new Error(`[bundle-openclaw] unsupported source path: ${sourcePath}`);
}

function copySourcePackageToOutput(sourcePkgDir) {
  const targetPkgDir = mapSourcePathToOutput(sourcePkgDir);
  fs.rmSync(targetPkgDir, { recursive: true, force: true });
  fs.mkdirSync(path.dirname(targetPkgDir), { recursive: true });
  fs.cpSync(sourcePkgDir, targetPkgDir, { recursive: true, dereference: true });
}

function collectDepsRecursively(sourcePkgDir, copiedSourceDirs, visitedRealDirs, stats) {
  const normalizedSource = path.resolve(sourcePkgDir);
  if (!copiedSourceDirs.has(normalizedSource)) {
    copySourcePackageToOutput(normalizedSource);
    copiedSourceDirs.add(normalizedSource);
    stats.copied += 1;
  }

  let realDir;
  try {
    realDir = fs.realpathSync(normalizedSource);
  } catch {
    realDir = normalizedSource;
  }
  if (visitedRealDirs.has(realDir)) return;
  visitedRealDirs.add(realDir);

  const pkg = readPackageJson(normalizedSource);
  if (!pkg) return;

  const directDeps = Object.keys(pkg.dependencies || {});
  const optionalDeps = new Set(Object.keys(pkg.optionalDependencies || {}));
  const allDeps = [...directDeps, ...optionalDeps];

  for (const pkgName of allDeps) {
    if (SKIP_PACKAGES.has(pkgName) || pkgName.startsWith('@types/')) continue;

    const depDir = resolvePackageDir(pkgName, normalizedSource);
    if (!depDir) {
      if (!optionalDeps.has(pkgName)) {
        console.warn(`[bundle-openclaw] missing dependency: ${pkgName}`);
      }
      continue;
    }

    collectDepsRecursively(depDir, copiedSourceDirs, visitedRealDirs, stats);
  }
}

function main() {
  if (!fs.existsSync(SOURCE_OPENCLAW)) {
    console.error('[bundle-openclaw] node_modules/openclaw not found. Run npm install first.');
    process.exit(1);
  }

  console.log('[bundle-openclaw] bundling openclaw runtime...');
  cleanDir(OUTPUT_ROOT);
  const copiedSourceDirs = new Set();
  const visitedRealDirs = new Set();
  const stats = { copied: 0 };

  collectDepsRecursively(SOURCE_OPENCLAW, copiedSourceDirs, visitedRealDirs, stats);

  // Keep proxy-agent family aligned with the root dependency graph used by the
  // development runtime. This avoids ESM/CJS export mismatches during startup.
  for (const pkgName of ROOT_OVERRIDE_PACKAGES) {
    const sourceDir = path.join(NODE_MODULES, ...pkgName.split('/'));
    if (!fs.existsSync(sourceDir)) continue;
    const targetDir = path.join(OUTPUT_NODE_MODULES, ...pkgName.split('/'));
    fs.rmSync(targetDir, { recursive: true, force: true });
    fs.mkdirSync(path.dirname(targetDir), { recursive: true });
    fs.cpSync(sourceDir, targetDir, { recursive: true, dereference: true });
  }

  console.log(`[bundle-openclaw] done. copied ${stats.copied} package directories to ${OUTPUT_ROOT}`);
}

main();
