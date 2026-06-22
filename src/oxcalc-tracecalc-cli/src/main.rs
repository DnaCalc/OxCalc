#![forbid(unsafe_code)]

use std::env;
use std::path::Path;
use std::process::ExitCode;

use oxcalc_core::grid_reference_machine::GridEngineMode;
use oxcalc_core::grid_runner::GridCorpusRunner;
use oxcalc_core::grid_scale::{GridScaleOptions, GridScaleProfile, GridScaleRunner};
use oxcalc_core::treecalc_runner::TreeCalcRunner as LocalTreeCalcRunner;
use oxcalc_core::treecalc_scale::{
    TreeCalcScaleOptions, TreeCalcScaleProfile, TreeCalcScaleRunner,
};
use oxcalc_core::upstream_host_runner::UpstreamHostRunner;
use oxcalc_tracecalc::independent_conformance::IndependentConformanceRunner;
use oxcalc_tracecalc::oracle_matrix::TraceCalcOracleMatrixRunner;
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
            "usage: oxcalc-tracecalc-cli <run-id> | tracecalc-oracle-matrix <run-id> | retained-failures <run-id> | treecalc <run-id> | oxfml-bridge <run-id> | upstream-host <run-id> | independent-conformance <run-id> | grid-seed <run-id> [--engine reference|optimized|both] | grid-scale <profile> <run-id> [--rows N] [--cols N] | treecalc-scale <profile> <run-id> [options]"
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
    if first_arg == "grid-seed" {
        let Some(run_id) = args.next() else {
            return Err(grid_seed_usage());
        };
        let engine = parse_grid_seed_engine(args)?;
        let repo_root = env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?
            .canonicalize()
            .map_err(|error| format!("failed to canonicalize repo root: {error}"))?;
        ensure_grid_seed_root(&repo_root)?;
        let runner = GridCorpusRunner::new();
        let summary = runner
            .execute_seed_corpus(&repo_root, &run_id, &engine)
            .map_err(|error| format!("grid seed run failed: {error}"))?;
        println!(
            "Grid seed run '{run_id}' ({}) wrote {} cases, {} expectation mismatches, {} differential mismatches, {} invalidation mismatches, and {} P-20 mismatches to docs/test-runs/core-engine/grid-seed/{run_id}.",
            summary.engine_mode.engine_arg(),
            summary.case_count,
            summary.expectation_mismatch_count,
            summary.differential_mismatch_count,
            summary.invalidation_mismatch_count,
            summary.p20_mismatch_count
        );
        return Ok(());
    }
    if first_arg == "grid-scale" {
        let Some(profile_arg) = args.next() else {
            return Err(grid_scale_usage());
        };
        let Some(run_id) = args.next() else {
            return Err(grid_scale_usage());
        };
        let Some(profile) = GridScaleProfile::parse(&profile_arg) else {
            return Err(format!(
                "unknown grid-scale profile '{profile_arg}'. {}",
                grid_scale_usage()
            ));
        };
        let options = parse_grid_scale_options(profile, run_id, args)?;
        let repo_root = env::current_dir()
            .map_err(|error| format!("failed to read current directory: {error}"))?
            .canonicalize()
            .map_err(|error| format!("failed to canonicalize repo root: {error}"))?;
        ensure_grid_scale_root(&repo_root)?;
        let runner = GridScaleRunner::new();
        let summary = runner
            .execute(&repo_root, options)
            .map_err(|error| format!("grid scale run failed: {error}"))?;
        println!(
            "Grid scale run '{}' ({}) wrote {} register assertions ({} failed) to {}.",
            summary.run_id,
            summary.profile,
            summary.register_assertion_count,
            summary.failed_register_assertion_count,
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
        "tracecalc-oracle-matrix" => {
            let Some(run_id) = args.next() else {
                return Err(
                    "usage: oxcalc-tracecalc-cli tracecalc-oracle-matrix <run-id>".to_string(),
                );
            };
            if args.next().is_some() {
                return Err(
                    "usage: oxcalc-tracecalc-cli tracecalc-oracle-matrix <run-id>".to_string(),
                );
            }
            ("tracecalc-oracle-matrix", run_id)
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
        "upstream-host" => {
            let Some(run_id) = args.next() else {
                return Err("usage: oxcalc-tracecalc-cli upstream-host <run-id>".to_string());
            };
            if args.next().is_some() {
                return Err("usage: oxcalc-tracecalc-cli upstream-host <run-id>".to_string());
            }
            ("upstream-host", run_id)
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
        "tracecalc-oracle-matrix" => ensure_repo_root(&repo_root)?,
        "treecalc" => ensure_treecalc_root(&repo_root)?,
        "oxfml-bridge" => ensure_oxfml_bridge_root(&repo_root)?,
        "upstream-host" => ensure_upstream_host_root(&repo_root)?,
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
        "tracecalc-oracle-matrix" => {
            let runner = TraceCalcOracleMatrixRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("TraceCalc oracle matrix run failed: {error}"))?;
            println!(
                "TraceCalc oracle matrix run '{run_id}' wrote {} matrix rows ({} covered, {} classified uncovered, {} excluded, {} failed/missing) to {}.",
                summary.matrix_row_count,
                summary.covered_row_count,
                summary.uncovered_row_count,
                summary.excluded_row_count,
                summary.missing_or_failed_row_count,
                summary.artifact_root
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
        "upstream-host" => {
            let runner = UpstreamHostRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("upstream-host run failed: {error}"))?;
            println!(
                "Upstream-host direct OxFml run '{run_id}' wrote {} cases ({} direct OxFml, {} LET/LAMBDA, {} W073 formatting guard, {} mismatches) to {}.",
                summary.fixture_case_count,
                summary.direct_oxfml_case_count,
                summary.let_lambda_case_count,
                summary.w073_typed_rule_case_count,
                summary.expectation_mismatch_count,
                summary.artifact_root
            );
        }
        "independent-conformance" => {
            let runner = IndependentConformanceRunner::new();
            let summary = runner
                .execute(&repo_root, &run_id)
                .map_err(|error| format!("independent conformance run failed: {error}"))?;
            if summary.w036_differential_row_count > 0 {
                println!(
                    "Independent conformance run '{run_id}' wrote {} comparison rows, {} W036 diversity rows, {} W036 differential rows, and {} promotion blockers to {}.",
                    summary.comparison_row_count,
                    summary.w036_diversity_row_count,
                    summary.w036_differential_row_count,
                    summary.w036_promotion_blocker_count,
                    summary.artifact_root
                );
            } else {
                println!(
                    "Independent conformance run '{run_id}' wrote {} comparison rows to {}.",
                    summary.comparison_row_count, summary.artifact_root
                );
            }
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

fn ensure_upstream_host_root(repo_root: &Path) -> Result<(), String> {
    let manifest_path =
        repo_root.join("docs/test-fixtures/core-engine/upstream-host/MANIFEST.json");
    if manifest_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root for upstream-host runs: missing {}",
            manifest_path.display()
        ))
    }
}

fn ensure_independent_conformance_root(repo_root: &Path) -> Result<(), String> {
    let trace_run = repo_root.join(
        "docs/test-runs/core-engine/tracecalc-reference-machine/w034-tracecalc-oracle-deepening-001/run_summary.json",
    );
    let tree_run = repo_root.join(
        "docs/test-runs/core-engine/treecalc-local/w034-independent-conformance-treecalc-001/run_summary.json",
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

fn ensure_grid_seed_root(repo_root: &Path) -> Result<(), String> {
    let workset_path =
        repo_root.join("docs/worksets/W061_STRICT_EXCEL_GRID_PLANNING_AND_REFERENCE_FLOOR.md");
    if workset_path.exists() {
        Ok(())
    } else {
        Err(format!(
            "current directory is not the OxCalc repo root for grid-seed runs: missing {}",
            workset_path.display()
        ))
    }
}

fn ensure_grid_scale_root(repo_root: &Path) -> Result<(), String> {
    ensure_grid_seed_root(repo_root).map_err(|message| message.replace("grid-seed", "grid-scale"))
}

fn parse_grid_seed_engine(mut args: impl Iterator<Item = String>) -> Result<String, String> {
    let mut engine = "both".to_string();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--engine" => {
                let Some(value) = args.next() else {
                    return Err(grid_seed_usage());
                };
                GridEngineMode::from_engine_arg(&value)
                    .map_err(|error| format!("{error}. {}", grid_seed_usage()))?;
                engine = value;
            }
            _ => {
                return Err(format!(
                    "unknown grid-seed option '{arg}'. {}",
                    grid_seed_usage()
                ));
            }
        }
    }
    Ok(engine)
}

fn parse_grid_scale_options(
    profile: GridScaleProfile,
    run_id: String,
    mut args: impl Iterator<Item = String>,
) -> Result<GridScaleOptions, String> {
    let mut options = GridScaleOptions::default_for(profile, run_id);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--rows" => options.rows = parse_u32_flag(&arg, &mut args, grid_scale_usage())?,
            "--cols" => options.cols = parse_u32_flag(&arg, &mut args, grid_scale_usage())?,
            _ => {
                return Err(format!(
                    "unknown grid-scale option '{arg}'. {}",
                    grid_scale_usage()
                ));
            }
        }
    }
    Ok(options)
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

