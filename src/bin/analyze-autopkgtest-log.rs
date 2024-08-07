use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    debug: bool,

    #[clap(short, long)]
    json: bool,

    #[clap(short, long, default_value = "5")]
    context: i32,

    path: std::path::PathBuf,
}

fn main() {
    let args = Args::parse();

    // Honor debug
    env_logger::Builder::from_default_env()
        .filter_level(if args.debug {
            log::LevelFilter::Debug
        } else if args.json {
            log::LevelFilter::Warn
        } else {
            log::LevelFilter::Info
        })
        .init();

    let log = std::fs::read_to_string(&args.path).expect("Failed to read log file");

    let lines = log.split('\n').collect::<Vec<_>>();

    let (r#match, testname, error, description) =
        buildlog_consultant::autopkgtest::find_autopkgtest_failure_description(&lines);

    if args.json {
        let ret = serde_json::json!({
            "testname": testname,
            "error": error,
            "description": description
        });
        if let Some(r#match) = r#match {
            ret["offset"] = r#match.offset;
        }
        std::io::stdout()
            .write_all(serde_json::to_string_pretty(&ret).unwrap().as_bytes())
            .unwrap();
    }

    if testname {
        log::info!("Test name: {}", testname);
    }
    if let Some(error) = error {
        log::info!("Error: {}", error);
    }
    if let Some(r#match) = r#match {
        log::info!("Failed line: {}:", r#match.lineno);
        for i in max(0, r#match.offset - args.context)
            ..min(lines.len(), r#match.offset + args.context + 1)
        {
            log::info!(
                " {}  {}",
                if r#match.offset == i { ">" } else { " " },
                lines[i].trim_end_matches('\n')
            );
        }
    }
}
