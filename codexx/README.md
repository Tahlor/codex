# Codexx Workspace

This directory is the fork-owned workspace for Codexx notes, helper scripts,
and deployment material. Keep upstream OpenAI source, docs, and workflows in
their normal locations unless a product change has to touch them.

## Layout

- `docs/`: Codexx requirements, implementation notes, and merge workflow docs.
- `scripts/`: fork maintenance helpers that are not upstream source scripts.
- `deployment/`: publishing and install notes for source, npm, and release
  artifact paths.

Use this folder for future fork-only planning and operations material. That
keeps the root and upstream-owned directories quieter when merging new OpenAI
release tags.

## Current Entry Points

- [Change requirements](./docs/change-requirements.md)
- [Single-repo workflow](./docs/single-repo-workflow.md)
- [Release and merge notes](./docs/release-and-merge.md)
- [Deployment notes](./deployment/README.md)
