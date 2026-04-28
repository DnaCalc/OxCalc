#![forbid(unsafe_code)]

//! Checked-in TreeCalc fixture loading for the local sequential runtime.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::formula::{TreeFormula, TreeFormulaBinding, TreeFormulaCatalog};
use crate::structural::{
    BindArtifactId, FormulaArtifactId, StructuralEdit, StructuralEditOutcome, StructuralError,
    StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
};
use crate::treecalc::{
    LocalTreeCalcEngine, LocalTreeCalcError, LocalTreeCalcInput, LocalTreeCalcRunArtifacts,
    derive_structural_invalidation_seeds,
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
    pub post_edit: Option<TreeCalcFixturePostEditPlan>,
    pub expected: TreeCalcFixtureExpected,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixturePostEditPlan {
    pub successor_snapshot_start_id: u64,
    pub edits: Vec<TreeCalcFixtureStructuralEdit>,
    pub expected_impacts: Vec<String>,
    #[serde(default)]
    pub expected_affected_node_ids: Option<Vec<Vec<u64>>>,
    pub expected: TreeCalcFixtureExpected,
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
    ReplaceFormulaAttachment {
        node_id: u64,
        formula_artifact_id: Option<String>,
        bind_artifact_id: Option<String>,
    },
    SetConstantValue {
        node_id: u64,
        constant_value: Option<String>,
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
    pub formula_artifact_id: Option<String>,
    pub bind_artifact_id: Option<String>,
    pub constant_value: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TreeCalcFixtureFormulaBinding {
    pub owner_node_id: u64,
    pub formula_artifact_id: String,
    pub bind_artifact_id: Option<String>,
    pub expression: TreeFormula,
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
    let structural_snapshot = StructuralSnapshot::create(
        StructuralSnapshotId(case.snapshot_id),
        TreeNodeId(case.root_node_id),
        case.nodes.iter().map(to_structural_node),
    )
    .map_err(|source| TreeCalcFixtureError::Structural {
        case_id: case.case_id.clone(),
        source,
    })?;

    let formula_catalog = TreeFormulaCatalog::new(case.formulas.iter().map(to_formula_binding));
    let seeded_published_values = case
        .seeded_published_values
        .iter()
        .map(|(node_id, value)| (TreeNodeId(*node_id), value.clone()))
        .collect::<BTreeMap<_, _>>();

    let initial_artifacts = engine
        .execute(LocalTreeCalcInput {
            structural_snapshot: structural_snapshot.clone(),
            formula_catalog: formula_catalog.clone(),
            seeded_published_values,
            invalidation_seeds: Vec::new(),
            candidate_result_id: format!("fixture:{}:candidate", case.case_id),
            publication_id: format!("fixture:{}:publication", case.case_id),
            compatibility_basis: format!("snapshot:{}", case.snapshot_id),
            artifact_token_basis: format!("snapshot:{}", case.snapshot_id),
            environment_context: crate::treecalc::LocalTreeCalcEnvironmentContext::default(),
        })
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
        .published_values
        .iter()
        .filter(|(node_id, _)| current_snapshot.try_get_node(**node_id).is_some())
        .map(|(node_id, value)| (*node_id, value.clone()))
        .collect::<BTreeMap<_, _>>();

    let rerun_snapshot = edit_outcomes
        .last()
        .map(|outcome| outcome.snapshot.clone())
        .unwrap_or_else(|| structural_snapshot.clone());

    let invalidation_seeds = derive_structural_invalidation_seeds(
        structural_snapshot,
        &rerun_snapshot,
        formula_catalog,
        &edit_outcomes,
    );

    let rerun_artifacts = engine
        .execute(LocalTreeCalcInput {
            structural_snapshot: rerun_snapshot.clone(),
            formula_catalog: formula_catalog.clone(),
            seeded_published_values,
            invalidation_seeds: invalidation_seeds.clone(),
            candidate_result_id: format!("fixture:{}:candidate:post_edit", case.case_id),
            publication_id: format!("fixture:{}:publication:post_edit", case.case_id),
            compatibility_basis: format!("snapshot:{}", plan.successor_snapshot_start_id),
            artifact_token_basis: format!("snapshot:{}", plan.successor_snapshot_start_id),
            environment_context: crate::treecalc::LocalTreeCalcEnvironmentContext::default(),
        })
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

fn to_structural_node(node: &TreeCalcFixtureNode) -> StructuralNode {
    StructuralNode {
        node_id: TreeNodeId(node.node_id),
        kind: parse_node_kind(&node.kind),
        symbol: node.symbol.clone(),
        parent_id: node.parent_id.map(TreeNodeId),
        child_ids: node.child_ids.iter().copied().map(TreeNodeId).collect(),
        formula_artifact_id: node
            .formula_artifact_id
            .as_ref()
            .map(|id| FormulaArtifactId(id.clone())),
        bind_artifact_id: node
            .bind_artifact_id
            .as_ref()
            .map(|id| BindArtifactId(id.clone())),
        constant_value: node.constant_value.clone(),
    }
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
        expression: binding.expression.clone(),
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
        TreeCalcFixtureStructuralEdit::ReplaceFormulaAttachment {
            node_id,
            formula_artifact_id,
            bind_artifact_id,
        } => StructuralEdit::ReplaceFormulaAttachment {
            node_id: TreeNodeId(*node_id),
            formula_artifact_id: formula_artifact_id
                .as_ref()
                .map(|id| FormulaArtifactId(id.clone())),
            bind_artifact_id: bind_artifact_id
                .as_ref()
                .map(|id| BindArtifactId(id.clone())),
        },
        TreeCalcFixtureStructuralEdit::SetConstantValue {
            node_id,
            constant_value,
        } => StructuralEdit::SetConstantValue {
            node_id: TreeNodeId(*node_id),
            constant_value: constant_value.clone(),
        },
        TreeCalcFixtureStructuralEdit::RemoveNode { node_id } => StructuralEdit::RemoveNode {
            node_id: TreeNodeId(*node_id),
        },
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::treecalc::LocalTreeCalcRunState;

    use super::*;

    #[test]
    fn checked_in_treecalc_fixtures_execute_against_local_runtime() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .unwrap();
        let manifest_path = repo_root.join("docs/test-fixtures/core-engine/treecalc/MANIFEST.json");
        let manifest = load_manifest(&manifest_path).unwrap();
        let engine = LocalTreeCalcEngine;

        assert_eq!(manifest.cases.len(), 14);

        for entry in &manifest.cases {
            let case_path = repo_root
                .join("docs/test-fixtures/core-engine/treecalc")
                .join(entry.path.replace('/', "\\"));
            let case = load_case(&case_path).unwrap();
            let execution = execute_fixture_case(&engine, &case).unwrap();
            let artifacts = &execution.initial_artifacts;

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
                    assert_eq!(actual_values.get(node_id), Some(expected_value));
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
                assert_eq!(actual_values.get(node_id), Some(expected_value));
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
}
