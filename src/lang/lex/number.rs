use super::trivia::trivia_width;
use super::LexResult;
use crate::source::{Position, Range, Source};
use crate::syntax_node::{SyntaxToken, TokenKind};

pub fn lex_number(source: &Source, range: Range) -> Option<LexResult> {
    let mut remaining_range = range;
    let init_range_start = range.start;

    while !remaining_range.is_empty() {
        let c = source.at(remaining_range.start);
        if c.is_ascii_digit() {
            remaining_range.start.advance(1);
        } else {
            break;
        }
    }

    if init_range_start == remaining_range.start {
        return None;
    }
    let token_width = Position::distance(remaining_range.start, init_range_start);
    let trailing_trivia_width = trivia_width(source, remaining_range);
    remaining_range.start.advance(trailing_trivia_width);

    Some(LexResult {
        token: SyntaxToken {
            kind: TokenKind::Number,
            leading_trivia_width: 0,
            token_width,
            trailing_trivia_width,
        },
        remaining_range,
    })
}

#[test]
fn test_number_token() {
    use super::test;
    use TokenKind::*;

    test("1", vec![Number]);
    test("42", vec![Number]);
    test("1 ", vec![Number]);
    test("42 ", vec![Number]);
}
