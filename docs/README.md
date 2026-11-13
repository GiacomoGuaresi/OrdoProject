# Ordo v2 — Documentazione di progetto

Documentazione delle decisioni per la riscrittura cross-platform (macOS, Windows, Linux)
di OrdoProject. È la base di riferimento per lo sviluppo.

| Documento | Contenuto |
|---|---|
| [01-prodotto.md](01-prodotto.md) | Visione, funzionalità, regole, conflitti, cronologia/undo, CLI, UX |
| [02-stack.md](02-stack.md) | Stack (Tauri 2 + Rust + React/TS), architettura, persistenza, test, struttura repo |
| [03-distribuzione.md](03-distribuzione.md) | Packaging, CI/CD, firma, aggiornamenti |
| [04-integrazione-os.md](04-integrazione-os.md) | Menu contestuale per OS, comando `ordo` nel PATH |
| [05-roadmap.md](05-roadmap.md) | Milestone M0–M4 e fuori scope |

## Sintesi delle decisioni
- **Ordo**: ordinamento **manuale** dei file tramite regole (glob/regex sul nome → cartella fissa).
- Ingressi: **menu contestuale** e **CLI `ordo`**; l'app grafica configura regole e gestisce la cronologia.
- **Conflitti** interattivi (sovrascrivi → cestino / mantieni entrambi / salta) con anteprima.
- **Cronologia + undo** per batch o singolo spostamento, SQLite, retention 90 gg.
- **Tauri 2 + Rust** (core condiviso) + **React/TS + shadcn/ui + Tailwind**.
- Installer standard non firmati, GitHub Actions, i18n EN/IT, licenza MIT.
