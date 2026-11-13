//! Ordo core library: rules, matching, file moves, conflicts, history and undo.

/// Version of the Ordo core library.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    #[test]
    fn version_is_set() {
        assert!(!super::VERSION.is_empty());
    }
}
