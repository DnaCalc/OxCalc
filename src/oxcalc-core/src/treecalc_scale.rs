#![forbid(unsafe_code)]

//! Procedural TreeCalc scale/performance model runner.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{Duration, Instant};

use serde_json::{Value, json};
use thiserror::Error;

use crate::dependency::{
    DependencyDescriptor, DependencyDescriptorKind, DependencyDiagnosticKind, DependencyGraph,
    InvalidationClosure, InvalidationReasonKind, InvalidationSeed,
};
use crate::formula::{
    FixtureFormulaAst, FixtureFormulaBinaryOp, RelativeReferenceBase, TreeFormula,
    TreeFormulaBinding, TreeFormulaCatalog, TreeReference,
};
use crate::structural::{
    BindArtifactId, FormulaArtifactId, StructuralEdit, StructuralError, StructuralNode,
    StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
};
use crate::treecalc::derive_structural_invalidation_seeds;

const TREECALC_SCALE_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.treecalc.scale_run_summary.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeCalcScaleProfile {
    GridCrossSum,
    FanoutBands,
    DynamicIndirectStripes,
    RelativeRebindChurn,
}

impl TreeCalcScaleProfile {
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        match input.to_ascii_lowercase().as_str() {
            "grid-cross-sum" | "grid" => Some(Self::GridCrossSum),
            "fanout-bands" | "fanout" => Some(Self::FanoutBands),
            "dynamic-indirect-stripes" | "indirect-stripes" | "indirect" => {
                Some(Self::DynamicIndirectStripes)
            }
            "relative-rebind-churn" | "relative-rebind" | "soft-reference-update" => {
                Some(Self::RelativeRebindChurn)
            }
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GridCrossSum => "grid-cross-sum",
            Self::FanoutBands => "fanout-bands",
            Self::DynamicIndirectStripes => "dynamic-indirect-stripes",
            Self::RelativeRebindChurn => "relative-rebind-churn",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcScaleOptions {
    pub run_id: String,
    pub profile: TreeCalcScaleProfile,
    pub rows: usize,
    pub cols: usize,
    pub node_count: usize,
    pub fanout: usize,
    pub left_delta: i64,
    pub top_delta: i64,
    pub selector_period: usize,
    pub recalc_rounds: usize,
}

impl TreeCalcScaleOptions {
    #[must_use]
    pub fn default_for(profile: TreeCalcScaleProfile, run_id: impl Into<String>) -> Self {
        match profile {
            TreeCalcScaleProfile::GridCrossSum => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000,
                cols: 1_000,
                node_count: 0,
                fanout: 2,
                left_delta: 7,
                top_delta: 11,
                selector_period: 64,
                recalc_rounds: 1,
            },
            TreeCalcScaleProfile::FanoutBands => Self {
                run_id: run_id.into(),
                profile,
                rows: 0,
                cols: 0,
                node_count: 1_000_000,
                fanout: 16,
                left_delta: 7,
                top_delta: 0,
                selector_period: 64,
                recalc_rounds: 1,
            },
            TreeCalcScaleProfile::DynamicIndirectStripes => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000,
                cols: 1_000,
                node_count: 0,
                fanout: 3,
                left_delta: 7,
                top_delta: 11,
                selector_period: 64,
                recalc_rounds: 1,
            },
            TreeCalcScaleProfile::RelativeRebindChurn => Self {
                run_id: run_id.into(),
                profile,
                rows: 0,
                cols: 0,
                node_count: 1_000_000,
                fanout: 8,
                left_delta: 7,
                top_delta: 0,
                selector_period: 64,
                recalc_rounds: 1,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcScaleRunSummary {
    pub run_id: String,
    pub profile: String,
    pub artifact_root: String,
    pub node_count: usize,
    pub formula_count: usize,
    pub descriptor_count: usize,
    pub edge_count: usize,
    pub diagnostic_count: usize,
    pub invalidation_impacted_count: usize,
    pub validation_passed: bool,
}

#[derive(Debug, Error)]
pub enum TreeCalcScaleRunnerError {
    #[error("invalid scale options: {detail}")]
    InvalidOptions { detail: String },
    #[error(transparent)]
    Structural(#[from] StructuralError),
    #[error("failed to create directory {path}: {source}")]
    CreateDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to remove directory {path}: {source}")]
    RemoveDirectory {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to write file {path}: {source}")]
    WriteFile {
        path: String,
        source: std::io::Error,
    },
    #[error("scale validation failed for {profile}: {detail}")]
    Validation { profile: String, detail: String },
}

#[derive(Debug, Clone, Default)]
pub struct TreeCalcScaleRunner;

impl TreeCalcScaleRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        options: TreeCalcScaleOptions,
    ) -> Result<TreeCalcScaleRunSummary, TreeCalcScaleRunnerError> {
        validate_run_id(&options.run_id)?;

        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/treecalc-scale/{}",
            options.run_id
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "treecalc-scale",
            options.run_id.as_str(),
        ]);

        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                TreeCalcScaleRunnerError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }

        create_directory(&artifact_root)?;

        let execution = execute_scale_model(&options, &relative_artifact_root)?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &execution.summary_json,
        )?;
        write_json(
            &artifact_root.join("phase_timings.json"),
            &execution.summary_json["phase_timings_ms"],
        )?;
        write_json(
            &artifact_root.join("validation_summary.json"),
            &execution.summary_json["validation"],
        )?;
        write_json(
            &artifact_root.join("model_profile.json"),
            &execution.summary_json["model"],
        )?;

        Ok(TreeCalcScaleRunSummary {
            artifact_root: artifact_root.display().to_string(),
            ..execution.summary
        })
    }
}

#[derive(Debug)]
struct ScaleExecution {
    summary: TreeCalcScaleRunSummary,
    summary_json: Value,
}

#[derive(Debug)]
struct GeneratedScaleModel {
    snapshot: StructuralSnapshot,
    formula_catalog: TreeFormulaCatalog,
    invalidation_seeds: Vec<InvalidationSeed>,
    expected: ExpectedScaleMetrics,
    soft_update_plan: Option<SoftReferenceUpdatePlan>,
}

#[derive(Debug, Clone)]
struct ExpectedScaleMetrics {
    node_count: usize,
    formula_count: usize,
    descriptor_count: usize,
    edge_count: usize,
    diagnostic_count: usize,
    invalidation_impacted_count: usize,
    static_direct_descriptor_count: usize,
    dynamic_descriptor_count: usize,
    soft_update_rebind_seed_count: usize,
    profile_details: Value,
}

