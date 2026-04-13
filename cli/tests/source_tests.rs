use skillx::source;

// ==================== URL Resolution ====================

#[test]
fn test_resolve_github_url() {
    let result = source::resolve("https://github.com/owner/repo/tree/main/path").unwrap();
    match result {
        source::SkillSource::GitHub {
            owner,
            repo,
            path,
            ref_,
        } => {
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
            assert_eq!(path, Some("path".into()));
            assert_eq!(ref_, Some("main".into()));
        }
        _ => panic!("expected GitHub source"),
    }
}

#[test]
fn test_resolve_gist_prefix() {
    let result = source::resolve("gist:abc123").unwrap();
    match result {
        source::SkillSource::Gist { id, revision } => {
            assert_eq!(id, "abc123");
            assert!(revision.is_none());
        }
        _ => panic!("expected Gist source"),
    }
}

#[test]
fn test_resolve_gist_prefix_with_revision() {
    let result = source::resolve("gist:abc123@rev456").unwrap();
    match result {
        source::SkillSource::Gist { id, revision } => {
            assert_eq!(id, "abc123");
            assert_eq!(revision, Some("rev456".into()));
        }
        _ => panic!("expected Gist source"),
    }
}

#[test]
fn test_resolve_gitlab_url() {
    let result = source::resolve("https://gitlab.com/owner/repo/-/tree/main/skills/pdf").unwrap();
    match result {
        source::SkillSource::GitLab {
            host,
            owner,
            repo,
            path,
            ref_,
        } => {
            assert_eq!(host, "gitlab.com");
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
            assert_eq!(path, Some("skills/pdf".into()));
            assert_eq!(ref_, Some("main".into()));
        }
        _ => panic!("expected GitLab source"),
    }
}

#[test]
fn test_resolve_gitlab_nested_group_url() {
    let result =
        source::resolve("https://gitlab.com/group/subgroup/project/-/tree/main/skills/pdf")
            .unwrap();
    match result {
        source::SkillSource::GitLab {
            host,
            owner,
            repo,
            path,
            ref_,
        } => {
            assert_eq!(host, "gitlab.com");
            assert_eq!(owner, "group/subgroup");
            assert_eq!(repo, "project");
            assert_eq!(path, Some("skills/pdf".into()));
            assert_eq!(ref_, Some("main".into()));
        }
        _ => panic!("expected GitLab source"),
    }
}

#[test]
fn test_resolve_bitbucket_url() {
    let result = source::resolve("https://bitbucket.org/owner/repo/src/main/skills/pdf").unwrap();
    match result {
        source::SkillSource::Bitbucket {
            owner,
            repo,
            path,
            ref_,
        } => {
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
            assert_eq!(path, Some("skills/pdf".into()));
            assert_eq!(ref_, Some("main".into()));
        }
        _ => panic!("expected Bitbucket source"),
    }
}

#[test]
fn test_resolve_codeberg_url() {
    let result = source::resolve("https://codeberg.org/owner/repo/src/branch/main/path").unwrap();
    match result {
        source::SkillSource::Gitea {
            host,
            owner,
            repo,
            path,
            ref_,
        } => {
            assert_eq!(host, "codeberg.org");
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
            assert_eq!(path, Some("path".into()));
            assert_eq!(ref_, Some("main".into()));
        }
        _ => panic!("expected Gitea source"),
    }
}

#[test]
fn test_resolve_gist_url() {
    let result = source::resolve("https://gist.github.com/user/abc123").unwrap();
    match result {
        source::SkillSource::Gist { id, revision } => {
            assert_eq!(id, "abc123");
            assert!(revision.is_none());
        }
        _ => panic!("expected Gist source"),
    }
}

#[test]
fn test_resolve_archive_zip_url() {
    let result = source::resolve("https://example.com/skill.zip").unwrap();
    match result {
        source::SkillSource::Archive { url, format } => {
            assert_eq!(url, "https://example.com/skill.zip");
            assert!(matches!(format, source::ArchiveFormat::Zip));
        }
        _ => panic!("expected Archive source"),
    }
}

