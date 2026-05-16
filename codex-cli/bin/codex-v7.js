#!/usr/bin/env node
process.env.CODEX_WRAPPER_MODE = "v7";
await import("./codex.js");
