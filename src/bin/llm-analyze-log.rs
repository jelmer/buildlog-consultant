use clap::Parser;
use std::io::BufRead;

#[derive(Clone, Debug, clap::ValueEnum)]
enum Backend {
    #[cfg(feature = "chatgpt")]
    Chatgpt,
    #[cfg(feature = "claude")]
    Claude,
}

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    debug: bool,

    #[clap(short, long)]
    backend: Option<Backend>,

    path: std::path::PathBuf,
}

fn detect_backend() -> Backend {
    #[cfg(feature = "claude")]
    if std::env::var("ANTHROPIC_API_KEY").is_ok() {
        return Backend::Claude;
    }
    #[cfg(feature = "chatgpt")]
    if std::env::var("OPENAI_API_KEY").is_ok() {
        return Backend::Chatgpt;
    }
    eprintln!("No API key found. Set ANTHROPIC_API_KEY or OPENAI_API_KEY.");
    std::process::exit(1);
}

fn main() {
    let args: Args = Args::parse();

    env_logger::builder()
        .filter_level(if args.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    let f = std::fs::File::open(&args.path).expect("Failed to open file");

    let reader = std::io::BufReader::new(f);

    let lines = reader
        .lines()
        .map(|l| l.expect("Failed to read line"))
        .collect::<Vec<_>>();

    let line_refs: Vec<&str> = lines.iter().map(|l| l.as_str()).collect();

    let backend = args.backend.unwrap_or_else(detect_backend);

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let result = match backend {
        #[cfg(feature = "chatgpt")]
        Backend::Chatgpt => {
            let key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
            runtime.block_on(buildlog_consultant::chatgpt::analyze(key, line_refs))
        }
        #[cfg(feature = "claude")]
        Backend::Claude => {
            let key = std::env::var("ANTHROPIC_API_KEY").expect("ANTHROPIC_API_KEY not set");
            runtime.block_on(buildlog_consultant::claude::analyze(key, line_refs))
        }
    };

    match result {
        Ok(Some(analysis)) => {
            log::info!("match: {}", analysis.r#match);
            if let Some(problem) = &analysis.problem {
                log::info!("problem: {}", problem);
            }
        }
        Ok(None) => log::info!("No match found"),
        Err(e) => {
            log::error!("Failed to analyze log: {}", e);
            std::process::exit(1);
        }
    }
}
