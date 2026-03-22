//! Integration tests for install, uninstall, list, and update commands.
//!
//! These tests use:
//! - `SKILLX_HOME` env var to isolate installed.json and cache from real `~/.skillx/`
//! - `--scope project` + temp project dir to isolate injected files from real home
//! - `--agent universal` to avoid needing real agents installed

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Create a skillx command with `SKILLX_HOME` set to a temp directory.
fn skillx_with_home(home: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("skillx").unwrap();
    cmd.env("SKILLX_HOME", home.path());
    cmd
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

/// Standard install args for a local fixture skill.
/// Uses --scope project so injected files land in the temp project dir.
fn install_skill(home: &TempDir, project: &TempDir) {
    skillx_with_home(home)
        .args([
            "install",
            &fixture("valid-skill"),
            "--skip-scan",
            "--agent",
            "universal",
            "--scope",
            "project",
            "--no-save",
        ])
        .current_dir(project.path())
        .assert()
        .success();
}

// ==================== install ====================

#[test]
fn test_install_local_skill() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    install_skill(&home, &project);

    // Verify installed.json was created
    let installed_path = home.path().join("installed.json");
    assert!(installed_path.exists(), "installed.json should be created");

    let content = fs::read_to_string(&installed_path).unwrap();
    assert!(
        content.contains("pdf-processing"),
        "installed.json should contain the skill name"
    );
}

#[test]
fn test_install_no_save_skips_toml() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    // Create a skillx.toml in the project
    fs::write(
        project.path().join("skillx.toml"),
        "[project]\nname = \"test\"\n\n[skills]\n",
    )
    .unwrap();

    // Install with --no-save (already in install_skill helper)
    install_skill(&home, &project);

    // skillx.toml should NOT have the skill added
    let toml_content = fs::read_to_string(project.path().join("skillx.toml")).unwrap();
    assert!(
        !toml_content.contains("pdf-processing"),
        "skillx.toml should not contain skill with --no-save"
    );
}

#[test]
fn test_install_saves_to_toml_by_default() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    // Create a skillx.toml
    fs::write(
        project.path().join("skillx.toml"),
        "[project]\nname = \"test\"\n\n[skills]\n",
    )
    .unwrap();

    // Install WITHOUT --no-save
    skillx_with_home(&home)
        .args([
            "install",
            &fixture("valid-skill"),
            "--skip-scan",
            "--agent",
            "universal",
            "--scope",
            "project",
        ])
        .current_dir(project.path())
        .assert()
        .success();

    // skillx.toml should have the skill
    let toml_content = fs::read_to_string(project.path().join("skillx.toml")).unwrap();
    assert!(
        toml_content.contains("pdf-processing"),
        "skillx.toml should contain the installed skill"
    );
}

#[test]
fn test_install_dev_flag() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join("skillx.toml"),
        "[project]\nname = \"test\"\n\n[skills]\n",
    )
    .unwrap();

    skillx_with_home(&home)
        .args([
            "install",
            &fixture("valid-skill"),
            "--skip-scan",
            "--agent",
            "universal",
            "--scope",
            "project",
            "--dev",
        ])
        .current_dir(project.path())
        .assert()
        .success();

    let toml_content = fs::read_to_string(project.path().join("skillx.toml")).unwrap();
    assert!(
        toml_content.contains("[skills.dev]"),
        "skillx.toml should have [skills.dev] section"
    );
}

#[test]
fn test_install_repeat_is_upgrade() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    // First install
    install_skill(&home, &project);

    // Second install — should succeed (upgrade)
    install_skill(&home, &project);
}

#[test]
fn test_install_from_toml_no_toml() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    // No skillx.toml → error
    skillx_with_home(&home)
        .arg("install")
        .current_dir(project.path())
        .assert()
        .failure()
        .stderr(predicates::str::contains("skillx.toml").or(predicates::str::contains("init")));
}

// ==================== uninstall ====================

#[test]
fn test_uninstall_nonexistent_skill() {
    let home = TempDir::new().unwrap();

    skillx_with_home(&home)
        .args(["uninstall", "nonexistent-skill"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not installed"));
}

#[test]
fn test_uninstall_installed_skill() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    install_skill(&home, &project);

    // Uninstall
    skillx_with_home(&home)
        .args(["uninstall", "pdf-processing"])
        .current_dir(project.path())
        .assert()
        .success();

    // Verify removed from installed.json
    let content = fs::read_to_string(home.path().join("installed.json")).unwrap();
    assert!(
        !content.contains("pdf-processing"),
        "skill should be removed from installed.json"
    );
}

#[test]
fn test_uninstall_keep_in_toml() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    fs::write(
        project.path().join("skillx.toml"),
        "[project]\nname = \"test\"\n\n[skills]\n",
    )
    .unwrap();

    // Install (saves to toml, no --no-save)
    skillx_with_home(&home)
        .args([
            "install",
            &fixture("valid-skill"),
            "--skip-scan",
            "--agent",
            "universal",
            "--scope",
            "project",
        ])
        .current_dir(project.path())
        .assert()
        .success();

    // Uninstall with --keep-in-toml
    skillx_with_home(&home)
        .args(["uninstall", "pdf-processing", "--keep-in-toml"])
        .current_dir(project.path())
        .assert()
        .success();

    // skillx.toml should still have the entry
    let toml_content = fs::read_to_string(project.path().join("skillx.toml")).unwrap();
    assert!(
        toml_content.contains("pdf-processing"),
        "skillx.toml should retain entry with --keep-in-toml"
    );
}

// ==================== list ====================

#[test]
fn test_list_empty() {
    let home = TempDir::new().unwrap();

    skillx_with_home(&home)
        .arg("list")
        .assert()
        .success()
        .stderr(
            predicates::str::contains("No skills")
                .or(predicates::str::contains("0"))
                .or(predicates::str::contains("empty")),
        );
}

#[test]
fn test_list_after_install() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    install_skill(&home, &project);

    // List should show the skill
    skillx_with_home(&home)
        .arg("list")
        .assert()
        .success()
        .stderr(predicates::str::contains("pdf-processing"));
}

#[test]
fn test_list_json() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    install_skill(&home, &project);

    // List as JSON
    let output = skillx_with_home(&home)
        .args(["list", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let parsed: serde_json::Value =
        serde_json::from_slice(&output).expect("--json should produce valid JSON");
    assert!(parsed.is_array() || parsed.is_object());
}

// ==================== update ====================

#[test]
fn test_update_empty() {
    let home = TempDir::new().unwrap();

    skillx_with_home(&home)
        .arg("update")
        .assert()
        .success()
        .stderr(predicates::str::contains("No skills"));
}

#[test]
fn test_update_dry_run() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    install_skill(&home, &project);

    // Get installed.json content before update
    let before = fs::read_to_string(home.path().join("installed.json")).unwrap();

    // Dry run should not change anything
    skillx_with_home(&home)
        .args(["update", "--dry-run"])
        .assert()
        .success();

    let after = fs::read_to_string(home.path().join("installed.json")).unwrap();
    assert_eq!(before, after, "dry-run should not modify installed.json");
}

#[test]
fn test_update_nonexistent_skill() {
    let home = TempDir::new().unwrap();
    let project = TempDir::new().unwrap();

    install_skill(&home, &project);

    skillx_with_home(&home)
        .args(["update", "nonexistent-skill-xyz"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("not installed"));
}
