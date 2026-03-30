import fs from 'node:fs/promises';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
import sharp from 'sharp';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const projectRoot = path.resolve(__dirname, '..');

const outputDir = path.join(projectRoot, 'src-tauri', 'resources', 'installer', 'windows');
const logoPath = path.join(projectRoot, 'images', 'DragonClaw-logo.png');
const headerPngPath = path.join(outputDir, 'header.png');
const sidebarPngPath = path.join(outputDir, 'sidebar.png');

async function ensureDir(target) {
  await fs.mkdir(target, { recursive: true });
}

function headerSvg() {
  return `
<svg width="150" height="57" viewBox="0 0 150 57" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#0F172A" />
      <stop offset="62%" stop-color="#14532D" />
      <stop offset="100%" stop-color="#166534" />
    </linearGradient>
    <linearGradient id="line" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0%" stop-color="#34D399" stop-opacity="0.1" />
      <stop offset="100%" stop-color="#A7F3D0" stop-opacity="0.8" />
    </linearGradient>
  </defs>
  <rect x="0" y="0" width="150" height="57" rx="4" fill="url(#bg)" />
  <circle cx="132" cy="12" r="28" fill="#A7F3D0" fill-opacity="0.14" />
  <circle cx="146" cy="42" r="22" fill="#34D399" fill-opacity="0.11" />
  <rect x="44" y="39" width="94" height="1.5" rx="0.75" fill="url(#line)" />
  <text x="44" y="23" fill="#ECFDF5" font-size="12" font-weight="700" font-family="Segoe UI, Microsoft YaHei, sans-serif">DragonClaw</text>
  <text x="44" y="35" fill="#BBF7D0" font-size="8" font-weight="500" font-family="Segoe UI, Microsoft YaHei, sans-serif">AI Assistant Installer</text>
</svg>
`;
}

function sidebarSvg() {
  return `
<svg width="164" height="314" viewBox="0 0 164 314" xmlns="http://www.w3.org/2000/svg">
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0%" stop-color="#052E16" />
      <stop offset="45%" stop-color="#14532D" />
      <stop offset="100%" stop-color="#0F172A" />
    </linearGradient>
    <linearGradient id="accent" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0%" stop-color="#6EE7B7" stop-opacity="0.8" />
      <stop offset="100%" stop-color="#22C55E" stop-opacity="0.15" />
    </linearGradient>
  </defs>
  <rect x="0" y="0" width="164" height="314" fill="url(#bg)" />
  <circle cx="130" cy="38" r="48" fill="#10B981" fill-opacity="0.18" />
  <circle cx="25" cy="254" r="72" fill="#34D399" fill-opacity="0.12" />
  <rect x="14" y="188" width="136" height="2" rx="1" fill="url(#accent)" />

  <text x="16" y="214" fill="#ECFDF5" font-size="18" font-weight="700" font-family="Segoe UI, Microsoft YaHei, sans-serif">DragonClaw</text>
  <text x="16" y="236" fill="#D1FAE5" font-size="11" font-weight="500" font-family="Segoe UI, Microsoft YaHei, sans-serif">更聪明的桌面 AI 助手</text>

  <text x="16" y="268" fill="#A7F3D0" font-size="9" font-weight="500" font-family="Segoe UI, Microsoft YaHei, sans-serif">• 安装后自动创建快捷方式</text>
  <text x="16" y="283" fill="#A7F3D0" font-size="9" font-weight="500" font-family="Segoe UI, Microsoft YaHei, sans-serif">• 支持微信 / 飞书 / 企业微信等渠道</text>
  <text x="16" y="298" fill="#A7F3D0" font-size="9" font-weight="500" font-family="Segoe UI, Microsoft YaHei, sans-serif">• 默认使用中文安装向导</text>
</svg>
`;
}

async function generate() {
  await ensureDir(outputDir);

  const logoHeader = await sharp(logoPath)
    .resize(28, 28, { fit: 'cover' })
    .png()
    .toBuffer();

  const logoSidebar = await sharp(logoPath)
    .resize(108, 108, { fit: 'cover' })
    .png()
    .toBuffer();

  await sharp(Buffer.from(headerSvg()))
    .composite([{ input: logoHeader, left: 10, top: 14 }])
    .png()
    .toFile(headerPngPath);

  await sharp(Buffer.from(sidebarSvg()))
    .composite([
      { input: logoSidebar, left: 28, top: 56 },
      {
        input: Buffer.from(
          `<svg width="164" height="314" xmlns="http://www.w3.org/2000/svg"><circle cx="82" cy="110" r="58" fill="#bbf7d0" fill-opacity="0.08" /></svg>`
        ),
        left: 0,
        top: 0
      }
    ])
    .png()
    .toFile(sidebarPngPath);

  console.log(`Generated NSIS assets:\n- ${headerPngPath}\n- ${sidebarPngPath}`);
}

generate().catch((error) => {
  console.error(error);
  process.exit(1);
});
