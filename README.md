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
pnpm tauri dev
```

Checks run in CI:

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cd app && pnpm lint && pnpm typecheck && pnpm test
```

## License

[MIT](LICENSE)
