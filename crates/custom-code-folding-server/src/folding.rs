use tower_lsp::lsp_types::FoldingRange;
use tower_lsp::lsp_types::FoldingRangeKind;

use crate::config::FoldPattern;

struct StackEntry {
    pattern_idx: usize,
    line: u32,
}

pub fn compute_folding_ranges(text: &str, patterns: &[FoldPattern]) -> Vec<FoldingRange> {
    let mut stack: Vec<StackEntry> = Vec::new();
    let mut ranges: Vec<FoldingRange> = Vec::new();

    for (line_num, line) in text.lines().enumerate() {
        let line_num = line_num as u32;

        // Check start patterns first
        let mut matched_start = false;
        for (idx, pattern) in patterns.iter().enumerate() {
            if pattern.start.is_match(line) {
                stack.push(StackEntry {
                    pattern_idx: idx,
                    line: line_num,
                });
                matched_start = true;
                break;
            }
        }

        if matched_start {
            continue;
        }

        // Check end patterns
        for (idx, pattern) in patterns.iter().enumerate() {
            if pattern.end.is_match(line) {
                // Pop the most recent matching start for this pattern
                if let Some(pos) = stack.iter().rposition(|e| e.pattern_idx == idx) {
                    let entry = stack.remove(pos);
                    ranges.push(FoldingRange {
                        start_line: entry.line,
                        start_character: None,
                        end_line: line_num,
                        end_character: None,
                        kind: Some(FoldingRangeKind::Region),
                        collapsed_text: None,
                    });
                }
                break;
            }
        }
    }

    ranges
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{FoldConfig, FoldConfigRaw};

    fn default_patterns() -> Vec<FoldPattern> {
        FoldConfig::from_raw(None).patterns
    }

    #[test]
    fn test_basic_region() {
        let text = "code\n# +++ Section 1\nline1\nline2\n# ---\nmore code";
        let patterns = default_patterns();
        let ranges = compute_folding_ranges(text, &patterns);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 1);
        assert_eq!(ranges[0].end_line, 4);
        assert_eq!(ranges[0].collapsed_text, None);
        assert_eq!(ranges[0].kind, Some(FoldingRangeKind::Region));
    }

    #[test]
    fn test_nested_regions() {
        let text = "\
# +++ Outer
  # +++ Inner
  code
  # ---
# ---";
        let patterns = default_patterns();
        let ranges = compute_folding_ranges(text, &patterns);
        assert_eq!(ranges.len(), 2);

        // Inner closes first (returned first in ranges)
        assert_eq!(ranges[0].start_line, 1);
        assert_eq!(ranges[0].end_line, 3);

        assert_eq!(ranges[1].start_line, 0);
        assert_eq!(ranges[1].end_line, 4);
    }

    #[test]
    fn test_unmatched_markers() {
        let text = "# +++ Unmatched start\ncode\n# --- extra end\n# ---";
        let patterns = default_patterns();
        let ranges = compute_folding_ranges(text, &patterns);
        // Should get one range from start to the first end, second end is unmatched
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);
    }

    #[test]
    fn test_multiple_pattern_types() {
        let text = "\
# +++ Plus section
code
# ---
// #region Named region
more code
// #endregion";
        let patterns = default_patterns();
        let ranges = compute_folding_ranges(text, &patterns);
        assert_eq!(ranges.len(), 2);

        assert_eq!(ranges[0].start_line, 0);
        assert_eq!(ranges[0].end_line, 2);

        assert_eq!(ranges[1].start_line, 3);
        assert_eq!(ranges[1].end_line, 5);
    }

    #[test]
    fn test_empty_document() {
        let patterns = default_patterns();
        let ranges = compute_folding_ranges("", &patterns);
        assert!(ranges.is_empty());
    }

    #[test]
    fn test_collapsed_text_is_none() {
        let text = "// +++ My Label Here\ncode\n// ---";
        let patterns = default_patterns();
        let ranges = compute_folding_ranges(text, &patterns);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].collapsed_text, None);
    }

    #[test]
    fn test_whitespace_variants() {
        // Extra spaces
        let text = "#  +++  Spaced Label\ncode\n#  ---";
        let patterns = default_patterns();
        let ranges = compute_folding_ranges(text, &patterns);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].collapsed_text, None);

        // Leading whitespace (indented)
        let text2 = "    # +++ Indented\n    code\n    # ---";
        let ranges2 = compute_folding_ranges(text2, &patterns);
        assert_eq!(ranges2.len(), 1);
        assert_eq!(ranges2[0].collapsed_text, None);
    }

    #[test]
    fn test_slash_comment_style() {
        let text = "// +++ Slash section\ncode\n// ---";
        let patterns = default_patterns();
        let ranges = compute_folding_ranges(text, &patterns);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].collapsed_text, None);
    }

    #[test]
    fn test_region_with_hash_comment() {
        let text = "# #region Hash Region\ncode\n# #endregion";
        let patterns = default_patterns();
        let ranges = compute_folding_ranges(text, &patterns);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].collapsed_text, None);
    }

    #[test]
    fn test_custom_pattern() {
        let raw = FoldConfigRaw {
            include_defaults: false,
            patterns: vec![crate::config::FoldPatternRaw {
                name: "begin-end".to_string(),
                start: r"^\s*//\s*BEGIN\s+(?P<label>.*?)\s*$".to_string(),
                end: r"^\s*//\s*END".to_string(),
            }],
        };
        let config = FoldConfig::from_raw(Some(raw));
        let text = "// BEGIN My Block\ncode\n// END";
        let ranges = compute_folding_ranges(text, &config.patterns);
        assert_eq!(ranges.len(), 1);
        assert_eq!(ranges[0].collapsed_text, None);
    }
}
