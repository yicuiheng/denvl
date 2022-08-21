mod skip;
mod test_util;

mod additive_expr;
mod let_expr;
mod multive_expr;
mod primary_expr;

use crate::diagnostic::{
    extra_token_error, missed_token_error, unexpected_token_error, Diagnostic,
};
use crate::lex::{lex, trivia::trivia_width, LexResult};
use crate::source::{Range, Source};
use crate::syntax_node::{SyntaxNode, SyntaxToken, TokenKind};
use std::collections::VecDeque;

/*

expr =
  | "let" IDENT "=" expr ";" expr
  | additive_expr
  ;

additive_expr =
  | additive_expr ("+"|"-") additive_expr
  | multive_expr
  ;

multive_expr =
  | multive_expr ("*"|"/") multive_expr
  | primary_expr
  ;

primary_expr =
  | IDENT
  | NUM
  | "(" expr ")"
  ;

*/

pub struct ParseResult {
    pub node: SyntaxNode,
    pub diagnostics: VecDeque<Diagnostic>,
    pub remaining_range: Range,
}

pub struct Parser {
    // is_let_init: bool, TODO: ';' の抜けを補完する。e.g. let a = 1 \n 2 は let a = 1; 2 と解釈されるようにする。
}

pub fn parse(source: &Source) -> (SyntaxNode, VecDeque<Diagnostic>) {
    let mut parser = Parser::new();
    let mut range = source.range();
    let leading_trivia_width = trivia_width(source, range);
    range.start.advance(leading_trivia_width);

    let ParseResult {
        mut node,
        diagnostics,
        remaining_range,
    } = parser.parse_toplevel(source, range);
    assert!(remaining_range.is_empty());
    node.extend_leading_trivia_width(leading_trivia_width);
    (node, diagnostics)
}

impl Parser {
    pub fn new() -> Parser {
        Parser { /* is_let_init: false */ }
    }

    fn parse_toplevel(&mut self, source: &Source, range: Range) -> ParseResult {
        let (skipped_width, mut diagnostics, remaining_range) =
            skip::until_expr_begin(source, range);

        let ParseResult {
            mut node,
            diagnostics: mut diagnostics_,
            remaining_range,
        } = self.parse_expr(source, remaining_range);
        node.extend_leading_trivia_width(skipped_width);
        diagnostics.append(&mut diagnostics_);

        ParseResult {
            node,
            diagnostics,
            remaining_range,
        }
    }

    fn parse_expr(&mut self, source: &Source, range: Range) -> ParseResult {
        let LexResult { token, .. } = lex(source, range);
        use TokenKind::*;
        match token.kind {
            Let => self.parse_let_expr(source, range),
            Ident | Number | OpenParen => self.parse_additive_expr(source, range),
            _ => {
                // TODO: 'let' や識別子以外が来た時のエラー回復
                let mut diagnostics = VecDeque::new();
                diagnostics.push_back(unexpected_token_error(
                    range,
                    vec![Let, Ident, Number, OpenParen],
                    TokenKind::Error,
                ));
                ParseResult {
                    node: SyntaxNode::Error {
                        token: SyntaxToken {
                            kind: TokenKind::Error,
                            leading_trivia_width: 0,
                            token_width: range.width(),
                            trailing_trivia_width: 0,
                        },
                    },
                    diagnostics,
                    remaining_range: Range {
                        start: range.end,
                        end: range.end,
                    },
                }
            }
        }
    }
}

fn peek_token_kinds(source: &Source, mut range: Range, n: usize) -> Vec<TokenKind> {
    let mut result = vec![];
    for _ in 0..n {
        let LexResult {
            token,
            remaining_range,
            ..
        } = lex(source, range);
        range = remaining_range;
        result.push(token.kind);
    }
    result
}

#[cfg(test)]
use std::assert_matches::assert_matches;

#[test]
fn only_trivia() {
    let source = Source::from_str(r" ");
    let (node, diagnostics) = parse(&source);
    assert_eq!(diagnostics.len(), 1);
    assert_matches!(node, SyntaxNode::Error { .. });
}
