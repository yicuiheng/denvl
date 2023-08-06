use crate::source::{starts_with, Position, Range, Source};

pub fn trivia_width(source: &Source, range: Range) -> usize {
    let mut remaining_range = range;
    let start = remaining_range.start;
    while !remaining_range.is_empty() {
        let c = source.at(remaining_range.start);
        if c.is_whitespace() || c == '\t' {
            remaining_range.start.advance(1);
        } else if starts_with(source, "//", &remaining_range) {
            remaining_range.start.advance(2);
            remaining_range.skip_until(|range: &Range| starts_with(source, "\n", range));
            if starts_with(source, "\n", &remaining_range) {
                remaining_range.start.advance(1);
            }
        } else if starts_with(source, "/*", &remaining_range) {
            remaining_range.start.advance(2);
            remaining_range.skip_until(|range: &Range| starts_with(source, "*/", range));
            if starts_with(source, "*/", &remaining_range) {
                remaining_range.start.advance(2);
            }
        } else {
            break;
        }
    }
    Position::distance(remaining_range.start, start)
}

#[test]
fn test_trivia_width() {
    fn test(src: &str, expected: usize) {
        let source = Source::from_str(src);
        let actual = trivia_width(&source, source.range());
        assert_eq!(expected, actual);
    }

    test("  a", 2);
    test("\n \n a", 4);
    test(" // comment\n", 12);
    test(" /* comment */ a", 15);
}
