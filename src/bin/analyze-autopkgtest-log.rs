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

    let (r#match, testname, error, description) =
        buildlog_consultant::autopkgtest::find_autopkgtest_failure_description(lines.clone());

    if args.json {
        let mut ret = serde_json::json!({
            "testname": testname,
            "error": error,
            "description": description
        });
        if let Some(ref r#match) = r#match {
            ret["offset"] = serde_json::value::Value::Number(r#match.offset().into());
        }
        std::io::stdout()
            .write_all(serde_json::to_string_pretty(&ret).unwrap().as_bytes())
            .unwrap();
    }

    if let Some(testname) = testname {
        log::info!("Test name: {}", testname);
    }
    if let Some(error) = error {
        log::info!("Error: {}", error);
    }
    if let Some(r#match) = r#match {
        buildlog_consultant::highlight_lines(&lines, r#match.as_ref(), args.context);
    }
}
