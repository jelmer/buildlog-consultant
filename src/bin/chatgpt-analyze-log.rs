use clap::Parser;
use std::io::BufRead;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    debug: bool,
    path: std::path::PathBuf,
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

    let openai_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_KEY not set");

    let runtime = tokio::runtime::Runtime::new().unwrap();

    let m = runtime.block_on(buildlog_consultant::chatgpt::analyze(
        openai_key,
        lines.iter().map(|l| l.as_str()).collect::<Vec<_>>(),
    ));

    if let Some(m) = m {
        log::info!("match: {}", m);
    }
}
