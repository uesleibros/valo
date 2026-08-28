//! Case-insensitive name handling.
//!
//! Basic-family languages compare identifiers without regard to case, so nearly
//! every symbol table in the compiler and runtime is keyed by a case-folded
//! name. Folding therefore sits on the hottest path in the interpreter, and the
//! rules have to be identical everywhere: the lexer, the semantic analyzer, and
//! the interpreter must agree on what two names being "the same" means.

/// Longest identifier that [`with_folded`] folds without touching the heap.
const STACK_LEN: usize = 96;

/// Returns the case-folded lookup key for `name`.
///
/// Prefer [`with_folded`] on hot paths; this allocates.
#[inline]
pub fn fold(name: &str) -> String {
    name.to_lowercase()
}

/// Calls `f` with the case-folded lookup key for `name`, without allocating for
/// the identifier shapes that occur in practice.
///
/// Valo identifiers are short and ASCII, so they fold into a stack buffer.
/// Anything longer or non-ASCII falls back to [`fold`].
#[inline]
pub fn with_folded<R>(name: &str, f: impl FnOnce(&str) -> R) -> R {
    let bytes = name.as_bytes();
    if bytes.len() <= STACK_LEN && bytes.is_ascii() {
        let mut buffer = [0u8; STACK_LEN];
        let folded = &mut buffer[..bytes.len()];
        folded.copy_from_slice(bytes);
        folded.make_ascii_lowercase();
        if let Ok(folded) = std::str::from_utf8(folded) {
            return f(folded);
        }
    }
    f(&fold(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folding_is_case_insensitive() {
        assert_eq!(fold("MyVariable"), "myvariable");
        assert_eq!(with_folded("MyVariable", str::to_string), "myvariable");
    }

    #[test]
    fn with_folded_agrees_with_fold_past_the_stack_buffer() {
        let long_name = "A".repeat(STACK_LEN * 2);
        assert_eq!(with_folded(&long_name, str::to_string), fold(&long_name));
    }

    #[test]
    fn with_folded_agrees_with_fold_for_non_ascii_names() {
        for name in ["Ácido", "ÖRE", "naïve"] {
            assert_eq!(with_folded(name, str::to_string), fold(name));
        }
    }
}
