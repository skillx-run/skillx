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
    let cli =
        Cli::try_parse_from(["skillx", "run", "./skill", "-f", "prompt.txt"]).unwrap();
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

    // Test: unknown command should fail
    assert!(Cli::try_parse_from(["skillx", "unknown"]).is_err());
}
