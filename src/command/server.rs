use crate::consts;
use crate::diagnostic::Diagnostic;
use crate::named_pipe::NamedPipeServer;
use crate::parse;
use crate::source::{Range, Source};
use std::collections::VecDeque;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::{thread, time};

pub fn exec() {
    let error = PathBuf::from(crate::consts::COMMON_NAME).join("error.txt");
    let mut error_file = File::options()
        .create(true)
        .read(false)
        .write(true)
        .append(true)
        .open(&error)
        .unwrap();

    let pipe_name = PathBuf::from(consts::COMMON_NAME);
    let mut server = match NamedPipeServer::create(pipe_name) {
        Ok(pipe) => pipe,
        Err(e) => {
            writeln!(error_file, "failed to create named pipe server. {e:?}").unwrap();
            return;
        }
    };

    let daemonize = daemonize::Daemonize::new();

    if let Err(e) = daemonize.start() {
        writeln!(error_file, "daemonize failed. {e}").unwrap();
        return;
    };

    server
        .writeline(crate::consts::SERVER_STARTING_HEADER.to_string())
        .unwrap();

    loop {
        let line = match server.readline() {
            Ok(line) => line,
            Err(e) => {
                writeln!(error_file, "error: {e}").unwrap();
                unreachable!()
            }
        };
        if line == "shutdown" {
            server.writeline("done".to_string()).unwrap();
            break;
        }
        let path = PathBuf::from(line);
        exec_impl(&mut server, path);
        thread::sleep(time::Duration::from_millis(1000));
        server.writeline("done".to_string()).unwrap();
        thread::sleep(time::Duration::from_millis(1000));
    }
}

fn exec_impl(server: &mut NamedPipeServer, path: PathBuf) {
    let source = Source::new(path.as_path()).expect("fail to read file");
    let (syntax_node, diagnostics) = parse::parse(&source);

    print_diagnostics(server, path.to_str().unwrap(), &source, diagnostics).unwrap();

    server
        .writeline(format!("parsed tree: {syntax_node:?}"))
        .unwrap();
}

fn print_diagnostics(
    server: &mut NamedPipeServer,
    filename: &str,
    source: &Source,
    mut diagnostics: VecDeque<Diagnostic>,
) -> Result<(), std::io::Error> {
    diagnostics.make_contiguous().sort_by_key(|lhs| lhs.pos());
    let mut line = 0;
    let mut column = 1;
    let mut range = source.range();

    while !range.is_empty() {
        let c = source.at(range.start);
        let diagnostic = diagnostics.front();
        if let Some(diagnostic) = diagnostic {
            if diagnostic.pos() == range.start {
                print_diagnostic(server, filename, source, line, column, diagnostic)?;
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
    Ok(())
}

fn print_diagnostic(
    server: &mut NamedPipeServer,
    filename: &str,
    source: &Source,
    line: usize,
    column: usize,
    diagnostic: &Diagnostic,
) -> Result<(), std::io::Error> {
    server.writeline(format!(
        "error at {}({}:{}) {}",
        filename,
        line,
        column,
        diagnostic.make_msg()
    ))?;
    let end = diagnostic.pos();
    let mut start = end;
    start.backward(column);
    server.writeline(format!(
        "> {}",
        source.get(&Range { start, end }).iter().collect::<String>()
    ))?;
    server.writeline(format!("{}^", " ".repeat(column + 2)))?;
    Ok(())
}
