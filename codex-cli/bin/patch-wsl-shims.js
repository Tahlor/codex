#!/usr/bin/env node

import { existsSync, readFileSync, statSync, writeFileSync } from "fs";
import { dirname, join } from "path";
import { fileURLToPath } from "url";

const COMMANDS = [
  "codexx",
  "codexxx",
  "codex-v5",
  "codex-v6",
  "codex-v7",
  "codexx-v5",
  "codexx-v6",
  "codexx-v7",
];
const MARKER = "# Codexx WSL interop";

if (process.platform === "win32") {
  patchWindowsShims();
}

function patchWindowsShims() {
  const scriptPath = fileURLToPath(import.meta.url);
  const packageRoot = dirname(dirname(scriptPath));
  const shimDirs = candidateShimDirs(packageRoot);

  for (const shimDir of shimDirs) {
    for (const command of COMMANDS) {
      patchShim(join(shimDir, command));
    }
  }
}

function candidateShimDirs(packageRoot) {
  const candidates = new Set();
  addIfSet(candidates, process.env.npm_config_prefix);
  addIfSet(
    candidates,
    process.env.npm_config_prefix &&
      join(process.env.npm_config_prefix, "node_modules", ".bin"),
  );
  addIfSet(
    candidates,
    process.env.npm_config_local_prefix &&
      join(process.env.npm_config_local_prefix, "node_modules", ".bin"),
  );
  addIfSet(candidates, join(packageRoot, "..", ".bin"));
  return [...candidates];
}

function addIfSet(candidates, value) {
  if (value) {
    candidates.add(value);
  }
}

function patchShim(shimPath) {
  if (!existsSync(shimPath) || !isFile(shimPath)) {
    return;
  }

  const content = readFileSync(shimPath, "utf8");
  if (!content.startsWith("#!/bin/sh\n") || content.includes(MARKER)) {
    return;
  }

  const lines = content.split(/\r?\n/);
  const fallbackIndex = lines.findIndex((line) =>
    /^\s*exec node\s+/.test(line),
  );
  if (fallbackIndex === -1) {
    return;
  }

  const targetMatch = lines[fallbackIndex].match(/"\$basedir\/([^"]+)"/);
  if (!targetMatch) {
    return;
  }

  const nodeCheckIndex = lines.findIndex(
    (line) => line.includes("[ -x") && line.includes("$basedir/node"),
  );
  if (nodeCheckIndex === -1 || nodeCheckIndex > fallbackIndex) {
    return;
  }

  const targetRelativePath = targetMatch[1];
  const helperLines = [
    MARKER,
    "is_codexx_wsl() {",
    '  if [ -n "${WSL_DISTRO_NAME-}" ] || [ -n "${WSL_INTEROP-}" ]; then',
    "    return 0",
    "  fi",
    '  case "$(uname -r 2>/dev/null || true) $(uname -v 2>/dev/null || true)" in',
    "    *[Mm]icrosoft*|*[Ww][Ss][Ll]*) return 0 ;;",
    "  esac",
    "  return 1",
    "}",
    "",
  ];
  lines.splice(nodeCheckIndex, 0, ...helperLines);

  const adjustedFallbackIndex = fallbackIndex + helperLines.length;
  const wslExecLines = [
    "  if is_codexx_wsl && command -v wslpath >/dev/null 2>&1 && command -v node.exe >/dev/null 2>&1; then",
    '    basedir_win="$(wslpath -w "$basedir" 2>/dev/null || true)"',
    '    if [ -n "$basedir_win" ]; then',
    `      exec node.exe "$basedir_win/${targetRelativePath}" "$@"`,
    "    fi",
    "  fi",
  ];
  lines.splice(adjustedFallbackIndex, 0, ...wslExecLines);

  writeFileSync(shimPath, lines.join("\n"), "utf8");
}

function isFile(path) {
  try {
    return statSync(path).isFile();
  } catch {
    return false;
  }
}
