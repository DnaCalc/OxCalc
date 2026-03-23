#![forbid(unsafe_code)]

use std::env;
use std::path::Path;
use std::process::ExitCode;

use oxcalc_core::treecalc_runner::TreeCalcRunner as LocalTreeCalcRunner;
use oxcalc_tracecalc::retained_failures::TraceCalcRetainedFailureRunner;
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
    let Some(first_arg) = args.next() else {
        return Err(
            "usage: oxcalc-tracecalc-cli <run-id> | retained-failures <run-id> | treecalc <run-id>"
                .to_string(),
        );
    };
    let (mode, run_id) = match first_arg.as_str() {
        "treecalc" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli treecalc <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli treecalc <run-id>".to_string());
            }
            ("treecalc", run_id)
        }
        "retained-failures" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli retained-failures <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli retained-failures <run-id>".to_string());
            }
            ("retained-failures", run_id)
        }
        run_id => {
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli <run-id>".to_string());
            }
            ("tracecalc", run_id.to_string())
        }
    };

    let repo_root = env::current_dir()
        .map_err(|error| format!("failed to read current directory: {error}"))?
        .canonicalize()
        .map_err(|error| format!("failed to canonicalize repo root: {error}"))?;

    match mode {
        "retained-failures" => ensure_retained_failure_root(&repo_root)?,
        "treecalc" => ensure_treecalc_root(&repo_root)?,
        _ => ensure_repo_root(&repo_root)?,
    }

    match mode {
        "treecalc" => {
            let runner = LocalTreeCalcRunner::new();
            let summary = runner
                .execute_manifest(&repo_root, &run_id)
                .map_err(|error| format!("treecalc run failed: {error}"))?;
            println!(
                "TreeCalc local run '{run_id}' wrote {} cases to {}.",
                summary.case_count, summary.artifact_root
            );
        }
        "retained-failures" => {
            let runner = TraceCalcRetainedFailureRunner::new();
            let summary = runner
                .execute_manifest(&repo_root, &run_id)
                .map_err(|error| format!("retained-failure run failed: {error}"))?;
            println!(
                "TraceCalc retained-failure run '{run_id}' wrote {} cases to {}.",
                summary.case_count, summary.artifact_root
            );
        }
        _ => {
            let runner = TraceCalcRunner::new();
            let summary = runner
                .execute_manifest(&repo_root, &run_id, None, None)
                .map_err(|error| format!("tracecalc run failed: {error}"))?;
            println!(
                "TraceCalc Rust run '{run_id}' wrote {} scenarios to {}.",
                summary.scenario_count, summary.artifact_root
            );
        }
    }

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

fn ensure_retained_failure_root(repo_root: &Path) -> Result<(), String> {
    let manifest_path =
        repo_root.join("docs/test-fixtures/core-engine/tracecalc-retained-failures/MANIFEST.json");
    if manifest_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root for retained-failure runs: missing {}",
            manifest_path.display()
        ))
    }
}

fn ensure_treecalc_root(repo_root: &Path) -> Result<(), String> {
    let manifest_path = repo_root.join("docs/test-fixtures/core-engine/treecalc/MANIFEST.json");
    if manifest_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root for treecalc runs: missing {}",
            manifest_path.display()
        ))
    }
}
