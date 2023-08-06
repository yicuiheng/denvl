use super::*;

/*
primary_expr =
  | IDENT
  | NUM
  | "(" expr ")"
  ;
*/

impl Parser {
    pub fn parse_primary_expr(&mut self, source: &Source, mut range: Range) -> ParseResult {
        let LexResult {
            token,
            remaining_range,
        } = lex(source, range);
        range = remaining_range;

        let mut diagnostics = VecDeque::new();

        let mut node = match token.kind {
            TokenKind::Number => SyntaxNode::Int { token },
            TokenKind::Ident => SyntaxNode::Var { token },
            TokenKind::OpenParen => {
                let mut open_paren_token = token;
                let (skipped_width, mut diagnostics_, range_) =
                    skip::until_not_error(source, range);
                diagnostics.append(&mut diagnostics_);
                range = range_;
                open_paren_token.trailing_trivia_width += skipped_width;

                let mut inner_expr_result = self.parse_expr(source, range);
                range = inner_expr_result.remaining_range;
                diagnostics.append(&mut inner_expr_result.diagnostics);
                let inner_expr = inner_expr_result.node;

                let LexResult {
                    token,
                    remaining_range,
                } = lex(source, range);

                let (close_paren_token, range_) = if token.kind == TokenKind::CloseParen {
                    (token, remaining_range)
                } else {
                    diagnostics
                        .push_back(missed_token_error(range.start, vec![TokenKind::CloseParen]));
                    (
                        SyntaxToken {
                            kind: TokenKind::CloseParen,
                            leading_trivia_width: 0,
                            token_width: 0,
                            trailing_trivia_width: 0,
                        },
                        range,
                    )
                };
                range = range_;

                SyntaxNode::Paren {
                    open_paren_token,
                    inner_expr: Box::new(inner_expr),
                    close_paren_token,
                }
            }
            _ => unreachable!(),
        };
        let (skipped_width, mut diagnostics_, range) = skip::until_not_error(source, range);
        diagnostics.append(&mut diagnostics_);
        node.extend_trailing_trivia_width(skipped_width);

        ParseResult {
            node,
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
    use crate::syntax_node::SyntaxNode;

    fn parse(source: &Source, range: Range) -> ParseResult {
        let mut parser = Parser::new();
        parser.parse_primary_expr(source, range)
    }

    #[test]
    fn test_normal_scenarios() {
        check_tree_pattern!(parse, "1", SyntaxNode::Int { .. });
        check_tree_pattern!(parse, "a", SyntaxNode::Var { .. });
        check_tree_pattern!(
            parse,
            "(a)",
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
    }

    #[test]
    fn test_normal_scenarios_with_trivia() {
        check_tree_pattern!(parse, "1 ", SyntaxNode::Int { .. });
        check_tree_pattern!(parse, "1 // comment ", SyntaxNode::Int { .. });
        check_tree_pattern!(parse, "1 /* comment */ ", SyntaxNode::Int { .. });

        check_tree_pattern!(parse, "a ", SyntaxNode::Var { .. });
        check_tree_pattern!(parse, "a // comment ", SyntaxNode::Var { .. });
        check_tree_pattern!(parse, "a /* comment */", SyntaxNode::Var { .. });

        check_tree_pattern!(
            parse,
            "( a ) ",
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "( // comment\n a ) ",
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "( a // comment\n ) ",
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "( a ) // comment",
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "( /* comment */ a ) ",
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "( a /* comment */ ) ",
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
        check_tree_pattern!(
            parse,
            "( a ) /* comment */ ",
            SyntaxNode::Paren {
                inner_expr: box SyntaxNode::Var { .. },
                ..
            }
        );
    }

    #[test]
    fn test_normal_scenarios_with_error() {
        check_tree_and_diagnostic_pattern!(
            parse,
            "1 @",
            SyntaxNode::Int { .. },
            Diagnostic::Error(DiagnosticError::UnknownToken { .. })
        );
        check_tree_and_diagnostic_pattern!(
            parse,
            "a @",
            SyntaxNode::Var { .. },
            Diagnostic::Error(DiagnosticError::UnknownToken { .. })
        );

        check_tree_and_diagnostic_pattern!(
            parse,
            "(a",
            SyntaxNode::Paren { .. },
            Diagnostic::Error(DiagnosticError::MissedToken { .. })
        );
    }
}
