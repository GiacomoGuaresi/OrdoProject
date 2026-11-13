# Integrazione con il sistema operativo

L'integrazione con il menu contestuale si installa/rimuove **dall'app** (Impostazioni).

## Comando `ordo` nel PATH
Pulsante in Impostazioni **"Installa il comando `ordo` nel terminale"** (modello VS Code):
| OS | Meccanismo |
|---|---|
| macOS | symlink in `/usr/local/bin/ordo` (richiesta privilegi) o `~/.local/bin/ordo` |
| Windows | aggiunta della cartella del binario al **PATH utente** (HKCU, no admin) |
| Linux (.deb) | binario già installato in `/usr/bin/ordo` dal pacchetto |
| Linux (AppImage) | symlink in `~/.local/bin/ordo` |

Disponibile anche il rispettivo "Rimuovi comando".

## macOS — Quick Actions (Services)
- Workflow Automator generati dall'app in `~/Library/Services/`.
- Non richiedono firma del codice; visibili nel menu contestuale di Finder sotto **"Azioni rapide"**.
- Voci previste: ordina file/selezione, ordina contenuto cartella.
- Il workflow invoca l'eseguibile con i path selezionati (struttura corretta del bundle
  `.workflow`, a differenza dello script attuale).
- Finder Sync Extension (voce al primo livello) **non** prevista per ora.

## Windows — verbi nel registro (HKCU)
- Chiavi in `HKCU\Software\Classes\...` scritte dall'app, **senza privilegi admin**:
  - `*\shell\Ordo...` → file (singolo o selezione multipla, aggregata via single-instance);
  - `Directory\shell\Ordo...` → cartella selezionata (ordina il contenuto, primo livello);
  - `Directory\Background\shell\Ordo...` → sfondo della cartella aperta.
- Su Windows 11 le voci compaiono sotto **"Mostra altre opzioni"**.
- Testi delle voci localizzati in base alla lingua dell'app al momento dell'installazione.
- Path dell'eseguibile risolto dinamicamente (nessun path hardcoded come nel vecchio `.reg`).
- Menu moderno Windows 11 (IExplorerCommand) **non** previsto: richiede firma.

## Linux — file manager supportati
| File manager | Meccanismo | Percorso |
|---|---|---|
| **Nautilus** (GNOME) | Script Nautilus (riceve selezione via `NAUTILUS_SCRIPT_SELECTED_FILE_PATHS`) | `~/.local/share/nautilus/scripts/` |
| **Dolphin** (KDE) | Service menu `.desktop` (`%F` per selezione multipla) | `~/.local/share/kio/servicemenus/` |

- L'app rileva quali file manager sono presenti e installa solo le integrazioni pertinenti.
- Nemo / Caja / Thunar **non** supportati inizialmente (la CLI resta disponibile).
- Nota: Nautilus scripts compaiono nel sottomenu "Scripts"; valutare in futuro `nautilus-python`.
