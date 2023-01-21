use super::trivia::trivia_width;
use super::LexResult;
use crate::source::{starts_with, Range, Source};
use crate::syntax_node::{SyntaxToken, TokenKind};

pub fn lex_keyword(source: &Source, range: Range) -> Option<LexResult> {
    let mut remaining_range = range;
    use std::collections::HashMap;
    let keywords: HashMap<&str, TokenKind> = vec![("let", TokenKind::Let)].into_iter().collect();

    for (str, kind) in keywords {
        if !starts_with(source, str, &remaining_range) {
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
fn test_keyword_token() {
    use super::test;
    use TokenKind::*;

    test("let", vec![Let]);
    test("let ", vec![Let]);
}
