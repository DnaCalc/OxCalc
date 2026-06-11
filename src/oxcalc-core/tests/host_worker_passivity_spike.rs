//! Passivity-spike timings for DNA TreeCalc stack-requirements W5
//! `host-worker-calc` (ROADMAP open question 5).
//!
//! Measures synchronous `OxCalcTreeContext::recalculate` wall-clock at model
//! sizes the host worker decision cares about, for three shapes:
//! a deep dependency chain (worst-case topological depth), a wide fan
//! (worst-case sibling breadth under one aggregate), and an incremental
//! single-edit recalc on the chain.
//!
//! Run manually with:
//! `cargo test -p oxcalc-core --test host_worker_passivity_spike -- --ignored --nocapture`
//!
//! Findings are recorded in
//! `docs/spec/core-engine/CORE_ENGINE_HOST_WORKER_PASSIVITY_SPIKE.md`; this
//! test stays `#[ignore]`d so routine CI never pays for it.

#![forbid(unsafe_code)]

use std::time::Instant;

use oxcalc_core::consumer::{OxCalcTreeContext, OxCalcTreeNodeCreate, OxCalcTreeWorkspaceCreate};

fn chain_context(
    n: usize,
) -> (
    OxCalcTreeContext,
    oxcalc_core::consumer::OxCalcTreeWorkspaceId,
    oxcalc_core::structural::TreeNodeId,
) {
    let mut context = OxCalcTreeContext::default();
    let workspace_id = context
        .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:spike-chain"))
        .expect("create workspace");
    let mut mid_id = context
        .add_node(&workspace_id, OxCalcTreeNodeCreate::new("N0", "=1"))
        .expect("add chain seed");
    for index in 1..n {
        let symbol = format!("N{index}");
        let formula = format!("=N{}+1", index - 1);
        let id = context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new(&symbol, &formula))
            .expect("add chain node");
        if index == n / 2 {
            mid_id = id;
        }
    }
    (context, workspace_id, mid_id)
}

fn fan_context(
    n: usize,
) -> (
    OxCalcTreeContext,
    oxcalc_core::consumer::OxCalcTreeWorkspaceId,
) {
    let mut context = OxCalcTreeContext::default();
    let workspace_id = context
        .create_workspace(OxCalcTreeWorkspaceCreate::new("workspace:spike-fan"))
        .expect("create workspace");
    for index in 0..n {
        let symbol = format!("L{index}");
        context
            .add_node(&workspace_id, OxCalcTreeNodeCreate::new(&symbol, "=1"))
            .expect("add fan leaf");
    }
    context
        .add_node(
            &workspace_id,
            OxCalcTreeNodeCreate::new("TotalSum", "=L0+L1+L2+L3+L4+L5+L6+L7"),
        )
        .expect("add fan aggregate");
    (context, workspace_id)
}

fn timed_recalc(
    context: &mut OxCalcTreeContext,
    workspace_id: &oxcalc_core::consumer::OxCalcTreeWorkspaceId,
    label: &str,
) {
    let start = Instant::now();
    let outcome = context.recalculate(workspace_id).expect("recalculate");
    let elapsed = start.elapsed();
    println!(
        "spike: {label}: run_state={:?} evaluated={} wall={:?}",
        outcome.run_state,
        outcome.evaluation_order.len(),
        elapsed,
    );
    // Top phases by recorded engine time, to localize the cost.
    let mut phases: Vec<_> = outcome.phase_timings_micros.iter().collect();
    phases.sort_by(|a, b| b.1.cmp(a.1));
    for (key, micros) in phases.iter().take(6) {
        println!("spike: {label}:   phase {key:?} = {}ms", *micros / 1000);
    }
}

#[test]
#[ignore = "passivity-spike timing run; execute manually with --ignored --nocapture"]
fn spike_chain_cold_and_incremental_recalc_timings() {
    for n in [100usize, 200, 400] {
        let build_start = Instant::now();
        let (mut context, workspace_id, mid_id) = chain_context(n);
        println!("spike: chain n={n}: build wall={:?}", build_start.elapsed());

        timed_recalc(&mut context, &workspace_id, &format!("chain n={n} cold"));
        timed_recalc(&mut context, &workspace_id, &format!("chain n={n} warm"));

        // Incremental: edit one node in the middle of the chain and recalc.
        context
            .set_node_formula_text(&workspace_id, mid_id, "=N0+100")
            .expect("edit mid-chain node");
        timed_recalc(
            &mut context,
            &workspace_id,
            &format!("chain n={n} incremental mid-edit"),
        );
    }
}

#[test]
#[ignore = "passivity-spike timing run; execute manually with --ignored --nocapture"]
fn spike_fan_cold_recalc_timings() {
    for n in [1_000usize, 5_000, 20_000] {
        let build_start = Instant::now();
        let (mut context, workspace_id) = fan_context(n);
        println!("spike: fan n={n}: build wall={:?}", build_start.elapsed());
        timed_recalc(&mut context, &workspace_id, &format!("fan n={n} cold"));
        timed_recalc(&mut context, &workspace_id, &format!("fan n={n} warm"));
    }
}
