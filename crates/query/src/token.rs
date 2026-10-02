use logos::Logos;

#[derive(Debug, PartialEq, Clone, Hash, Eq, Logos)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("-")]
    Except,

    #[token("|")]
    Or,

    #[token("&")]
    And,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[regex("[\\p{Cased_Letter}0-9][\\p{Cased_Letter}0-9-]*", capture_slice)]
    Substring(String),

    #[regex("[\\p{Cased_Letter}0-9-]+\\*", capture_prefix)]
    Prefix(String),

    #[regex("\"[^\"]+\"", capture_quoted_string)]
    #[regex("'[^']+'", capture_quoted_string)]
    ExactString(String),
}

fn capture_quoted_string(lex: &logos::Lexer<Token>) -> String {
    let slice = lex.slice();
    slice[1..slice.len() - 1].to_lowercase()
}

fn capture_slice(lex: &logos::Lexer<Token>) -> String {
    lex.slice().to_lowercase()
}

fn capture_prefix(lex: &logos::Lexer<Token>) -> String {
    let s = lex.slice().strip_suffix("*");

    s.unwrap().to_lowercase()
}