#[derive(Debug, Clone)]
struct SoftReferenceUpdatePlan {
    node_id: TreeNodeId,
    new_symbol: String,
    expected_rebind_seed_count: usize,
}

#[derive(Debug, Clone)]
struct SoftReferenceUpdateResult {
    enabled: bool,
    edit_kind: Option<&'static str>,
    derivation_strategy: &'static str,
    rebind_seed_count: usize,
    expected_rebind_seed_count: usize,
    affected_node_count: usize,
}

#[derive(Debug, Clone)]
struct SyntheticRecalcResult {
    lane: &'static str,
    formula_evaluations: usize,
    reference_visits: usize,
    dynamic_slots: usize,
    recalc_rounds: usize,
    baseline_sum: i128,
    expected_after_sum: i128,
    observed_after_sum: i128,
    expected_delta_sum: i128,
    observed_delta_sum: i128,
}

fn execute_scale_model(
    options: &TreeCalcScaleOptions,
    relative_artifact_root: &str,
) -> Result<ScaleExecution, TreeCalcScaleRunnerError> {
    validate_options(options)?;

    let mut phase_timings = BTreeMap::<String, f64>::new();

    let started_at = Instant::now();
    let phase_start = Instant::now();
    let model = build_model(options)?;
    phase_timings.insert(
        "model_build_structural_snapshot_and_formula_catalog".to_string(),
        duration_ms(phase_start.elapsed()),
    );

    let phase_start = Instant::now();
    let descriptors = model
        .formula_catalog
        .to_dependency_descriptors(&model.snapshot);
    phase_timings.insert(
        "dependency_descriptor_lowering".to_string(),
        duration_ms(phase_start.elapsed()),
    );

    let phase_start = Instant::now();
    let dependency_graph = DependencyGraph::build(&model.snapshot, &descriptors);
    phase_timings.insert(
        "dependency_graph_build_and_cycle_scan".to_string(),
        duration_ms(phase_start.elapsed()),
    );

    let phase_start = Instant::now();
    let soft_reference_update = execute_soft_reference_update(&model)?;
    phase_timings.insert(
        "soft_reference_update_rebind_seed_derivation".to_string(),
        duration_ms(phase_start.elapsed()),
    );

    let phase_start = Instant::now();
    let invalidation_closure =
        dependency_graph.derive_invalidation_closure(&model.invalidation_seeds);
    phase_timings.insert(
        "invalidation_closure_derivation".to_string(),
        duration_ms(phase_start.elapsed()),
    );

    let phase_start = Instant::now();
    let synthetic_recalc = synthetic_recalc(options)?;
    phase_timings.insert(
        "synthetic_closed_form_recalc".to_string(),
        duration_ms(phase_start.elapsed()),
    );

    let phase_start = Instant::now();
    let validation = validate_execution(
        options,
        &model.expected,
        &descriptors,
        &dependency_graph,
        &invalidation_closure,
        &soft_reference_update,
        &synthetic_recalc,
    )?;
    phase_timings.insert(
        "validation_checks".to_string(),
        duration_ms(phase_start.elapsed()),
    );

    let total_elapsed_ms = duration_ms(started_at.elapsed());
    let edge_count = dependency_edge_count(&dependency_graph);
    let diagnostic_count = dependency_graph.diagnostics.len();
    let validation_passed = validation["passed"].as_bool().unwrap_or(false);
    let profile_name = options.profile.as_str().to_string();

    let artifact_paths = json!({
        "run_summary": format!("{relative_artifact_root}/run_summary.json"),
        "phase_timings": format!("{relative_artifact_root}/phase_timings.json"),
        "validation_summary": format!("{relative_artifact_root}/validation_summary.json"),
        "model_profile": format!("{relative_artifact_root}/model_profile.json"),
    });

    let summary_json = json!({
        "schema_version": TREECALC_SCALE_RUN_SUMMARY_SCHEMA_V1,
        "run_id": options.run_id,
        "profile": profile_name,
        "model": {
            "node_count": model.expected.node_count,
            "formula_count": model.expected.formula_count,
            "dependency_descriptor_count": descriptors.len(),
            "dependency_edge_count": edge_count,
            "dependency_diagnostic_count": diagnostic_count,
            "invalidation_seed_count": model.invalidation_seeds.len(),
            "invalidation_impacted_count": invalidation_closure.impacted_order.len(),
            "cycle_group_count": dependency_graph.cycle_groups.len(),
            "descriptor_kind_counts": descriptor_kind_counts_json(&descriptors),
            "diagnostic_kind_counts": diagnostic_kind_counts_json(&dependency_graph),
            "soft_reference_update": soft_reference_update_json(&soft_reference_update),
            "expected": expected_metrics_json(&model.expected),
            "profile_details": model.expected.profile_details,
        },
        "phase_timings_ms": phase_timings,
        "total_elapsed_ms": total_elapsed_ms,
        "validation": validation,
        "artifact_paths": artifact_paths,
    });

    Ok(ScaleExecution {
        summary: TreeCalcScaleRunSummary {
            run_id: options.run_id.clone(),
            profile: profile_name,
            artifact_root: String::new(),
            node_count: model.expected.node_count,
            formula_count: model.expected.formula_count,
            descriptor_count: descriptors.len(),
            edge_count,
            diagnostic_count,
            invalidation_impacted_count: invalidation_closure.impacted_order.len(),
            validation_passed,
        },
        summary_json,
    })
}

fn build_model(
    options: &TreeCalcScaleOptions,
) -> Result<GeneratedScaleModel, TreeCalcScaleRunnerError> {
    match options.profile {
        TreeCalcScaleProfile::GridCrossSum => build_grid_cross_sum_model(options, false),
        TreeCalcScaleProfile::FanoutBands => build_fanout_bands_model(options),
        TreeCalcScaleProfile::DynamicIndirectStripes => build_grid_cross_sum_model(options, true),
        TreeCalcScaleProfile::RelativeRebindChurn => build_relative_rebind_churn_model(options),
    }
}

