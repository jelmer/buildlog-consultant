use clap::Parser;
use std::cmp::{max, min};

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    debug: bool,
    #[clap(short, long, default_value = "5")]
    context: usize,
    path: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();

    env_logger::Builder::from_default_env()
        .filter_level(if args.debug {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        })
        .init();

    let log = std::fs::read_to_string(&args.path).expect("Failed to read log file");

    let lines = log.split('\n').collect::<Vec<_>>();

    let (r#match, error) = buildlog_consultant::apt::find_apt_get_failure(lines);

    if let Some(error) = error.as_ref() {
        log::info!("Error: {}", error);
    }

    if let Some(r#match) = r#match {
        log::info!("Failed line: {}", r#match.lineno);

        for i in max(0, r#match.offset - args.context)..min(lines.len(), r#match.offset + args.context + 1) {
            log::info!(
                "{} {}",
                if r#match.offset == i { ">" } else { " " },
                lines[i].trim_end_matches('\n')
            );
        }
    }
}
