//! English/Italian messages of the CLI.

use ordo_core::{ErrorCode, Lang, MoveStatus};

/// Message catalog: (key, English, Italian). `{}` placeholders are filled in order.
const MESSAGES: &[(&str, &str, &str)] = &[
    // sort
    ("sort.nothing", "Nothing to sort.", "Niente da ordinare."),
    ("sort.moved", "{} sorted", "{} ordinati"),
    (
        "sort.ignored",
        "{} ignored (no rule)",
        "{} ignorati (nessuna regola)",
    ),
    ("sort.skipped", "{} skipped", "{} saltati"),
    (
        "sort.in_place",
        "{} already in place",
        "{} già al loro posto",
    ),
    ("sort.errors", "{} errors", "{} errori"),
    (
        "sort.batch",
        "Batch #{} — undo with `ordo undo {}`",
        "Batch #{} — annulla con `ordo undo {}`",
    ),
    // conflicts
    (
        "conflict.title",
        "Conflict: \"{}\" already exists in {}",
        "Conflitto: \"{}\" esiste già in {}",
    ),
    (
        "conflict.title_undo",
        "Undo conflict: \"{}\" is already present in {}",
        "Conflitto nell'annullamento: \"{}\" è già presente in {}",
    ),
    (
        "conflict.identical",
        "The two files are identical.",
        "I due file sono identici.",
    ),
    (
        "conflict.different",
        "existing: {} · {}    incoming: {} · {}",
        "esistente: {} · {}    nuovo: {} · {}",
    ),
    (
        "conflict.choices",
        "[o] overwrite  [k] keep both  [s] skip   (uppercase: apply to all)",
        "[o] sovrascrivi  [k] mantieni entrambi  [s] salta   (maiuscola: applica a tutti)",
    ),
    (
        "conflict.invalid",
        "Please answer o, k or s.",
        "Rispondi o, k oppure s.",
    ),
    // undo
    ("undo.nothing", "Nothing to undo.", "Niente da annullare."),
    (
        "undo.done",
        "Batch #{}: {} restored",
        "Batch #{}: {} ripristinati",
    ),
    (
        "undo.not_undoable",
        "{} not undoable (file no longer at destination)",
        "{} non annullabili (file non più nella destinazione)",
    ),
    ("undo.skipped", "{} skipped", "{} saltati"),
    ("undo.errors", "{} errors", "{} errori"),
    ("undo.move_done", "Restored to {}", "Ripristinato in {}"),
    (
        "undo.move_not_undoable",
        "Not undoable: the file is no longer at {}",
        "Non annullabile: il file non è più in {}",
    ),
    ("undo.move_skipped", "Skipped.", "Saltato."),
    (
        "undo.move_not_applicable",
        "This move is not in the \"done\" state.",
        "Questo spostamento non è nello stato \"eseguito\".",
    ),
    (
        "undo.trash_restored",
        "The overwritten file was put back from the trash.",
        "Il file sovrascritto è stato ripristinato dal cestino.",
    ),
    (
        "undo.trash_manual",
        "The overwritten file {} is in the trash: restore it manually.",
        "Il file sovrascritto {} è nel cestino: ripristinalo manualmente.",
    ),
    // history
    (
        "history.empty",
        "The history is empty.",
        "La cronologia è vuota.",
    ),
    (
        "history.header",
        "ID      DATE              ORIGIN        MOVED  UNDONE  IGNORED  ERRORS",
        "ID      DATA              ORIGINE       SPOST. ANNULL. IGNOR.   ERRORI",
    ),
    // rules
    (
        "rules.empty",
        "No rules. Add one with `ordo rules add`.",
        "Nessuna regola. Aggiungine una con `ordo rules add`.",
    ),
    (
        "rules.added",
        "Rule \"{}\" added at position {}.",
        "Regola \"{}\" aggiunta in posizione {}.",
    ),
    (
        "rules.removed",
        "Rule \"{}\" removed.",
        "Regola \"{}\" rimossa.",
    ),
    (
        "rules.moved",
        "Rule \"{}\" moved to position {}.",
        "Regola \"{}\" spostata in posizione {}.",
    ),
    (
        "rules.enabled",
        "Rule \"{}\" enabled.",
        "Regola \"{}\" abilitata.",
    ),
    (
        "rules.disabled",
        "Rule \"{}\" disabled.",
        "Regola \"{}\" disabilitata.",
    ),
    (
        "rules.imported",
        "{} rules imported.",
        "{} regole importate.",
    ),
    (
        "rules.exported",
        "{} rules exported to {}.",
        "{} regole esportate in {}.",
    ),
    (
        "rules.case_sensitive",
        "case-sensitive",
        "maiuscole/minuscole",
    ),
    ("rules.off", "off", "off"),
    // generic
    ("error", "Error", "Errore"),
];

