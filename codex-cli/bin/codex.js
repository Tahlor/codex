#!/usr/bin/env node
// Unified entry point for the Codex CLI.

import { spawn } from "node:child_process";
import { existsSync } from "fs";
import { createRequire } from "node:module";
import path from "path";
import { fileURLToPath } from "url";

// __dirname equivalent in ESM
const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const require = createRequire(import.meta.url);

const PLATFORM_PACKAGE_BY_TARGET = {
  "x86_64-unknown-linux-musl": "codexx-linux-x64",
  "aarch64-unknown-linux-musl": "codexx-linux-arm64",
  "x86_64-apple-darwin": "codexx-darwin-x64",
  "aarch64-apple-darwin": "codexx-darwin-arm64",
  "x86_64-pc-windows-msvc": "codexx-win32-x64",
  "aarch64-pc-windows-msvc": "codexx-win32-arm64",
};

const { platform, arch } = process;

function targetTripleFor(platformName, architecture) {
  switch (platformName) {
    case "linux":
    case "android":
      switch (architecture) {
        case "x64":
          return "x86_64-unknown-linux-musl";
        case "arm64":
          return "aarch64-unknown-linux-musl";
        default:
          return null;
      }
    case "darwin":
      switch (architecture) {
        case "x64":
          return "x86_64-apple-darwin";
        case "arm64":
          return "aarch64-apple-darwin";
        default:
          return null;
      }
    case "win32":
      switch (architecture) {
        case "x64":
          return "x86_64-pc-windows-msvc";
        case "arm64":
          return "aarch64-pc-windows-msvc";
        default:
          return null;
      }
    default:
      return null;
  }
}

function isWsl() {
  if (platform !== "linux") {
    return false;
  }
  if (process.env.WSL_DISTRO_NAME || process.env.WSL_INTEROP) {
    return true;
  }
  try {
    return existsSync("/proc/sys/fs/binfmt_misc/WSLInterop");
  } catch {
    return false;
  }
}

function isWslWindowsMountPath(value) {
  return /^\/mnt\/[a-z](?:\/|$)/i.test(value);
}

function wslWindowsTargetTriple() {
  if (!isWsl() || !isWslWindowsMountPath(__dirname)) {
    return null;
  }
  return targetTripleFor("win32", arch);
}

const targetTriple = wslWindowsTargetTriple() ?? targetTripleFor(platform, arch);

if (!targetTriple) {
  throw new Error(`Unsupported platform: ${platform} (${arch})`);
}

const platformPackage = PLATFORM_PACKAGE_BY_TARGET[targetTriple];
if (!platformPackage) {
  throw new Error(`Unsupported target triple: ${targetTriple}`);
}

const targetIsWindows = targetTriple.endsWith("-pc-windows-msvc");
const codexBinaryName = targetIsWindows ? "codex.exe" : "codex";
const localVendorRoot = path.join(__dirname, "..", "vendor");
const localBinaryPath = path.join(
  localVendorRoot,
  targetTriple,
  "codex",
  codexBinaryName,
);
const checkoutDebugRoot = path.join(
  __dirname,
  "..",
  "..",
  "codex-rs",
  "target",
  "debug",
);
const checkoutDebugBinaryPath = path.join(checkoutDebugRoot, codexBinaryName);
const checkoutDebugDepsRoot = path.join(checkoutDebugRoot, "deps");
const checkoutDebugDepsBinaryPath = path.join(
  checkoutDebugDepsRoot,
  codexBinaryName,
);
const checkoutActiveDebugRoot = path.join(
  __dirname,
  "..",
  "..",
  "codex-rs",
  "target",
  "codexx-active",
  "debug",
);
const checkoutActiveDebugBinaryPath = path.join(
  checkoutActiveDebugRoot,
  codexBinaryName,
);
const additionalDirs = [];

let binaryPath;
if (existsSync(checkoutActiveDebugBinaryPath)) {
  binaryPath = checkoutActiveDebugBinaryPath;
  additionalDirs.push(checkoutActiveDebugRoot);
  const localPathDir = path.join(localVendorRoot, targetTriple, "path");
  if (existsSync(localPathDir)) {
    additionalDirs.push(localPathDir);
  }
} else if (existsSync(checkoutDebugDepsBinaryPath)) {
  binaryPath = checkoutDebugDepsBinaryPath;
  additionalDirs.push(checkoutDebugDepsRoot, checkoutDebugRoot);
  const localPathDir = path.join(localVendorRoot, targetTriple, "path");
  if (existsSync(localPathDir)) {
    additionalDirs.push(localPathDir);
  }
} else if (existsSync(checkoutDebugBinaryPath)) {
  binaryPath = checkoutDebugBinaryPath;
  additionalDirs.push(checkoutDebugRoot);
  const localPathDir = path.join(localVendorRoot, targetTriple, "path");
  if (existsSync(localPathDir)) {
    additionalDirs.push(localPathDir);
  }
} else {
  let vendorRoot;
  try {
    const packageJsonPath = require.resolve(`${platformPackage}/package.json`);
    vendorRoot = path.join(path.dirname(packageJsonPath), "vendor");
  } catch {
    if (existsSync(localBinaryPath)) {
      vendorRoot = localVendorRoot;
    } else {
      const packageManager = detectPackageManager();
      const updateCommand =
        packageManager === "bun"
          ? "bun install -g codexx@latest"
          : "npm install -g codexx@latest";
      throw new Error(
        `Missing optional dependency ${platformPackage}. Reinstall Codex: ${updateCommand}`,
      );
    }
  }

  if (!vendorRoot) {
    const packageManager = detectPackageManager();
    const updateCommand =
      packageManager === "bun"
        ? "bun install -g codexx@latest"
        : "npm install -g codexx@latest";
    throw new Error(
      `Missing optional dependency ${platformPackage}. Reinstall Codex: ${updateCommand}`,
    );
  }

  const archRoot = path.join(vendorRoot, targetTriple);
  binaryPath = path.join(archRoot, "codex", codexBinaryName);
  const pathDir = path.join(archRoot, "path");
  if (existsSync(pathDir)) {
    additionalDirs.push(pathDir);
  }
}