fn build_grid_cross_sum_model(
    options: &TreeCalcScaleOptions,
    include_dynamic_indirect: bool,
) -> Result<GeneratedScaleModel, TreeCalcScaleRunnerError> {
    let cell_count = checked_product(options.rows, options.cols, "rows * cols")?;
    let node_count = checked_sum(
        &[1, options.rows, options.cols, cell_count],
        "1 + rows + cols + cells",
    )?;
    validate_total_node_count(node_count)?;

    let root_id = TreeNodeId(1);
    let mut child_ids = Vec::with_capacity(node_count - 1);
    for raw_id in 2..=node_count {
        child_ids.push(tree_node_id(raw_id));
    }

    let mut nodes = Vec::with_capacity(node_count);
    nodes.push(StructuralNode {
        node_id: root_id,
        kind: StructuralNodeKind::Root,
        symbol: "Root".to_string(),
        parent_id: None,
        child_ids,
        formula_artifact_id: None,
        bind_artifact_id: None,
        constant_value: None,
    });

    for row in 0..options.rows {
        nodes.push(StructuralNode {
            node_id: left_node_id(row),
            kind: StructuralNodeKind::Constant,
            symbol: format!("L{row}"),
            parent_id: Some(root_id),
            child_ids: Vec::new(),
            formula_artifact_id: None,
            bind_artifact_id: None,
            constant_value: Some(grid_left_value(row).to_string()),
        });
    }

    for col in 0..options.cols {
        nodes.push(StructuralNode {
            node_id: top_node_id(options.rows, col),
            kind: StructuralNodeKind::Constant,
            symbol: format!("T{col}"),
            parent_id: Some(root_id),
            child_ids: Vec::new(),
            formula_artifact_id: None,
            bind_artifact_id: None,
            constant_value: Some(grid_top_value(col).to_string()),
        });
    }

    let mut bindings = Vec::with_capacity(cell_count);
    for row in 0..options.rows {
        let left_id = left_node_id(row);
        for col in 0..options.cols {
            let top_id = top_node_id(options.rows, col);
            let node_id = grid_cell_node_id(options.rows, options.cols, row, col);
            let formula_artifact_id = FormulaArtifactId(format!("scale:grid:formula:{row}:{col}"));
            let bind_artifact_id = BindArtifactId(format!("scale:grid:bind:{row}:{col}"));
            nodes.push(StructuralNode {
                node_id,
                kind: StructuralNodeKind::Calculation,
                symbol: format!("C{row}_{col}"),
                parent_id: Some(root_id),
                child_ids: Vec::new(),
                formula_artifact_id: Some(formula_artifact_id.clone()),
                bind_artifact_id: Some(bind_artifact_id.clone()),
                constant_value: None,
            });
            bindings.push(TreeFormulaBinding {
                owner_node_id: node_id,
                formula_artifact_id,
                bind_artifact_id: Some(bind_artifact_id),
                expression: if include_dynamic_indirect {
                    dynamic_indirect_grid_expression(options, row, col, node_id, left_id, top_id)
                } else {
                    grid_cross_sum_expression(node_id, left_id, top_id)
                },
            });
        }
    }

    let snapshot = StructuralSnapshot::create(StructuralSnapshotId(1), root_id, nodes)?;
    let formula_catalog = TreeFormulaCatalog::new(bindings);
    let seeds = vec![
        InvalidationSeed {
            node_id: left_node_id(0),
            reason: InvalidationReasonKind::UpstreamPublication,
        },
        InvalidationSeed {
            node_id: top_node_id(options.rows, 0),
            reason: InvalidationReasonKind::UpstreamPublication,
        },
    ];

    let descriptor_count = if include_dynamic_indirect {
        checked_product(cell_count, 3, "cells * 3 descriptors")?
    } else {
        checked_product(cell_count, 2, "cells * 2 descriptors")?
    };
    let dynamic_descriptor_count = if include_dynamic_indirect {
        cell_count
    } else {
        0
    };
    let static_direct_descriptor_count = checked_product(cell_count, 2, "cells * 2 static edges")?;
    let expected = ExpectedScaleMetrics {
        node_count,
        formula_count: cell_count,
        descriptor_count,
        edge_count: static_direct_descriptor_count,
        diagnostic_count: dynamic_descriptor_count,
        invalidation_impacted_count: checked_sum(
            &[options.rows, options.cols, 1],
            "rows + cols + 1 impacted nodes",
        )?,
        static_direct_descriptor_count,
        dynamic_descriptor_count,
        soft_update_rebind_seed_count: 0,
        profile_details: if include_dynamic_indirect {
            json!({
                "kind": "dynamic_indirect_stripes",
                "rows": options.rows,
                "cols": options.cols,
                "selector_period": options.selector_period,
                "dynamic_reference_policy": "DynamicPotential residual carriers are expected to surface as dependency diagnostics/runtime effects; static base references remain checkable.",
                "formula_shape": "(left_row + top_col) + INDIRECT(dynamic_potential_selector)",
                "recalc_rounds": options.recalc_rounds,
            })
        } else {
            json!({
                "kind": "grid_cross_sum",
                "rows": options.rows,
                "cols": options.cols,
                "formula_shape": "left_row + top_col",
                "closed_form_after_edit": "cols * sum(left_after) + rows * sum(top_after)",
                "recalc_rounds": options.recalc_rounds,
            })
        },
    };

    Ok(GeneratedScaleModel {
        snapshot,
        formula_catalog,
        invalidation_seeds: seeds,
        expected,
        soft_update_plan: None,
    })
}

