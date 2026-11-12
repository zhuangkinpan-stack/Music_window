// 把 tauri build 产出的 NSIS 安装包按版本号收集到 artifacts/，避免手工复制。
import { copyFileSync, mkdirSync, readdirSync, readFileSync, statSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = dirname(dirname(fileURLToPath(import.meta.url)));
const bundleDir = join(root, "src-tauri", "target", "release", "bundle", "nsis");
const artifactsDir = join(root, "artifacts");

const version = JSON.parse(
  readFileSync(join(root, "src-tauri", "tauri.conf.json"), "utf8"),
).version;

let candidates = [];
try {
  candidates = readdirSync(bundleDir)
    .filter((name) => name.toLowerCase().endsWith(".exe"))
    .map((name) => join(bundleDir, name))
    .sort((a, b) => statSync(b).mtimeMs - statSync(a).mtimeMs);
} catch {
  console.error(`未找到安装包目录：${bundleDir}`);
  process.exit(1);
}

if (candidates.length === 0) {
  console.error(`安装包目录为空：${bundleDir}`);
  process.exit(1);
}

const suffix = process.argv.includes("--offline") ? "-offline" : "";
const source = candidates[0];
const target = join(artifactsDir, `流光_${version}_x64${suffix}-setup.exe`);

mkdirSync(artifactsDir, { recursive: true });
copyFileSync(source, target);
console.log(`已收集安装包：${target}`);
