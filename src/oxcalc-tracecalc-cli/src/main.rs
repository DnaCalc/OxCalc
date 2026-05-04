#![forbid(unsafe_code)]

use std::env;
use std::path::Path;
use std::process::ExitCode;

use oxcalc_core::treecalc_runner::TreeCalcRunner as LocalTreeCalcRunner;
use oxcalc_core::treecalc_scale::{
    TreeCalcScaleOptions, TreeCalcScaleProfile, TreeCalcScaleRunner,
};
use oxcalc_tracecalc::independent_conformance::IndependentConformanceRunner;
use oxcalc_tracecalc::oxfml_fixture_bridge::OxFmlFixtureBridgeRunner;
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
            "usage: oxcalc-tracecalc-cli <run-id> | retained-failures <run-id> | treecalc <run-id> | oxfml-bridge <run-id> | independent-conformance <run-id> | treecalc-scale <profile> <run-id> [options]"
                .to_string(),
        );
    };
    if first_arg == "treecalc-scale" {
        let Some(profile_arg) = args.next() else {
            return Err(treecalc_scale_usage());
        };
        let Some(run_id) = args.next() else {
            return Err(treecalc_scale_usage());
        };
        let Some(profile) = TreeCalcScaleProfile::parse(&profile_arg) else {
            return Err(format!(
                "unknown treecalc-scale profile '{profile_arg}'. {}",
                treecalc_scale_usage()
            ));
        };
        let options = parse_treecalc_scale_options(profile, run_id, args)?;
        let repo_root = env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?
            .canonicalize()
            .map_err(|error| format!("failed to canonicalize repo root: {error}"))?;
        ensure_treecalc_root(&repo_root)?;
        let runner = TreeCalcScaleRunner::new();
        let summary = runner
            .execute(&repo_root, options)
            .map_err(|error| format!("treecalc scale run failed: {error}"))?;
        println!(
            "TreeCalc scale run '{}' ({}) wrote {} formulas, {} descriptors, {} edges to {}.",
            summary.run_id,
            summary.profile,
            summary.formula_count,
            summary.descriptor_count,
            summary.edge_count,
            summary.artifact_root
        );
        return Ok(());
    }

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
        "oxfml-bridge" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli oxfml-bridge <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli oxfml-bridge <run-id>".to_string());
            }
            ("oxfml-bridge", run_id)
        }
        "independent-conformance" => {
            let Some(run_id) = args.next() else {
                return Err(
                    "usage: oxcalc-tracecalc-cli independent-conformance <run-id>".to_string(),
                );
            };
            if args.next().is_some() {
                return Err(
                    "usage: oxcalc-tracecalc-cli independent-conformance <run-id>".to_string(),
                );
            }
            ("independent-conformance", run_id)
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
        "oxfml-bridge" => ensure_oxfml_bridge_root(&repo_root)?,
        "independent-conformance" => ensure_independent_conformance_root(&repo_root)?,
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
        "oxfml-bridge" => {
            let runner = OxFmlFixtureBridgeRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("OxFml fixture bridge run failed: {error}"))?;
            println!(
                "OxFml fixture bridge run '{run_id}' projected {} fixture cases across {} families to {}.",
                summary.fixture_case_count, summary.family_count, summary.artifact_root
            );
        }
        "independent-conformance" => {
            let runner = IndependentConformanceRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("independent conformance run failed: {error}"))?;
            println!(
                "Independent conformance run '{run_id}' wrote {} comparison rows to {}.",
                summary.comparison_row_count, summary.artifact_root
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

fn ensure_oxfml_bridge_root(repo_root: &Path) -> Result<(), String> {
    let fixture_path =
        repo_root.join("../OxFml/crates/oxfml_core/tests/fixtures/fec_commit_replay_cases.json");
    if fixture_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root for OxFml fixture bridge runs: missing {}",
            fixture_path.display()
        ))
    }
}

fn ensure_independent_conformance_root(repo_root: &Path) -> Result<(), String> {
    let trace_run = repo_root.join(
        "docs/test-runs/core-engine/tracecalc-reference-machine/post-w033-let-lambda-carrier-witness-001/run_summary.json",
    );
    let tree_run = repo_root.join(
        "docs/test-runs/core-engine/treecalc-local/post-w033-independent-conformance-treecalc-001/run_summary.json",
    );
    if trace_run.exists() && tree_run.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not ready for independent conformance: missing {} or {}",
            trace_run.display(),
            tree_run.display()
        ))
    }
}

fn parse_treecalc_scale_options(
    profile: TreeCalcScaleProfile,
    run_id: String,
    mut args: impl Iterator<Item = String>,
) -> Result<TreeCalcScaleOptions, String> {
    let mut options = TreeCalcScaleOptions::default_for(profile, run_id);

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rows" => options.rows = parse_usize_flag(&arg, &mut args)?,
            "--cols" => options.cols = parse_usize_flag(&arg, &mut args)?,
            "--nodes" | "--node-count" => options.node_count = parse_usize_flag(&arg, &mut args)?,
            "--fanout" => options.fanout = parse_usize_flag(&arg, &mut args)?,
            "--left-delta" => options.left_delta = parse_i64_flag(&arg, &mut args)?,
            "--top-delta" => options.top_delta = parse_i64_flag(&arg, &mut args)?,
            "--selector-period" => options.selector_period = parse_usize_flag(&arg, &mut args)?,
            "--recalc-rounds" => options.recalc_rounds = parse_usize_flag(&arg, &mut args)?,
            _ => {
                return Err(format!(
                    "unknown treecalc-scale option '{arg}'. {}",
                    treecalc_scale_usage()
                ));
            }
        }
    }

    Ok(options)
}

fn parse_usize_flag(flag: &str, args: &mut impl Iterator<Item = String>) -> Result<usize, String> {
    let Some(value) = args.next() else {
        return Err(format!(
            "missing value for {flag}. {}",
            treecalc_scale_usage()
        ));
    };
    value
        .parse::<usize>()
        .map_err(|error| format!("invalid value for {flag}: {value} ({error})"))
}

fn parse_i64_flag(flag: &str, args: &mut impl Iterator<Item = String>) -> Result<i64, String> {
    let Some(value) = args.next() else {
        return Err(format!(
            "missing value for {flag}. {}",
            treecalc_scale_usage()
        ));
    };
    value
        .parse::<i64>()
        .map_err(|error| format!("invalid value for {flag}: {value} ({error})"))
}

fn treecalc_scale_usage() -> String {
    "usage: oxcalc-tracecalc-cli treecalc-scale <grid-cross-sum|fanout-bands|dynamic-indirect-stripes|relative-rebind-churn> <run-id> [--rows N] [--cols N] [--nodes N] [--fanout N] [--left-delta N] [--top-delta N] [--selector-period N] [--recalc-rounds N]".to_string()
}
