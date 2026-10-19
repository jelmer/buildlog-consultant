use clap::Parser;
use std::io::Write;

#[derive(Parser)]
struct Args {
    #[clap(short, long)]
    debug: bool,

    #[clap(short, long)]
    json: bool,

    #[clap(short, long, default_value = "5")]
    context: usize,

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
        .format_timestamp(None)
        .format_level(false)
        .format_target(false)
        .init();

    let log = std::fs::read_to_string(&args.path).expect("Failed to read log file");

    let lines = log.split('\n').collect::<Vec<_>>();

    let result =
        buildlog_consultant::autopkgtest::find_autopkgtest_failure_description(lines.clone());

    if args.json {
        let mut ret = serde_json::json!({
            "testname": result.as_ref().and_then(|r| r.testname.as_ref()),
            "error": result.as_ref().and_then(|r| r.failure.problem.as_ref()),
            "description": result.as_ref().and_then(|r| r.description.as_ref()),
        });
        if let Some(r) = result.as_ref() {
            ret["offset"] = serde_json::value::Value::Number(r.failure.r#match.offset().into());
        }
        std::io::stdout()
            .write_all(serde_json::to_string_pretty(&ret).unwrap().as_bytes())
            .unwrap();
    }

    let Some(result) = result else {
        return;
    };
    if let Some(testname) = result.testname {
        log::info!("Test name: {}", testname);
    }
    if let Some(error) = result.failure.problem {
        log::info!("Error: {}", error);
    }
    buildlog_consultant::highlight_lines(&lines, result.failure.r#match.as_ref(), args.context);
}