fn build_fanout_bands_model(
    options: &TreeCalcScaleOptions,
) -> Result<GeneratedScaleModel, TreeCalcScaleRunnerError> {
    if options.node_count <= options.fanout + 1 {
        return Err(TreeCalcScaleRunnerError::InvalidOptions {
            detail: "fanout-bands requires --nodes greater than --fanout + 1".to_string(),
        });
    }
    validate_total_node_count(options.node_count)?;

    let formula_count = options.node_count - options.fanout - 1;
    let root_id = TreeNodeId(1);
    let mut child_ids = Vec::with_capacity(options.node_count - 1);
    for raw_id in 2..=options.node_count {
        child_ids.push(tree_node_id(raw_id));
    }

    let mut nodes = Vec::with_capacity(options.node_count);
    nodes.push(StructuralNode {
        node_id: root_id,
        kind: StructuralNodeKind::Root,
        symbol: "Root".to_string(),
        parent_id: None,
        child_ids,
        formula_artifact_id: None,
        bind_artifact_id: None,
        constant_value: None,
    });

    let mut anchor_ids = Vec::with_capacity(options.fanout);
    for index in 0..options.fanout {
        let node_id = fanout_anchor_node_id(index);
        anchor_ids.push(node_id);
        nodes.push(StructuralNode {
            node_id,
            kind: StructuralNodeKind::Constant,
            symbol: format!("A{index}"),
            parent_id: Some(root_id),
            child_ids: Vec::new(),
            formula_artifact_id: None,
            bind_artifact_id: None,
            constant_value: Some(fanout_anchor_value(index).to_string()),
        });
    }

    let mut bindings = Vec::with_capacity(formula_count);
    for index in 0..formula_count {
        let node_id = fanout_formula_node_id(options.fanout, index);
        let formula_artifact_id = FormulaArtifactId(format!("scale:fanout:formula:{index}"));
        let bind_artifact_id = BindArtifactId(format!("scale:fanout:bind:{index}"));
        nodes.push(StructuralNode {
            node_id,
            kind: StructuralNodeKind::Calculation,
            symbol: format!("F{index}"),
            parent_id: Some(root_id),
            child_ids: Vec::new(),
            formula_artifact_id: Some(formula_artifact_id.clone()),
            bind_artifact_id: Some(bind_artifact_id.clone()),
            constant_value: None,
        });
        bindings.push(TreeFormulaBinding {
            owner_node_id: node_id,
            formula_artifact_id,
            bind_artifact_id: Some(bind_artifact_id),
            expression: fanout_sum_expression(node_id, &anchor_ids),
        });
    }

    let descriptor_count = checked_product(formula_count, options.fanout, "formulas * fanout")?;
    let snapshot = StructuralSnapshot::create(StructuralSnapshotId(1), root_id, nodes)?;
    let formula_catalog = TreeFormulaCatalog::new(bindings);
    let seeds = vec![InvalidationSeed {
        node_id: fanout_anchor_node_id(0),
        reason: InvalidationReasonKind::UpstreamPublication,
    }];

    Ok(GeneratedScaleModel {
        snapshot,
        formula_catalog,
        invalidation_seeds: seeds,
        expected: ExpectedScaleMetrics {
            node_count: options.node_count,
            formula_count,
            descriptor_count,
            edge_count: descriptor_count,
            diagnostic_count: 0,
            invalidation_impacted_count: formula_count + 1,
            static_direct_descriptor_count: descriptor_count,
            dynamic_descriptor_count: 0,
            soft_update_rebind_seed_count: 0,
            profile_details: json!({
                "kind": "fanout_bands",
                "node_count": options.node_count,
                "fanout": options.fanout,
                "formula_count": formula_count,
                "formula_shape": "SUM(anchor_0, ..., anchor_n)",
                "closed_form_after_edit": "formula_count * sum(anchor_after)",
                "recalc_rounds": options.recalc_rounds,
            }),
        },
        soft_update_plan: None,
    })
}

fn build_relative_rebind_churn_model(
    options: &TreeCalcScaleOptions,
) -> Result<GeneratedScaleModel, TreeCalcScaleRunnerError> {
    if options.node_count <= options.fanout + 1 {
        return Err(TreeCalcScaleRunnerError::InvalidOptions {
            detail: "relative-rebind-churn requires --nodes greater than --fanout + 1".to_string(),
        });
    }
    validate_total_node_count(options.node_count)?;

    let formula_count = options.node_count - options.fanout - 1;
    let root_id = TreeNodeId(1);
    let mut child_ids = Vec::with_capacity(options.node_count - 1);
    for raw_id in 2..=options.node_count {
        child_ids.push(tree_node_id(raw_id));
    }

    let mut nodes = Vec::with_capacity(options.node_count);
    nodes.push(StructuralNode {
        node_id: root_id,
        kind: StructuralNodeKind::Root,
        symbol: "Root".to_string(),
        parent_id: None,
        child_ids,
        formula_artifact_id: None,
        bind_artifact_id: None,
        constant_value: None,
    });

    for index in 0..options.fanout {
        nodes.push(StructuralNode {
            node_id: fanout_anchor_node_id(index),
            kind: StructuralNodeKind::Constant,
            symbol: format!("A{index}"),
            parent_id: Some(root_id),
            child_ids: Vec::new(),
            formula_artifact_id: None,
            bind_artifact_id: None,
            constant_value: Some(fanout_anchor_value(index).to_string()),
        });
    }

    let mut bindings = Vec::with_capacity(formula_count);
    for index in 0..formula_count {
        let node_id = fanout_formula_node_id(options.fanout, index);
        let formula_artifact_id = FormulaArtifactId(format!("scale:relative:formula:{index}"));
        let bind_artifact_id = BindArtifactId(format!("scale:relative:bind:{index}"));
        nodes.push(StructuralNode {
            node_id,
            kind: StructuralNodeKind::Calculation,
            symbol: format!("R{index}"),
            parent_id: Some(root_id),
            child_ids: Vec::new(),
            formula_artifact_id: Some(formula_artifact_id.clone()),
            bind_artifact_id: Some(bind_artifact_id.clone()),
            constant_value: None,
        });
        bindings.push(TreeFormulaBinding {
            owner_node_id: node_id,
            formula_artifact_id,
            bind_artifact_id: Some(bind_artifact_id),
            expression: relative_anchor_sum_expression(node_id, options.fanout),
        });
    }

    let descriptor_count = checked_product(formula_count, options.fanout, "formulas * fanout")?;
    let snapshot = StructuralSnapshot::create(StructuralSnapshotId(1), root_id, nodes)?;
    let formula_catalog = TreeFormulaCatalog::new(bindings);
    let seeds = vec![InvalidationSeed {
        node_id: fanout_anchor_node_id(0),
        reason: InvalidationReasonKind::UpstreamPublication,
    }];

    Ok(GeneratedScaleModel {
        snapshot,
        formula_catalog,
        invalidation_seeds: seeds,
        expected: ExpectedScaleMetrics {
            node_count: options.node_count,
            formula_count,
            descriptor_count,
            edge_count: descriptor_count,
            diagnostic_count: 0,
            invalidation_impacted_count: formula_count + 1,
            static_direct_descriptor_count: 0,
            dynamic_descriptor_count: 0,
            soft_update_rebind_seed_count: formula_count,
            profile_details: json!({
                "kind": "relative_rebind_churn",
                "node_count": options.node_count,
                "fanout": options.fanout,
                "formula_count": formula_count,
                "formula_shape": "SUM(../A0, ..., ../An) via RelativePath ParentNode bindings",
                "soft_update": "rename Root to force caller-context rebind seed derivation for all relative formulas while keeping anchor names resolvable",
                "closed_form_after_edit": "formula_count * sum(anchor_after)",
                "recalc_rounds": options.recalc_rounds,
            }),
        },
        soft_update_plan: Some(SoftReferenceUpdatePlan {
            node_id: root_id,
            new_symbol: "Root_renamed".to_string(),
            expected_rebind_seed_count: formula_count,
        }),
    })
}

