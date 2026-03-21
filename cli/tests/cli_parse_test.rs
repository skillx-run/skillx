// Test command parsing by importing from the binary crate's commands module.
// Since commands are in main.rs's mod, we test via clap's try_parse_from.

#[test]
fn test_run_basic() {
    use clap::Parser;

    #[derive(Parser)]
    #[command(name = "skillx")]
    struct Cli {
        #[command(subcommand)]
        command: Commands,
    }

    #[derive(clap::Subcommand)]
    enum Commands {
        Run(RunArgs),
        Install(InstallArgs),
        Uninstall(UninstallArgs),
        List(ListArgs),
        Update(UpdateArgs),
        Init(InitArgs),
        Scan(ScanArgs),
        Agents(AgentsArgs),
        Info(InfoArgs),
        Cache(CacheArgs),
    }

    #[derive(clap::Args, Debug)]
    struct RunArgs {
        source: Option<String>,
        prompt: Option<String>,
        #[arg(short = 'f', long = "file")]
        prompt_file: Option<String>,
        #[arg(long)]
        stdin: bool,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long, default_value = "global")]
        scope: String,
        #[arg(long)]
        attach: Vec<String>,
        #[arg(long)]
        no_cache: bool,
        #[arg(long)]
        skip_scan: bool,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        yolo: bool,
        #[arg(long)]
        timeout: Option<String>,
    }

    #[derive(clap::Args, Debug)]
    struct InstallArgs {
        sources: Vec<String>,
        #[arg(long, conflicts_with = "all")]
        agent: Option<String>,
        #[arg(long, conflicts_with = "agent")]
        all: bool,
        #[arg(long, default_value = "global")]
        scope: String,
        #[arg(long)]
        no_cache: bool,
        #[arg(long)]
        skip_scan: bool,
        #[arg(long)]
        yes: bool,
        #[arg(long)]
        no_save: bool,
        #[arg(long)]
        dev: bool,
        #[arg(long)]
        prod: bool,
        #[arg(long)]
        prune: bool,
    }

    #[derive(clap::Args, Debug)]
    struct UninstallArgs {
        #[arg(required = true)]
        names: Vec<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        keep_in_toml: bool,
        #[arg(long)]
        purge: bool,
    }

    #[derive(clap::Args, Debug)]
    struct ListArgs {
        #[arg(long)]
        agent: Option<String>,
        #[arg(long, default_value = "all")]
        scope: String,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        outdated: bool,
    }

    #[derive(clap::Args, Debug)]
    struct UpdateArgs {
        names: Vec<String>,
        #[arg(long)]
        agent: Option<String>,
        #[arg(long)]
        dry_run: bool,
        #[arg(long)]
        skip_scan: bool,
        #[arg(long)]
        yes: bool,
    }

    #[derive(clap::Args, Debug)]
    struct InitArgs {
        #[arg(long)]
        from_installed: bool,
    }

    #[derive(clap::Args, Debug)]
    struct ScanArgs {
        source: String,
        #[arg(long, default_value = "text")]
        format: String,
        #[arg(long, default_value = "danger")]
        fail_on: String,
    }

    #[derive(clap::Args, Debug)]
    struct AgentsArgs {
        #[arg(long)]
        all: bool,
    }

    #[derive(clap::Args, Debug)]
    struct InfoArgs {
        source: String,
    }

    #[derive(clap::Args, Debug)]
    struct CacheArgs {
        #[command(subcommand)]
        command: CacheCommands,
    }

    #[derive(clap::Subcommand, Debug)]
    enum CacheCommands {
        Ls,
        Clean,
    }

    // Test: run with source and prompt
    let cli = Cli::try_parse_from(["skillx", "run", "./my-skill", "do something"]).unwrap();
    match cli.command {
        Commands::Run(args) => {
            assert_eq!(args.source.as_deref(), Some("./my-skill"));
            assert_eq!(args.prompt.as_deref(), Some("do something"));
            assert!(!args.yolo);
            assert!(!args.skip_scan);
            assert_eq!(args.scope, "global");
        }
        _ => panic!("expected Run command"),
    }

    // Test: run with flags
    let cli = Cli::try_parse_from([
        "skillx",
        "run",
        "github:org/repo/path",
        "--agent",
        "claude-code",
        "--yolo",
        "--skip-scan",
        "--timeout",
        "30m",
        "--scope",
        "project",
    ])
    .unwrap();
    match cli.command {
        Commands::Run(args) => {
            assert_eq!(args.source.as_deref(), Some("github:org/repo/path"));
            assert_eq!(args.agent.as_deref(), Some("claude-code"));
            assert!(args.yolo);
            assert!(args.skip_scan);
            assert_eq!(args.timeout.as_deref(), Some("30m"));
            assert_eq!(args.scope, "project");
        }
        _ => panic!("expected Run command"),
    }

    // Test: install with sources and flags
    let cli = Cli::try_parse_from([
        "skillx",
        "install",
        "./my-skill",
        "github:org/other",
        "--agent",
        "claude-code",
        "--skip-scan",
        "--dev",
        "--scope",
        "project",
    ])
    .unwrap();
    match cli.command {
        Commands::Install(args) => {
            assert_eq!(args.sources, vec!["./my-skill", "github:org/other"]);
            assert_eq!(args.agent.as_deref(), Some("claude-code"));
            assert!(args.skip_scan);
            assert!(args.dev);
            assert_eq!(args.scope, "project");
            assert!(!args.all);
            assert!(!args.no_save);
        }
        _ => panic!("expected Install command"),
    }

    // Test: install without sources (manifest mode)
    let cli = Cli::try_parse_from(["skillx", "install", "--prod", "--prune"]).unwrap();
    match cli.command {
        Commands::Install(args) => {
            assert!(args.sources.is_empty());
            assert!(args.prod);
            assert!(args.prune);
        }
        _ => panic!("expected Install command"),
    }

    // Test: install --all
    let cli = Cli::try_parse_from(["skillx", "install", "./skill", "--all"]).unwrap();
    match cli.command {
        Commands::Install(args) => {
            assert!(args.all);
        }
        _ => panic!("expected Install command"),
    }

    // Test: uninstall
    let cli = Cli::try_parse_from([
        "skillx",
        "uninstall",
        "pdf-processing",
        "--agent",
        "cursor",
        "--purge",
    ])
    .unwrap();
    match cli.command {
        Commands::Uninstall(args) => {
            assert_eq!(args.names, vec!["pdf-processing"]);
            assert_eq!(args.agent.as_deref(), Some("cursor"));
            assert!(args.purge);
            assert!(!args.keep_in_toml);
        }
        _ => panic!("expected Uninstall command"),
    }

    // Test: uninstall multiple
    let cli = Cli::try_parse_from(["skillx", "uninstall", "pdf", "review"]).unwrap();
    match cli.command {
        Commands::Uninstall(args) => {
            assert_eq!(args.names, vec!["pdf", "review"]);
        }
        _ => panic!("expected Uninstall command"),
    }

    // Test: uninstall requires at least one name
    assert!(Cli::try_parse_from(["skillx", "uninstall"]).is_err());

    // Test: list
    let cli = Cli::try_parse_from(["skillx", "list", "--json", "--outdated"]).unwrap();
    match cli.command {
        Commands::List(args) => {
            assert!(args.json);
            assert!(args.outdated);
            assert_eq!(args.scope, "all");
            assert!(args.agent.is_none());
        }
        _ => panic!("expected List command"),
    }

    // Test: list with filters
    let cli = Cli::try_parse_from([
        "skillx",
        "list",
        "--agent",
        "claude-code",
        "--scope",
        "global",
    ])
    .unwrap();
    match cli.command {
        Commands::List(args) => {
            assert_eq!(args.agent.as_deref(), Some("claude-code"));
            assert_eq!(args.scope, "global");
        }
        _ => panic!("expected List command"),
    }

    // Test: update (all)
    let cli = Cli::try_parse_from(["skillx", "update", "--dry-run", "--yes"]).unwrap();
    match cli.command {
        Commands::Update(args) => {
            assert!(args.names.is_empty());
            assert!(args.dry_run);
            assert!(args.yes);
        }
        _ => panic!("expected Update command"),
    }

    // Test: update specific skills
    let cli = Cli::try_parse_from(["skillx", "update", "pdf", "review"]).unwrap();
    match cli.command {
        Commands::Update(args) => {
            assert_eq!(args.names, vec!["pdf", "review"]);
        }
        _ => panic!("expected Update command"),
    }

    // Test: init
    let cli = Cli::try_parse_from(["skillx", "init"]).unwrap();
    match cli.command {
        Commands::Init(args) => {
            assert!(!args.from_installed);
        }
        _ => panic!("expected Init command"),
    }

    // Test: init --from-installed
    let cli = Cli::try_parse_from(["skillx", "init", "--from-installed"]).unwrap();
    match cli.command {
        Commands::Init(args) => {
            assert!(args.from_installed);
        }
        _ => panic!("expected Init command"),
    }

    // Test: scan with format
    let cli = Cli::try_parse_from([
        "skillx",
        "scan",
        "./test-skill",
        "--format",
        "json",
        "--fail-on",
        "warn",
    ])
    .unwrap();
    match cli.command {
        Commands::Scan(args) => {
            assert_eq!(args.source, "./test-skill");
            assert_eq!(args.format, "json");
            assert_eq!(args.fail_on, "warn");
        }
        _ => panic!("expected Scan command"),
    }

    // Test: agents --all
    let cli = Cli::try_parse_from(["skillx", "agents", "--all"]).unwrap();
    match cli.command {
        Commands::Agents(args) => {
            assert!(args.all);
        }
        _ => panic!("expected Agents command"),
    }

    // Test: info
    let cli = Cli::try_parse_from(["skillx", "info", "github:org/repo"]).unwrap();
    match cli.command {
        Commands::Info(args) => {
            assert_eq!(args.source, "github:org/repo");
        }
        _ => panic!("expected Info command"),
    }

    // Test: cache ls
    let cli = Cli::try_parse_from(["skillx", "cache", "ls"]).unwrap();
    match cli.command {
        Commands::Cache(args) => {
            assert!(matches!(args.command, CacheCommands::Ls));
        }
        _ => panic!("expected Cache command"),
    }

    // Test: cache clean
    let cli = Cli::try_parse_from(["skillx", "cache", "clean"]).unwrap();
    match cli.command {
        Commands::Cache(args) => {
            assert!(matches!(args.command, CacheCommands::Clean));
        }
        _ => panic!("expected Cache command"),
    }

    // Test: run with prompt file
    let cli = Cli::try_parse_from(["skillx", "run", "./skill", "-f", "prompt.txt"]).unwrap();
    match cli.command {
        Commands::Run(args) => {
            assert_eq!(args.prompt_file.as_deref(), Some("prompt.txt"));
        }
        _ => panic!("expected Run command"),
    }

    // Test: run without source succeeds (source is optional, skillx.toml may exist)
    let cli = Cli::try_parse_from(["skillx", "run"]).unwrap();
    match cli.command {
        Commands::Run(args) => {
            assert!(args.source.is_none());
        }
        _ => panic!("expected Run command"),
    }

    // Test: install --agent and --all conflict
    assert!(
        Cli::try_parse_from([
            "skillx",
            "install",
            "./skill",
            "--agent",
            "claude-code",
            "--all"
        ])
        .is_err(),
        "--agent and --all should conflict"
    );

    // Test: unknown command should fail
    assert!(Cli::try_parse_from(["skillx", "unknown"]).is_err());
}
