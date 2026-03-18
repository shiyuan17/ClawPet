#!/usr/bin/env node
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(__dirname, '..');
const NODE_MODULES = path.join(ROOT, 'node_modules');
const OUTPUT_ROOT = path.join(ROOT, 'src-tauri', 'resources', 'openclaw-plugins');

const SKIP_PACKAGES = new Set(['typescript', '@playwright/test']);
const PLUGINS = [
  { npmName: '@soimy/dingtalk', pluginId: 'dingtalk' },
  { npmName: '@wecom/wecom-openclaw-plugin', pluginId: 'wecom' },
  { npmName: '@sliverp/qqbot', pluginId: 'qqbot' },
  { npmName: '@larksuite/openclaw-lark', pluginId: 'feishu-openclaw-plugin' },
];

function cleanDir(dir) {
  fs.rmSync(dir, { recursive: true, force: true });
  fs.mkdirSync(dir, { recursive: true });
}

function readPackageJson(pkgDir) {
  const pkgPath = path.join(pkgDir, 'package.json');
  if (!fs.existsSync(pkgPath)) return null;
  try {
    return JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
  } catch {
    return null;
  }
}

function resolvePackageDir(pkgName, parentPkgDir) {
  const parentNodeModules = path.join(parentPkgDir, 'node_modules', ...pkgName.split('/'));
  if (fs.existsSync(parentNodeModules)) return parentNodeModules;

  const rootNodeModules = path.join(NODE_MODULES, ...pkgName.split('/'));
  if (fs.existsSync(rootNodeModules)) return rootNodeModules;

  return null;
}

function copyPackage(pkgDir, pkgName, outputNodeModules, copiedByName) {
  if (copiedByName.has(pkgName)) return;
  copiedByName.add(pkgName);

  const dest = path.join(outputNodeModules, ...pkgName.split('/'));
  fs.mkdirSync(path.dirname(dest), { recursive: true });
  fs.cpSync(pkgDir, dest, { recursive: true, dereference: true });
}

function collectDepsRecursively(pkgDir, outputNodeModules, copiedByName, visitedRealDirs) {
  let realDir;
  try {
    realDir = fs.realpathSync(pkgDir);
  } catch {
    realDir = pkgDir;
  }

  if (visitedRealDirs.has(realDir)) return;
  visitedRealDirs.add(realDir);

  const pkg = readPackageJson(pkgDir);
  if (!pkg) return;

  const deps = {
    ...(pkg.dependencies || {}),
    ...(pkg.optionalDependencies || {}),
  };

  for (const pkgName of Object.keys(deps)) {
    if (SKIP_PACKAGES.has(pkgName) || pkgName.startsWith('@types/')) continue;

    const depDir = resolvePackageDir(pkgName, pkgDir);
    if (!depDir) continue;

    copyPackage(depDir, pkgName, outputNodeModules, copiedByName);
    collectDepsRecursively(depDir, outputNodeModules, copiedByName, visitedRealDirs);
  }
}

function patchManifestIdIfNeeded(pluginDir, pluginId) {
  const manifestPath = path.join(pluginDir, 'openclaw.plugin.json');
  if (!fs.existsSync(manifestPath)) return;

  try {
    const parsed = JSON.parse(fs.readFileSync(manifestPath, 'utf8'));
    if (pluginId === 'wecom' && parsed.id === 'wecom-openclaw-plugin') {
      parsed.id = 'wecom';
      fs.writeFileSync(manifestPath, `${JSON.stringify(parsed, null, 2)}\n`, 'utf8');
    }
  } catch {
    // ignore
  }
}

function bundleOnePlugin(plugin) {
  const sourceDir = path.join(NODE_MODULES, ...plugin.npmName.split('/'));
  if (!fs.existsSync(sourceDir)) {
    console.warn(`[bundle-openclaw-plugins] skip missing plugin: ${plugin.npmName}`);
    return;
  }

  const outputDir = path.join(OUTPUT_ROOT, plugin.pluginId);
  fs.rmSync(outputDir, { recursive: true, force: true });
  fs.mkdirSync(outputDir, { recursive: true });
  fs.cpSync(sourceDir, outputDir, { recursive: true, dereference: true });

  const outputNodeModules = path.join(outputDir, 'node_modules');
  fs.mkdirSync(outputNodeModules, { recursive: true });

  const copiedByName = new Set();
  const visitedRealDirs = new Set();
  collectDepsRecursively(sourceDir, outputNodeModules, copiedByName, visitedRealDirs);
  patchManifestIdIfNeeded(outputDir, plugin.pluginId);

  console.log(`[bundle-openclaw-plugins] ${plugin.pluginId}: copied ${copiedByName.size} dependency packages`);
}

function main() {
  console.log('[bundle-openclaw-plugins] bundling plugin mirrors...');
  cleanDir(OUTPUT_ROOT);

  for (const plugin of PLUGINS) {
    bundleOnePlugin(plugin);
  }

  console.log(`[bundle-openclaw-plugins] done -> ${OUTPUT_ROOT}`);
}

main();