#[test]
fn test_resolve_archive_tar_gz_url() {
    let result = source::resolve("https://example.com/skill.tar.gz").unwrap();
    match result {
        source::SkillSource::Archive { url, format } => {
            assert_eq!(url, "https://example.com/skill.tar.gz");
            assert!(matches!(format, source::ArchiveFormat::TarGz));
        }
        _ => panic!("expected Archive source"),
    }
}

#[test]
fn test_resolve_skills_directory_url() {
    let result = source::resolve("https://skills.sh/some/skill").unwrap();
    assert!(matches!(
        result,
        source::SkillSource::SkillsDirectory { .. }
    ));
}

#[test]
fn test_resolve_speculative_gitea() {
    let result =
        source::resolve("https://mygitea.example.com/owner/repo/src/branch/main/path").unwrap();
    match result {
        source::SkillSource::Gitea {
            host, owner, repo, ..
        } => {
            assert_eq!(host, "mygitea.example.com");
            assert_eq!(owner, "owner");
            assert_eq!(repo, "repo");
        }
        _ => panic!("expected Gitea source"),
    }
}

#[test]
fn test_resolve_unknown_url_fallback() {
    let result = source::resolve("https://example.com/some/path").unwrap();
    assert!(matches!(result, source::SkillSource::Archive { .. }));
}

#[test]
fn test_resolve_bare_name_error() {
    let result = source::resolve("my-skill");
    assert!(result.is_err());
}

// ==================== Priority ====================

#[test]
fn test_resolve_local_path_priority() {
    // Local path (./) takes priority over everything
    let result = source::resolve("./nonexistent-dir");
    assert!(matches!(result, Ok(source::SkillSource::Local(_))));
}

#[test]
fn test_resolve_github_prefix_priority() {
    // github: prefix takes priority over URL matching
    let result = source::resolve("github:owner/repo").unwrap();
    assert!(matches!(result, source::SkillSource::GitHub { .. }));
}

// ==================== Error Display ====================

#[test]
fn test_error_display_gitlab() {
    let err = skillx::error::SkillxError::GitLabApi("test error".into());
    assert_eq!(err.to_string(), "GitLab API error: test error");
}

#[test]
fn test_error_display_bitbucket() {
    let err = skillx::error::SkillxError::BitbucketApi("test error".into());
    assert_eq!(err.to_string(), "Bitbucket API error: test error");
}

#[test]
fn test_error_display_gitea() {
    let err = skillx::error::SkillxError::GiteaApi("test error".into());
    assert_eq!(err.to_string(), "Gitea API error: test error");
}

#[test]
fn test_error_display_gist() {
    let err = skillx::error::SkillxError::GistApi("test error".into());
    assert_eq!(err.to_string(), "Gist API error: test error");
}

#[test]
fn test_error_display_archive() {
    let err = skillx::error::SkillxError::Archive("test error".into());
    assert_eq!(err.to_string(), "archive error: test error");
}

#[test]
fn test_error_display_unsupported_url() {
    let err = skillx::error::SkillxError::UnsupportedUrl("test".into());
    assert_eq!(err.to_string(), "unsupported URL: test");
}

// ==================== Agent Registry ====================

#[tokio::test]
async fn test_registry_has_32_adapters() {
    let registry = skillx::agent::registry::AgentRegistry::new_default();
    // 10 Tier1+2 + 21 Tier3 + 1 universal = 32
    assert_eq!(registry.all().len(), 32);
}

#[tokio::test]
async fn test_registry_tier2_agents_by_name() {
    let registry = skillx::agent::registry::AgentRegistry::new_default();
    assert!(registry.get("gemini-cli").is_some());
    assert!(registry.get("opencode").is_some());
    assert!(registry.get("amp").is_some());
    assert!(registry.get("windsurf").is_some());
    assert!(registry.get("cline").is_some());
    assert!(registry.get("roo").is_some());
}

#[tokio::test]
async fn test_registry_universal_is_last() {
    let registry = skillx::agent::registry::AgentRegistry::new_default();
    let all = registry.all();
    assert_eq!(all.last().unwrap().name(), "universal");
}

// ==================== Agent Adapter Properties ====================

