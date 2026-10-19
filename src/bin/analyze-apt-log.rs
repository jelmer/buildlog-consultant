use clap::Parser;

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
        .format_timestamp(None)
        .format_level(false)
        .format_target(false)
        .init();

    let log = std::fs::read_to_string(&args.path).expect("Failed to read log file");

    let lines = log.split_inclusive('\n').collect::<Vec<_>>();

    let Some(failure) = buildlog_consultant::apt::find_apt_get_failure(&lines) else {
        return;
    };

    if let Some(error) = failure.problem.as_ref() {
        log::info!("Error: {}", error);
    }

    buildlog_consultant::highlight_lines(&lines, failure.r#match.as_ref(), args.context);
}
