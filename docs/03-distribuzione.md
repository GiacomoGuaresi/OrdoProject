# Distribuzione e packaging

## Formati per piattaforma (generati da `tauri-bundler`)
| OS | Formato | Note |
|---|---|---|
| macOS | `.dmg` contenente `.app` | CLI inclusa nel bundle |
| Windows | Installer **NSIS** o **MSI** | L'installer può registrare il menu contestuale |
| Linux | **AppImage** + **.deb** | |

- Nessuna pubblicazione su package manager (Homebrew, winget, Flathub/AUR) per ora.
- Nessuna versione portable.

## CI/CD — GitHub Actions
- **Su ogni PR / push**: matrice `macos-latest`, `windows-latest`, `ubuntu-latest`
  - Rust: `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`
  - Frontend: lint (oxlint), typecheck (`tsc`), test (Vitest)
  - Build di verifica dell'app
- **Su tag `v*`**: build degli installer per le tre piattaforme (`tauri-action`)
  e creazione di una **GitHub Release in bozza** con tutti gli artefatti.
- macOS: build universal (Apple Silicon + Intel) — da confermare.

## Firma del codice
- **Non firmata** nella prima fase.
- README con istruzioni per il primo avvio:
  - macOS: click destro → Apri (oppure `xattr -dr com.apple.quarantine`);
  - Windows: SmartScreen → "Ulteriori informazioni" → "Esegui comunque".
- La pipeline di build va predisposta per aggiungere in futuro firma/notarizzazione
  (Apple Developer ID, Azure Trusted Signing o certificato OV) senza ristrutturazioni.

## Aggiornamenti
- All'avvio dell'**app grafica** (non da menu contestuale / CLI) controllo delle
  **GitHub Releases** per nuove versioni.
- Se disponibile: banner non invasivo con link alla pagina di download.
- Nessun auto-update automatico (eventuale `tauri-plugin-updater` in futuro).
- Controllo disattivabile dalle impostazioni.
