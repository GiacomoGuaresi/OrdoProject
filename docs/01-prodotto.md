# Prodotto — Visione e funzionalità

## Identità
| Voce | Valore |
|---|---|
| Nome prodotto | **Ordo** |
| Comando CLI | `ordo` |
| Identificatore (bundle id) | `io.github.giacomoguaresi.ordo` |
| Repository | `GiacomoGuaresi/OrdoProject` (nome invariato) |
| Licenza | **MIT** (file `LICENSE` da aggiungere) |

## Migrazione dalla v1 (Electron)
- **Nessuna migrazione**: `rules.json` e `move_log.csv` della v1 non vengono letti né importati.

## Primo avvio
- Viene creato un **set di regole di default** che puntano alle **cartelle standard dell'OS**
  (risolte via API di sistema, quindi già localizzate: Documenti, Immagini, Musica, Video, Download):
  documenti, fogli di calcolo, presentazioni, immagini, audio, video, archivi, installer
  (le estensioni degli installer variano per OS: `.exe/.msi`, `.dmg/.pkg`, `.deb/.rpm/.AppImage`).
- Le cartelle di destinazione **non vengono create** al primo avvio, solo al primo spostamento.
- **Onboarding** breve: spiega il funzionamento e propone di installare il menu contestuale.

## Piattaforme target
- macOS, Windows, Linux

## Lingua
- Internazionalizzazione (i18n) predisposta da subito.
- Lingue iniziali: **inglese** e **italiano**; default = lingua del sistema operativo (fallback EN).
- Si applica a: app, notifiche, CLI, voci del menu contestuale.

