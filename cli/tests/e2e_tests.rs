use assert_cmd::Command;
use predicates::prelude::*;

fn skillx() -> Command {
    Command::cargo_bin("skillx").unwrap()
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
    let dir = tempfile::TempDir::new().unwrap();
    skillx()
        .arg("init")
        .current_dir(dir.path())
        .assert()
        .success();
    assert!(dir.path().join("skillx.toml").exists());
}

#[test]
fn test_init_fails_if_exists() {
    let dir = tempfile::TempDir::new().unwrap();
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
