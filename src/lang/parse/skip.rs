use crate::diagnostic::{extra_token_error, unknown_token_error, Diagnostic};
use crate::lex::{lex, LexResult};
use crate::source::{Range, Source};
use crate::syntax_node::{self, TokenKind};
use std::collections::VecDeque;

// expected に含まれるトークンが出現するまでスキップする
// (スキップした長さ, 診断情報, スキップ後の範囲) を返す
pub fn until(
    source: &Source,
    mut range: Range,
    expected: Vec<TokenKind>,
) -> (usize, VecDeque<Diagnostic>, Range) {
    let mut diagnostics = vec![].into_iter().collect();
    let mut skipped_width: usize = 0;
    while !range.is_empty() {
        let LexResult {
            token,
            remaining_range,
        } = lex(source, range);

        if expected.contains(&token.kind) {
            return (skipped_width, diagnostics, range);
        } else {
            skipped_width += token.full_width();

            // 期待していないトークンそのものの範囲
            let token_range = Range {
                start: range.start,
                end: range.start + token.token_width,
            };
            range = remaining_range;

            if token.kind == TokenKind::Error {
                diagnostics.push_back(unknown_token_error(token_range));
            } else {
                diagnostics.push_back(extra_token_error(token_range, token.kind));
            }
        }
    }
    (0, diagnostics, range)
}

const EXPR_BEGIN_TOKEN_KINDS: [TokenKind; 4] = [
    TokenKind::Ident,
    TokenKind::Number,
    TokenKind::Let,
    TokenKind::OpenParen,
];

pub fn until_expr_begin(source: &Source, range: Range) -> (usize, VecDeque<Diagnostic>, Range) {
    until(source, range, EXPR_BEGIN_TOKEN_KINDS.into_iter().collect())
}

pub fn until_expr_begin_or(
    source: &Source,
    range: Range,
    mut expected: Vec<TokenKind>,
) -> (usize, VecDeque<Diagnostic>, Range) {
    expected.append(&mut EXPR_BEGIN_TOKEN_KINDS.into_iter().collect());
    until(source, range, expected)
}

pub fn until_not_error(source: &Source, range: Range) -> (usize, VecDeque<Diagnostic>, Range) {
    let expected = syntax_node::ALL_TOKEN_KINDS
        .into_iter()
        .filter(|kind| kind != &TokenKind::Error)
        .collect();
    until(source, range, expected)
}

#[test]
fn test_skip_until() {
    fn test(expected: TokenKind, expected_skipped_num: usize, expected_skipped_width: usize) {
        const SOURCE_STR: &str = r#"1 a let ( ) = ; + - * / @"#;
        let source = Source::from_str(SOURCE_STR);
        let (skipped_width, diagnostics, range) = until(&source, source.range(), vec![expected]);
        assert_eq!(source.range().width(), skipped_width + range.width());
        assert_eq!(expected_skipped_num, diagnostics.len());
        assert_eq!(expected_skipped_width, skipped_width);
    }
    test(TokenKind::Number, 0, 0);
    test(TokenKind::Ident, 1, 2);
    test(TokenKind::Let, 2, 4);
    test(TokenKind::OpenParen, 3, 8);
    test(TokenKind::CloseParen, 4, 10);
    test(TokenKind::Equal, 5, 12);
    test(TokenKind::Semicolon, 6, 14);
    test(TokenKind::Plus, 7, 16);
    test(TokenKind::Minus, 8, 18);
    test(TokenKind::Ast, 9, 20);
    test(TokenKind::Slash, 10, 22);
    test(TokenKind::Error, 11, 24);
}