## Modalità di funzionamento
- **Solo manuale**: l'ordinamento avviene esclusivamente su richiesta dell'utente
  (menu contestuale del file manager e/o dall'app).
- Nessun servizio in background né cartelle sorvegliate.

## Punti di ingresso dell'ordinamento
- **Menu contestuale** del file manager (Finder, Esplora risorse, file manager Linux):
  file singolo, selezione multipla, cartella, sfondo della cartella aperta.
- **CLI** (es. `ordo sort <path>`), utilizzabile da terminale e script.
- L'app grafica **non** avvia ordinamenti (niente drag & drop né pulsante "ordina"):
  serve a configurare regole e a consultare/annullare la cronologia.

## CLI `ordo` — comandi (prima versione)
| Comando | Descrizione |
|---|---|
| `ordo sort <path...> [--on-conflict=keep-both\|overwrite\|skip]` | Ordina file e/o contenuto di cartelle (primo livello) |
| `ordo undo [batch-id] [--move <id>]` | Annulla un batch (default: l'ultimo) o un singolo spostamento |
| `ordo history [--limit N] [--json]` | Elenca i batch recenti |
| `ordo rules list\|add\|remove\|move\|enable\|disable` | Gestione regole da terminale |
| `ordo integration install\|uninstall\|status` | Installa/rimuove l'integrazione con il menu contestuale |

Opzioni globali: `--json` (output strutturato dove sensato), `--lang`, `--help`, `--version`.
Exit code distinti per: successo, successo parziale (alcuni errori), errore.

## Feedback all'utente (menu contestuale)
- Al termine: **notifica nativa di sistema** con riepilogo (es. "12 file ordinati, 3 ignorati").
- Azione **"Annulla"** nella notifica, dove la piattaforma lo supporta.
- Click sulla notifica → apre l'app sulla cronologia (batch appena eseguito).
- Errori: riportati nella notifica e dettagliati nella cronologia.

## App grafica — sezioni e funzioni
- **Regole**: lista ordinata, modifica, abilita/disabilita, drag & drop.
  - **Import/export** delle regole su file (backup / condivisione tra computer).
- **Cronologia**: batch, dettaglio spostamenti, undo (batch o singolo), "mostra nella cartella".
- **Impostazioni**:
  - **Installa/rimuovi integrazione menu contestuale** dall'app (niente script manuali).
  - **Tema** chiaro/scuro: segue il sistema, forzabile.
  - Lingua (default: sistema).
- Nessun "tester regole".

## Ambito dell'ordinamento
- Ordinando una cartella si considerano **solo i file al primo livello**.
- Le sottocartelle (e il loro contenuto) non vengono mai toccate.
- **File nascosti e di sistema sempre esclusi** (dotfile, attributo "hidden" su Windows,
  `.DS_Store`, `desktop.ini`, `Thumbs.db`, ecc.): non valutati e non conteggiati.
  Vale anche se selezionati esplicitamente? → **sì, esclusi comunque** (default prudente).

## Organizzazione delle regole
- **Una sola lista ordinata**; si applica la **prima regola che corrisponde** (first match wins).
- Ogni regola ha: **nome**, **abilitata on/off**, pattern, destinazione.
- Riordinamento tramite drag & drop nell'app.
- Nessun profilo / set multiplo.

## Modello delle regole — condizioni
- La condizione si applica **solo al nome del file**.
- Due tipi di pattern: **glob** (es. `*.{png,jpg}`) e **regex** (es. `^invoice_\d{4}\.pdf$`).
- Nessuna condizione su dimensione, data o tipo MIME.
- Matching **case-insensitive di default**; opzione per regola **"distingui maiuscole"**.

## File senza regola corrispondente
- Restano al loro posto, ma vengono **conteggiati** nel riepilogo (notifica/CLI)
  e **registrati nel batch** in cronologia come "ignorati".
- Nessuna regola di fallback / catch-all.

## Modello delle regole — azione
- Unica azione: **sposta** il file in una **cartella di destinazione fissa** (statica, senza variabili).
- Lo spostamento deve funzionare **anche tra volumi diversi** (copia + verifica + rimozione).
- Se il file si trova **già nella cartella di destinazione** della regola, non viene toccato.

### Destinazione inesistente
- Volume/disco radice presente → la cartella viene **creata automaticamente**.
- Volume non montato (es. disco esterno scollegato) → file **saltato e segnalato come errore**;
  non vengono mai create cartelle "fantasma" in posizioni errate.

## Cronologia e annullamento
- Ogni esecuzione dell'ordinamento genera un **batch** registrato in cronologia
  (data/ora, origine dell'invocazione, elenco degli spostamenti).
- **Undo** possibile a due livelli:
  - intero batch;
  - singolo spostamento.
- Casi limite dell'undo:
  - file spostato **non più presente** nella destinazione → voce marcata **"non annullabile"**, si prosegue;
  - percorso originale **occupato** → si riusa il **dialog conflitti** (Sovrascrivi / Mantieni entrambi / Salta);
  - cartella d'origine non più esistente → viene ricreata (stessa logica "volume presente" delle destinazioni).
- Un batch può risultare **parzialmente annullato**; la cronologia mostra lo stato di ogni spostamento
  (eseguito / annullato / non annullabile / saltato / errore).
- Nessuna modalità di anteprima / dry run.
- **Retention** configurabile: default **90 giorni** (opzione "per sempre");
  pulizia automatica dei batch scaduti + pulsante "svuota cronologia".

## Conflitti di nome nella destinazione
Quando esiste già un file con lo stesso nome, l'app **chiede all'utente** con un dialog:
- **Sovrascrivi** il file esistente;
- **Mantieni entrambi** (il nuovo file viene rinominato, es. `nome (1).ext`);
- **Salta** (il file resta dov'è; registrato come "saltato" nel batch).

Il dialog permette di **aprire/ispezionare i due file** prima di decidere
(idealmente con anteprima rapida: Quick Look su macOS, equivalente/app predefinita su Windows e Linux).

### Contenuto del dialog conflitti
- I due file **affiancati** (esistente vs nuovo): nome, dimensione, data di modifica.
- **Miniatura** dove possibile (immagini; PDF/video se supportato dalla piattaforma).
- Indicazione **"file identici"** se il contenuto coincide (es. confronto hash).
- Pulsante **"Anteprima"**: Quick Look su macOS (`qlmanage -p`); app predefinita su Windows/Linux.
- Azioni: Sovrascrivi · Mantieni entrambi · Salta · (checkbox) applica a tutti.

Con più conflitti nello stesso batch: **un dialog per conflitto**, con checkbox
**"applica a tutti i conflitti rimanenti"**.

In caso di **sovrascrittura**, il file preesistente viene spostato nel **cestino di sistema**
(mai eliminato definitivamente). L'undo di quello spostamento tenta di ripristinarlo dal cestino;
se non è possibile, indica all'utente dove si trova.

> Implicazione: anche l'invocazione da menu contestuale deve poter mostrare UI.

### Conflitti da CLI
- Terminale **interattivo** (TTY): prompt testuale (sovrascrivi / mantieni entrambi / salta / applica a tutti).
- **Non interattivo**: flag `--on-conflict=keep-both|overwrite|skip` (default `keep-both`).
  Il flag, se passato, vale anche in modalità interattiva e sopprime il prompt.
