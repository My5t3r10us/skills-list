<p align="center">
  <img src="src-tauri/icons/icon.ico" width="72" alt="skills-list icon" />
</p>

# skills-list

*Catalog, group, export, and install Codex skills from a desktop app or CLI.*

![Rust](https://img.shields.io/badge/Rust-2021-b7410e?style=flat-square&logo=rust&logoColor=white)
![TypeScript](https://img.shields.io/badge/TypeScript-5.6-3178c6?style=flat-square&logo=typescript&logoColor=white)
![Tauri](https://img.shields.io/badge/Tauri-v2-24c8db?style=flat-square&logo=tauri&logoColor=white)
![Vite](https://img.shields.io/badge/Vite-6-646cff?style=flat-square&logo=vite&logoColor=white)

[Features](#features) - [Getting started](#getting-started) - [CLI usage](#cli-usage) - [Skill format](#skill-format) - [Data layout](#data-layout)

`skills-list` is a lightweight Tauri v2 app backed by a shared Rust core. It keeps a local catalog of Codex skills, lets you organize them into installable groups, and copies selected skills into a project's `.agents/skills` folder.

The same catalog is available through the desktop interface and the `skills-list` CLI, so you can browse and curate visually, then automate the same workflows from a terminal.

## Features

- **Local skill catalog** - Import one skill folder or recursively discover many folders containing `SKILL.md`.
- **Command skills** - Save external install commands beside local skills for tools managed elsewhere.
- **Groups** - Combine multiple skills into named starter packs and install them together.
- **Project installation** - Install to `<project>/.agents/skills` by default, or override the target directory.
- **Export support** - Copy a local skill from the catalog back out to another folder.
- **Shared engine** - The Tauri app and CLI both use the `skills-core` Rust crate.
- **Web preview mode** - Run the Vite UI outside Tauri with demo data while iterating on the frontend.

> [!IMPORTANT]
> Command skills can execute arbitrary shell commands. The desktop app opens command skills in PowerShell so interactive prompts stay visible. The CLI previews commands before running them; use `--yes` only when you trust the catalog entry.

## Getting started

### Prerequisites

- [Node.js LTS](https://nodejs.org/) and npm
- [Rust](https://www.rust-lang.org/tools/install)
- Tauri v2 system dependencies for your platform

### Install dependencies

```bash
npm install
```

### Run the desktop app

```bash
npm run tauri dev
```

This starts the Vite frontend and opens the Tauri shell with filesystem access for importing, exporting, and installing skills.

### Run the web preview

```bash
npm run dev
```

Open the local Vite URL in your browser. In this mode, the UI uses demo catalog data and system-level actions are disabled.

### Build

```bash
npm run build
npm run tauri build
```

The Windows `.exe` installer installs both the desktop app and the
`skills-list` CLI. It adds the app install directory to the current user's
`PATH`, so open a new terminal after installation before running
`skills-list`.

During development, you can also install the CLI directly:

```bash
npm run cli:install
```

This installs `skills-list.exe` into Cargo's bin directory, usually
`%USERPROFILE%\.cargo\bin`.

### Test

```bash
cargo test
```

## CLI usage

During development, run the CLI through Cargo:

```bash
cargo run -p skills-cli -- <command>
```

Search the catalog:

```bash
cargo run -p skills-cli -- search tauri
cargo run -p skills-cli -- search tauri --json
```

Import skills:

```bash
cargo run -p skills-cli -- import C:\path\to\skill-or-skills-folder
```

Add a command skill:

```bash
cargo run -p skills-cli -- add-command "Team Bootstrap" \
  --description "Install shared skills" \
  --command "codex skill install team-bootstrap" \
  --tag onboarding
```

Create and manage groups:

```bash
cargo run -p skills-cli -- group create Starter --description "Base project skills"
cargo run -p skills-cli -- group add starter tauri-v2
cargo run -p skills-cli -- group remove starter tauri-v2
cargo run -p skills-cli -- group delete starter
```

Install one skill or a group:

```bash
cargo run -p skills-cli -- install skill tauri-v2 --project C:\path\to\project
cargo run -p skills-cli -- install group starter --project C:\path\to\project --overwrite
```

Command skills run in the current terminal when installed from the CLI. If the
underlying command asks for input, answer it directly in PowerShell. The `--yes`
flag only skips the skills-list confirmation; it does not answer prompts from
the underlying command.

Use a portable or test catalog:

```bash
cargo run -p skills-cli -- --data-dir C:\path\to\catalog search rust
```

You can also set `SKILLS_LIST_DATA_DIR` instead of passing `--data-dir`.

## Skill format

Each local skill is a folder containing a `SKILL.md` file. Optional YAML frontmatter is used to populate catalog metadata:

```markdown
---
name: Tauri v2
description: Build and debug Tauri desktop apps
version: 1.0.0
tags:
  - tauri
  - rust
  - desktop
---

# Tauri v2
```

If a field is missing, `skills-list` falls back to a sensible default: the folder name for `name`, no description, no version, and no tags.

## Data layout

The app and CLI store the catalog in the same platform data directory by default
using the app identifier `dev.skillslist.app`:

```text
catalog.json
library/
  skills/
    <skill-id>/
      SKILL.md
```

Local installations copy skill folders into the target project:

```text
<project>/
  .agents/
    skills/
      <skill-id>/
        SKILL.md
```

> [!NOTE]
> Skill and group references accept either the generated id or the exact name, case-insensitively. Generated ids are slugified from names, with suffixes like `-2` added when needed.

## Project structure

```text
crates/
  skills-core/    Shared catalog, import, export, group, and install logic
  skills-cli/     Command-line interface
src/              Vite/TypeScript frontend
src-tauri/        Tauri shell and command bindings
```
