use once_cell::sync::Lazy;
use std::collections::HashSet;

static RUST: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let keywords = [
        "as",
        "use",
        "extern crate",
        "break",
        "const",
        "continue",
        "crate",
        "else",
        "if",
        "if let",
        "enum",
        "extern",
        "false",
        "fn",
        "for",
        "if",
        "impl",
        "in",
        "for",
        "let",
        "loop",
        "match",
        "mod",
        "move",
        "mut",
        "pub",
        "impl",
        "ref",
        "return",
        "Self",
        "self",
        "static",
        "struct",
        "super",
        "trait",
        "true",
        "type",
        "unsafe",
        "use",
        "where",
        "while",
        "abstract",
        "alignof",
        "become",
        "box",
        "do",
        "final",
        "macro",
        "offsetof",
        "override",
        "priv",
        "proc",
        "pure",
        "sizeof",
        "typeof",
        "unsized",
        "virtual",
        "yield",
    ];
    let mut set = HashSet::new();
    set.extend(keywords);
    set
});

static JS: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let keywords = [
        "await",
        "break",
        "case",
        "catch",
        "class",
        "const",
        "continue",
        "debugger",
        "default",
        "delete",
        "do",
        "else",
        "enum",
        "export",
        "extends",
        "false",
        "finally",
        "for",
        "function",
        "if",
        "implements",
        "import",
        "in",
        "instanceof",
        "interface",
        "let",
        "new",
        "null",
        "package",
        "private",
        "protected",
        "public",
        "return",
        "super",
        "switch",
        "static",
        "this",
        "throw",
        "try",
        "True",
        "typeof",
        "var",
        "void",
        "while",
        "with",
        "yield",
    ];
    let mut set = HashSet::new();
    set.extend(keywords);
    set
});
static RUBY: Lazy<HashSet<&'static str>> = Lazy::new(|| {
    let keywords = [
        "BEGIN", "END", "alias", "and", "begin", "break", "case", "class", "def", "module", "next",
        "nil", "not", "or", "redo", "rescue", "retry", "return", "elsif", "end", "false", "ensure",
        "for", "if", "true", "undef", "unless", "do", "else", "super", "then", "until", "when",
        "while", "defined?", "self",
    ];
    let mut set = HashSet::new();
    set.extend(keywords);
    set
});

pub fn from_lang(lang: &str) -> HashSet<&'static str> {
    match lang {
        "rust" => RUST.clone(),
        "javascript" => JS.clone(),
        "ruby" => RUBY.clone(),
        _ => HashSet::new(),
    }
}
