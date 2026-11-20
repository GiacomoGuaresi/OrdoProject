# Distribuzione e packaging

## Formati per piattaforma (generati da `tauri-bundler`)
| OS | Formato | Note |
|---|---|---|
| macOS | `.dmg` contenente `.app`, una build per Apple Silicon e una per Intel | CLI inclusa nel bundle |
| Windows | Installer **NSIS**, installazione per utente (senza admin) | Il menu contestuale si attiva dall'app (HKCU) |
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
- macOS: due build separate (`aarch64` e `x86_64`) invece di una universal: più semplice con la CLI
  inclusa come binario esterno (una universal richiederebbe anche una CLI universal via `lipo`).

### Icone
- Sorgente: `assets/icon.png` (logo su sfondo trasparente) → icone Windows/Linux (`pnpm tauri icon`).
- macOS: `assets/icon-macos.png`, il logo su un riquadro scuro arrotondato secondo la griglia
  delle icone Apple, generato con `python3 assets/make_macos_icon.py`; il `.icns` risultante è in
  `app/src-tauri/icons/macos/` ed è usato solo da `tauri.macos.conf.json`.

### Dettagli del packaging
- La CLI `ordo` è inclusa come *external binary* di Tauri, accanto all'eseguibile dell'app
  (`ordo-app`): `Contents/MacOS/` su macOS, cartella di installazione su Windows, `/usr/bin` nel `.deb`.
- La configurazione di bundle è in `app/src-tauri/tauri.bundle.conf.json` (usata da `pnpm bundle`
  e dal workflow di release): compila la CLI in release (`app/scripts/prepare-cli.mjs`).
- Nome dell'eseguibile dell'app fissato a `ordo-app` (`mainBinaryName`): evita la collisione
  `Ordo`/`ordo` sui file system case-insensitive di macOS.
- AppImage: il comando `ordo` installato dalle Impostazioni punta dentro il mount temporaneo
  dell'AppImage e smette di funzionare quando l'app si chiude; per l'uso da terminale su Linux
  è consigliato il pacchetto `.deb`. (Da migliorare.)

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
