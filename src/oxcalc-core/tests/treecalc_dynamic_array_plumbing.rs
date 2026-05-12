#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use oxcalc_core::coordinator::{RejectKind, RuntimeEffectFamily};
use oxcalc_core::dependency::DependencyDiagnosticKind;
use oxcalc_core::recalc::OverlayKind;
use oxcalc_core::formula::{FormulaBinaryOp, TreeFormula, TreeFormulaBinding, TreeFormulaCatalog, TreeReference};
use oxcalc_core::structural::{
    BindArtifactId, FormulaArtifactId, StructuralNode, StructuralNodeKind, StructuralSnapshot,
    StructuralSnapshotId, TreeNodeId,
};
use oxcalc_core::treecalc::{
    LocalTreeCalcEngine, LocalTreeCalcEnvironmentContext, LocalTreeCalcInput, LocalTreeCalcRunState,
};

fn formula(owner_node_id: u64, artifact: &str, expression: TreeFormula) -> TreeFormulaBinding {
    TreeFormulaBinding {
        owner_node_id: TreeNodeId(owner_node_id),
        formula_artifact_id: FormulaArtifactId(artifact.to_string()),
        bind_artifact_id: Some(BindArtifactId(format!("bind:{artifact}"))),
        expression,
    }
}

fn direct(node_id: u64) -> TreeFormula {
    TreeFormula::Reference(TreeReference::DirectNode {
        target_node_id: TreeNodeId(node_id),
    })
}

fn literal(value: &str) -> TreeFormula {
    TreeFormula::Literal {
        value: value.to_string(),
    }
}

#[test]
fn randarray_spill_shape_is_explicit_treecalc_plumbing_boundary() {
    let snapshot = StructuralSnapshot::create(
        StructuralSnapshotId(90),
        TreeNodeId(1),
        [
            StructuralNode {
                node_id: TreeNodeId(1),
                kind: StructuralNodeKind::Root,
                symbol: "Root".to_string(),
                parent_id: None,
                child_ids: vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)],
                formula_artifact_id: None,
                bind_artifact_id: None,
                constant_value: None,
            },
            StructuralNode {
                node_id: TreeNodeId(2),
                kind: StructuralNodeKind::Calculation,
                symbol: "A".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
                formula_artifact_id: Some(FormulaArtifactId("formula:A:randarray".to_string())),
                bind_artifact_id: Some(BindArtifactId("bind:A:randarray".to_string())),
                constant_value: None,
            },
            StructuralNode {
                node_id: TreeNodeId(3),
                kind: StructuralNodeKind::Calculation,
                symbol: "B".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
                formula_artifact_id: Some(FormulaArtifactId("formula:B:A_plus_1".to_string())),
                bind_artifact_id: Some(BindArtifactId("bind:B:A_plus_1".to_string())),
                constant_value: None,
            },
            StructuralNode {
                node_id: TreeNodeId(4),
                kind: StructuralNodeKind::Calculation,
                symbol: "C".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
                formula_artifact_id: Some(FormulaArtifactId("formula:C:sum_A_B".to_string())),
                bind_artifact_id: Some(BindArtifactId("bind:C:sum_A_B".to_string())),
                constant_value: None,
            },
            StructuralNode {
                node_id: TreeNodeId(5),
                kind: StructuralNodeKind::Calculation,
                symbol: "D".to_string(),
                parent_id: Some(TreeNodeId(1)),
                child_ids: vec![],
                formula_artifact_id: Some(FormulaArtifactId("formula:D:index_A_2_2".to_string())),
                bind_artifact_id: Some(BindArtifactId("bind:D:index_A_2_2".to_string())),
                constant_value: None,
            },
        ],
    )
    .expect("snapshot should be valid");

    let catalog = TreeFormulaCatalog::new([
        formula(
            2,
            "formula:A:randarray",
            TreeFormula::Reference(TreeReference::ShapeTopology {
                carrier_id: "carrier:A:randarray:spill".to_string(),
                detail: "RANDARRAY(5,5) produces volatile 5x5 spill shape for A".to_string(),
            }),
        ),
        formula(
            3,
            "formula:B:A_plus_1",
            TreeFormula::Binary {
                op: FormulaBinaryOp::Add,
                left: Box::new(direct(2)),
                right: Box::new(literal("1")),
            },
        ),
        formula(
            4,
            "formula:C:sum_A_B",
            TreeFormula::FunctionCall {
                function_name: "SUM".to_string(),
                arguments: vec![direct(2), direct(3)],
                may_introduce_dynamic_dependencies: false,
            },
        ),
        formula(
            5,
            "formula:D:index_A_2_2",
            TreeFormula::FunctionCall {
                function_name: "INDEX".to_string(),
                arguments: vec![direct(2), literal("2"), literal("2")],
                may_introduce_dynamic_dependencies: false,
            },
        ),
    ]);

    let artifacts = LocalTreeCalcEngine
        .execute(LocalTreeCalcInput {
            structural_snapshot: snapshot,
            formula_catalog: catalog,
            seeded_published_values: BTreeMap::new(),
            seeded_published_runtime_effects: vec![],
            invalidation_seeds: vec![],
            candidate_result_id: "fixture:dynamic-array-plumbing:candidate".to_string(),
            publication_id: "fixture:dynamic-array-plumbing:publication".to_string(),
            compatibility_basis: "showcase:randarray-spill-plumbing:v1".to_string(),
            artifact_token_basis: "showcase:randarray-spill-plumbing:v1".to_string(),
            environment_context: LocalTreeCalcEnvironmentContext::default(),
        })
        .expect("TreeCalc should return rejected artifacts, not crash");

    assert_eq!(artifacts.result_state, LocalTreeCalcRunState::Rejected);
    assert!(artifacts.publication_bundle.is_none());
    assert_eq!(
        artifacts.reject_detail.as_ref().map(|detail| detail.kind.clone()),
        Some(RejectKind::HostInjectedFailure)
    );

    let graph = &artifacts.dependency_graph;
    assert!(graph.cycle_groups.is_empty());
    assert!(graph.diagnostics.iter().any(|diagnostic| {
        diagnostic.kind == DependencyDiagnosticKind::ShapeTopologyReference
            && diagnostic.detail.contains("RANDARRAY(5,5)")
    }));

    let b_edges = graph.edges_by_owner.get(&TreeNodeId(3)).expect("B edges");
    assert!(b_edges.iter().any(|edge| edge.target_node_id == TreeNodeId(2)));
    let c_edges = graph.edges_by_owner.get(&TreeNodeId(4)).expect("C edges");
    assert!(c_edges.iter().any(|edge| edge.target_node_id == TreeNodeId(2)));
    assert!(c_edges.iter().any(|edge| edge.target_node_id == TreeNodeId(3)));
    let d_edges = graph.edges_by_owner.get(&TreeNodeId(5)).expect("D edges");
    assert!(d_edges.iter().any(|edge| edge.target_node_id == TreeNodeId(2)));

    assert!(artifacts.runtime_effects.iter().any(|effect| {
        effect.family == RuntimeEffectFamily::ShapeTopology
            && effect.kind == "runtime_effect.shape_topology_reference"
            && effect.detail.contains("RANDARRAY(5,5)")
    }));
    assert!(artifacts
        .runtime_effect_overlays
        .iter()
        .any(|overlay| overlay.key.overlay_kind == OverlayKind::ShapeTopology));

    for node_id in [2, 3, 4, 5] {
        assert!(
            artifacts.invalidation_closure.records.contains_key(&TreeNodeId(node_id)),
            "node {node_id} should be in invalidation closure"
        );
    }
}
