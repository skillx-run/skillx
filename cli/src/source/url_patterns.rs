use crate::config::CustomUrlPattern;

/// URL source type classification for skill source resolution.
#[derive(Debug, Clone, PartialEq)]
pub enum UrlSourceType {
    GitHub,
    GitLab,
    Bitbucket,
    Gitea,
    Gist,
    SourceHut,
    HuggingFace,
    SkillsDirectory,
}

/// A pattern mapping a domain to a source type.
pub struct UrlPattern {
    pub domain: &'static str,
    pub source_type: UrlSourceType,
}

/// Built-in domain-to-source-type mappings.
///
/// Order matters: more specific domains should come first.
pub static URL_PATTERNS: &[UrlPattern] = &[
    // GitHub
    UrlPattern {
        domain: "github.com",
        source_type: UrlSourceType::GitHub,
    },
    // GitHub Gist
    UrlPattern {
        domain: "gist.github.com",
        source_type: UrlSourceType::Gist,
    },
    // GitLab
    UrlPattern {
        domain: "gitlab.com",
        source_type: UrlSourceType::GitLab,
    },
    // Bitbucket
    UrlPattern {
        domain: "bitbucket.org",
        source_type: UrlSourceType::Bitbucket,
    },
    // Gitea / Forgejo / Codeberg
    UrlPattern {
        domain: "codeberg.org",
        source_type: UrlSourceType::Gitea,
    },
    // SourceHut
    UrlPattern {
        domain: "git.sr.ht",
        source_type: UrlSourceType::SourceHut,
    },
    // HuggingFace
    UrlPattern {
        domain: "huggingface.co",
        source_type: UrlSourceType::HuggingFace,
    },
    // Skills directory platforms
    UrlPattern {
        domain: "skills.sh",
        source_type: UrlSourceType::SkillsDirectory,
    },
    UrlPattern {
        domain: "skillsmp.com",
        source_type: UrlSourceType::SkillsDirectory,
    },
    UrlPattern {
        domain: "clawhub.ai",
        source_type: UrlSourceType::SkillsDirectory,
    },
    UrlPattern {
        domain: "lobehub.com",
        source_type: UrlSourceType::SkillsDirectory,
    },
    UrlPattern {
        domain: "skillhub.club",
        source_type: UrlSourceType::SkillsDirectory,
    },
    UrlPattern {
        domain: "agentskillshub.dev",
        source_type: UrlSourceType::SkillsDirectory,
    },
    UrlPattern {
        domain: "agentskills.so",
        source_type: UrlSourceType::SkillsDirectory,
    },
    UrlPattern {
        domain: "mcpmarket.com",
        source_type: UrlSourceType::SkillsDirectory,
    },
    UrlPattern {
        domain: "skillsdirectory.com",
        source_type: UrlSourceType::SkillsDirectory,
    },
    UrlPattern {
        domain: "prompts.chat",
        source_type: UrlSourceType::SkillsDirectory,
    },
];

/// Look up the source type for a given domain.
///
/// Checks exact match first, then checks if the domain ends with `.{pattern_domain}`.
pub fn lookup_domain(domain: &str) -> Option<&UrlSourceType> {
    let domain_lower = domain.to_lowercase();

    // Check gist.github.com before github.com (more specific first)
    for pattern in URL_PATTERNS {
        if domain_lower == pattern.domain {
            return Some(&pattern.source_type);
        }
    }

    // Check subdomain match (e.g., gitlab.mycompany.com → not matched,
    // but we don't do subdomain matching for security reasons)
    None
}

/// Map a source type string from config to `UrlSourceType`.
fn parse_source_type_str(s: &str) -> Option<UrlSourceType> {
    match s.to_lowercase().as_str() {
        "github" => Some(UrlSourceType::GitHub),
        "gitlab" => Some(UrlSourceType::GitLab),
        "bitbucket" => Some(UrlSourceType::Bitbucket),
        "gitea" => Some(UrlSourceType::Gitea),
        "sourcehut" => Some(UrlSourceType::SourceHut),
        "huggingface" => Some(UrlSourceType::HuggingFace),
        _ => None,
    }
}