fn parse_u32_flag(
    flag: &str,
    args: &mut impl Iterator<Item = String>,
    usage: String,
) -> Result<u32, String> {
    let Some(value) = args.next() else {
        return Err(format!("missing value for {flag}. {usage}"));
    };
    value
        .parse::<u32>()
        .map_err(|error| format!("invalid value for {flag}: {value} ({error})"))
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

fn grid_seed_usage() -> String {
    "usage: oxcalc-tracecalc-cli grid-seed <run-id> [--engine reference|optimized|both]".to_string()
}

fn grid_scale_usage() -> String {
    "usage: oxcalc-tracecalc-cli grid-scale <sparse-whole-column|full-column-1m|sparse-singletons|zig-zag-1m|dense-values|repeated-r1c1|fill-down-r1c1|pascal-r1c1-1m|boring-1mx10|direct-r1c1-1m|unary-r1c1-1m|argument-aggregate-r1c1-1m|math-function-r1c1-1m|mod-function-r1c1-1m|rounding-function-r1c1-1m|integer-function-r1c1-1m|log-function-r1c1-1m|trig-function-r1c1-1m|angle-function-r1c1-1m|reference-function-r1c1-1m|logical-function-r1c1-1m|if-logical-r1c1-1m|two-left-r1c1-1m|absolute-r1c1-1m|division-r1c1-1m|decimal-r1c1-1m|recursive-binary-r1c1-1m|if-r1c1-1m|if-branch-r1c1-1m|nested-if-r1c1-1m|iferror-r1c1-1m|comparison-r1c1-1m|comparison-expression-r1c1-1m|comparison-iferror-r1c1-1m|sum-row-r1c1-1m|sumsq-row-r1c1-1m|count-row-r1c1-1m|product-row-r1c1-1m|average-row-r1c1-1m|min-max-row-r1c1-1m|sum-window-r1c1-1m|division-error-r1c1-1m|division-error-propagation-r1c1-1m|aggregate-error-r1c1-1m|text-function-r1c1-1m|index-function-r1c1-1m|match-function-r1c1-1m|vlookup-function-r1c1-1m|insert-storm-1m|publication-delta-1m|tile-stream-64k|viewport-64k-of-1m|cow-retention-1m|plan-cache-rounds-1m|range-invalidation-1m|range-query-1m|sum-pyramid-1m|dirty-rect-1m|hide-storm-1m|spill-anchor-1m|spill-blockage-1m|aggregate-context-1m|spill-epoch-1m|filter-spill-1m|sequence-spill-1m> <run-id> [--rows N] [--cols N]".to_string()
}
