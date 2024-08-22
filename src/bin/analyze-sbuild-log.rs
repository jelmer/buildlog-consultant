use buildlog_consultant::sbuild::{worker_failure_from_sbuild_log, SbuildLog};
use buildlog_consultant::{Match, Problem};
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
struct Args {
    #[clap(short, long, default_value = "5")]
    /// Number of lines of context to show.
    context: usize,
    #[clap(short, long)]
    /// Output JSON.
    json: bool,
    #[clap(short, long)]
    /// Enable debug output.
    debug: bool,

    #[clap(long)]
    /// Dump the build log to a file.
    dump: bool,

    /// The path to the build log to analyze.
    path: Option<PathBuf>,
}

fn as_json(m: Option<&dyn Match>, problem: Option<&dyn Problem>) -> serde_json::Value {
    let mut ret = serde_json::Map::new();
    if let Some(m) = m {
        ret.insert(
            "lineno".to_string(),
            serde_json::Value::Number(serde_json::Number::from(m.lineno())),
        );
        ret.insert(
            "line".to_string(),
            serde_json::Value::String(m.line().clone()),
        );
        ret.insert(
            "origin".to_string(),
            serde_json::Value::String(m.origin().to_string()),
        );
    }
    if let Some(problem) = problem {
        ret.insert(
            "problem".to_string(),
            serde_json::Value::String(problem.kind().to_string()),
        );
        ret.insert("details".to_string(), problem.json());
    }
    serde_json::Value::Object(ret)
}

pub fn main() -> Result<(), i8> {
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

    let sbuildlog: SbuildLog = if let Some(path) = args.path.as_deref() {
        std::fs::File::open(path)
            .expect("Failed to open log file")
            .try_into()
            .expect("Failed to parse log file")
    } else {
        std::io::BufReader::new(std::io::stdin().lock())
            .try_into()
            .expect("Failed to parse log file")
    };

    if args.debug {
        println!("{:?}", sbuildlog.summary());
    }

    if args.dump {
        println!("{:?}", sbuildlog);
    }

    let failed_stage = sbuildlog.get_failed_stage();

    if let Some(failed_stage) = failed_stage {
        log::info!("Failed stage: {}", failed_stage);
    } else {
        log::info!("No failed stage found");
    }

    let failure = worker_failure_from_sbuild_log(&sbuildlog);

    if let Some(error) = failure.error.as_ref() {
        log::info!("Error: {}", error);
    } else {
        log::debug!("No error found");
    }

    if args.json {
        let ret = as_json(
            failure.r#match.as_ref().map(|m| m.as_ref()),
            failure.error.as_ref().map(|p| p.as_ref()),
        );
        serde_json::to_writer_pretty(std::io::stdout(), &ret).expect("Failed to write JSON");
    }

    if let (Some(m), Some(s)) = (failure.r#match.as_ref(), failure.section.as_ref()) {
        buildlog_consultant::highlight_lines(&s.lines(), m.as_ref(), args.context);
    } else {
        assert!(failure.r#match.is_some());
        assert!(failure.section.is_some());
        log::info!("No specific issue found");
    }

    if let Some(problem) = failure.error.as_ref() {
        log::info!("Identified issue: {}: {}", problem.kind(), problem);
    }

    Ok(())
}
