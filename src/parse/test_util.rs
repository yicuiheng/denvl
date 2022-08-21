#[cfg(test)]
macro_rules! check_tree_pattern {
    ($parse:expr, $src:expr, $node_pat:pat) => {
        let source = crate::source::Source::from_str($src);
        let parse_result = $parse(&source, source.range());
        assert!(parse_result.remaining_range.is_empty());
        assert!(parse_result.diagnostics.is_empty());
        std::assert_matches::assert_matches!(parse_result.node, $node_pat);

        let restored_str = parse_result.node.restore(&source);
        assert_eq!(restored_str, format!("{}\n", $src));
    };
}

#[cfg(test)]
pub(crate) use check_tree_pattern;

#[cfg(test)]
macro_rules! check_tree_and_diagnostic_pattern {
    ($parse:expr, $src:expr, $node_pat:pat, $($diag_pat:pat)*) => {
        let source = crate::source::Source::from_str($src);
        let mut parse_result = $parse(&source, source.range());
        assert!(parse_result.remaining_range.is_empty());
        $(
            assert!(!parse_result.diagnostics.is_empty());
            let diagnostic = parse_result.diagnostics.pop_front().unwrap();
            std::assert_matches::assert_matches!(diagnostic, $diag_pat);
        )*
        std::assert_matches::assert_matches!(parse_result.node, $node_pat);
    };
}

#[cfg(test)]
pub(crate) use check_tree_and_diagnostic_pattern;
