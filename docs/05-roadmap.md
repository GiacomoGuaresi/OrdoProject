# Roadmap

Ogni milestone produce qualcosa di utilizzabile e rilasciabile.

## M0 — Preparazione repository
- Tag `v1-electron` sullo stato attuale di `master`.
- Branch `v2`; rimozione codice Electron.
- Scaffolding: Cargo workspace, `crates/ordo-core`, `crates/ordo-cli`, `app/` (Tauri 2 + React + TS + pnpm).
- `LICENSE` (MIT), README aggiornato, CI di base (fmt, clippy, test sulle 3 piattaforme).

## M1 — `ordo-core` + CLI
- Modello regole (nome, enabled, glob/regex, case-sensitive opzionale, destinazione) + persistenza JSON.
- Regole di default con cartelle standard OS.
- Matching (first match wins), esclusione file nascosti/di sistema, solo primo livello.
- Spostamento robusto (rename → fallback copia+verifica+rimozione cross-volume), gestione volume assente.
- Conflitti: keep-both / overwrite (esistente → cestino) / skip.
- Cronologia SQLite (batch + spostamenti, stati), retention, undo batch/singolo con casi limite.
- CLI: `sort`, `undo`, `history`, `rules`, prompt TTY e `--on-conflict`, `--json`, exit code.
- i18n EN/IT dei messaggi.
- Test estesi su core e CLI.

## M2 — App Tauri
- Sezione **Regole** (CRUD, enable/disable, drag & drop, import/export).
- Sezione **Cronologia** (batch, dettaglio, undo, mostra nella cartella).
- Sezione **Impostazioni** (tema, lingua, retention, svuota cronologia, controllo aggiornamenti).
- Tema chiaro/scuro, i18n EN/IT.
- Banner nuova versione da GitHub Releases.

## M3 — Integrazione menu contestuale
- Modalità `--sort` dell'app: avvio senza finestra, single-instance + aggregazione.
- Dialog conflitti (affiancati, miniature, "identici", anteprima/Quick Look, applica a tutti).
- Notifica di riepilogo con "Annulla" (processo vivo ~10 s) e click → cronologia.
- Installazione/rimozione da Impostazioni e da `ordo integration`:
  macOS Quick Actions, Windows registro HKCU, Linux Nautilus + Dolphin.
- "Installa comando `ordo`" nel PATH.

## M4 — Distribuzione
- Bundle: `.dmg` (macOS), NSIS/MSI (Windows), AppImage + `.deb` (Linux), CLI inclusa.
- Workflow release su tag `v*` → GitHub Release in bozza.
- Onboarding primo avvio.
- Istruzioni per app non firmata; predisposizione firma futura.

## Fuori scope (per ora)
- Cartelle sorvegliate / automazione in background
- Condizioni su dimensione/data/MIME, destinazioni dinamiche, azioni diverse da "sposta"
- Profili multipli di regole, tester regole, dry run
- Firma del codice, auto-update, package manager
- Finder Sync Extension, menu moderno Windows 11, Nemo/Caja/Thunar
- Migrazione dati dalla v1
