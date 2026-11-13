# Stack tecnologico

## Scelta principale: Tauri 2 + Rust

### Motivazioni
- **Avvio rapido**: ogni invocazione dal menu contestuale avvia un processo → serve startup minimo.
- **CLI nativa** che condivide lo stesso motore dell'app (crate Rust comune).
- **Binari leggeri** (~5–10 MB vs >150 MB di Electron).
- Accesso nativo a funzioni di sistema: cestino, notifiche, Quick Look / anteprima, dialog.

### Alternative scartate
| Opzione | Motivo dello scarto |
|---|---|
| Electron + TS | Pesante, avvio lento per ogni click da menu contestuale, CLI non nativa |
| Flutter | Supporto desktop e integrazioni di sistema meno mature |
| .NET + Avalonia | Binari più pesanti; nessun vantaggio decisivo rispetto a Tauri |

## Frontend (webview Tauri)
- **React + TypeScript**
- **shadcn/ui** (componenti Radix copiati nel repo) + **Tailwind CSS**
- Tema chiaro/scuro gestito via classi Tailwind, sincronizzato con il sistema.
- Tipi condivisi Rust → TS generati automaticamente (strumento da definire: `specta`/`ts-rs`).

## Persistenza
Directory dati per-utente standard del sistema (es. via crate `directories`):
- macOS: `~/Library/Application Support/<app>/`
- Windows: `%APPDATA%\<app>\`
- Linux: `$XDG_CONFIG_HOME/<app>/` (config) e `$XDG_DATA_HOME/<app>/` (dati)

| Dato | Formato | Note |
|---|---|---|
| Regole | File JSON | Leggibile, versione dello schema inclusa; base per import/export |
| Impostazioni | File JSON | Tema, lingua, retention, ecc. |
| Cronologia (batch + spostamenti) | **SQLite** (`rusqlite`) | WAL mode per accessi concorrenti da più processi; query, retention, undo |

## Invocazioni multiple dal menu contestuale
Problema: su Windows (e alcuni file manager Linux) una selezione di N file lancia N processi.

Soluzione: **single-instance + aggregazione**
- Il primo processo diventa "principale" (es. `tauri-plugin-single-instance` / lock + IPC locale).
- I processi successivi inoltrano i propri path al principale e terminano subito.
- Il principale attende una breve finestra di debounce (~300 ms, da tarare) e processa
  **tutti i path in un unico batch** → una notifica, un undo, una sequenza di dialog conflitti.

## Architettura di alto livello
- `ordo-core` (crate Rust): regole, matching, spostamento, conflitti, cronologia/undo.
- `ordo-cli` (binario Rust): interfaccia a riga di comando sopra `ordo-core`.
- `ordo-app` (Tauri 2): UI web + comandi Tauri sopra `ordo-core`.

## Binari e flussi di esecuzione
Due eseguibili distinti, stesso core:

| Binario | Ruolo | UI |
|---|---|---|
| `ordo` (da `ordo-cli`) | CLI pura per terminale e script | Solo testo (prompt TTY) |
| App Tauri (`Ordo`) | GUI + target del menu contestuale | Finestra, dialog, notifiche |

### Flusso "menu contestuale"
1. Il file manager invoca l'app con `--sort <path...>` (o `--sort-dir <path>`).
2. Single-instance: se esiste già un'istanza, i path vengono inoltrati e il processo esce.
3. L'istanza principale **non apre la finestra principale**; attende il debounce e crea un batch.
4. Esegue l'ordinamento tramite `ordo-core`; apre **solo** il dialog conflitti se necessario.
5. Mostra la notifica di riepilogo con azione **"Annulla"**.
6. Il processo **resta vivo ~10 s** per ricevere "Annulla" o click sulla notifica, poi esce.
   - Su Windows si usa anche l'attivazione via protocollo/AUMID se il processo è già uscito.
   - Dove le azioni nelle notifiche non sono supportate: click → apre la cronologia sul batch.

### Flusso "app"
Avvio senza argomenti → finestra principale (Regole, Cronologia, Impostazioni).

## Struttura del repository
Cargo workspace alla radice + frontend gestito con **pnpm**.

```
/
├── Cargo.toml            # workspace
├── crates/
│   ├── ordo-core/        # libreria: dominio e logica
│   └── ordo-cli/         # binario CLI
├── app/
│   ├── package.json      # frontend React + TS (pnpm)
│   ├── src/              # UI
│   └── src-tauri/        # crate Tauri (dipende da ordo-core)
└── docs/
```

## Strategia di test
- **`ordo-core`** — copertura alta, test unitari + integrazione su filesystem temporanei (`tempfile`):
  matching glob/regex e case-sensitivity, esclusione file nascosti, spostamento (incluso
  fallback copia+rimozione cross-volume simulato), conflitti, cestino, cronologia, undo, retention.
- **`ordo-cli`** — test di integrazione con `assert_cmd` (flag, exit code, output).
- **Frontend** — test dei componenti principali con **Vitest** + Testing Library.
- Nessun E2E nella prima fase.

## Gestione del rewrite nel repository
- Prima di iniziare: tag **`v1-electron`** sull'ultimo commit della versione Electron.
- Sviluppo su branch **`v2`**: il codice Electron viene rimosso e sostituito dalla nuova struttura.
- A rewrite completato: merge di `v2` su `master`.
