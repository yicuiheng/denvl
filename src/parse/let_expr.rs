use super::*;

// "let" IDENT "=" expr ";" expr

impl Parser {
    pub fn parse_let_expr(&mut self, source: &Source, mut range: Range) -> ParseResult {
        let LexResult {
            token,
            remaining_range,
        } = lex(source, range);
        assert_eq!(token.kind, TokenKind::Let);
        let let_token = token;
        range = remaining_range;

        let mut diagnostics = VecDeque::new();

        let (skipped_width, mut diagnostics_, mut range) =
            skip::until_expr_begin_or(source, range, vec![TokenKind::Equal]);
        diagnostics.append(&mut diagnostics_);

        let tokens = peek_token_kinds(source, range, 2);
        let (mut ident_token, equal_token) = match tokens.as_slice() {
            &[TokenKind::Ident, TokenKind::Equal] => {
                let ident_lex_result = lex(source, range);
                assert_eq!(ident_lex_result.token.kind, TokenKind::Ident);
                range = ident_lex_result.remaining_range;
                let equal_lex_result = lex(source, range);
                assert_eq!(equal_lex_result.token.kind, TokenKind::Equal);
                range = equal_lex_result.remaining_range;
                (ident_lex_result.token, equal_lex_result.token)
            }
            &[TokenKind::Ident, _] => {
                let ident_lex_result = lex(source, range);
                assert_eq!(ident_lex_result.token.kind, TokenKind::Ident);
                range = ident_lex_result.remaining_range;
                diagnostics.push_back(missed_token_error(range.start, vec![TokenKind::Equal]));
                (
                    ident_lex_result.token,
                    SyntaxToken::make_empty(TokenKind::Equal),
                )
            }
            &[TokenKind::Equal, _] => {
                diagnostics.push_back(missed_token_error(range.start, vec![TokenKind::Ident]));
                let equal_lex_result = lex(source, range);
                assert_eq!(equal_lex_result.token.kind, TokenKind::Equal);
                range = equal_lex_result.remaining_range;
                (
                    SyntaxToken::make_empty(TokenKind::Ident),
                    equal_lex_result.token,
                )
            }
            _ => {
                let mut result = self.parse_expr(source, range);
                let LexResult {
                    remaining_range, ..
                } = lex(source, range);
                let let_token_range = Range {
                    start: range.start,
                    end: remaining_range.start,
                };
                result
                    .diagnostics
                    .push_back(extra_token_error(let_token_range, TokenKind::Let));
                return result;
            }
        };
        ident_token.leading_trivia_width += skipped_width;

        let mut init_result = self.parse_expr(source, range);
        let init_expr = init_result.node;
        range = init_result.remaining_range;
        diagnostics.append(&mut init_result.diagnostics);

        let (skipped_width, mut diagnostics_, mut range) =
            skip::until_expr_begin_or(source, range, vec![TokenKind::Semicolon]);

        diagnostics.append(&mut diagnostics_);

        let semicolon_token = {
            let semicolon_lex_result = lex(source, range);
            if semicolon_lex_result.token.kind == TokenKind::Semicolon {
                range = semicolon_lex_result.remaining_range;
                let mut token = semicolon_lex_result.token;
                token.leading_trivia_width += skipped_width;
                token
            } else {
                diagnostics.push_back(missed_token_error(range.start, vec![TokenKind::Semicolon]));
                SyntaxToken::make_empty(TokenKind::Semicolon)
            }
        };

        let mut body_result = self.parse_expr(source, range);
        range = body_result.remaining_range;
        diagnostics.append(&mut body_result.diagnostics);

        let mut body_expr = body_result.node;

        if semicolon_token.full_width() == 0 {
            body_expr.extend_leading_trivia_width(skipped_width)
        }

        ParseResult {
            node: SyntaxNode::Let {
                let_token,
                ident_token,
                equal_token,
                init_expr: Box::new(init_expr),
                semicolon_token,
                body_expr: Box::new(body_expr),
            },
            diagnostics,
            remaining_range: range,
        }
    }
}

#[cfg(test)]
mod test {
    use super::test_util::*;
    use super::{ParseResult, Parser, Range, Source};
    use crate::diagnostic::{Diagnostic, DiagnosticError};
    use crate::syntax_node::{SyntaxNode, TokenKind};

    fn parse(source: &Source, range: Range) -> ParseResult {
        let mut parser = Parser::new();
        parser.parse_let_expr(source, range)
    }

    #[test]
    fn test_normal_scenarios() {
        check_tree_pattern!(
            parse,
            "let a = 1; a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
    }

    #[test]
    fn test_normal_scenarios_with_trivia() {
        check_tree_pattern!(
            parse,
            "let a = 1; a ",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let // comment \n a = 1; a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a // comment \n = 1; a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a = // comment \n 1; a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a = 1 // comment \n ; a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a = 1; // comment \n a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a = 1; a // comment",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );

        check_tree_pattern!(
            parse,
            "let /* comment */ a = 1; a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a /* comment */ = 1; a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a = /* comment */ 1; a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a = 1 /* comment */ ; a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a = 1; /* comment */ a",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "let a = 1; a /* comment */",
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
    }

    #[test]
    fn test_normal_scenarios_with_errors() {
        check_tree_and_diagnostic_pattern!(
            parse,
            "let 1",
            SyntaxNode::Int { .. },
            Diagnostic::Error(DiagnosticError::ExtraToken {
                kind: TokenKind::Let,
                ..
            })
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "let = 1; a", // 識別子が抜けている
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::MissedToken { .. })
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "let a 1; a", // '=' が抜けている
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::MissedToken { .. })
        );
        // TODO: ';' の抜けを補完する
        /* check_tree_and_diagnostic_pattern!(
            parse,
            "let a = 1\n a", // ';' が抜けている
            SyntaxNode::Let {
                init_expr: box SyntaxNode::Int { .. },
                body_expr: box SyntaxNode::Var { .. },
                ..
            },
            Diagnostic::Error(DiagnosticError::MissedToken { .. })
        ); */
    }
}
