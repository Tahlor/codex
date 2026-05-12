#!/usr/bin/env node
process.env.CODEX_WRAPPER_MODE = "v6";
await import("./codex.js");
