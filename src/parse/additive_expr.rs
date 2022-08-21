use super::*;

/*
additive_expr =
  | additive_expr ("+"|"-") additive_expr
  | multive_expr
  ;
*/

impl Parser {
    pub fn parse_additive_expr(&mut self, source: &Source, mut range: Range) -> ParseResult {
        let lhs_expr_result = self.parse_multive_expr(source, range);
        range = lhs_expr_result.remaining_range;
        let mut diagnostics = lhs_expr_result.diagnostics;
        let mut expr = lhs_expr_result.node;

        loop {
            let (skipped_width, mut diagnostics_, range_) = skip::until_expr_begin_or(
                source,
                range,
                vec![
                    TokenKind::Semicolon,
                    TokenKind::CloseParen,
                    TokenKind::Error,
                    TokenKind::Plus,
                    TokenKind::Minus,
                ],
            );
            range = range_;
            diagnostics.append(&mut diagnostics_);

            let (mut binop_token, range_) = if let Some((binop_token, mut diagnostics_, range)) =
                self.parse_additive_operator(source, range)
            {
                diagnostics.append(&mut diagnostics_);
                (binop_token, range)
            } else {
                return ParseResult {
                    node: expr,
                    remaining_range: range,
                    diagnostics,
                };
            };
            range = range_;
            binop_token.leading_trivia_width += skipped_width;

            let mut rhs_expr_result = self.parse_multive_expr(source, range);
            diagnostics.append(&mut rhs_expr_result.diagnostics);
            range = rhs_expr_result.remaining_range;
            let rhs_expr = rhs_expr_result.node;

            let mut tmp = SyntaxNode::Error {
                token: SyntaxToken::make_empty(TokenKind::Error),
            };
            std::mem::swap(&mut tmp, &mut expr);
            expr = SyntaxNode::BinOp {
                lhs_expr: Box::new(tmp),
                binop_token,
                rhs_expr: Box::new(rhs_expr),
            };
        }
    }

    fn parse_additive_operator(
        &mut self,
        source: &Source,
        range: Range,
    ) -> Option<(SyntaxToken, VecDeque<Diagnostic>, Range)> {
        if range.is_empty() {
            return None;
        }

        let (skipped_width, mut diagnostics, mut range) = skip::until_expr_begin_or(
            source,
            range,
            vec![
                TokenKind::Plus,
                TokenKind::Minus,
                TokenKind::Semicolon,
                TokenKind::CloseParen,
            ],
        );
        let LexResult {
            mut token,
            remaining_range,
        } = lex(source, range);

        match token.kind {
            TokenKind::Plus | TokenKind::Minus => {
                range = remaining_range;
                token.leading_trivia_width += skipped_width;

                let (skipped_width, mut diagnostics_, range) = skip::until_not_error(source, range);
                diagnostics.append(&mut diagnostics_);
                token.trailing_trivia_width += skipped_width;
                return Some((token, diagnostics, range));
            }
            TokenKind::Semicolon | TokenKind::CloseParen => return None,
            _ => {
                // 式の始まり
                // 演算子書き忘れ
                // '+' だとみなしてエラーを追加する

                let token_range_start = range.start;
                let mut token_range_end = token_range_start;
                token_range_end.advance(token.token_width);
                let token_range = Range {
                    start: token_range_start,
                    end: token_range_end,
                };
                diagnostics.push_back(missed_token_error(
                    token_range.start,
                    vec![
                        TokenKind::Plus,
                        TokenKind::Minus,
                        TokenKind::Ast,
                        TokenKind::Slash,
                    ],
                ));
                let token = SyntaxToken {
                    kind: TokenKind::Plus,
                    leading_trivia_width: skipped_width,
                    token_width: 0,
                    trailing_trivia_width: 0,
                };

                return Some((token, diagnostics, range));
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::test_util::*;
    use super::{ParseResult, Parser, Range, Source};
    use crate::diagnostic::{Diagnostic, DiagnosticError};
    use crate::syntax_node::{SyntaxNode, SyntaxToken, TokenKind};

    fn parse(source: &Source, range: Range) -> ParseResult {
        let mut parser = Parser::new();
        parser.parse_additive_expr(source, range)
    }

    #[test]
    fn test_normal_scenarios() {
        check_tree_pattern!(
            parse,
            "a + 1",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                binop_token: SyntaxToken {
                    kind: TokenKind::Plus,
                    ..
                }
            }
        );
        check_tree_pattern!(
            parse,
            "a - 1",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                binop_token: SyntaxToken {
                    kind: TokenKind::Minus,
                    ..
                }
            }
        );

        check_tree_pattern!(
            parse,
            "a + 1 - 2",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::BinOp {
                    lhs_expr: box SyntaxNode::Var { .. },
                    rhs_expr: box SyntaxNode::Int { .. },
                    binop_token: SyntaxToken {
                        kind: TokenKind::Plus,
                        ..
                    }
                },
                rhs_expr: box SyntaxNode::Int { .. },
                binop_token: SyntaxToken {
                    kind: TokenKind::Minus,
                    ..
                }
            }
        );
    }