#[test]
fn test_gemini_cli_adapter_properties() {
    let adapter = skillx::agent::gemini_cli::GeminiCliAdapter;
    use skillx::agent::AgentAdapter;
    assert_eq!(adapter.name(), "gemini-cli");
    assert_eq!(adapter.display_name(), "Gemini CLI");
    assert_eq!(
        adapter.lifecycle_mode(),
        skillx::agent::LifecycleMode::ManagedProcess
    );
    assert!(adapter.supports_auto_approve());
    assert_eq!(adapter.auto_approve_args(), vec!["--yolo"]);
}

#[test]
fn test_opencode_adapter_properties() {
    let adapter = skillx::agent::opencode::OpenCodeAdapter;
    use skillx::agent::AgentAdapter;
    assert_eq!(adapter.name(), "opencode");
    assert_eq!(adapter.display_name(), "OpenCode");
    assert_eq!(
        adapter.lifecycle_mode(),
        skillx::agent::LifecycleMode::ManagedProcess
    );
    assert!(!adapter.supports_auto_approve());
}

#[test]
fn test_amp_adapter_properties() {
    let adapter = skillx::agent::amp::AmpAdapter;
    use skillx::agent::AgentAdapter;
    assert_eq!(adapter.name(), "amp");
    assert_eq!(adapter.display_name(), "Amp");
    assert_eq!(
        adapter.lifecycle_mode(),
        skillx::agent::LifecycleMode::ManagedProcess
    );
    assert!(adapter.supports_auto_approve());
    assert_eq!(adapter.auto_approve_args(), vec!["--dangerously-allow-all"]);
}

#[test]
fn test_windsurf_adapter_properties() {
    let adapter = skillx::agent::windsurf::WindsurfAdapter;
    use skillx::agent::AgentAdapter;
    assert_eq!(adapter.name(), "windsurf");
    assert_eq!(adapter.display_name(), "Windsurf");
    assert_eq!(
        adapter.lifecycle_mode(),
        skillx::agent::LifecycleMode::FileInjectAndWait
    );
    assert!(!adapter.supports_auto_approve());
}

#[test]
fn test_cline_adapter_properties() {
    let adapter = skillx::agent::cline::ClineAdapter;
    use skillx::agent::AgentAdapter;
    assert_eq!(adapter.name(), "cline");
    assert_eq!(adapter.display_name(), "Cline");
    assert_eq!(
        adapter.lifecycle_mode(),
        skillx::agent::LifecycleMode::FileInjectAndWait
    );
    assert!(!adapter.supports_auto_approve());
}

#[test]
fn test_roo_adapter_properties() {
    let adapter = skillx::agent::roo::RooAdapter;
    use skillx::agent::AgentAdapter;
    assert_eq!(adapter.name(), "roo");
    assert_eq!(adapter.display_name(), "Roo Code");
    assert_eq!(
        adapter.lifecycle_mode(),
        skillx::agent::LifecycleMode::FileInjectAndWait
    );
    assert!(!adapter.supports_auto_approve());
}

// ==================== Agent Inject Paths ====================

#[test]
fn test_gemini_inject_path_project() {
    let adapter = skillx::agent::gemini_cli::GeminiCliAdapter;
    use skillx::agent::AgentAdapter;
    let path = adapter.inject_path("my-skill", &skillx::types::Scope::Project);
    assert_eq!(path, std::path::PathBuf::from(".gemini/skills/my-skill"));
}

#[test]
fn test_opencode_inject_path_project() {
    let adapter = skillx::agent::opencode::OpenCodeAdapter;
    use skillx::agent::AgentAdapter;
    let path = adapter.inject_path("my-skill", &skillx::types::Scope::Project);
    assert_eq!(path, std::path::PathBuf::from(".opencode/skills/my-skill"));
}

#[test]
fn test_amp_inject_path_project() {
    let adapter = skillx::agent::amp::AmpAdapter;
    use skillx::agent::AgentAdapter;
    let path = adapter.inject_path("my-skill", &skillx::types::Scope::Project);
    assert_eq!(path, std::path::PathBuf::from(".agents/skills/my-skill"));
}

