use crate::error::{Result, SkillxError};
use crate::source::{SkillSource, SkillsDirectoryPlatform};

/// Resolve a skills directory platform URL to its underlying source.
///
/// This function makes HTTP requests to the platform to extract the
/// actual source repository (typically a GitHub URL).
///
/// Strategy (by priority):
/// 1. API endpoint (if available, e.g., skillsmp.com /api/v1/skills/)
/// 2. HTML parsing with scraper — extract GitHub links
/// 3. Meta tag / Open Graph extraction
pub async fn resolve_skills_directory(
    platform: &SkillsDirectoryPlatform,
    path: &str,
) -> Result<SkillSource> {
    let domain = platform_domain(platform);
    let url = format!("https://{domain}{path}");

    let client = reqwest::Client::builder()
        .user_agent("skillx/0.2")
        .build()
        .map_err(|e| SkillxError::Network(format!("failed to create HTTP client: {e}")))?;

    // Strategy 1: Try API endpoint for platforms that have one
    if let Some(source) = try_api_endpoint(&client, platform, path).await? {
        return Ok(source);
    }

    // Strategy 2+3: Fetch HTML page and extract GitHub link
    let resp = client.get(&url).send().await.map_err(|e| {
        SkillxError::Network(format!("failed to fetch {url}: {e}"))
    })?;

    if !resp.status().is_success() {
        return Err(SkillxError::Source(format!(
            "skills directory returned HTTP {} for {url}",
            resp.status()
        )));
    }

    let html = resp.text().await.map_err(|e| {
        SkillxError::Network(format!("failed to read response from {url}: {e}"))
    })?;

    // Try to extract GitHub URL from HTML
    if let Some(github_url) = extract_github_url_from_html(&html) {
        return crate::source::url::resolve_url(&github_url);
    }

    Err(SkillxError::Source(format!(
        "could not extract source repository from {url}. The platform may not link to a GitHub source."
    )))
}

/// Get the domain for a skills directory platform.
fn platform_domain(platform: &SkillsDirectoryPlatform) -> &'static str {
    match platform {
        SkillsDirectoryPlatform::SkillsSh => "skills.sh",
        SkillsDirectoryPlatform::SkillsMp => "skillsmp.com",
        SkillsDirectoryPlatform::ClawHub => "clawhub.ai",
        SkillsDirectoryPlatform::LobeHub => "lobehub.com",
        SkillsDirectoryPlatform::SkillHub => "skillhub.club",
        SkillsDirectoryPlatform::AgentSkillsHub => "agentskillshub.dev",
        SkillsDirectoryPlatform::AgentSkillsSo => "agentskills.so",
        SkillsDirectoryPlatform::McpMarket => "mcpmarket.com",
        SkillsDirectoryPlatform::SkillsDirectory => "skillsdirectory.com",
        SkillsDirectoryPlatform::PromptsChat => "prompts.chat",
    }
}

/// Try API-based resolution for platforms with known APIs.
async fn try_api_endpoint(
    client: &reqwest::Client,
    platform: &SkillsDirectoryPlatform,
    path: &str,
) -> Result<Option<SkillSource>> {
    match platform {
        SkillsDirectoryPlatform::SkillsMp => {
            // skillsmp.com has /api/v1/skills/ endpoint
            let slug = path.trim_start_matches('/').trim_end_matches('/');
            let api_url = format!("https://skillsmp.com/api/v1/skills/{slug}");
            let resp = client.get(&api_url).send().await;
            if let Ok(resp) = resp {
                if resp.status().is_success() {
                    if let Ok(body) = resp.json::<serde_json::Value>().await {
                        if let Some(github_url) = body["github_url"]
                            .as_str()
                            .or_else(|| body["source_url"].as_str())
                            .or_else(|| body["repository_url"].as_str())
                        {
                            return Ok(Some(
                                crate::source::url::resolve_url(github_url)?,
                            ));
                        }
                    }
                }
            }
            Ok(None)
        }
        _ => Ok(None),
    }
}

/// Extract a GitHub URL from an HTML page using the scraper crate.
fn extract_github_url_from_html(html: &str) -> Option<String> {
    let document = scraper::Html::parse_document(html);

    // Strategy 1: Look for <a href="https://github.com/..."> links
    let a_selector = scraper::Selector::parse("a[href]").ok()?;
    for element in document.select(&a_selector) {
        if let Some(href) = element.value().attr("href") {
            if is_github_repo_url(href) {
                return Some(href.to_string());
            }
        }
    }

    // Strategy 2: Look for meta tags with GitHub URLs
    let meta_selector =
        scraper::Selector::parse(r#"meta[property="og:url"], meta[name="source"], meta[property="og:see_also"]"#)
            .ok()?;
    for element in document.select(&meta_selector) {
        if let Some(content) = element.value().attr("content") {
            if is_github_repo_url(content) {
                return Some(content.to_string());
            }
        }
    }

    None
}

/// Check if a URL looks like a GitHub repository URL.
fn is_github_repo_url(url: &str) -> bool {
    if !url.starts_with("https://github.com/") && !url.starts_with("http://github.com/") {
        return false;
    }
    // Must have at least owner/repo
    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .unwrap_or("");
    let parts: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    parts.len() >= 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_github_repo_url() {
        assert!(is_github_repo_url("https://github.com/owner/repo"));
        assert!(is_github_repo_url(
            "https://github.com/owner/repo/tree/main/path"
        ));
        assert!(!is_github_repo_url("https://github.com/"));
        assert!(!is_github_repo_url("https://gitlab.com/owner/repo"));
        assert!(!is_github_repo_url("not-a-url"));
    }

    #[test]
    fn test_extract_github_url_from_html() {
        let html = r#"
            <html>
            <body>
                <a href="https://github.com/owner/repo/tree/main/skills/pdf">View on GitHub</a>
            </body>
            </html>
        "#;
        let result = extract_github_url_from_html(html);
        assert_eq!(
            result,
            Some("https://github.com/owner/repo/tree/main/skills/pdf".into())
        );
    }

    #[test]
    fn test_extract_github_url_from_meta() {
        let html = r#"
            <html>
            <head>
                <meta property="og:see_also" content="https://github.com/owner/repo" />
            </head>
            <body></body>
            </html>
        "#;
        let result = extract_github_url_from_html(html);
        assert_eq!(
            result,
            Some("https://github.com/owner/repo".into())
        );
    }

    #[test]
    fn test_extract_no_github_url() {
        let html = r#"
            <html>
            <body>
                <a href="https://example.com">Not GitHub</a>
            </body>
            </html>
        "#;
        assert!(extract_github_url_from_html(html).is_none());
    }

    #[test]
    fn test_platform_domain() {
        assert_eq!(
            platform_domain(&SkillsDirectoryPlatform::SkillsSh),
            "skills.sh"
        );
        assert_eq!(
            platform_domain(&SkillsDirectoryPlatform::SkillsMp),
            "skillsmp.com"
        );
    }
}
