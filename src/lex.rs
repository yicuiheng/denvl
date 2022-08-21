mod ident;
mod keyword;
mod mark;
mod number;
pub mod trivia;

use std::collections::VecDeque;

use crate::diagnostic::{Diagnostic, DiagnosticError};
use crate::source::{Position, Range, Source};
use crate::syntax_node::{SyntaxToken, TokenKind};
use ident::lex_ident;
use keyword::lex_keyword;
use mark::lex_mark;
use number::lex_number;

use self::trivia::trivia_width;

pub struct LexResult {
    pub token: SyntaxToken,
    pub remaining_range: Range,
}

pub fn lex(source: &Source, range: Range) -> LexResult {
    match skip_until_success(source, range) {
        Ok(lex_result) => lex_result,
        Err(error_width) => {
            let mut range = range;
            let error_range_start = range.start;
            range.start.advance(error_width);
            let mut error_range_end = error_range_start;
            error_range_end.advance(error_width);
            let trivia_width = trivia_width(source, range);
            range.start.advance(trivia_width);
            let mut diagnostics = VecDeque::new();
            diagnostics.push_back(Diagnostic::Error(DiagnosticError::UnknownToken {
                range: Range {
                    start: error_range_start,
                    end: error_range_end,
                },
            }));
            LexResult {
                token: SyntaxToken {
                    kind: TokenKind::Error,
                    leading_trivia_width: 0,
                    token_width: error_width,
                    trailing_trivia_width: trivia_width,
                },
                remaining_range: range,
            }
        }
    }
}

fn skip_until_success(source: &Source, range: Range) -> Result<LexResult, usize> {
    fn aux(source: &Source, range: Range) -> Option<LexResult> {
        lex_number(source, range)
            .or_else(|| lex_ident(source, range))
            .or_else(|| lex_keyword(source, range))
            .or_else(|| lex_mark(source, range))
    }

    let init_range_start = range.start;
    let mut range = range;
    while !range.is_empty() {
        if let Some(lex_result) = aux(source, range) {
            let skipped_width = Position::distance(range.start, init_range_start);
            if skipped_width == 0 {
                return Ok(lex_result);
            } else {
                return Err(skipped_width);
            }
        } else {
            range.start.advance(1);
        }
    }
    Err(Position::distance(range.start, init_range_start))
}

#[cfg(test)]
fn test(src: &str, expecteds: Vec<crate::syntax_node::TokenKind>) {
    let source = Source::from_str(src);
    let mut range = source.range();
    for expected in expecteds {
        let LexResult {
            token,
            remaining_range,
        } = lex(&source, range);
        range = remaining_range;
        let actual = token;
        assert_eq!(actual.kind, expected);
    }
    assert!(range.is_empty());
}

#[test]
fn test_mark_token() {
    use TokenKind::*;

    test("@", vec![Error]);
    test("@ ", vec![Error]);
    test("@ @ ", vec![Error, Error]);
    test("@\n@", vec![Error, Error]);
    test("@\n\n@", vec![Error, Error]);
}