/// Look up the source type for a domain, checking custom patterns first.
///
/// Custom patterns from config.toml take priority over built-in patterns.
pub fn lookup_domain_with_custom(
    domain: &str,
    custom: &[CustomUrlPattern],
) -> Option<UrlSourceType> {
    let domain_lower = domain.to_lowercase();

    // Custom patterns take priority
    for pattern in custom {
        if domain_lower == pattern.domain.to_lowercase() {
            match parse_source_type_str(&pattern.source_type) {
                Some(st) => return Some(st),
                None => {
                    eprintln!(
                        "warning: custom URL pattern for '{}' has unknown source_type '{}' (expected: github, gitlab, bitbucket, gitea, sourcehut, huggingface)",
                        pattern.domain, pattern.source_type
                    );
                }
            }
        }
    }

    // Fall back to built-in (lookup_domain lowercases internally, passing
    // domain_lower avoids an extra allocation when the input is already lowercase)
    lookup_domain(&domain_lower).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_github() {
        assert_eq!(lookup_domain("github.com"), Some(&UrlSourceType::GitHub));
    }

    #[test]
    fn test_lookup_gist() {
        assert_eq!(lookup_domain("gist.github.com"), Some(&UrlSourceType::Gist));
    }

    #[test]
    fn test_lookup_gitlab() {
        assert_eq!(lookup_domain("gitlab.com"), Some(&UrlSourceType::GitLab));
    }

    #[test]
    fn test_lookup_bitbucket() {
        assert_eq!(
            lookup_domain("bitbucket.org"),
            Some(&UrlSourceType::Bitbucket)
        );
    }

    #[test]
    fn test_lookup_codeberg() {
        assert_eq!(lookup_domain("codeberg.org"), Some(&UrlSourceType::Gitea));
    }

    #[test]
    fn test_lookup_skills_directory_platforms() {
        let platforms = [
            "skills.sh",
            "skillsmp.com",
            "clawhub.ai",
            "lobehub.com",
            "skillhub.club",
            "agentskillshub.dev",
            "agentskills.so",
            "mcpmarket.com",
            "skillsdirectory.com",
            "prompts.chat",
        ];
        for domain in platforms {
            assert_eq!(
                lookup_domain(domain),
                Some(&UrlSourceType::SkillsDirectory),
                "expected SkillsDirectory for {domain}"
            );
        }
    }

    #[test]
    fn test_lookup_unknown() {
        assert_eq!(lookup_domain("example.com"), None);
        assert_eq!(lookup_domain("unknown.org"), None);
    }

    #[test]
    fn test_lookup_case_insensitive() {
        assert_eq!(lookup_domain("GitHub.com"), Some(&UrlSourceType::GitHub));
        assert_eq!(lookup_domain("GITLAB.COM"), Some(&UrlSourceType::GitLab));
    }

    #[test]
    fn test_lookup_sourcehut() {
        assert_eq!(lookup_domain("git.sr.ht"), Some(&UrlSourceType::SourceHut));
    }

    #[test]
    fn test_lookup_huggingface() {
        assert_eq!(
            lookup_domain("huggingface.co"),
            Some(&UrlSourceType::HuggingFace)
        );
    }

    #[test]
    fn test_lookup_domain_with_custom_overrides_builtin() {
        // Custom pattern overrides codeberg.org from Gitea to GitLab
        let custom = vec![CustomUrlPattern {
            domain: "codeberg.org".to_string(),
            source_type: "gitlab".to_string(),
        }];
        assert_eq!(
            lookup_domain_with_custom("codeberg.org", &custom),
            Some(UrlSourceType::GitLab)
        );
    }

    #[test]
    fn test_lookup_domain_with_custom_new_domain() {
        let custom = vec![CustomUrlPattern {
            domain: "mygitea.company.com".to_string(),
            source_type: "gitea".to_string(),
        }];
        assert_eq!(
            lookup_domain_with_custom("mygitea.company.com", &custom),
            Some(UrlSourceType::Gitea)
        );
    }

    #[test]
    fn test_lookup_domain_with_custom_fallback_builtin() {
        let custom = vec![CustomUrlPattern {
            domain: "other.example.com".to_string(),
            source_type: "gitea".to_string(),
        }];
        // github.com not in custom, should fallback to builtin
        assert_eq!(
            lookup_domain_with_custom("github.com", &custom),
            Some(UrlSourceType::GitHub)
        );
    }

    #[test]
    fn test_lookup_domain_with_custom_empty_list() {
        let custom: Vec<CustomUrlPattern> = vec![];
        assert_eq!(
            lookup_domain_with_custom("github.com", &custom),
            Some(UrlSourceType::GitHub)
        );
        assert_eq!(lookup_domain_with_custom("unknown.org", &custom), None);
    }
}
