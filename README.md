# Ordo

<img src="assets/icon.png" width="160" alt="Ordo icon">

**Ordo** sorts files into folders using simple rules, straight from your file
manager's context menu or from the terminal. It runs on macOS, Windows and Linux.

> **Status:** v2 is being rewritten from scratch with Tauri 2 and Rust on the
> `v2` branch. The old Electron version is tagged
> [`v1-electron`](https://github.com/GiacomoGuaresi/OrdoProject/tree/v1-electron).

## How it works

- You keep **one ordered list of rules**. Each rule matches the file name with a
  **glob** (`*.{jpg,png}`) or a **regex** (`^invoice_\d{4}\.pdf$`) and moves the
  file to a fixed folder. The first rule that matches wins.
- Sorting runs only when you ask: right-click a file, a selection or a folder,
  or use the `ordo` command.
- Every run is saved in the **history**, and you can **undo** a whole run or a
  single move.
- When a file with the same name already exists, Ordo asks you whether to
  overwrite it, keep both or skip. Overwritten files go to the system trash.

## Install

Download the installer for your system from the
[Releases page](https://github.com/GiacomoGuaresi/OrdoProject/releases):

| System | File |
|---|---|
| macOS (Apple Silicon) | `Ordo_<version>_aarch64.dmg` |
| macOS (Intel) | `Ordo_<version>_x64.dmg` |
| Windows | `Ordo_<version>_x64-setup.exe` |
| Linux | `.AppImage` or `.deb` |

On first launch Ordo offers to add **Sort with Ordo** to your file manager's
context menu. You can add or remove it at any time in **Settings → File manager**,
together with the `ordo` terminal command.

### Opening Ordo the first time (unsigned builds)

The builds are not code-signed yet, so the system warns you once:

- **macOS**: right-click `Ordo.app` → **Open** → **Open**. Or, from a terminal:
  `xattr -dr com.apple.quarantine /Applications/Ordo.app`
- **Windows**: in the SmartScreen dialog click **More info** → **Run anyway**.
- **Linux (AppImage)**: make it executable (`chmod +x Ordo_*.AppImage`) and run it.

## Usage

**From the file manager**: select files or a folder, right-click and choose
**Sort with Ordo** (macOS: under *Quick Actions*; Windows 11: under *Show more
options*; Nautilus: under *Scripts*). Ordo asks only if a file with the same
name already exists, then shows a summary with **Undo**.

**From the terminal**:

```bash
ordo sort ~/Desktop                # sort the files on the desktop
ordo sort *.pdf --on-conflict skip # never overwrite, skip clashes
ordo history                       # recent runs
ordo undo                          # undo the latest run
ordo rules list                    # show the rules
ordo rules add --name Invoices --regex '^invoice_\d{4}\.pdf$' --dest ~/Documents/Invoices --position 1
ordo integration install           # add the context-menu entry
```

Exit codes: `0` success, `1` error, `2` wrong usage, `3` partial success.
Add `--json` to any command for machine-readable output.

## Repository layout

```
Cargo.toml          Cargo workspace
crates/ordo-core    library: rules, matching, moves, history, undo
crates/ordo-cli     the `ordo` command-line tool
app/                desktop app (React + TypeScript, pnpm)
app/src-tauri       Tauri crate
docs/               design docs (in Italian)
```

## Development

Requirements: Rust (stable), Node.js 20+, pnpm, and the
[Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) for your OS.

```bash
# Rust core and CLI
cargo test --workspace
cargo run -p ordo-cli -- --help

# Desktop app
cd app
pnpm install
pnpm tauri dev                       # ORDO_HOME=/tmp/ordo keeps your real data untouched
pnpm tauri dev -- -- --sort ~/Desktop # try the context-menu flow

# Installers for the current platform (app + bundled CLI)
pnpm bundle
```

TypeScript types shared with Rust live in `app/src/bindings` and are generated
with `pnpm bindings`.

Releases: push a tag such as `v2.0.0` (matching `app/package.json`); the
*Release* workflow builds every platform and creates a draft GitHub Release.

Checks run in CI:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && pnpm lint && pnpm typecheck && pnpm test
```

## License

[MIT](LICENSE)