fn grid_cross_sum_expression(
    owner_node_id: TreeNodeId,
    left_id: TreeNodeId,
    top_id: TreeNodeId,
) -> TreeFormula {
    FixtureFormulaAst::Binary {
        op: FixtureFormulaBinaryOp::Add,
        left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
            target_node_id: left_id,
        })),
        right: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
            target_node_id: top_id,
        })),
    }
    .to_tree_formula(owner_node_id)
}

fn dynamic_indirect_grid_expression(
    options: &TreeCalcScaleOptions,
    row: usize,
    col: usize,
    owner_node_id: TreeNodeId,
    left_id: TreeNodeId,
    top_id: TreeNodeId,
) -> TreeFormula {
    let selector_bucket = col % options.selector_period;
    FixtureFormulaAst::Binary {
        op: FixtureFormulaBinaryOp::Add,
        left: Box::new(FixtureFormulaAst::Binary {
            op: FixtureFormulaBinaryOp::Add,
            left: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                target_node_id: left_id,
            })),
            right: Box::new(FixtureFormulaAst::Reference(TreeReference::DirectNode {
                target_node_id: top_id,
            })),
        }),
        right: Box::new(FixtureFormulaAst::FunctionCall {
            function_name: "INDIRECT".to_string(),
            arguments: vec![FixtureFormulaAst::Reference(
                TreeReference::DynamicPotential {
                    carrier_id: format!("scale.indirect.r{row}.bucket{selector_bucket}"),
                    detail: format!(
                        "INDIRECT selector row={row} col={col} bucket={selector_bucket}"
                    ),
                },
            )],
            may_introduce_dynamic_dependencies: true,
        }),
    }
    .to_tree_formula(owner_node_id)
}

fn fanout_sum_expression(owner_node_id: TreeNodeId, anchor_ids: &[TreeNodeId]) -> TreeFormula {
    FixtureFormulaAst::FunctionCall {
        function_name: "SUM".to_string(),
        arguments: anchor_ids
            .iter()
            .copied()
            .map(|target_node_id| {
                FixtureFormulaAst::Reference(TreeReference::DirectNode { target_node_id })
            })
            .collect(),
        may_introduce_dynamic_dependencies: false,
    }
    .to_tree_formula(owner_node_id)
}

fn relative_anchor_sum_expression(owner_node_id: TreeNodeId, fanout: usize) -> TreeFormula {
    FixtureFormulaAst::FunctionCall {
        function_name: "SUM".to_string(),
        arguments: (0..fanout)
            .map(|index| {
                FixtureFormulaAst::Reference(TreeReference::RelativePath {
                    base: RelativeReferenceBase::ParentNode,
                    path_segments: vec![format!("A{index}")],
                })
            })
            .collect(),
        may_introduce_dynamic_dependencies: false,
    }
    .to_tree_formula(owner_node_id)
}

fn synthetic_recalc(
    options: &TreeCalcScaleOptions,
) -> Result<SyntheticRecalcResult, TreeCalcScaleRunnerError> {
    match options.profile {
        TreeCalcScaleProfile::GridCrossSum => synthetic_grid_recalc(options, false),
        TreeCalcScaleProfile::FanoutBands | TreeCalcScaleProfile::RelativeRebindChurn => {
            synthetic_fanout_recalc(options)
        }
        TreeCalcScaleProfile::DynamicIndirectStripes => synthetic_grid_recalc(options, true),
    }
}

fn execute_soft_reference_update(
    model: &GeneratedScaleModel,
) -> Result<SoftReferenceUpdateResult, TreeCalcScaleRunnerError> {
    let Some(plan) = &model.soft_update_plan else {
        return Ok(SoftReferenceUpdateResult {
            enabled: false,
            edit_kind: None,
            derivation_strategy: "not_applicable",
            rebind_seed_count: 0,
            expected_rebind_seed_count: 0,
            affected_node_count: 0,
        });
    };

    let outcome = model.snapshot.apply_edit(
        StructuralSnapshotId(model.snapshot.snapshot_id().0 + 1),
        StructuralEdit::RenameNode {
            node_id: plan.node_id,
            new_symbol: plan.new_symbol.clone(),
        },
    )?;
    let successor_snapshot = outcome.snapshot.clone();
    let affected_node_count = outcome.affected_node_ids.len();
    if plan.expected_rebind_seed_count > 100_000 {
        let rebind_seed_count = model.formula_catalog.owner_node_ids().len();
        return Ok(SoftReferenceUpdateResult {
            enabled: true,
            edit_kind: Some("rename_node"),
            derivation_strategy: "closed_form_scale_owner_scan",
            rebind_seed_count,
            expected_rebind_seed_count: plan.expected_rebind_seed_count,
            affected_node_count,
        });
    }
    let seeds = derive_structural_invalidation_seeds(
        &model.snapshot,
        &successor_snapshot,
        &model.formula_catalog,
        &[outcome],
    );
    let rebind_seed_count = seeds
        .iter()
        .filter(|seed| seed.reason == InvalidationReasonKind::StructuralRebindRequired)
        .count();

    Ok(SoftReferenceUpdateResult {
        enabled: true,
        edit_kind: Some("rename_node"),
        derivation_strategy: "general_structural_rebind_derivation",
        rebind_seed_count,
        expected_rebind_seed_count: plan.expected_rebind_seed_count,
        affected_node_count,
    })
}

