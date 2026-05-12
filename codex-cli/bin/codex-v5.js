#!/usr/bin/env node
process.env.CODEX_WRAPPER_MODE = "v5";
await import("./codex.js");
