/// URL source type classification for skill source resolution.
#[derive(Debug, Clone, PartialEq)]
pub enum UrlSourceType {
    GitHub,
    GitLab,
    Bitbucket,
    Gitea,
    Gist,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookup_github() {
        assert_eq!(lookup_domain("github.com"), Some(&UrlSourceType::GitHub));
    }

    #[test]
    fn test_lookup_gist() {
        assert_eq!(
            lookup_domain("gist.github.com"),
            Some(&UrlSourceType::Gist)
        );
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
}