#[test]
fn test_windsurf_inject_path_project() {
    let adapter = skillx::agent::windsurf::WindsurfAdapter;
    use skillx::agent::AgentAdapter;
    let path = adapter.inject_path("my-skill", &skillx::types::Scope::Project);
    assert_eq!(path, std::path::PathBuf::from(".windsurf/skills/my-skill"));
}

#[test]
fn test_cline_inject_path_project() {
    let adapter = skillx::agent::cline::ClineAdapter;
    use skillx::agent::AgentAdapter;
    let path = adapter.inject_path("my-skill", &skillx::types::Scope::Project);
    assert_eq!(path, std::path::PathBuf::from(".cline/skills/my-skill"));
}

#[test]
fn test_roo_inject_path_project() {
    let adapter = skillx::agent::roo::RooAdapter;
    use skillx::agent::AgentAdapter;
    let path = adapter.inject_path("my-skill", &skillx::types::Scope::Project);
    assert_eq!(path, std::path::PathBuf::from(".roo/skills/my-skill"));
}

// ==================== SARIF Formatter ====================

#[test]
fn test_sarif_empty_report() {
    let report = skillx::scanner::ScanReport { findings: vec![] };
    let sarif = skillx::scanner::report::SarifFormatter::format(&report);
    let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
    assert_eq!(parsed["version"], "2.1.0");
    assert_eq!(parsed["runs"][0]["results"].as_array().unwrap().len(), 0);
}

#[test]
fn test_sarif_with_findings() {
    use skillx::scanner::{Finding, RiskLevel, ScanReport};

    let report = ScanReport {
        findings: vec![
            Finding {
                rule_id: "SC-003".to_string(),
                level: RiskLevel::Danger,
                message: "suspicious eval usage".to_string(),
                file: "scripts/run.sh".to_string(),
                line: Some(10),
                context: None,
            },
            Finding {
                rule_id: "MD-001".to_string(),
                level: RiskLevel::Warn,
                message: "prompt injection detected".to_string(),
                file: "SKILL.md".to_string(),
                line: Some(5),
                context: None,
            },
        ],
    };

    let sarif = skillx::scanner::report::SarifFormatter::format(&report);
    let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();

    assert_eq!(parsed["version"], "2.1.0");

    let results = parsed["runs"][0]["results"].as_array().unwrap();
    assert_eq!(results.len(), 2);

    // Check first result
    assert_eq!(results[0]["ruleId"], "SC-003");
    assert_eq!(results[0]["level"], "error");
    assert_eq!(results[0]["message"]["text"], "suspicious eval usage");

    // Check second result
    assert_eq!(results[1]["ruleId"], "MD-001");
    assert_eq!(results[1]["level"], "warning");
}

#[test]
fn test_sarif_level_mapping() {
    use skillx::scanner::{Finding, RiskLevel, ScanReport};

    let levels = vec![
        (RiskLevel::Pass, "none"),
        (RiskLevel::Info, "note"),
        (RiskLevel::Warn, "warning"),
        (RiskLevel::Danger, "error"),
        (RiskLevel::Block, "error"),
    ];

    for (risk_level, expected_sarif) in levels {
        let report = ScanReport {
            findings: vec![Finding {
                rule_id: "TEST".to_string(),
                level: risk_level,
                message: "test".to_string(),
                file: "test.md".to_string(),
                line: None,
                context: None,
            }],
        };

        let sarif = skillx::scanner::report::SarifFormatter::format(&report);
        let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
        assert_eq!(
            parsed["runs"][0]["results"][0]["level"], expected_sarif,
            "RiskLevel::{:?} should map to '{}'",
            risk_level, expected_sarif
        );
    }
}

#[test]
fn test_sarif_schema() {
    let report = skillx::scanner::ScanReport { findings: vec![] };
    let sarif = skillx::scanner::report::SarifFormatter::format(&report);
    let parsed: serde_json::Value = serde_json::from_str(&sarif).unwrap();
    assert!(parsed["$schema"]
        .as_str()
        .unwrap()
        .contains("sarif-schema-2.1.0"));
    assert_eq!(parsed["runs"][0]["tool"]["driver"]["name"], "skillx");
}
