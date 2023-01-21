use crate::source::{Position, Range};
use crate::syntax_node::TokenKind;

#[derive(Debug, PartialEq, Eq)]
pub enum Diagnostic {
    Error(DiagnosticError),
    Warning(DiagnosticWarning),
}

impl Diagnostic {
    pub fn pos(&self) -> Position {
        match self {
            Diagnostic::Error(DiagnosticError::UnexpectedToken { range, .. }) => range.start,
            Diagnostic::Error(DiagnosticError::MissedToken { pos, .. }) => *pos,
            Diagnostic::Error(DiagnosticError::UnknownToken { range }) => range.start,
            Diagnostic::Error(DiagnosticError::ExtraToken { range, .. }) => range.start,
            Diagnostic::Error(DiagnosticError::Unknown { range }) => range.start,
            Diagnostic::Warning(..) => Position::start(),
        }
    }

    pub fn make_msg(&self) -> String {
        match self {
            Diagnostic::Error(DiagnosticError::UnexpectedToken {
                expected, actual, ..
            }) => {
                format!("unexpected token. expected {expected:?}, but actual is {actual:?}")
            }
            Diagnostic::Error(DiagnosticError::MissedToken { expected, .. }) => {
                format!("missing expected token. expected {expected:?}")
            }
            Diagnostic::Error(DiagnosticError::UnknownToken { .. }) => "unknown token".to_string(),
            Diagnostic::Error(DiagnosticError::ExtraToken { .. }) => "extra token".to_string(),
            Diagnostic::Error(DiagnosticError::Unknown { .. }) => "unknown error".to_string(),
            Diagnostic::Warning(..) => todo!(),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiagnosticError {
    UnexpectedToken {
        // 期待していないトークン
        range: Range,
        expected: Vec<TokenKind>,
        actual: TokenKind,
    },
    MissedToken {
        // 必要なトークンがない
        pos: Position,
        expected: Vec<TokenKind>,
    },
    UnknownToken {
        // 字句解析エラー
        range: Range,
    },
    ExtraToken {
        // 余分なトークン (e.g. '1 + = 2' の '=')
        range: Range,
        kind: TokenKind,
    },
    Unknown {
        range: Range,
    },
}

#[derive(Debug, PartialEq, Eq)]
pub enum DiagnosticWarning {}

pub fn unexpected_token_error(
    range: Range,
    expected: Vec<TokenKind>,
    actual: TokenKind,
) -> Diagnostic {
    Diagnostic::Error(DiagnosticError::UnexpectedToken {
        range,
        expected,
        actual,
    })
}

pub fn missed_token_error(pos: Position, expected: Vec<TokenKind>) -> Diagnostic {
    Diagnostic::Error(DiagnosticError::MissedToken { pos, expected })
}

pub fn unknown_token_error(range: Range) -> Diagnostic {
    Diagnostic::Error(DiagnosticError::UnknownToken { range })
}

pub fn extra_token_error(range: Range, kind: TokenKind) -> Diagnostic {
    Diagnostic::Error(DiagnosticError::ExtraToken { range, kind })
}
