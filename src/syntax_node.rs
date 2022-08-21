#[cfg(test)]
use crate::source::{Position, Range, Source};

#[derive(Debug)]
pub enum SyntaxNode {
    Int {
        token: SyntaxToken,
    },
    Var {
        token: SyntaxToken,
    },
    Let {
        let_token: SyntaxToken,
        ident_token: SyntaxToken,
        equal_token: SyntaxToken,
        init_expr: Box<SyntaxNode>,
        semicolon_token: SyntaxToken,
        body_expr: Box<SyntaxNode>,
    },
    BinOp {
        lhs_expr: Box<SyntaxNode>,
        binop_token: SyntaxToken,
        rhs_expr: Box<SyntaxNode>,
    },
    Paren {
        open_paren_token: SyntaxToken,
        inner_expr: Box<SyntaxNode>,
        close_paren_token: SyntaxToken,
    },
    Error {
        token: SyntaxToken,
    },
}

impl SyntaxNode {
    pub fn extend_leading_trivia_width(&mut self, n: usize) {
        use SyntaxNode::*;
        match self {
            Int { token } => token.leading_trivia_width += n,
            Var { token } => token.leading_trivia_width += n,
            Let { let_token, .. } => let_token.leading_trivia_width += n,
            BinOp { lhs_expr, .. } => lhs_expr.extend_leading_trivia_width(n),
            Paren {
                open_paren_token, ..
            } => open_paren_token.leading_trivia_width += n,
            Error { token } => token.leading_trivia_width += n,
        }
    }

    pub fn extend_trailing_trivia_width(&mut self, n: usize) {
        use SyntaxNode::*;
        match self {
            Int { token } => token.trailing_trivia_width += n,
            Var { token } => token.trailing_trivia_width += n,
            Let { body_expr, .. } => body_expr.extend_trailing_trivia_width(n),
            BinOp { rhs_expr, .. } => rhs_expr.extend_trailing_trivia_width(n),
            Paren {
                close_paren_token, ..
            } => close_paren_token.trailing_trivia_width += n,
            Error { token } => token.trailing_trivia_width += n,
        }
    }

    #[cfg(test)]
    pub fn restore(&self, source: &Source) -> String {
        let (pos, result) = string_from_node(source, Position::start(), self);
        assert_eq!(pos, source.range().end);
        result
    }
}

#[cfg(test)]
fn string_from_node(source: &Source, pos: Position, node: &SyntaxNode) -> (Position, String) {
    use SyntaxNode::*;
    match node {
        Int { token } | Var { token } | Error { token } => string_from_token(source, pos, token),
        Let {
            let_token,
            ident_token,
            equal_token,
            init_expr,
            semicolon_token,
            body_expr,
        } => {
            let mut result = String::new();
            let (pos, let_str) = string_from_token(source, pos, let_token);
            result += &let_str;
            let (pos, ident_str) = string_from_token(source, pos, ident_token);
            result += &ident_str;
            let (pos, equal_str) = string_from_token(source, pos, equal_token);
            result += &equal_str;
            let (pos, init_str) = string_from_node(source, pos, &init_expr);
            result += &init_str;
            let (pos, semicolon_str) = string_from_token(source, pos, semicolon_token);
            result += &semicolon_str;
            let (pos, body_str) = string_from_node(source, pos, &body_expr);
            result += &body_str;
            (pos, result)
        }
        BinOp {
            lhs_expr,
            binop_token,
            rhs_expr,
        } => {
            let mut result = String::new();
            let (pos, lhs_str) = string_from_node(source, pos, lhs_expr);
            result += &lhs_str;
            let (pos, binop_str) = string_from_token(source, pos, binop_token);
            result += &binop_str;
            let (pos, rhs_expr) = string_from_node(source, pos, rhs_expr);
            result += &rhs_expr;
            (pos, result)
        }
        Paren {
            open_paren_token,
            inner_expr,
            close_paren_token,
        } => {
            let mut result = String::new();
            let (pos, open_paren_str) = string_from_token(source, pos, open_paren_token);
            result += &open_paren_str;
            let (pos, inner_str) = string_from_node(source, pos, inner_expr);
            result += &inner_str;
            let (pos, close_paren_str) = string_from_token(source, pos, close_paren_token);
            result += &close_paren_str;
            (pos, result)
        }
    }
}

#[cfg(test)]
fn string_from_token(
    source: &Source,
    mut pos: Position,
    token: &SyntaxToken,
) -> (Position, String) {
    let token_start_pos = pos;
    pos.advance(token.full_width());
    let token_end_pos = pos;
    let token_range = Range {
        start: token_start_pos,
        end: token_end_pos,
    };
    (pos, source.get(&token_range).into_iter().collect())
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum TokenKind {
    Number,
    Ident,
    Let,
    OpenParen,
    CloseParen,
    Equal,
    Semicolon,
    Plus,
    Minus,
    Ast,
    Slash,
    Error,
}

pub const ALL_TOKEN_KINDS: [TokenKind; 12] = [
    TokenKind::Number,
    TokenKind::Ident,
    TokenKind::Let,
    TokenKind::OpenParen,
    TokenKind::CloseParen,
    TokenKind::Equal,
    TokenKind::Semicolon,
    TokenKind::Plus,
    TokenKind::Minus,
    TokenKind::Ast,
    TokenKind::Slash,
    TokenKind::Error,
];

#[derive(Debug, PartialEq, Eq)]
pub struct SyntaxToken {
    pub kind: TokenKind,
    pub leading_trivia_width: usize,
    pub token_width: usize,
    pub trailing_trivia_width: usize,
}

impl SyntaxToken {
    pub fn full_width(&self) -> usize {
        self.leading_trivia_width + self.token_width + self.trailing_trivia_width
    }

    pub fn make_empty(kind: TokenKind) -> Self {
        SyntaxToken {
            kind,
            leading_trivia_width: 0,
            token_width: 0,
            trailing_trivia_width: 0,
        }
    }
}