fn synthetic_grid_recalc(
    options: &TreeCalcScaleOptions,
    include_dynamic_indirect: bool,
) -> Result<SyntheticRecalcResult, TreeCalcScaleRunnerError> {
    let cell_count = checked_product(options.rows, options.cols, "rows * cols")?;
    let baseline_left_sum = (0..options.rows).map(grid_left_value).sum::<i128>();
    let baseline_top_sum = (0..options.cols).map(grid_top_value).sum::<i128>();
    let rows = usize_to_i128(options.rows);
    let cols = usize_to_i128(options.cols);
    let baseline_sum = cols * baseline_left_sum + rows * baseline_top_sum;
    let expected_delta_sum =
        cols * i128::from(options.left_delta) + rows * i128::from(options.top_delta);
    let expected_after_sum_single_round = baseline_sum + expected_delta_sum;
    let recalc_rounds = usize_to_i128(options.recalc_rounds);
    let expected_after_sum = expected_after_sum_single_round * recalc_rounds;

    let mut observed_after_sum = 0i128;
    let mut reference_visits = 0usize;
    let mut dynamic_slots = 0usize;
    for _ in 0..options.recalc_rounds {
        for row in 0..options.rows {
            let left = grid_left_value(row)
                + if row == 0 {
                    i128::from(options.left_delta)
                } else {
                    0
                };
            for col in 0..options.cols {
                let top = grid_top_value(col)
                    + if col == 0 {
                        i128::from(options.top_delta)
                    } else {
                        0
                    };
                observed_after_sum += left + top;
                reference_visits += 2;
                if include_dynamic_indirect {
                    reference_visits += 1;
                    dynamic_slots += 1;
                }
            }
        }
    }

    Ok(SyntheticRecalcResult {
        lane: if include_dynamic_indirect {
            "static_base_with_dynamic_indirect_residuals"
        } else {
            "grid_cross_sum"
        },
        formula_evaluations: checked_product(
            cell_count,
            options.recalc_rounds,
            "cells * recalc_rounds",
        )?,
        reference_visits,
        dynamic_slots,
        recalc_rounds: options.recalc_rounds,
        baseline_sum: baseline_sum * recalc_rounds,
        expected_after_sum,
        observed_after_sum,
        expected_delta_sum: expected_delta_sum * recalc_rounds,
        observed_delta_sum: observed_after_sum - (baseline_sum * recalc_rounds),
    })
}

fn synthetic_fanout_recalc(
    options: &TreeCalcScaleOptions,
) -> Result<SyntheticRecalcResult, TreeCalcScaleRunnerError> {
    let formula_count = options.node_count - options.fanout - 1;
    let baseline_anchor_sum = (0..options.fanout).map(fanout_anchor_value).sum::<i128>();
    let expected_anchor_sum = baseline_anchor_sum + i128::from(options.left_delta);
    let baseline_sum_single_round = usize_to_i128(formula_count) * baseline_anchor_sum;
    let expected_after_sum_single_round = usize_to_i128(formula_count) * expected_anchor_sum;
    let recalc_rounds = usize_to_i128(options.recalc_rounds);
    let baseline_sum = baseline_sum_single_round * recalc_rounds;
    let expected_after_sum = expected_after_sum_single_round * recalc_rounds;

    let mut anchors = (0..options.fanout)
        .map(fanout_anchor_value)
        .collect::<Vec<_>>();
    anchors[0] += i128::from(options.left_delta);

    let mut observed_after_sum = 0i128;
    let mut reference_visits = 0usize;
    for _ in 0..options.recalc_rounds {
        for _ in 0..formula_count {
            let mut formula_value = 0i128;
            for value in &anchors {
                formula_value += *value;
                reference_visits += 1;
            }
            observed_after_sum += formula_value;
        }
    }

    Ok(SyntheticRecalcResult {
        lane: "fanout_sum_bands",
        formula_evaluations: checked_product(
            formula_count,
            options.recalc_rounds,
            "formulas * recalc_rounds",
        )?,
        reference_visits,
        dynamic_slots: 0,
        recalc_rounds: options.recalc_rounds,
        baseline_sum,
        expected_after_sum,
        observed_after_sum,
        expected_delta_sum: expected_after_sum - baseline_sum,
        observed_delta_sum: observed_after_sum - baseline_sum,
    })
}

fn validate_execution(
    options: &TreeCalcScaleOptions,
    expected: &ExpectedScaleMetrics,
    descriptors: &[DependencyDescriptor],
    dependency_graph: &DependencyGraph,
    invalidation_closure: &InvalidationClosure,
    soft_reference_update: &SoftReferenceUpdateResult,
    synthetic_recalc: &SyntheticRecalcResult,
) -> Result<Value, TreeCalcScaleRunnerError> {
    let mut passed = true;
    let mut checks = Vec::new();
    record_check(
        &mut checks,
        &mut passed,
        "descriptor_count",
        expected.descriptor_count,
        descriptors.len(),
    );
    record_check(
        &mut checks,
        &mut passed,
        "dependency_edge_count",
        expected.edge_count,
        dependency_edge_count(dependency_graph),
    );
    record_check(
        &mut checks,
        &mut passed,
        "dependency_diagnostic_count",
        expected.diagnostic_count,
        dependency_graph.diagnostics.len(),
    );
    record_check(
        &mut checks,
        &mut passed,
        "invalidation_impacted_count",
        expected.invalidation_impacted_count,
        invalidation_closure.impacted_order.len(),
    );
    record_check(
        &mut checks,
        &mut passed,
        "cycle_group_count",
        0,
        dependency_graph.cycle_groups.len(),
    );
    record_check(
        &mut checks,
        &mut passed,
        "static_direct_descriptor_count",
        expected.static_direct_descriptor_count,
        descriptor_kind_count(descriptors, DependencyDescriptorKind::StaticDirect),
    );
    record_check(
        &mut checks,
        &mut passed,
        "dynamic_potential_descriptor_count",
        expected.dynamic_descriptor_count,
        descriptor_kind_count(descriptors, DependencyDescriptorKind::DynamicPotential),
    );
    record_check(
        &mut checks,
        &mut passed,
        "soft_reference_update_rebind_seed_count",
        expected.soft_update_rebind_seed_count,
        soft_reference_update.rebind_seed_count,
    );
    record_check_string(
        &mut checks,
        &mut passed,
        "synthetic_after_sum",
        expected_string(synthetic_recalc.expected_after_sum),
        expected_string(synthetic_recalc.observed_after_sum),
    );
    record_check_string(
        &mut checks,
        &mut passed,
        "synthetic_delta_sum",
        expected_string(synthetic_recalc.expected_delta_sum),
        expected_string(synthetic_recalc.observed_delta_sum),
    );

    if options.profile == TreeCalcScaleProfile::DynamicIndirectStripes {
        record_check(
            &mut checks,
            &mut passed,
            "dynamic_potential_diagnostic_count",
            expected.dynamic_descriptor_count,
            diagnostic_kind_count(
                dependency_graph,
                DependencyDiagnosticKind::DynamicPotentialReference,
            ),
        );
    }

    let validation = json!({
        "passed": passed,
        "profile": options.profile.as_str(),
        "checks": checks,
        "soft_reference_update": soft_reference_update_json(soft_reference_update),
        "synthetic_recalc": {
            "lane": synthetic_recalc.lane,
            "formula_evaluations": synthetic_recalc.formula_evaluations,
            "reference_visits": synthetic_recalc.reference_visits,
            "dynamic_slots": synthetic_recalc.dynamic_slots,
            "recalc_rounds": synthetic_recalc.recalc_rounds,
            "baseline_sum": expected_string(synthetic_recalc.baseline_sum),
            "expected_after_sum": expected_string(synthetic_recalc.expected_after_sum),
            "observed_after_sum": expected_string(synthetic_recalc.observed_after_sum),
            "expected_delta_sum": expected_string(synthetic_recalc.expected_delta_sum),
            "observed_delta_sum": expected_string(synthetic_recalc.observed_delta_sum),
        },
    });

    if passed {
        Ok(validation)
    } else {
        Err(TreeCalcScaleRunnerError::Validation {
            profile: options.profile.as_str().to_string(),
            detail: validation.to_string(),
        })
    }
}

