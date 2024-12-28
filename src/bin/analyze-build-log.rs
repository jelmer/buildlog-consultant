use buildlog_consultant::common::find_build_failure_description;
use buildlog_consultant::{Match, Problem};
use clap::Parser;
use std::cmp::{max, min};
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
        .format_timestamp(None)
        .format_level(false)
        .format_target(false)
        .init();

    let log = if let Some(path) = args.path.as_deref() {
        std::fs::read_to_string(path).expect("Failed to read log file")
    } else {
        use std::io::Read;
        let mut log = String::new();
        std::io::stdin()
            .read_to_string(&mut log)
            .expect("Failed to read log from stdin");
        log
    };

    let lines = log.split_inclusive('\n').collect::<Vec<_>>();

    let (m, problem) = find_build_failure_description(lines.clone());

    if args.json {
        let ret = as_json(
            m.as_ref().map(|m| m.as_ref()),
            problem.as_ref().map(|p| p.as_ref()),
        );
        serde_json::to_writer_pretty(std::io::stdout(), &ret).expect("Failed to write JSON");
    } else {
        if let Some(m) = m {
            if m.linenos().len() == 1 {
                log::info!("Issue found at line {}:", m.lineno());
            } else {
                log::info!(
                    "Issue found at lines {}-{}:",
                    m.linenos().first().unwrap(),
                    m.linenos().last().unwrap()
                );
            }
            for i in max(0, m.offsets()[0] - args.context)
                ..min(lines.len(), m.offsets().last().unwrap() + args.context + 1)
            {
                log::info!(
                    " {}  {}",
                    if m.offsets().contains(&i) { ">" } else { " " },
                    lines[i].trim_end_matches('\n')
                );
            }
        } else {
            log::info!("No issues found");
        }

        if let Some(problem) = problem {
            log::info!("Identified issue: {}: {}", problem.kind(), problem);
        }
    }

    Ok(())
}