/// Translator for one language.
#[derive(Debug, Clone, Copy)]
pub struct Tr(pub Lang);

impl Tr {
    /// Returns the message for `key`, with `{}` placeholders replaced by `args`.
    pub fn t(&self, key: &str, args: &[&dyn std::fmt::Display]) -> String {
        let template = MESSAGES
            .iter()
            .find(|(k, _, _)| *k == key)
            .map(|(_, en, it)| if self.0 == Lang::It { *it } else { *en })
            .unwrap_or(key);
        let mut out = String::with_capacity(template.len());
        let mut args = args.iter();
        let mut rest = template;
        while let Some(pos) = rest.find("{}") {
            out.push_str(&rest[..pos]);
            match args.next() {
                Some(a) => out.push_str(&a.to_string()),
                None => out.push_str("{}"),
            }
            rest = &rest[pos + 2..];
        }
        out.push_str(rest);
        out
    }

    pub fn error_code(&self, code: ErrorCode) -> &'static str {
        let it = self.0 == Lang::It;
        match code {
            ErrorCode::SourceMissing if it => "file non trovato",
            ErrorCode::SourceMissing => "file not found",
            ErrorCode::VolumeMissing if it => "disco di destinazione non collegato",
            ErrorCode::VolumeMissing => "destination drive not connected",
            ErrorCode::DestinationMissing if it => "file non più nella destinazione",
            ErrorCode::DestinationMissing => "file no longer at destination",
            ErrorCode::Trash if it => "impossibile spostare nel cestino",
            ErrorCode::Trash => "cannot move to trash",
            ErrorCode::Io if it => "errore di I/O",
            ErrorCode::Io => "I/O error",
        }
    }

    pub fn status(&self, status: MoveStatus) -> &'static str {
        let it = self.0 == Lang::It;
        match status {
            MoveStatus::Done if it => "spostato",
            MoveStatus::Done => "moved",
            MoveStatus::Undone if it => "annullato",
            MoveStatus::Undone => "undone",
            MoveStatus::NotUndoable if it => "non annullabile",
            MoveStatus::NotUndoable => "not undoable",
            MoveStatus::Skipped if it => "saltato",
            MoveStatus::Skipped => "skipped",
            MoveStatus::Ignored if it => "ignorato",
            MoveStatus::Ignored => "ignored",
            MoveStatus::AlreadyInPlace if it => "già al suo posto",
            MoveStatus::AlreadyInPlace => "already in place",
            MoveStatus::Error if it => "errore",
            MoveStatus::Error => "error",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fills_placeholders() {
        let tr = Tr(Lang::En);
        assert_eq!(
            tr.t("sort.batch", &[&7, &7]),
            "Batch #7 — undo with `ordo undo 7`"
        );
        assert_eq!(Tr(Lang::It).t("sort.moved", &[&3]), "3 ordinati");
        assert_eq!(tr.t("missing.key", &[]), "missing.key");
    }

    #[test]
    fn every_message_has_matching_placeholders() {
        for (key, en, it) in MESSAGES {
            assert_eq!(en.matches("{}").count(), it.matches("{}").count(), "{key}");
            assert_eq!(
                MESSAGES.iter().filter(|(k, _, _)| k == key).count(),
                1,
                "duplicate key {key}"
            );
        }
    }
}