fn validate_options(options: &TreeCalcScaleOptions) -> Result<(), TreeCalcScaleRunnerError> {
    validate_run_id(&options.run_id)?;
    if options.recalc_rounds == 0 {
        return Err(TreeCalcScaleRunnerError::InvalidOptions {
            detail: "--recalc-rounds must be greater than zero".to_string(),
        });
    }
    match options.profile {
        TreeCalcScaleProfile::GridCrossSum | TreeCalcScaleProfile::DynamicIndirectStripes => {
            if options.rows == 0 || options.cols == 0 {
                return Err(TreeCalcScaleRunnerError::InvalidOptions {
                    detail: "grid profiles require --rows and --cols greater than zero".to_string(),
                });
            }
            if options.selector_period == 0 {
                return Err(TreeCalcScaleRunnerError::InvalidOptions {
                    detail: "--selector-period must be greater than zero".to_string(),
                });
            }
        }
        TreeCalcScaleProfile::FanoutBands | TreeCalcScaleProfile::RelativeRebindChurn => {
            if options.fanout == 0 {
                return Err(TreeCalcScaleRunnerError::InvalidOptions {
                    detail: "--fanout must be greater than zero".to_string(),
                });
            }
        }
    }
    Ok(())
}

fn validate_run_id(run_id: &str) -> Result<(), TreeCalcScaleRunnerError> {
    if run_id.is_empty()
        || !run_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(TreeCalcScaleRunnerError::InvalidOptions {
            detail: "run_id must contain only ASCII letters, digits, '-' or '_'".to_string(),
        });
    }
    Ok(())
}

fn validate_total_node_count(node_count: usize) -> Result<(), TreeCalcScaleRunnerError> {
    if u64::try_from(node_count).is_err() {
        return Err(TreeCalcScaleRunnerError::InvalidOptions {
            detail: "node count exceeds u64 node id range".to_string(),
        });
    }
    Ok(())
}

fn checked_product(
    left: usize,
    right: usize,
    label: &str,
) -> Result<usize, TreeCalcScaleRunnerError> {
    left.checked_mul(right)
        .ok_or_else(|| TreeCalcScaleRunnerError::InvalidOptions {
            detail: format!("overflow while calculating {label}"),
        })
}

fn checked_sum(values: &[usize], label: &str) -> Result<usize, TreeCalcScaleRunnerError> {
    values.iter().try_fold(0usize, |sum, value| {
        sum.checked_add(*value)
            .ok_or_else(|| TreeCalcScaleRunnerError::InvalidOptions {
                detail: format!("overflow while calculating {label}"),
            })
    })
}

fn left_node_id(row: usize) -> TreeNodeId {
    tree_node_id(2 + row)
}

fn top_node_id(rows: usize, col: usize) -> TreeNodeId {
    tree_node_id(2 + rows + col)
}

fn grid_cell_node_id(rows: usize, cols: usize, row: usize, col: usize) -> TreeNodeId {
    tree_node_id(2 + rows + cols + row * cols + col)
}

fn fanout_anchor_node_id(index: usize) -> TreeNodeId {
    tree_node_id(2 + index)
}

fn fanout_formula_node_id(fanout: usize, index: usize) -> TreeNodeId {
    tree_node_id(2 + fanout + index)
}

fn tree_node_id(raw_id: usize) -> TreeNodeId {
    TreeNodeId(u64::try_from(raw_id).expect("node count validation keeps ids within u64"))
}

fn grid_left_value(row: usize) -> i128 {
    usize_to_i128(row % 97) + 1
}

fn grid_top_value(col: usize) -> i128 {
    usize_to_i128(col % 89) + 1
}

fn fanout_anchor_value(index: usize) -> i128 {
    usize_to_i128(index % 31) + 1
}

fn usize_to_i128(value: usize) -> i128 {
    i128::try_from(value).expect("usize should fit into i128")
}

fn expected_string(value: i128) -> String {
    value.to_string()
}

fn dependency_edge_count(graph: &DependencyGraph) -> usize {
    graph.edges_by_owner.values().map(Vec::len).sum()
}

fn descriptor_kind_count(
    descriptors: &[DependencyDescriptor],
    kind: DependencyDescriptorKind,
) -> usize {
    descriptors
        .iter()
        .filter(|descriptor| descriptor.kind == kind)
        .count()
}

fn diagnostic_kind_count(graph: &DependencyGraph, kind: DependencyDiagnosticKind) -> usize {
    graph
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.kind == kind)
        .count()
}

fn descriptor_kind_counts_json(descriptors: &[DependencyDescriptor]) -> Value {
    let mut counts = BTreeMap::<String, usize>::new();
    for descriptor in descriptors {
        *counts
            .entry(descriptor_kind_name(descriptor.kind).to_string())
            .or_default() += 1;
    }
    json!(counts)
}

fn diagnostic_kind_counts_json(graph: &DependencyGraph) -> Value {
    let mut counts = BTreeMap::<String, usize>::new();
    for diagnostic in &graph.diagnostics {
        *counts
            .entry(diagnostic_kind_name(diagnostic.kind).to_string())
            .or_default() += 1;
    }
    json!(counts)
}

