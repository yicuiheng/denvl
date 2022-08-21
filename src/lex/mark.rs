use super::trivia::trivia_width;
use super::LexResult;
use crate::source::{starts_with, Range, Source};
use crate::syntax_node::{SyntaxToken, TokenKind};

pub fn lex_mark(source: &Source, range: Range) -> Option<LexResult> {
    let mut remaining_range = range;
    use std::collections::HashMap;
    let operators: HashMap<&str, TokenKind> = vec![
        ("+", TokenKind::Plus),
        ("-", TokenKind::Minus),
        ("*", TokenKind::Ast),
        ("/", TokenKind::Slash),
        ("=", TokenKind::Equal),
        ("(", TokenKind::OpenParen),
        (")", TokenKind::CloseParen),
        (";", TokenKind::Semicolon),
    ]
    .into_iter()
    .collect();

    for (str, kind) in operators {
        if !starts_with(source, &str, &remaining_range) {
            continue;
        }
        remaining_range.start.advance(str.len());

        let trailing_trivia_width = trivia_width(source, remaining_range);
        remaining_range.start.advance(trailing_trivia_width);

        return Some(LexResult {
            token: SyntaxToken {
                kind,
                leading_trivia_width: 0,
                token_width: str.len(),
                trailing_trivia_width,
            },
            remaining_range,
        });
    }

    None
}

#[test]
fn test_mark_token() {
    use super::test;
    use TokenKind::*;

    test("+", vec![Plus]);
    test("-", vec![Minus]);
    test("*", vec![Ast]);
    test("/", vec![Slash]);
    test("=", vec![Equal]);
    test(";", vec![Semicolon]);
    test("(", vec![OpenParen]);
    test(")", vec![CloseParen]);

    test("+ ", vec![Plus]);
    test("- ", vec![Minus]);
    test("* ", vec![Ast]);
    test("/ ", vec![Slash]);
    test("= ", vec![Equal]);
    test("; ", vec![Semicolon]);
    test("( ", vec![OpenParen]);
    test(") ", vec![CloseParen]);
}
