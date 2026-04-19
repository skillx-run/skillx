use assert_cmd::Command;
use predicates::prelude::*;
use std::time::Duration;
use tempfile::TempDir;

fn skillx() -> Command {
    Command::cargo_bin("skillx").unwrap()
}

/// Create a skillx command with `SKILLX_HOME` set to a temp directory for isolation.
fn skillx_with_home(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("skillx").unwrap();
    cmd.env("SKILLX_HOME", home.path());
    cmd
}

/// Absolute path to an example skill directory.
fn example_skill(name: &str) -> String {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap()
        .join("examples")
        .join("skills")
        .join(name)
        .to_string_lossy()
        .to_string()
}

/// Absolute path to a first-party skill directory (top-level `skills/`).
fn firstparty_skill(name: &str) -> String {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .parent()
        .unwrap()
        .join("skills")
        .join(name)
        .to_string_lossy()
        .to_string()
}

/// Absolute path to a test fixture directory.
fn fixture(name: &str) -> String {
    let manifest = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest
        .join("tests")
        .join("fixtures")
        .join(name)
        .to_string_lossy()
        .to_string()
}

// ==================== CLI Basics ====================

#[test]
fn test_help() {
    skillx().arg("--help").assert().success();
}

#[test]
fn test_version() {
    skillx()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicates::str::contains("skillx"));
}

#[test]
fn test_unknown_command() {
    skillx().arg("foobar").assert().failure().code(2);
}

// ==================== scan ====================

#[test]
fn test_scan_valid_skill() {
    skillx()
        .args(["scan", "./tests/fixtures/valid-skill"])
        .assert()
        .success();
}

#[test]
fn test_scan_dangerous_skill_default() {
    // dangerous-skill has BLOCK findings (SC-010, SC-011); default fail-on is danger
    skillx()
        .args(["scan", "./tests/fixtures/dangerous-skill"])
        .assert()
        .failure();
}

#[test]
fn test_scan_dangerous_skill_fail_on_warn() {
    skillx()
        .args([
            "scan",
            "./tests/fixtures/dangerous-skill",
            "--fail-on",
            "warn",
        ])
        .assert()
        .failure();
}

