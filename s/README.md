<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->

# Project scripts

The `s/` directory holds the repeated project workflows. Run these from anywhere inside the repository - each script moves to the repo root before doing work.

## `s/test`

Runs the local CI gate for the Rust crate:

```bash
s/test
```

This checks formatting, clippy with default and MCP features, and tests default, leaf (`--no-default-features`), and MCP builds. Use this before commits that touch Rust code.

## `s/docs`

Serves the documentation site locally with hot reload:

```bash
s/docs
```

First-time setup:

```bash
python3 -m venv .venv
./.venv/bin/pip install -r requirements.txt
```

The script picks the first free port in `8000-8030`, prints the chosen URL, and opens the browser unless you pass your own Zensical arguments.

## `s/install`

Installs the `clincalc` binary from the current checkout:

```bash
s/install --force
```

This wraps `cargo install --locked --path .` and forwards any extra `cargo install` flags.

## `s/gui-dev`

Runs the Tauri desktop GUI in development mode:

```bash
s/gui-dev
```

First-time setup:

```bash
cd gui
npm install
```

You also need the Tauri system prerequisites for your operating system.

## `s/version++`

Cuts a release bump using the house-style CI auto-tag cascade:

```bash
s/version++ patch
s/version++ minor --pr
s/version++ major --auto-merge
```

The script requires a clean `main`, runs the pre-release Rust gate, bumps the Rust crate and GUI version-bearing files, regenerates `CHANGELOG.md`, commits `chore(release): vX.Y.Z`, and lands that commit on `main` directly or via a release PR depending on branch protection. The `auto-tag.yml` workflow then creates the tag and invokes the release and crates.io publish workflows.
