use thiserror::Error;

pub type Result<T> = std::result::Result<T, SkillxError>;

#[derive(Error, Debug)]
pub enum SkillxError {
    #[error("source error: {0}")]
    Source(String),

    #[error("skill not found: {0}")]
    SkillNotFound(String),

    #[error("invalid source format: {0}")]
    InvalidSource(String),

    #[error("frontmatter parse error: {0}")]
    FrontmatterParse(String),

    #[error("scan error: {0}")]
    Scan(String),

    #[error("scan blocked: risk level BLOCK detected")]
    ScanBlocked,

    #[error("agent error: {0}")]
    Agent(String),

    #[error("no agent detected")]
    NoAgentDetected,

    #[error("session error: {0}")]
    Session(String),

    #[error("cache error: {0}")]
    Cache(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("network error: {0}")]
    Network(String),

    #[error("GitHub API error: {0}")]
    GitHubApi(String),

    #[error("GitLab API error: {0}")]
    GitLabApi(String),

    #[error("Bitbucket API error: {0}")]
    BitbucketApi(String),

    #[error("Gitea API error: {0}")]
    GiteaApi(String),

    #[error("Gist API error: {0}")]
    GistApi(String),

    #[error("archive error: {0}")]
    Archive(String),

    #[error("unsupported URL: {0}")]
    UnsupportedUrl(String),

    #[error("rate limited: {0}")]
    RateLimited(String),

    #[error("timeout after {0}")]
    Timeout(String),

    #[error("user cancelled")]
    UserCancelled,

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}
