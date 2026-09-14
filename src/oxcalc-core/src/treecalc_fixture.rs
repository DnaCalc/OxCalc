#![forbid(unsafe_code)]

//! Checked-in TreeCalc fixture loading for the local sequential runtime.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use oxfunc_core::value::{CalcValue, ExcelText};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::dependency::InvalidationReasonKind;
use crate::formula::{FixtureFormulaAst, TreeFormulaBinding, TreeFormulaCatalog};
use crate::structural::{
    BindArtifactId, FormulaArtifactId, StructuralEdit, StructuralEditOutcome, StructuralError,
    StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
};
use crate::treecalc::{
    LocalTreeCalcEngine, LocalTreeCalcEnvironmentContext, LocalTreeCalcError, LocalTreeCalcInput,
    LocalTreeCalcLayerSnapshotIds, LocalTreeCalcRunArtifacts,
    derive_structural_invalidation_seeds_for_catalogs,
};
use crate::workbook_settings::IterationSettings;
use crate::workspace_revision::{
    GridInputSnapshot, NamespaceSnapshot, NodeInputRecord, NodeInputSnapshot, WorkspaceRevision,
};

const TREECALC_FIXTURE_MANIFEST_SCHEMA_V1: &str = "oxcalc.treecalc.fixture_manifest.v1";
const TREECALC_FIXTURE_CASE_SCHEMA_V1: &str = "oxcalc.treecalc.fixture_case.v1";

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixtureManifest {
    pub schema_version: String,
    pub corpus_id: String,
    pub base_path: String,
    pub cases: Vec<TreeCalcFixtureManifestCase>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixtureManifestCase {
    pub case_id: String,
    pub path: String,
    #[serde(default)]
    pub tags: Vec<String>,
    /// Declared open gap: the case's `expected` block is a retained target
    /// (for the W048 cases, a real Excel observation) that the engine is known
    /// NOT to meet yet, and the named bead owns closing it. The runner and the
    /// fixture test report such a case as an open typed divergence — never as
    /// a pass, and never silently: a declared gap whose expectation the engine
    /// starts to meet is itself reported as a mismatch, so the marker cannot
    /// outlive the gap.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub open_gap: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixtureCase {
    pub schema_version: String,
    pub case_id: String,
    pub description: String,
    pub snapshot_id: u64,
    pub root_node_id: u64,
    pub nodes: Vec<TreeCalcFixtureNode>,
    #[serde(default)]
    pub formulas: Vec<TreeCalcFixtureFormulaBinding>,
    #[serde(default)]
    pub seeded_published_values: BTreeMap<u64, String>,
    #[serde(default)]
    pub input_values: BTreeMap<u64, String>,
    #[serde(default)]
    pub compatibility_basis: Option<String>,
    /// Workbook iterative-calculation settings the host requested for the
    /// case's runs. Absent means Excel's default (iteration disabled). The
    /// W048 Excel-observed iterative cases declare the exact profile Excel was
    /// observed under (`docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001`).
    #[serde(default)]
    pub iteration_settings: Option<IterationSettings>,
    #[serde(default)]
    pub post_edit: Option<TreeCalcFixturePostEditPlan>,
    pub expected: TreeCalcFixtureExpected,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixturePostEditPlan {
    pub successor_snapshot_start_id: u64,
    #[serde(default)]
    pub edits: Vec<TreeCalcFixtureStructuralEdit>,
    #[serde(default)]
    pub input_values: Option<BTreeMap<u64, String>>,
    #[serde(default)]
    pub formulas: Option<Vec<TreeCalcFixtureFormulaBinding>>,
    #[serde(default)]
    pub invalidation_seeds: Vec<TreeCalcFixtureInvalidationSeed>,
    #[serde(default)]
    pub expected_impacts: Vec<String>,
    #[serde(default)]
    pub expected_affected_node_ids: Option<Vec<Vec<u64>>>,
    pub expected: TreeCalcFixtureExpected,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixtureInvalidationSeed {
    pub node_id: u64,
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TreeCalcFixtureStructuralEdit {
    RenameNode {
        node_id: u64,
        new_symbol: String,
    },
    MoveNode {
        node_id: u64,
        new_parent_id: u64,
        new_index: Option<usize>,
    },
    RemoveNode {
        node_id: u64,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixtureNode {
    pub node_id: u64,
    pub kind: String,
    pub symbol: String,
    pub parent_id: Option<u64>,
    #[serde(default)]
    pub child_ids: Vec<u64>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixtureFormulaBinding {
    pub owner_node_id: u64,
    pub formula_artifact_id: String,
    pub bind_artifact_id: Option<String>,
    pub expression: FixtureFormulaAst,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixtureExpected {
    pub result_state: String,
    #[serde(default)]
    pub published_values: Option<BTreeMap<u64, String>>,
    #[serde(default)]
    pub evaluation_order: Option<Vec<u64>>,
    #[serde(default)]
    pub reject_kind: Option<String>,
    #[serde(default)]
    pub runtime_effect_kinds: Option<Vec<String>>,
    #[serde(default)]
    pub runtime_effect_overlay_kinds: Option<Vec<String>>,
}

#[derive(Debug, Clone)]
pub struct TreeCalcFixtureExecution {
    pub initial_artifacts: LocalTreeCalcRunArtifacts,
    pub post_edit: Option<TreeCalcFixturePostEditExecution>,
}

#[derive(Debug, Clone)]
pub struct TreeCalcFixturePostEditExecution {
    pub edit_outcomes: Vec<StructuralEditOutcome>,
    pub invalidation_seeds: Vec<crate::dependency::InvalidationSeed>,
    pub rerun_artifacts: LocalTreeCalcRunArtifacts,
}

#[derive(Debug, Error)]
pub enum TreeCalcFixtureError {
    #[error("failed to read {path}: {source}")]
    Read {
        path: String,
        source: std::io::Error,
    },
    #[error("failed to parse json from {path}: {source}")]
    Parse {
        path: String,
        source: serde_json::Error,
    },
    #[error("unsupported manifest schema version '{schema_version}'")]
    UnsupportedManifestSchema { schema_version: String },
    #[error("unsupported fixture case schema version '{schema_version}'")]
    UnsupportedCaseSchema { schema_version: String },
    #[error("structural error in fixture case '{case_id}': {source}")]
    Structural {
        case_id: String,
        #[source]
        source: StructuralError,
    },
    #[error("fixture case '{case_id}' failed: {source}")]
    Runtime {
        case_id: String,
        #[source]
        source: LocalTreeCalcError,
    },
    #[error("fixture case '{case_id}' uses unsupported invalidation reason '{reason}'")]
    UnsupportedInvalidationReason { case_id: String, reason: String },
}

pub fn load_manifest(path: &Path) -> Result<TreeCalcFixtureManifest, TreeCalcFixtureError> {
    let text = fs::read_to_string(path).map_err(|source| TreeCalcFixtureError::Read {
        path: path.display().to_string(),
        source,
    })?;
    let manifest = serde_json::from_str::<TreeCalcFixtureManifest>(&text).map_err(|source| {
        TreeCalcFixtureError::Parse {
            path: path.display().to_string(),
            source,
        }
    })?;
    if manifest.schema_version != TREECALC_FIXTURE_MANIFEST_SCHEMA_V1 {
        return Err(TreeCalcFixtureError::UnsupportedManifestSchema {
            schema_version: manifest.schema_version,
        });
    }
    Ok(manifest)
}

pub fn load_case(path: &Path) -> Result<TreeCalcFixtureCase, TreeCalcFixtureError> {
    let text = fs::read_to_string(path).map_err(|source| TreeCalcFixtureError::Read {
        path: path.display().to_string(),
        source,
    })?;
    let case = serde_json::from_str::<TreeCalcFixtureCase>(&text).map_err(|source| {
        TreeCalcFixtureError::Parse {
            path: path.display().to_string(),
            source,
        }
    })?;
    if case.schema_version != TREECALC_FIXTURE_CASE_SCHEMA_V1 {
        return Err(TreeCalcFixtureError::UnsupportedCaseSchema {
            schema_version: case.schema_version,
        });
    }
    Ok(case)
}

pub fn execute_fixture_case(
    engine: &LocalTreeCalcEngine,
    case: &TreeCalcFixtureCase,
) -> Result<TreeCalcFixtureExecution, TreeCalcFixtureError> {
    let structural_snapshot = case_structural_snapshot(case)?;
    let formula_catalog = TreeFormulaCatalog::new(case.formulas.iter().map(to_formula_binding));
    let input_values = to_input_values(&case.input_values);

    let initial_artifacts = engine
        .execute(case_initial_runtime_input(
            case,
            fixture_environment_context(case, case_runtime_policy_id(case)),
        )?)
        .map_err(|source| TreeCalcFixtureError::Runtime {
            case_id: case.case_id.clone(),
            source,
        })?;

    let post_edit = case
        .post_edit
        .as_ref()
        .map(|plan| {
            execute_post_edit_plan(
                engine,
                case,
                plan,
                &structural_snapshot,
                &formula_catalog,
                &input_values,
                &initial_artifacts,
            )
        })
        .transpose()?;

    Ok(TreeCalcFixtureExecution {
        initial_artifacts,
        post_edit,
    })
}

fn execute_post_edit_plan(
    engine: &LocalTreeCalcEngine,
    case: &TreeCalcFixtureCase,
    plan: &TreeCalcFixturePostEditPlan,
    structural_snapshot: &StructuralSnapshot,
    formula_catalog: &TreeFormulaCatalog,
    initial_input_values: &BTreeMap<TreeNodeId, String>,
    initial_artifacts: &LocalTreeCalcRunArtifacts,
) -> Result<TreeCalcFixturePostEditExecution, TreeCalcFixtureError> {
    let mut current_snapshot = structural_snapshot.clone();
    let mut edit_outcomes = Vec::new();

    for (index, edit) in plan.edits.iter().enumerate() {
        let successor_snapshot_id =
            StructuralSnapshotId(plan.successor_snapshot_start_id + u64::try_from(index).unwrap());
        let outcome = current_snapshot
            .apply_edit(successor_snapshot_id, to_structural_edit(edit))
            .map_err(|source| TreeCalcFixtureError::Structural {
                case_id: case.case_id.clone(),
                source,
            })?;
        current_snapshot = outcome.snapshot.clone();
        edit_outcomes.push(outcome);
    }

    let seeded_published_values = initial_artifacts
        .published_calc_values
        .iter()
        .filter(|(node_id, _)| current_snapshot.try_get_node(**node_id).is_some())
        .map(|(node_id, value)| (*node_id, value.clone()))
        .collect::<BTreeMap<_, _>>();

    let rerun_snapshot = edit_outcomes
        .last()
        .map(|outcome| outcome.snapshot.clone())
        .unwrap_or_else(|| structural_snapshot.clone());
    let rerun_formula_catalog = plan
        .formulas
        .as_ref()
        .map(|formulas| TreeFormulaCatalog::new(formulas.iter().map(to_formula_binding)))
        .unwrap_or_else(|| formula_catalog.clone());
    let rerun_input_values = plan
        .input_values
        .as_ref()
        .map(to_input_values)
        .unwrap_or_else(|| initial_input_values.clone());

    let invalidation_seeds = if !plan.invalidation_seeds.is_empty() {
        plan.invalidation_seeds
            .iter()
            .map(|seed| to_invalidation_seed(case, seed))
            .collect::<Result<Vec<_>, _>>()?
    } else {
        let mut seeds = derive_structural_invalidation_seeds_for_catalogs(
            structural_snapshot,
            &rerun_snapshot,
            formula_catalog,
            &rerun_formula_catalog,
            &edit_outcomes,
        );
        seeds.extend(derive_input_value_invalidation_seeds(
            initial_input_values,
            &rerun_input_values,
        ));
        dedupe_fixture_invalidation_seeds(seeds)
    };

    let seeded_published_runtime_effects = initial_artifacts
        .publication_bundle
        .as_ref()
        .map(|bundle| bundle.published_runtime_effects.clone())
        .unwrap_or_default();

    let rerun_artifacts = engine
        .execute(fixture_runtime_input(
            &format!("fixture:{}:post_edit", case.case_id),
            rerun_snapshot.clone(),
            rerun_formula_catalog,
            rerun_input_values,
            seeded_published_values,
            seeded_published_runtime_effects,
            invalidation_seeds.clone(),
            format!("fixture:{}:candidate:post_edit", case.case_id),
            format!("fixture:{}:publication:post_edit", case.case_id),
            fixture_environment_context(
                case,
                format!(
                    "fixture-runtime-policy:{}",
                    plan.successor_snapshot_start_id
                ),
            ),
        ))
        .map_err(|source| TreeCalcFixtureError::Runtime {
            case_id: case.case_id.clone(),
            source,
        })?;

    Ok(TreeCalcFixturePostEditExecution {
        edit_outcomes,
        invalidation_seeds,
        rerun_artifacts,
    })
}

fn case_structural_snapshot(
    case: &TreeCalcFixtureCase,
) -> Result<StructuralSnapshot, TreeCalcFixtureError> {
    StructuralSnapshot::create(
        StructuralSnapshotId(case.snapshot_id),
        TreeNodeId(case.root_node_id),
        case.nodes.iter().map(to_structural_node),
    )
    .map_err(|source| TreeCalcFixtureError::Structural {
        case_id: case.case_id.clone(),
        source,
    })
}

/// The runtime policy id the case's initial run declares (its
/// `compatibility_basis`, or a snapshot-derived default).
fn case_runtime_policy_id(case: &TreeCalcFixtureCase) -> String {
    case.compatibility_basis
        .clone()
        .unwrap_or_else(|| format!("fixture-runtime-policy:{}", case.snapshot_id))
}

/// The engine input for a case's initial run under `environment_context`:
/// the case's structure, formulas, literal inputs, and seeded prior
/// publication, with no invalidation seeds (a full structural recalc).
pub(crate) fn case_initial_runtime_input(
    case: &TreeCalcFixtureCase,
    environment_context: LocalTreeCalcEnvironmentContext,
) -> Result<LocalTreeCalcInput, TreeCalcFixtureError> {
    let structural_snapshot = case_structural_snapshot(case)?;
    let formula_catalog = TreeFormulaCatalog::new(case.formulas.iter().map(to_formula_binding));
    let input_values = to_input_values(&case.input_values);
    Ok(fixture_runtime_input(
        &format!("fixture:{}", case.case_id),
        structural_snapshot,
        formula_catalog,
        input_values,
        case_seeded_published_values(case),
        Vec::new(),
        Vec::new(),
        format!("fixture:{}:candidate", case.case_id),
        format!("fixture:{}:publication", case.case_id),
        environment_context,
    ))
}

/// The case's seeded prior publication as calc values.
pub(crate) fn case_seeded_published_values(
    case: &TreeCalcFixtureCase,
) -> BTreeMap<TreeNodeId, CalcValue> {
    case.seeded_published_values
        .iter()
        .map(|(node_id, value)| (TreeNodeId(*node_id), fixture_string_to_calc_value(value)))
        .collect()
}

/// The environment context a fixture case runs under: the case's runtime
/// policy id plus its declared workbook iteration settings (Excel default when
/// absent).
fn fixture_environment_context(
    case: &TreeCalcFixtureCase,
    runtime_policy_id: String,
) -> LocalTreeCalcEnvironmentContext {
    LocalTreeCalcEnvironmentContext::default()
        .with_runtime_policy_id(runtime_policy_id)
        .with_iteration_settings(case.iteration_settings.unwrap_or_default())
}

fn to_structural_node(node: &TreeCalcFixtureNode) -> StructuralNode {
    StructuralNode {
        node_id: TreeNodeId(node.node_id),
        kind: parse_node_kind(&node.kind),
        symbol: node.symbol.clone(),
        parent_id: node.parent_id.map(TreeNodeId),
        child_ids: node.child_ids.iter().copied().map(TreeNodeId).collect(),
        role: None,
        is_meta: false,
    }
}

fn to_input_values(input_values: &BTreeMap<u64, String>) -> BTreeMap<TreeNodeId, String> {
    input_values
        .iter()
        .map(|(node_id, value)| (TreeNodeId(*node_id), value.clone()))
        .collect()
}

fn fixture_string_to_calc_value(value: &str) -> CalcValue {
    if let Ok(number) = value.parse::<f64>() {
        CalcValue::number(number)
    } else if let Ok(logical) = value.parse::<bool>() {
        CalcValue::logical(logical)
    } else {
        CalcValue::text(ExcelText::from_interop_assignment(value))
    }
}

fn fixture_workspace_revision(
    workspace_id: &str,
    structural_snapshot: StructuralSnapshot,
    input_values: &BTreeMap<TreeNodeId, String>,
) -> WorkspaceRevision {
    let records = structural_snapshot
        .nodes()
        .keys()
        .map(|node_id| {
            input_values.get(node_id).map_or_else(
                || NodeInputRecord::empty(*node_id, 1),
                |value| NodeInputRecord::literal(*node_id, value.clone(), 1),
            )
        })
        .collect::<Vec<_>>();
    WorkspaceRevision::new(
        workspace_id,
        structural_snapshot,
        NodeInputSnapshot::create(records).expect("fixture node-input records are unique"),
        GridInputSnapshot::default(),
        NamespaceSnapshot::current_absent(),
    )
}

#[allow(clippy::too_many_arguments)]
fn fixture_runtime_input(
    workspace_id: &str,
    structural_snapshot: StructuralSnapshot,
    formula_catalog: TreeFormulaCatalog,
    input_values: BTreeMap<TreeNodeId, String>,
    publication_values: BTreeMap<TreeNodeId, CalcValue>,
    publication_runtime_effects: Vec<crate::coordinator::RuntimeEffect>,
    invalidation_seeds: Vec<crate::dependency::InvalidationSeed>,
    candidate_result_id: String,
    publication_id: String,
    environment_context: LocalTreeCalcEnvironmentContext,
) -> LocalTreeCalcInput {
    let workspace_revision =
        fixture_workspace_revision(workspace_id, structural_snapshot, &input_values);
    LocalTreeCalcInput {
        layer_snapshot_ids: LocalTreeCalcLayerSnapshotIds::current_absent_for_revision(
            &workspace_revision,
        ),
        workspace_revision,
        formula_catalog,
        formula_dependency_descriptors: None,
        table_snapshots: BTreeMap::new(),
        static_dependency_shape_updates: Vec::new(),
        publication_calc_values: publication_values,
        publication_runtime_effects,
        invalidation_seeds,
        previous_arg_preparation_profile_version: None,
        candidate_result_id,
        publication_id,
        environment_context,
    }
}

fn derive_input_value_invalidation_seeds(
    predecessor: &BTreeMap<TreeNodeId, String>,
    successor: &BTreeMap<TreeNodeId, String>,
) -> Vec<crate::dependency::InvalidationSeed> {
    predecessor
        .keys()
        .chain(successor.keys())
        .copied()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .filter(|node_id| predecessor.get(node_id) != successor.get(node_id))
        .map(|node_id| crate::dependency::InvalidationSeed {
            node_id,
            reason: InvalidationReasonKind::UpstreamPublication,
        })
        .collect()
}

fn dedupe_fixture_invalidation_seeds(
    seeds: Vec<crate::dependency::InvalidationSeed>,
) -> Vec<crate::dependency::InvalidationSeed> {
    let mut seen = std::collections::BTreeSet::new();
    seeds
        .into_iter()
        .filter(|seed| seen.insert((seed.node_id, format!("{:?}", seed.reason))))
        .collect()
}

fn parse_node_kind(kind: &str) -> StructuralNodeKind {
    match kind {
        "root" => StructuralNodeKind::Root,
        "container" => StructuralNodeKind::Container,
        "constant" => StructuralNodeKind::Constant,
        _ => StructuralNodeKind::Calculation,
    }
}

fn to_formula_binding(binding: &TreeCalcFixtureFormulaBinding) -> TreeFormulaBinding {
    TreeFormulaBinding {
        owner_node_id: TreeNodeId(binding.owner_node_id),
        formula_artifact_id: FormulaArtifactId(binding.formula_artifact_id.clone()),
        bind_artifact_id: binding
            .bind_artifact_id
            .as_ref()
            .map(|id| BindArtifactId(id.clone())),
        expression: binding
            .expression
            .to_tree_formula(TreeNodeId(binding.owner_node_id)),
    }
}

fn to_structural_edit(edit: &TreeCalcFixtureStructuralEdit) -> StructuralEdit {
    match edit {
        TreeCalcFixtureStructuralEdit::RenameNode {
            node_id,
            new_symbol,
        } => StructuralEdit::RenameNode {
            node_id: TreeNodeId(*node_id),
            new_symbol: new_symbol.clone(),
        },
        TreeCalcFixtureStructuralEdit::MoveNode {
            node_id,
            new_parent_id,
            new_index,
        } => StructuralEdit::MoveNode {
            node_id: TreeNodeId(*node_id),
            new_parent_id: TreeNodeId(*new_parent_id),
            new_index: *new_index,
        },
        TreeCalcFixtureStructuralEdit::RemoveNode { node_id } => StructuralEdit::RemoveNode {
            node_id: TreeNodeId(*node_id),
        },
    }
}

fn to_invalidation_seed(
    case: &TreeCalcFixtureCase,
    seed: &TreeCalcFixtureInvalidationSeed,
) -> Result<crate::dependency::InvalidationSeed, TreeCalcFixtureError> {
    Ok(crate::dependency::InvalidationSeed {
        node_id: TreeNodeId(seed.node_id),
        reason: parse_invalidation_reason(case, &seed.reason)?,
    })
}

fn parse_invalidation_reason(
    case: &TreeCalcFixtureCase,
    reason: &str,
) -> Result<InvalidationReasonKind, TreeCalcFixtureError> {
    let parsed = match reason {
        "StructuralRebindRequired" | "structural_rebind_required" => {
            InvalidationReasonKind::StructuralRebindRequired
        }
        "StructuralRecalcOnly" | "structural_recalc_only" => {
            InvalidationReasonKind::StructuralRecalcOnly
        }
        "UpstreamPublication" | "upstream_publication" => {
            InvalidationReasonKind::UpstreamPublication
        }
        "ExternallyInvalidated" | "externally_invalidated" => {
            InvalidationReasonKind::ExternallyInvalidated
        }
        "TreeReferenceMembershipChanged" | "tree_reference_membership_changed" => {
            InvalidationReasonKind::TreeReferenceMembershipChanged
        }
        "TreeReferenceOrderChanged" | "tree_reference_order_changed" => {
            InvalidationReasonKind::TreeReferenceOrderChanged
        }
        "StructuredTableContextChanged" | "structured_table_context_changed" => {
            InvalidationReasonKind::StructuredTableContextChanged
        }
        "StructuredTableRowMembershipChanged" | "structured_table_row_membership_changed" => {
            InvalidationReasonKind::StructuredTableRowMembershipChanged
        }
        "StructuredTableRowOrderChanged" | "structured_table_row_order_changed" => {
            InvalidationReasonKind::StructuredTableRowOrderChanged
        }
        "StructuredTableColumnChanged" | "structured_table_column_changed" => {
            InvalidationReasonKind::StructuredTableColumnChanged
        }
        "StructuredTableRegionChanged" | "structured_table_region_changed" => {
            InvalidationReasonKind::StructuredTableRegionChanged
        }
        "StructuredTableCallerContextChanged" | "structured_table_caller_context_changed" => {
            InvalidationReasonKind::StructuredTableCallerContextChanged
        }
        "DependencyAdded" | "dependency_added" => InvalidationReasonKind::DependencyAdded,
        "DependencyRemoved" | "dependency_removed" => InvalidationReasonKind::DependencyRemoved,
        "DependencyReclassified" | "dependency_reclassified" => {
            InvalidationReasonKind::DependencyReclassified
        }
        "DynamicDependencyActivated" | "dynamic_dependency_activated" => {
            InvalidationReasonKind::DynamicDependencyActivated
        }
        "DynamicDependencyReleased" | "dynamic_dependency_released" => {
            InvalidationReasonKind::DynamicDependencyReleased
        }
        "DynamicDependencyReclassified" | "dynamic_dependency_reclassified" => {
            InvalidationReasonKind::DynamicDependencyReclassified
        }
        _ => {
            return Err(TreeCalcFixtureError::UnsupportedInvalidationReason {
                case_id: case.case_id.clone(),
                reason: reason.to_string(),
            });
        }
    };
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::formula::FixtureFormulaPolicyClass;
    use crate::treecalc::LocalTreeCalcRunState;

    use super::*;

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap()
    }

    fn treecalc_fixture_path(repo_root: &Path, entry: &TreeCalcFixtureManifestCase) -> PathBuf {
        repo_root
            .join("docs/test-fixtures/core-engine/treecalc")
            .join(entry.path.replace('/', "\\"))
    }

    #[test]
    fn treecalc_fixture_policy_tags_match_representative_cases() {
        let repo_root = repo_root();
        let manifest_path = repo_root.join("docs/test-fixtures/core-engine/treecalc/MANIFEST.json");
        let manifest = load_manifest(&manifest_path).unwrap();

        let cases = [
            (
                "tc_local_lambda_host_sensitive_reject_001",
                "fixture-policy:opaque-oxfml-source",
                FixtureFormulaPolicyClass::OpaqueOxfmlSource,
            ),
            (
                "tc_local_w034_higher_order_let_lambda_publish_001",
                "fixture-policy:opaque-oxfml-source",
                FixtureFormulaPolicyClass::OpaqueOxfmlSource,
            ),
            (
                "tc_w048_excel_iter_two_node_order_001",
                "fixture-policy:legacy-structured-quarantine",
                FixtureFormulaPolicyClass::LegacyStructuredQuarantine,
            ),
        ];

        for (case_id, expected_tag, expected_policy) in cases {
            let entry = manifest
                .cases
                .iter()
                .find(|entry| entry.case_id == case_id)
                .unwrap_or_else(|| panic!("missing fixture manifest entry {case_id}"));
            assert!(
                entry.tags.iter().any(|tag| tag == expected_tag),
                "fixture {case_id} must carry {expected_tag}"
            );

            let case = load_case(&treecalc_fixture_path(&repo_root, entry)).unwrap();
            assert!(
                case.formulas
                    .iter()
                    .any(|binding| binding.expression.policy_class() == expected_policy),
                "fixture {case_id} must exercise {expected_policy:?}"
            );
        }
    }

    #[test]
    fn checked_in_treecalc_fixtures_execute_against_local_runtime() {
        let repo_root = repo_root();
        let manifest_path = repo_root.join("docs/test-fixtures/core-engine/treecalc/MANIFEST.json");
        let manifest = load_manifest(&manifest_path).unwrap();
        let engine = LocalTreeCalcEngine;

        assert_eq!(manifest.cases.len(), 37);

        let mut open_gap_case_ids = Vec::new();
        for entry in &manifest.cases {
            let case = load_case(&treecalc_fixture_path(&repo_root, entry)).unwrap();
            let execution = execute_fixture_case(&engine, &case).unwrap();
            let artifacts = &execution.initial_artifacts;

            if let Some(bead) = entry.open_gap.as_deref() {
                // A declared open gap: the retained expectation is a target the
                // engine does not meet yet. The suite is green only by declaring
                // the gap — the divergence must exist, and it must be the typed
                // engine answer the bead names, not any divergence.
                let mismatches =
                    crate::treecalc_runner::compare_expected(&case.expected, artifacts);
                assert!(
                    !mismatches.is_empty(),
                    "fixture {}: manifest declares open gap {bead} but the engine now meets the retained expectation; remove the open_gap marker",
                    case.case_id
                );
                assert_open_gap_answer_is_typed(bead, &case, artifacts);
                assert!(
                    case.post_edit.is_none(),
                    "fixture {}: an open_gap declaration covers the initial run only; a post-edit plan would be skipped silently",
                    case.case_id
                );
                open_gap_case_ids.push(case.case_id.clone());
                continue;
            }

            assert_eq!(
                match artifacts.result_state {
                    LocalTreeCalcRunState::Published => "published",
                    LocalTreeCalcRunState::VerifiedClean => "verified_clean",
                    LocalTreeCalcRunState::Rejected => "rejected",
                },
                case.expected.result_state
            );

            if let Some(expected_values) = &case.expected.published_values {
                let actual_values = artifacts
                    .published_values
                    .iter()
                    .map(|(node_id, value)| (node_id.0, value.clone()))
                    .collect::<BTreeMap<_, _>>();
                for (node_id, expected_value) in expected_values {
                    let actual_value = actual_values
                        .get(node_id)
                        .unwrap_or_else(|| panic!("missing published value for node {node_id}"));
                    assert!(
                        fixture_value_text_matches(actual_value, expected_value),
                        "published value mismatch for node {node_id}: expected {expected_value}, observed {actual_value}"
                    );
                }
            }

            if let Some(expected_order) = &case.expected.evaluation_order {
                assert_eq!(
                    artifacts
                        .evaluation_order
                        .iter()
                        .map(|node_id| node_id.0)
                        .collect::<Vec<_>>(),
                    *expected_order
                );
            }

            if let Some(expected_reject_kind) = &case.expected.reject_kind {
                let observed_reject_kind = artifacts
                    .reject_detail
                    .as_ref()
                    .map(|detail| format!("{:?}", detail.kind));
                assert_eq!(observed_reject_kind.as_ref(), Some(expected_reject_kind));
            }

            if let Some(expected_runtime_effect_kinds) = &case.expected.runtime_effect_kinds {
                let observed_runtime_effect_kinds = artifacts
                    .runtime_effects
                    .iter()
                    .map(|runtime_effect| runtime_effect.kind.clone())
                    .collect::<Vec<_>>();
                assert_eq!(
                    observed_runtime_effect_kinds,
                    *expected_runtime_effect_kinds
                );
            }

            if let Some(post_edit) = &case.post_edit {
                let post_edit_execution = execution.post_edit.as_ref().unwrap();
                let observed_impacts = post_edit_execution
                    .edit_outcomes
                    .iter()
                    .map(|outcome| format!("{:?}", outcome.impact))
                    .collect::<Vec<_>>();
                assert_eq!(observed_impacts, post_edit.expected_impacts);

                if let Some(expected_affected_node_ids) = &post_edit.expected_affected_node_ids {
                    let observed_affected_node_ids = post_edit_execution
                        .edit_outcomes
                        .iter()
                        .map(|outcome| {
                            outcome
                                .affected_node_ids
                                .iter()
                                .map(|node_id| node_id.0)
                                .collect::<Vec<_>>()
                        })
                        .collect::<Vec<_>>();
                    assert_eq!(observed_affected_node_ids, *expected_affected_node_ids);
                }

                assert_artifacts_match_expected(
                    &post_edit_execution.rerun_artifacts,
                    &post_edit.expected,
                    &format!("{}:post_edit", case.case_id),
                );
            }
        }

        assert_eq!(
            open_gap_case_ids,
            W048_EXCEL_ITERATIVE_CASE_IDS
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            "the only declared open gaps are the four W048 Excel-observed iterative cases (calc-9ouy.3)"
        );
    }

    /// The four W048 TreeCalc cases whose `expected` block is a real Excel
    /// observation (`docs/test-runs/excel-cycles/w048-excel-cycles-bitexact-001`)
    /// and whose probe id used to key a fixture-answered result table in the
    /// engine (withdrawn by calc-9ouy.11; the real engine is calc-9ouy.3).
    const W048_EXCEL_ITERATIVE_CASE_IDS: [&str; 4] = [
        "tc_w048_excel_iter_two_node_order_001",
        "tc_w048_excel_iter_three_node_order_001",
        "tc_w048_excel_iter_fraction_precision_001",
        "tc_w048_excel_ctro_indirect_iterative_self_001",
    ];

    /// The W048 Excel probe ids that used to select a hardcoded answer.
    const W048_EXCEL_ITERATIVE_PROBE_IDS: [&str; 4] = [
        "excel_iter_two_node_order_001",
        "excel_iter_three_node_order_001",
        "excel_iter_fraction_precision_001",
        "excel_ctro_indirect_iterative_self_001",
    ];

    /// Every declared open gap must have its typed engine answer coded here,
    /// so an `open_gap` marker can never act as a generic suppressor: the
    /// divergence has to be the one the owning bead names.
    fn assert_open_gap_answer_is_typed(
        bead: &str,
        case: &TreeCalcFixtureCase,
        artifacts: &LocalTreeCalcRunArtifacts,
    ) {
        match bead {
            "calc-9ouy.3" => {
                let requested = case.iteration_settings.unwrap_or_else(|| {
                    panic!(
                        "fixture {}: open gap calc-9ouy.3 cases declare the iteration_settings Excel was observed under",
                        case.case_id
                    )
                });
                assert!(requested.enabled, "fixture {}", case.case_id);
                assert_iterative_cycle_typed_not_implemented(
                    &case.case_id,
                    case,
                    requested,
                    artifacts,
                );
            }
            other => panic!(
                "fixture {}: open gap {other} has no typed engine answer declared in this test; add one before declaring the gap",
                case.case_id
            ),
        }
    }

    /// The engine's honest answer to an iterative cycle it cannot compute:
    /// a rejected run whose typed cause is `IterativeCalcNotImplemented`
    /// carrying the requested settings and the cycle members; no candidate,
    /// no publication, no fabricated value — the seeded prior publication is
    /// what remains visible — and no iteration trace.
    fn assert_iterative_cycle_typed_not_implemented(
        context: &str,
        case: &TreeCalcFixtureCase,
        requested: IterationSettings,
        artifacts: &LocalTreeCalcRunArtifacts,
    ) {
        assert_eq!(
            artifacts.result_state,
            LocalTreeCalcRunState::Rejected,
            "{context}"
        );
        let Some(LocalTreeCalcError::IterativeCalcNotImplemented {
            cycle_groups,
            iteration_settings,
        }) = &artifacts.reject_error
        else {
            panic!(
                "{context}: expected IterativeCalcNotImplemented, observed {:?}",
                artifacts.reject_error
            );
        };
        assert_eq!(*iteration_settings, requested, "{context}");
        let cycle_members = cycle_groups
            .iter()
            .flatten()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let formula_owners = case
            .formulas
            .iter()
            .map(|binding| TreeNodeId(binding.owner_node_id))
            .collect::<std::collections::BTreeSet<_>>();
        assert!(!cycle_members.is_empty(), "{context}");
        assert!(
            cycle_members.is_subset(&formula_owners),
            "{context}: cycle members {cycle_members:?} must be formula owners {formula_owners:?}"
        );
        assert_eq!(
            artifacts.reject_detail.as_ref().map(|detail| detail.kind),
            Some(crate::coordinator::RejectKind::SyntheticCycleReject),
            "{context}"
        );
        assert!(artifacts.candidate_result.is_none(), "{context}");
        assert!(artifacts.publication_bundle.is_none(), "{context}");
        assert!(artifacts.evaluation_order.is_empty(), "{context}");
        assert_eq!(
            artifacts.published_calc_values,
            case_seeded_published_values(case),
            "{context}: only the seeded prior publication may remain visible"
        );
        for diagnostic in &artifacts.diagnostics {
            assert!(
                !diagnostic.contains("cycle_iteration_trace")
                    && !diagnostic.contains("cycle.excel_match_iterative"),
                "{context}: no iteration trace may be emitted for an unimplemented iteration: {diagnostic}"
            );
        }
    }

    /// Two hand-built cyclic inputs that never had a probe id: a self-cycle
    /// and a two-node cycle with an acyclic downstream dependent.
    fn hand_built_cyclic_cases() -> Vec<TreeCalcFixtureCase> {
        let self_cycle = r#"{
          "schema_version": "oxcalc.treecalc.fixture_case.v1",
          "case_id": "hand_built_self_cycle_plus_one",
          "description": "A = A + 1 seeded from a prior publication of 4",
          "snapshot_id": 9001,
          "root_node_id": 1,
          "nodes": [
            { "node_id": 1, "kind": "root", "symbol": "Root", "parent_id": null, "child_ids": [2] },
            { "node_id": 2, "kind": "calculation", "symbol": "A", "parent_id": 1, "child_ids": [] }
          ],
          "formulas": [
            { "owner_node_id": 2, "formula_artifact_id": "formula:hand:A", "bind_artifact_id": null,
              "expression": { "Binary": { "op": "Add",
                "left": { "Reference": { "DirectNode": { "target_node_id": 2 } } },
                "right": { "Literal": { "value": "1" } } } } }
          ],
          "seeded_published_values": { "2": "4" },
          "expected": { "result_state": "rejected" }
        }"#;
        let two_node_with_downstream = r#"{
          "schema_version": "oxcalc.treecalc.fixture_case.v1",
          "case_id": "hand_built_two_node_cycle_with_downstream",
          "description": "A = B + 1, B = A * 2, C = A + B (C depends on the cycle but is not in it)",
          "snapshot_id": 9002,
          "root_node_id": 1,
          "nodes": [
            { "node_id": 1, "kind": "root", "symbol": "Root", "parent_id": null, "child_ids": [2, 3, 4] },
            { "node_id": 2, "kind": "calculation", "symbol": "A", "parent_id": 1, "child_ids": [] },
            { "node_id": 3, "kind": "calculation", "symbol": "B", "parent_id": 1, "child_ids": [] },
            { "node_id": 4, "kind": "calculation", "symbol": "C", "parent_id": 1, "child_ids": [] }
          ],
          "formulas": [
            { "owner_node_id": 2, "formula_artifact_id": "formula:hand:A", "bind_artifact_id": null,
              "expression": { "Binary": { "op": "Add",
                "left": { "Reference": { "DirectNode": { "target_node_id": 3 } } },
                "right": { "Literal": { "value": "1" } } } } },
            { "owner_node_id": 3, "formula_artifact_id": "formula:hand:B", "bind_artifact_id": null,
              "expression": { "Binary": { "op": "Multiply",
                "left": { "Reference": { "DirectNode": { "target_node_id": 2 } } },
                "right": { "Literal": { "value": "2" } } } } },
            { "owner_node_id": 4, "formula_artifact_id": "formula:hand:C", "bind_artifact_id": null,
              "expression": { "Binary": { "op": "Add",
                "left": { "Reference": { "DirectNode": { "target_node_id": 2 } } },
                "right": { "Reference": { "DirectNode": { "target_node_id": 3 } } } } } }
          ],
          "seeded_published_values": { "2": "7", "3": "3", "4": "10" },
          "expected": { "result_state": "rejected" }
        }"#;
        [self_cycle, two_node_with_downstream]
            .iter()
            .map(|text| serde_json::from_str::<TreeCalcFixtureCase>(text).unwrap())
            .collect()
    }

    /// Tripwire for the withdrawn fixture-answered iterative surface
    /// (calc-9ouy.11): for every iterative cycle — the four W048 Excel-observed
    /// cases and two fresh hand-built cycles — under every runtime policy id
    /// that used to select a hardcoded answer (each W048 probe id, the legacy
    /// `cycle.excel_match_iterative` token) and under a spread of enabled
    /// iteration profiles, the tree lane answers `IterativeCalcNotImplemented`
    /// and publishes nothing. With iteration disabled the same inputs answer
    /// `CycleDetected`, so the two are distinct. Re-introducing any lookup
    /// keyed on a probe/policy id string that publishes values fails this test.
    #[test]
    fn iterative_cycles_are_typed_not_implemented_never_fixture_answered() {
        let repo_root = repo_root();
        let manifest_path = repo_root.join("docs/test-fixtures/core-engine/treecalc/MANIFEST.json");
        let manifest = load_manifest(&manifest_path).unwrap();
        let engine = LocalTreeCalcEngine;

        let mut subjects = W048_EXCEL_ITERATIVE_CASE_IDS
            .iter()
            .map(|case_id| {
                let entry = manifest
                    .cases
                    .iter()
                    .find(|entry| entry.case_id == *case_id)
                    .unwrap_or_else(|| panic!("missing fixture manifest entry {case_id}"));
                load_case(&treecalc_fixture_path(&repo_root, entry)).unwrap()
            })
            .collect::<Vec<_>>();
        subjects.extend(hand_built_cyclic_cases());
        assert_eq!(subjects.len(), 6);

        let policy_ids = W048_EXCEL_ITERATIVE_PROBE_IDS
            .iter()
            .map(|probe_id| {
                format!(
                    "cycle.excel_match_iterative:excel:w048-excel-cycles-bitexact-001:{probe_id}"
                )
            })
            .chain([
                "cycle.excel_match_iterative".to_string(),
                "runtime-policy:default".to_string(),
            ])
            .collect::<Vec<_>>();
        let enabled_profiles = [
            (1, 0.001),
            (2, 0.001),
            (5, 0.001),
            (20, 0.000_001),
            (100, 0.001),
        ]
        .map(|(max_iterations, max_change)| IterationSettings {
            enabled: true,
            max_iterations,
            max_change,
        });

        let mut enabled_runs = 0usize;
        let mut disabled_runs = 0usize;
        for case in &subjects {
            for policy_id in &policy_ids {
                for requested in enabled_profiles {
                    let context = format!(
                        "{} under policy '{policy_id}' with {requested:?}",
                        case.case_id
                    );
                    let artifacts = engine
                        .execute(
                            case_initial_runtime_input(
                                case,
                                LocalTreeCalcEnvironmentContext::default()
                                    .with_runtime_policy_id(policy_id.clone())
                                    .with_iteration_settings(requested),
                            )
                            .unwrap(),
                        )
                        .unwrap_or_else(|error| panic!("{context}: {error}"));
                    assert_iterative_cycle_typed_not_implemented(
                        &context, case, requested, &artifacts,
                    );
                    enabled_runs += 1;
                }

                // Iteration off: the same cycle is the ordinary, distinct
                // `CycleDetected` rejection, still publishing nothing.
                let context = format!(
                    "{} under policy '{policy_id}' with iteration off",
                    case.case_id
                );
                let artifacts = engine
                    .execute(
                        case_initial_runtime_input(
                            case,
                            LocalTreeCalcEnvironmentContext::default()
                                .with_runtime_policy_id(policy_id.clone()),
                        )
                        .unwrap(),
                    )
                    .unwrap_or_else(|error| panic!("{context}: {error}"));
                assert_eq!(
                    artifacts.result_state,
                    LocalTreeCalcRunState::Rejected,
                    "{context}"
                );
                assert_eq!(
                    artifacts.reject_error,
                    Some(LocalTreeCalcError::CycleDetected),
                    "{context}"
                );
                assert!(artifacts.publication_bundle.is_none(), "{context}");
                assert_eq!(
                    artifacts.published_calc_values,
                    case_seeded_published_values(case),
                    "{context}"
                );
                disabled_runs += 1;
            }
        }
        assert_eq!(enabled_runs, 6 * 6 * 5);
        assert_eq!(disabled_runs, 6 * 6);
    }

    fn assert_artifacts_match_expected(
        artifacts: &LocalTreeCalcRunArtifacts,
        expected: &TreeCalcFixtureExpected,
        context: &str,
    ) {
        assert_eq!(
            match artifacts.result_state {
                LocalTreeCalcRunState::Published => "published",
                LocalTreeCalcRunState::VerifiedClean => "verified_clean",
                LocalTreeCalcRunState::Rejected => "rejected",
            },
            expected.result_state,
            "fixture context: {context}"
        );

        if let Some(expected_values) = &expected.published_values {
            let actual_values = artifacts
                .published_values
                .iter()
                .map(|(node_id, value)| (node_id.0, value.clone()))
                .collect::<BTreeMap<_, _>>();
            for (node_id, expected_value) in expected_values {
                let actual_value = actual_values
                    .get(node_id)
                    .unwrap_or_else(|| panic!("missing published value for node {node_id}"));
                assert!(
                    fixture_value_text_matches(actual_value, expected_value),
                    "fixture context: {context}; published value mismatch for node {node_id}: expected {expected_value}, observed {actual_value}"
                );
            }
        }

        if let Some(expected_order) = &expected.evaluation_order {
            assert_eq!(
                artifacts
                    .evaluation_order
                    .iter()
                    .map(|node_id| node_id.0)
                    .collect::<Vec<_>>(),
                *expected_order,
                "fixture context: {context}"
            );
        }

        if let Some(expected_reject_kind) = &expected.reject_kind {
            let observed_reject_kind = artifacts
                .reject_detail
                .as_ref()
                .map(|detail| format!("{:?}", detail.kind));
            assert_eq!(
                observed_reject_kind.as_ref(),
                Some(expected_reject_kind),
                "fixture context: {context}"
            );
        }

        if let Some(expected_runtime_effect_kinds) = &expected.runtime_effect_kinds {
            let observed_runtime_effect_kinds = artifacts
                .runtime_effects
                .iter()
                .map(|runtime_effect| runtime_effect.kind.clone())
                .collect::<Vec<_>>();
            assert_eq!(
                observed_runtime_effect_kinds, *expected_runtime_effect_kinds,
                "fixture context: {context}"
            );
        }

        if let Some(expected_runtime_effect_overlay_kinds) = &expected.runtime_effect_overlay_kinds
        {
            let observed_runtime_effect_overlay_kinds = artifacts
                .runtime_effect_overlays
                .iter()
                .map(|overlay| format!("{:?}", overlay.key.overlay_kind))
                .collect::<Vec<_>>();
            assert_eq!(
                observed_runtime_effect_overlay_kinds, *expected_runtime_effect_overlay_kinds,
                "fixture context: {context}"
            );
        }
    }

    fn fixture_value_text_matches(actual: &str, expected: &str) -> bool {
        actual == expected
            || actual
                .parse::<f64>()
                .ok()
                .zip(expected.parse::<f64>().ok())
                .is_some_and(|(actual, expected)| actual == expected)
    }
}