// Use an asynchronous spawn instead of spawnSync so that Node is able to
// respond to signals (e.g. Ctrl-C / SIGINT) while the native binary is
// executing. This allows us to forward those signals to the child process
// and guarantees that when either the child terminates or the parent
// receives a fatal signal, both processes exit in a predictable manner.

function getUpdatedPath(newDirs) {
  const pathSep = process.platform === "win32" ? ";" : ":";
  const existingPath = process.env.PATH || "";
  const updatedPath = [
    ...newDirs,
    ...existingPath.split(pathSep).filter(Boolean),
  ].join(pathSep);
  return updatedPath;
}

/**
 * Use heuristics to detect the package manager that was used to install Codex
 * in order to give the user a hint about how to update it.
 */
function detectPackageManager() {
  const userAgent = process.env.npm_config_user_agent || "";
  if (/\bbun\//.test(userAgent)) {
    return "bun";
  }

  const execPath = process.env.npm_execpath || "";
  if (execPath.includes("bun")) {
    return "bun";
  }

  if (
    __dirname.includes(".bun/install/global") ||
    __dirname.includes(".bun\\install\\global")
  ) {
    return "bun";
  }

  return userAgent ? "npm" : null;
}

const updatedPath = getUpdatedPath(additionalDirs);

const env = { ...process.env, PATH: updatedPath };
const packageManagerEnvVar =
  detectPackageManager() === "bun"
    ? "CODEX_MANAGED_BY_BUN"
    : "CODEX_MANAGED_BY_NPM";
env[packageManagerEnvVar] = "1";

function launcherModeArgs(userArgs) {
  if (userArgs.some((arg) => arg === "--mode" || arg.startsWith("--mode="))) {
    return [];
  }

  const wrapperMode = process.env.CODEX_WRAPPER_MODE;
  if (wrapperMode === "v5" || wrapperMode === "v6" || wrapperMode === "v7") {
    return ["--mode", wrapperMode];
  }

  const launcher = process.argv[1] || "";
  const launcherName = path.basename(launcher, path.extname(launcher));
  if (launcherName === "codex-v5" || launcherName === "codexx-v5") {
    return ["--mode", "v5"];
  }
  if (launcherName === "codex-v6" || launcherName === "codexx-v6") {
    return ["--mode", "v6"];
  }
  if (launcherName === "codex-v7" || launcherName === "codexx-v7") {
    return ["--mode", "v7"];
  }
  return [];
}

const userArgs = process.argv.slice(2);
const child = spawn(binaryPath, [...launcherModeArgs(userArgs), ...userArgs], {
  stdio: "inherit",
  env,
});

child.on("error", (err) => {
  // Typically triggered when the binary is missing or not executable.
  // Re-throwing here will terminate the parent with a non-zero exit code
  // while still printing a helpful stack trace.
  // eslint-disable-next-line no-console
  console.error(err);
  process.exit(1);
});

// Forward common termination signals to the child so that it shuts down
// gracefully. In the handler we temporarily disable the default behavior of
// exiting immediately; once the child has been signaled we simply wait for
// its exit event which will in turn terminate the parent (see below).
const forwardSignal = (signal) => {
  if (child.killed) {
    return;
  }
  try {
    child.kill(signal);
  } catch {
    /* ignore */
  }
};

["SIGINT", "SIGTERM", "SIGHUP"].forEach((sig) => {
  process.on(sig, () => forwardSignal(sig));
});

// When the child exits, mirror its termination reason in the parent so that
// shell scripts and other tooling observe the correct exit status.
// Wrap the lifetime of the child process in a Promise so that we can await
// its termination in a structured way. The Promise resolves with an object
// describing how the child exited: either via exit code or due to a signal.
const childResult = await new Promise((resolve) => {
  child.on("exit", (code, signal) => {
    if (signal) {
      resolve({ type: "signal", signal });
    } else {
      resolve({ type: "code", exitCode: code ?? 1 });
    }
  });
});

if (childResult.type === "signal") {
  // Re-emit the same signal so that the parent terminates with the expected
  // semantics (this also sets the correct exit code of 128 + n).
  process.kill(process.pid, childResult.signal);
} else {
  process.exit(childResult.exitCode);
}