#[test]
fn test_scan_json_format() {
    let output = skillx()
        .args(["scan", "./tests/fixtures/valid-skill", "--format", "json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value =
        serde_json::from_slice(&output).expect("stdout should be valid JSON");
    assert!(parsed["findings"].is_array());
}

#[test]
fn test_scan_sarif_format() {
    let output = skillx()
        .args(["scan", "./tests/fixtures/valid-skill", "--format", "sarif"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8_lossy(&output);
    assert!(
        text.contains("$schema"),
        "SARIF output should contain $schema"
    );
}

#[test]
fn test_scan_nonexistent() {
    skillx()
        .args(["scan", "./nonexistent-dir-that-does-not-exist"])
        .assert()
        .failure();
}

#[test]
fn test_scan_binary_skill() {
    // binary-skill has SC-001 (BLOCK) — scan fails at default danger threshold
    let output = skillx()
        .args(["scan", "./tests/fixtures/binary-skill", "--format", "json"])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON");
    let findings = parsed["findings"].as_array().expect("findings array");
    assert!(
        findings.iter().any(|f| f["rule_id"] == "SC-001"),
        "should detect SC-001 binary in binary-skill"
    );
}

// ==================== agents ====================

#[test]
fn test_agents_runs() {
    skillx().arg("agents").assert().success();
}

#[test]
fn test_agents_all() {
    skillx()
        .args(["agents", "--all"])
        .assert()
        .success()
        .stderr(predicates::str::contains("supported").or(predicates::str::contains("agent")));
}

// ==================== info ====================

#[test]
fn test_info_valid_skill() {
    skillx()
        .args(["info", "./tests/fixtures/valid-skill"])
        .assert()
        .success();
}

#[test]
fn test_info_nonexistent() {
    skillx()
        .args(["info", "./nonexistent-dir-that-does-not-exist"])
        .assert()
        .failure();
}

// ==================== init ====================

#[test]
fn test_init_creates_toml() {
    let dir = TempDir::new().unwrap();
    skillx()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();
    assert!(dir.path().join("skillx.toml").exists());
}

#[test]
fn test_init_fails_if_exists() {
    let dir = TempDir::new().unwrap();
    std::fs::write(
        dir.path().join("skillx.toml"),
        "[project]\nname = \"test\"\n",
    )
    .unwrap();
    skillx()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .failure();
}

// ==================== cache ====================

#[test]
fn test_cache_ls() {
    skillx().args(["cache", "ls"]).assert().success();
}

#[test]
fn test_cache_clean() {
    skillx().args(["cache", "clean"]).assert().success();
}

// ==================== Example Skills: scan ====================

#[test]
fn test_scan_example_hello_world() {
    skillx()
        .args(["scan", &example_skill("hello-world")])
        .assert()
        .success();
}

#[test]
fn test_scan_example_name_poem() {
    skillx()
        .args(["scan", &example_skill("name-poem")])
        .assert()
        .success();
}

#[test]
fn test_scan_example_code_review() {
    skillx()
        .args(["scan", &example_skill("code-review")])
        .assert()
        .success();
}

#[test]
fn test_scan_example_commit_message() {
    skillx()
        .args(["scan", &example_skill("commit-message")])
        .assert()
        .success();
}

#[test]
fn test_scan_example_testing_guide() {
    // Multi-file skill: SKILL.md + references/patterns.md
    skillx()
        .args(["scan", &example_skill("testing-guide")])
        .assert()
        .success();
}

#[test]
fn test_scan_firstparty_setup_skillx() {
    skillx()
        .args(["scan", &firstparty_skill("setup-skillx")])
        .assert()
        .success();
}

#[test]
fn test_scan_example_dangerous_blocked() {
    // dangerous-example has BLOCK-level findings from scripts/payload.sh
    skillx()
        .args(["scan", &example_skill("dangerous-example")])
        .assert()
        .failure();
}

#[test]
fn test_scan_example_dangerous_json_findings() {
    let output = skillx()
        .args([
            "scan",
            &example_skill("dangerous-example"),
            "--format",
            "json",
        ])
        .assert()
        .failure()
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("valid JSON");
    let findings = parsed["findings"].as_array().expect("findings array");
    // Should have multiple findings from both SKILL.md and scripts/payload.sh
    assert!(
        findings.len() > 1,
        "dangerous-example should have multiple findings"
    );
    // Should include script-level findings (SC-*) from payload.sh
    assert!(
        findings
            .iter()
            .any(|f| f["rule_id"].as_str().is_some_and(|r| r.starts_with("SC-"))),
        "should have script-level findings from payload.sh"
    );
    // Should include markdown-level findings (MD-*) from SKILL.md
    assert!(
        findings
            .iter()
            .any(|f| f["rule_id"].as_str().is_some_and(|r| r.starts_with("MD-"))),
        "should have markdown-level findings from SKILL.md"
    );
}

// ==================== Example Skills: info ====================

#[test]
fn test_info_example_name_poem() {
    skillx()
        .args(["info", &example_skill("name-poem")])
        .assert()
        .success()
        .stderr(predicates::str::contains("name-poem"));
}

// ==================== run command ====================

#[test]
fn test_run_full_lifecycle() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    // Full pipeline: fetch → inject → wait → cleanup
    skillx_with_home(&home)
        .args([
            "run",
            &example_skill("hello-world"),
            "test prompt",
            "--agent",
            "universal",
            "--scope",
            "project",
            "--skip-scan",
        ])
        .current_dir(project.path())
        .write_stdin("\n")
        .timeout(Duration::from_secs(30))
        .assert()
        .success()
        .stderr(predicates::str::contains("Cleanup complete"));

    // Injected skill directory should be cleaned up
    let skill_dir = project
        .path()
        .join(".agents")
        .join("skills")
        .join("hello-world");
    assert!(
        !skill_dir.exists(),
        "injected skill dir should be cleaned up after run"
    );

    // Session should be archived in history
    let history_dir = home.path().join("history");
    assert!(history_dir.exists(), "history dir should exist after run");
    let history_count = std::fs::read_dir(&history_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "json"))
        .count();
    assert_eq!(
        history_count, 1,
        "should have exactly one session in history"
    );
}

#[test]
fn test_run_example_with_scan_enabled() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    // Run WITH security scan (no --skip-scan)
    skillx_with_home(&home)
        .args([
            "run",
            &example_skill("hello-world"),
            "test prompt",
            "--agent",
            "universal",
            "--scope",
            "project",
        ])
        .current_dir(project.path())
        .write_stdin("\n")
        .timeout(Duration::from_secs(30))
        .assert()
        .success()
        .stderr(
            predicates::str::contains("Scanning")
                .and(predicates::str::contains("Cleanup complete")),
        );
}

#[test]
fn test_run_example_testing_guide_multi_file() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    // testing-guide has SKILL.md + references/patterns.md
    skillx_with_home(&home)
        .args([
            "run",
            &example_skill("testing-guide"),
            "test prompt",
            "--agent",
            "universal",
            "--scope",
            "project",
        ])
        .current_dir(project.path())
        .write_stdin("\n")
        .timeout(Duration::from_secs(30))
        .assert()
        .success()
        .stderr(predicates::str::contains("Cleanup complete"));
}

#[test]
fn test_run_danger_rejected_by_user() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    // dangerous-example triggers DANGER level.
    // In CI (CI=true is set), headless mode auto-refuses DANGER with exit 1.
    // Outside CI, the interactive gate would prompt for "yes" confirmation.
    let is_ci = std::env::var("CI").is_ok();

    let mut cmd = skillx_with_home(&home);
    cmd.args([
        "run",
        &example_skill("dangerous-example"),
        "test prompt",
        "--agent",
        "universal",
        "--scope",
        "project",
    ])
    .current_dir(project.path())
    .write_stdin("no\n")
    .timeout(Duration::from_secs(30));

    if is_ci {
        // Headless mode: DANGER auto-refused, exit 1
        cmd.assert()
            .failure()
            .stderr(predicates::str::contains("DANGER").and(predicates::str::contains("headless")));
    } else {
        // Interactive mode: user types "no", graceful cancellation (exit 0)
        cmd.assert().success().stderr(
            predicates::str::contains("DANGER").and(predicates::str::contains("Cancelled")),
        );
    }
}

#[test]
fn test_run_block_refused_by_gate() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    // dangerous-skill fixture has BLOCK-level findings (SC-010, SC-011).
    // Gate auto-refuses without any user input.
    skillx_with_home(&home)
        .args([
            "run",
            &fixture("dangerous-skill"),
            "test prompt",
            "--agent",
            "universal",
            "--scope",
            "project",
        ])
        .current_dir(project.path())
        .write_stdin("\n")
        .timeout(Duration::from_secs(30))
        .assert()
        .failure()
        .stderr(predicates::str::contains("BLOCK"));
}