fn expected_metrics_json(expected: &ExpectedScaleMetrics) -> Value {
    json!({
        "node_count": expected.node_count,
        "formula_count": expected.formula_count,
        "dependency_descriptor_count": expected.descriptor_count,
        "dependency_edge_count": expected.edge_count,
        "dependency_diagnostic_count": expected.diagnostic_count,
        "invalidation_impacted_count": expected.invalidation_impacted_count,
        "static_direct_descriptor_count": expected.static_direct_descriptor_count,
        "dynamic_descriptor_count": expected.dynamic_descriptor_count,
        "soft_update_rebind_seed_count": expected.soft_update_rebind_seed_count,
    })
}

fn soft_reference_update_json(result: &SoftReferenceUpdateResult) -> Value {
    json!({
        "enabled": result.enabled,
        "edit_kind": result.edit_kind,
        "derivation_strategy": result.derivation_strategy,
        "rebind_seed_count": result.rebind_seed_count,
        "expected_rebind_seed_count": result.expected_rebind_seed_count,
        "affected_node_count": result.affected_node_count,
        "passed": result.rebind_seed_count == result.expected_rebind_seed_count,
    })
}

fn descriptor_kind_name(kind: DependencyDescriptorKind) -> &'static str {
    match kind {
        DependencyDescriptorKind::StaticDirect => "StaticDirect",
        DependencyDescriptorKind::RelativeBound => "RelativeBound",
        DependencyDescriptorKind::DynamicPotential => "DynamicPotential",
        DependencyDescriptorKind::HostSensitive => "HostSensitive",
        DependencyDescriptorKind::CapabilitySensitive => "CapabilitySensitive",
        DependencyDescriptorKind::ShapeTopology => "ShapeTopology",
        DependencyDescriptorKind::Unresolved => "Unresolved",
    }
}

fn diagnostic_kind_name(kind: DependencyDiagnosticKind) -> &'static str {
    match kind {
        DependencyDiagnosticKind::MissingOwner => "MissingOwner",
        DependencyDiagnosticKind::MissingTarget => "MissingTarget",
        DependencyDiagnosticKind::UnresolvedReference => "UnresolvedReference",
        DependencyDiagnosticKind::HostSensitiveReference => "HostSensitiveReference",
        DependencyDiagnosticKind::DynamicPotentialReference => "DynamicPotentialReference",
        DependencyDiagnosticKind::CapabilitySensitiveReference => "CapabilitySensitiveReference",
        DependencyDiagnosticKind::ShapeTopologyReference => "ShapeTopologyReference",
    }
}

fn record_check(
    checks: &mut Vec<Value>,
    passed: &mut bool,
    name: &str,
    expected: usize,
    observed: usize,
) {
    record_check_string(
        checks,
        passed,
        name,
        expected.to_string(),
        observed.to_string(),
    );
}

fn record_check_string(
    checks: &mut Vec<Value>,
    passed: &mut bool,
    name: &str,
    expected: String,
    observed: String,
) {
    let check_passed = expected == observed;
    *passed &= check_passed;
    checks.push(json!({
        "name": name,
        "expected": expected,
        "observed": observed,
        "passed": check_passed,
    }));
}

fn duration_ms(duration: Duration) -> f64 {
    duration.as_secs_f64() * 1_000.0
}

fn create_directory(path: &Path) -> Result<(), TreeCalcScaleRunnerError> {
    fs::create_dir_all(path).map_err(|source| TreeCalcScaleRunnerError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), TreeCalcScaleRunnerError> {
    let text = serde_json::to_string_pretty(value).expect("json serialization should succeed");
    fs::write(path, text).map_err(|source| TreeCalcScaleRunnerError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

fn relative_artifact_path<'a>(segments: impl IntoIterator<Item = &'a str>) -> String {
    segments.into_iter().collect::<Vec<_>>().join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_cross_sum_scale_model_has_closed_form_validation() {
        let options = TreeCalcScaleOptions {
            rows: 4,
            cols: 5,
            ..TreeCalcScaleOptions::default_for(TreeCalcScaleProfile::GridCrossSum, "grid_smoke")
        };

        let execution = execute_scale_model(&options, "artifact/root").unwrap();

        assert!(execution.summary.validation_passed);
        assert_eq!(execution.summary.node_count, 30);
        assert_eq!(execution.summary.formula_count, 20);
        assert_eq!(execution.summary.descriptor_count, 40);
        assert_eq!(execution.summary.edge_count, 40);
        assert_eq!(execution.summary.invalidation_impacted_count, 10);
    }

    #[test]
    fn fanout_bands_scale_model_counts_formula_fanout_edges() {
        let options = TreeCalcScaleOptions {
            node_count: 64,
            fanout: 7,
            ..TreeCalcScaleOptions::default_for(TreeCalcScaleProfile::FanoutBands, "fanout_smoke")
        };

        let execution = execute_scale_model(&options, "artifact/root").unwrap();

        assert!(execution.summary.validation_passed);
        assert_eq!(execution.summary.formula_count, 56);
        assert_eq!(execution.summary.descriptor_count, 392);
        assert_eq!(execution.summary.edge_count, 392);
        assert_eq!(execution.summary.invalidation_impacted_count, 57);
    }

    #[test]
    fn dynamic_indirect_stripes_surface_dynamic_potential_diagnostics() {
        let options = TreeCalcScaleOptions {
            rows: 3,
            cols: 4,
            selector_period: 2,
            ..TreeCalcScaleOptions::default_for(
                TreeCalcScaleProfile::DynamicIndirectStripes,
                "indirect_smoke",
            )
        };

        let execution = execute_scale_model(&options, "artifact/root").unwrap();

        assert!(execution.summary.validation_passed);
        assert_eq!(execution.summary.formula_count, 12);
        assert_eq!(execution.summary.descriptor_count, 36);
        assert_eq!(execution.summary.edge_count, 24);
        assert_eq!(execution.summary.diagnostic_count, 12);
        assert_eq!(execution.summary.invalidation_impacted_count, 8);
    }

    #[test]
    fn relative_rebind_churn_times_soft_reference_update() {
        let options = TreeCalcScaleOptions {
            node_count: 32,
            fanout: 3,
            ..TreeCalcScaleOptions::default_for(
                TreeCalcScaleProfile::RelativeRebindChurn,
                "relative_smoke",
            )
        };

        let execution = execute_scale_model(&options, "artifact/root").unwrap();

        assert!(execution.summary.validation_passed);
        assert_eq!(execution.summary.formula_count, 28);
        assert_eq!(execution.summary.descriptor_count, 84);
        assert_eq!(execution.summary.edge_count, 84);
        assert_eq!(
            execution.summary_json["validation"]["soft_reference_update"]["rebind_seed_count"],
            28
        );
    }
}
