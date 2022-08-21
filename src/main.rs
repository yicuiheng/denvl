#![feature(assert_matches, box_patterns)]

mod diagnostic;
mod lex;
mod parse;
mod source;
mod syntax_node;

use crate::diagnostic::Diagnostic;
use crate::source::{Range, Source};
use std::collections::VecDeque;
use std::path::PathBuf;

fn main() {
    let args: Vec<_> = std::env::args().collect();
    let filename = args.get(1).expect("<filename> required");
    let pathbuf = PathBuf::from(filename.as_str());
    let source = Source::new(pathbuf.as_path()).expect("fail to read file");
    let (syntax_node, diagnostics) = parse::parse(&source);

    print_diagnostics(filename, &source, diagnostics);

    eprintln!("parsed tree: {:?}", syntax_node);
}

fn print_diagnostics(filename: &str, source: &Source, mut diagnostics: VecDeque<Diagnostic>) {
    diagnostics
        .make_contiguous()
        .sort_by(|lhs, rhs| lhs.pos().cmp(&rhs.pos()));
    let mut line = 0;
    let mut column = 1;
    let mut range = source.range();

    while !range.is_empty() {
        let c = source.at(range.start);
        let diagnostic = diagnostics.front();
        if let Some(diagnostic) = diagnostic {
            if diagnostic.pos() == range.start {
                print_diagnostic(filename, source, line, column, diagnostic);
                diagnostics.pop_front();
            }
            if c == '\n' {
                line += 1;
                column = 0;
            } else {
                column += 1;
            }
            range.start.advance(1);
        } else {
            break;
        }
    }
}

fn print_diagnostic(
    filename: &str,
    source: &Source,
    line: usize,
    column: usize,
    diagnostic: &Diagnostic,
) {
    eprintln!(
        "error at {}({}:{}) {}",
        filename,
        line,
        column,
        diagnostic.make_msg()
    );
    let end = diagnostic.pos();
    let mut start = end;
    start.backward(column);
    eprintln!(
        "> {}",
        source
            .get(&Range { start, end })
            .into_iter()
            .collect::<String>()
    );
    eprintln!("{}^", " ".repeat(column + 2));
}
