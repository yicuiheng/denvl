use super::trivia::trivia_width;
use super::LexResult;
use crate::source::{Position, Range, Source};
use crate::syntax_node::{SyntaxToken, TokenKind};

pub fn lex_ident(source: &Source, range: Range) -> Option<LexResult> {
    let mut remaining_range = range;
    let init_range_start = range.start;

    let c = source.at(remaining_range.start);
    if !c.is_ascii_alphabetic() && c != '_' {
        return None;
    }
    remaining_range.start.advance(1);

    while !remaining_range.is_empty() {
        let c = source.at(remaining_range.start);
        if c.is_ascii_alphanumeric() || c == '_' {
            remaining_range.start.advance(1);
        } else {
            break;
        }
    }

    if init_range_start == remaining_range.start {
        return None;
    }

    // 予約語は識別子ではない
    if let &['l', 'e', 't'] = source.get(&Range {
        start: init_range_start,
        end: remaining_range.start,
    }) {
        return None;
    }

    let token_width = Position::distance(remaining_range.start, init_range_start);
    let trailing_trivia_width = trivia_width(source, remaining_range);
    remaining_range.start.advance(trailing_trivia_width);

    Some(LexResult {
        token: SyntaxToken {
            kind: TokenKind::Ident,
            leading_trivia_width: 0,
            token_width,
            trailing_trivia_width,
        },
        remaining_range,
    })
}

#[test]
fn test_ident_token() {
    use super::test;
    use TokenKind::*;

    test("a", vec![Ident]);
    test("hoge", vec![Ident]);
    test("hoge42", vec![Ident]);
    test("lethoge", vec![Ident]);

    test("a  ", vec![Ident]);
    test("a// comment", vec![Ident]);
    test("a/* comment */", vec![Ident]);
    test("a // comment", vec![Ident]);
    test("a /* comment */", vec![Ident]);
    test("a \n", vec![Ident]);
    test("a \n ", vec![Ident]);
    test("a // \n ", vec![Ident]);
    test("a  \n // \n  ", vec![Ident]);
    test("a  \n /* comment */  \n  ", vec![Ident]);

    test("a b", vec![Ident, Ident]);
    test("a @", vec![Ident, Error]);
}
