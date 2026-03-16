#![forbid(unsafe_code)]

use std::env;
use std::path::Path;
use std::process::ExitCode;

use oxcalc_tracecalc::runner::TraceCalcRunner;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(run_id) = args.next() else {
        return Err("usage: oxcalc-tracecalc-cli <run-id>".to_string());
    };
    if args.next().is_some() {
        return Err("usage: oxcalc-tracecalc-cli <run-id>".to_string());
    }

    let repo_root = env::current_dir()
        .map_err(|error| format!("failed to read current directory: {error}"))?
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize repo root: {error}"))?;
    ensure_repo_root(&repo_root)?;

    let runner = TraceCalcRunner::new();
    let summary = runner
        .execute_manifest(&repo_root, &run_id, None, None)
        .map_err(|error| format!("tracecalc run failed: {error}"))?;
    println!(
        "TraceCalc Rust run '{run_id}' wrote {} scenarios to {}.",
        summary.scenario_count, summary.artifact_root
    );
    Ok(())
}

fn ensure_repo_root(repo_root: &Path) -> Result<(), String> {
    let manifest_path = repo_root.join("docs/test-corpus/core-engine/tracecalc/MANIFEST.json");
    if manifest_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root: missing {}",
            manifest_path.display()
        ))
    }
}