    #[test]
    fn test_operator_precedence() {
        check_tree_pattern!(
            parse,
            "1 + 2 * 3",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Int { .. },
                binop_token: SyntaxToken {
                    kind: TokenKind::Plus,
                    ..
                },
                rhs_expr: box SyntaxNode::BinOp {
                    lhs_expr: box SyntaxNode::Int { .. },
                    binop_token: SyntaxToken {
                        kind: TokenKind::Ast,
                        ..
                    },
                    rhs_expr: box SyntaxNode::Int { .. },
                }
            }
        );
    }

    #[test]
    fn test_normal_scenarios_with_trivia() {
        check_tree_pattern!(
            parse,
            "a + 1 ",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "a // comment\n + 1 ",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "a + // comment\n 1 ",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "a + 1 // comment",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "a /* comment */ + 1 ",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "a + /* comment*/ 1 ",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "a + 1 /* comment */",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            }
        );
    }

    #[test]
    fn test_normal_scenarios_with_errors() {
        check_tree_and_diagnostic_pattern!(
            parse,
            "a @ + 1 ",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::UnknownToken { .. })
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "a + @ 1 ",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::UnknownToken { .. })
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "a + 1 @",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Var { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::UnknownToken { .. })
        );

        check_tree_and_diagnostic_pattern!(
            parse,
            "1 2", // 演算子が抜けている
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Int { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::MissedToken { .. })
        );

        check_tree_and_diagnostic_pattern!(
            parse,
            "1 @ 2", // 演算子が抜けており、解釈できないトークンがある
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Int { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::UnknownToken { .. })
            Diagnostic::Error(DiagnosticError::MissedToken { .. })
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "1 + @ 2",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Int { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::UnknownToken { .. })
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "1 @ + 2",
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Int { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::UnknownToken { .. })
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "1 @ = + 2", // 余分な '=' トークンがある
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Int { .. },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::UnknownToken { .. })
            Diagnostic::Error(DiagnosticError::ExtraToken { .. })
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "1 + + 2", // 式が抜けている
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::BinOp {
                    lhs_expr: box SyntaxNode::Int { .. },
                    rhs_expr: box SyntaxNode::Error { .. },
                    ..
                },
                rhs_expr: box SyntaxNode::Int { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::MissedToken { .. }) // TODO: 式がないことを表せるようにする
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "1 * + 2", // 式が抜けている
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::BinOp {
                    lhs_expr: box SyntaxNode::Int { .. },
                    binop_token: SyntaxToken {
                        kind: TokenKind::Ast,
                        ..
                    },
                    rhs_expr: box SyntaxNode::Error { .. },
                },
                binop_token: SyntaxToken {
                    kind: TokenKind::Plus,
                    ..
                },
                rhs_expr: box SyntaxNode::Int { .. },
            },
            Diagnostic::Error(DiagnosticError::MissedToken { .. }) // TODO: 式がないことを表せるようにする
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "1 + * 2", // 式が抜けている
            SyntaxNode::BinOp {
                lhs_expr: box SyntaxNode::Int { .. },
                binop_token: SyntaxToken {
                    kind: TokenKind::Plus,
                    ..
                },
                rhs_expr: box SyntaxNode::BinOp {
                    lhs_expr: box SyntaxNode::Error { .. },
                    binop_token: SyntaxToken {
                        kind: TokenKind::Ast,
                        ..
                    },
                    rhs_expr: box SyntaxNode::Int { .. },
                },
            },
            Diagnostic::Error(DiagnosticError::MissedToken { .. }) // TODO: 式がないことを表せるようにする
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "(1+)", // 式が抜けている
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::BinOp {
                    lhs_expr: box SyntaxNode::Int { .. },
                    rhs_expr: box SyntaxNode::Error { .. },
                    ..
                },
                ..
            },
            Diagnostic::Error(DiagnosticError::MissedToken { .. }) // TODO: 式がないことを表せるようにする
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "(let a = 1 + ; a)", // 式が抜けている
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::Let {
                    init_expr: box SyntaxNode::BinOp {
                        lhs_expr: box SyntaxNode::Int { .. },
                        rhs_expr: box SyntaxNode::Error { .. },
                        ..
                    },
                    ..
                },
                ..
            },
            Diagnostic::Error(DiagnosticError::MissedToken { .. }) // TODO: 式がないことを表せるようにする
        );
    }
}
