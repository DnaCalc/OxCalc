//! Runtime dependency tracing for calc-time realized grid references.

use std::cell::Cell;

use super::*;
use crate::grid::coords::ExcelGridAxisRef;
use oxfml_core::binding::{BoundExpr, ProfileReferenceRecord, ReferenceExpr};
use oxfml_core::consumer::runtime::RuntimeFormulaResult;
use oxfml_core::semantics::FunctionPlanBinding;
use oxfunc_core::function::{FecDependencyProfile, HostInteractionClass};
use oxfunc_core::resolver::ReferenceSystemCapabilities;

#[derive(Debug, Clone, PartialEq)]
pub struct GridFormulaEvaluationOutcome {
    pub value: CalcValue,
    pub trace: GridRuntimeDependencyTrace,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridRuntimeDependencyTrace {
    pub realized_dependencies: BTreeSet<GridDependency>,
    pub volatile: bool,
    pub external_pending: bool,
    pub external_subscriptions: BTreeSet<GridRuntimeExternalSubscription>,
    pub resolution_effects: Vec<GridRuntimeResolutionEffect>,
    /// F1: set when a formula-level bound-tree classification determined
    /// that EVERY runtime-realizing reference in this formula (INDIRECT/
    /// OFFSET-class calls) is consumed solely as an argument to a metadata
    /// function (ROW/COLUMN/ROWS/COLUMNS). When true,
    /// `overlay_dependencies_excluding_structural` installs
    /// `realized_dependencies` as invalidation-only
    /// `GridDependency::ReferenceMetadata` edges instead of ordinary value
    /// edges, so a metadata-only consumer of a runtime-realized reference
    /// does not create a false effective-dependency cycle and does not get
    /// spuriously dirtied by unrelated value edits to the realized cells.
    /// See `grid_formula_runtime_realized_dependencies_are_metadata_only`.
    pub runtime_realized_dependencies_are_metadata_only: bool,
}

impl GridRuntimeDependencyTrace {
    #[must_use]
    pub fn is_external_pending(&self) -> bool {
        self.external_pending || !self.external_subscriptions.is_empty()
    }

    #[must_use]
    pub fn with_external_subscription(
        mut self,
        subscription: GridRuntimeExternalSubscription,
    ) -> Self {
        self.external_pending = true;
        self.external_subscriptions.insert(subscription);
        self
    }

    pub fn add_external_subscriptions_from_runtime_result(
        &mut self,
        result: &RuntimeFormulaResult,
    ) {
        let subscriptions = grid_runtime_external_subscriptions_from_result(result);
        if !subscriptions.is_empty() {
            self.external_pending = true;
            self.external_subscriptions.extend(subscriptions);
        }
    }

    #[must_use]
    pub fn overlay_dependencies_excluding_structural(
        &self,
        structural_dependencies: &[GridDependency],
    ) -> BTreeSet<GridDependency> {
        let filtered = self.realized_dependencies.iter().filter(|dependency| {
            !structural_dependencies
                .iter()
                .any(|structural| grid_dependency_covers(structural, dependency))
        });
        if self.runtime_realized_dependencies_are_metadata_only {
            // F1: every runtime-realizing reference in this formula is
            // consumed solely as a metadata-function argument, so its
            // realized dependencies install as invalidation-only
            // `ReferenceMetadata` edges rather than ordinary value edges.
            return filtered
                .cloned()
                .map(|dependency| GridDependency::ReferenceMetadata(Box::new(dependency)))
                .collect();
        }
        filtered.cloned().collect()
    }
}

#[must_use]
pub fn grid_runtime_external_subscriptions_from_result(
    result: &RuntimeFormulaResult,
) -> BTreeSet<GridRuntimeExternalSubscription> {
    if !result
        .semantic_plan
        .execution_profile
        .contains_external_event_dependence
    {
        return BTreeSet::new();
    }

    let formula_stable_id = result.source.formula_stable_id.0.as_str();
    let mut subscriptions = result
        .semantic_plan
        .function_bindings
        .iter()
        .filter(|binding| function_binding_projects_external_subscription(binding))
        .map(|binding| {
            let topic_component =
                grid_runtime_subscription_component(&binding.function_name.to_ascii_lowercase());
            GridRuntimeExternalSubscription::new(
                format!("topic:{topic_component}"),
                format!(
                    "subscription:{}:{}",
                    grid_runtime_subscription_component(formula_stable_id),
                    topic_component
                ),
                format!(
                    "semantic_plan_external_provider:{}:{}",
                    binding.function_name, binding.function_id
                ),
            )
        })
        .collect::<BTreeSet<_>>();

    if subscriptions.is_empty() {
        subscriptions.insert(GridRuntimeExternalSubscription::new(
            "topic:external-reference",
            format!(
                "subscription:{}:external-reference",
                grid_runtime_subscription_component(formula_stable_id)
            ),
            "semantic_plan_external_event_dependence",
        ));
    }

    subscriptions
}

fn function_binding_projects_external_subscription(binding: &FunctionPlanBinding) -> bool {
    binding.host_interaction == HostInteractionClass::ExternalProvider
        || binding.fec_dependency_profile == FecDependencyProfile::ExternalProvider
        || binding.surface_fec_dependency_profile == FecDependencyProfile::ExternalProvider
}

fn grid_runtime_subscription_component(input: &str) -> String {
    input
        .chars()
        .map(|character| match character {
            'a'..='z' | 'A'..='Z' | '0'..='9' | ':' | '-' | '_' | '.' => character,
            _ => '_',
        })
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct GridRuntimeExternalSubscription {
    pub topic_id: String,
    pub subscription_handle: String,
    pub topic_descriptor: String,
}

impl GridRuntimeExternalSubscription {
    #[must_use]
    pub fn new(
        topic_id: impl Into<String>,
        subscription_handle: impl Into<String>,
        topic_descriptor: impl Into<String>,
    ) -> Self {
        Self {
            topic_id: topic_id.into(),
            subscription_handle: subscription_handle.into(),
            topic_descriptor: topic_descriptor.into(),
        }
    }

    #[must_use]
    pub fn topic(topic_id: impl Into<String>) -> Self {
        let topic_id = topic_id.into();
        Self {
            subscription_handle: format!("subscription:{topic_id}"),
            topic_descriptor: topic_id.clone(),
            topic_id,
        }
    }
}

fn runtime_identity_dependencies_for_text(
    text: &str,
    resolved_dependencies: &BTreeSet<GridDependency>,
    bounds: ExcelGridBounds,
    name_keys: Vec<String>,
    name_resolution: GridNameDependencyScopeResolution,
) -> BTreeSet<GridDependency> {
    let text = text.trim();
    if let Some(table_name) = explicit_table_name_from_structured_reference(text) {
        return grid_named_structural_dependency(table_name, bounds, resolved_dependencies.clone());
    }
    // Runtime text resolved via the caller-local table-column fallback
    // (e.g. `INDIRECT("Amount")` while the calling cell sits inside a
    // table with an `Amount` column and no real defined name of that
    // text): install the real Table/TableIdentity structural anchor for
    // the owning table, plus a defensive NameIdentity edge for every
    // candidate name key so a later shadowing name-create still retargets
    // this consumer through the ordinary name-lifecycle seed path. See D4.
    if let GridNameDependencyScopeResolution::CallerLocalTableColumn(table_name) = &name_resolution
    {
        let mut dependencies =
            grid_named_structural_dependency(table_name, bounds, resolved_dependencies.clone());
        dependencies.extend(
            name_keys
                .into_iter()
                .map(GridNameIdentityDependency::from_key)
                .map(GridDependency::NameIdentity),
        );
        return dependencies;
    }
    grid_name_structural_dependencies(name_keys, resolved_dependencies.clone(), bounds)
}

fn runtime_name_identity_dependency_for_text(
    text: &str,
    bounds: ExcelGridBounds,
    name_keys: Vec<String>,
    name_resolution: GridNameDependencyScopeResolution,
) -> BTreeSet<GridDependency> {
    let text = text.trim();
    if let Some(table_name) = explicit_table_name_from_structured_reference(text) {
        return GridTableIdentityDependency::new(table_name, bounds)
            .map(GridDependency::TableIdentity)
            .into_iter()
            .collect();
    }
    // Same rationale as `runtime_identity_dependencies_for_text` above, for
    // the unresolved-text error path: the failed lookup still names the
    // owning table's TableIdentity plus NameIdentity for all candidates.
    if let GridNameDependencyScopeResolution::CallerLocalTableColumn(table_name) = &name_resolution
    {
        let mut dependencies: BTreeSet<GridDependency> =
            GridTableIdentityDependency::new(table_name, bounds)
                .map(GridDependency::TableIdentity)
                .into_iter()
                .collect();
        dependencies.extend(
            name_keys
                .into_iter()
                .map(GridNameIdentityDependency::from_key)
                .map(GridDependency::NameIdentity),
        );
        return dependencies;
    }
    name_keys
        .into_iter()
        .map(GridNameIdentityDependency::from_key)
        .map(GridDependency::NameIdentity)
        .collect()
}

impl GridInvalidationRef {
    pub fn replace_overlay_dependencies_from_trace(
        &mut self,
        dependent: ExcelGridCellAddress,
        trace: &GridRuntimeDependencyTrace,
    ) -> Result<GridOverlayDependencyUpdate, GridRefError> {
        let structural_dependencies =
            self.semantic_dependencies_for_layer(GridDependencyLayer::Structural, &dependent);
        let old_dependencies = self
            .semantic_dependencies_for_layer(GridDependencyLayer::CalcOverlay, &dependent)
            .into_iter()
            .collect::<BTreeSet<_>>();
        let mut overlay_dependencies =
            trace.overlay_dependencies_excluding_structural(&structural_dependencies);
        overlay_dependencies.extend(
            old_dependencies
                .iter()
                .filter(|dependency| matches!(dependency, GridDependency::SpillBlocker(_)))
                .cloned(),
        );
        self.set_volatile_root(dependent.clone(), trace.volatile)?;
        self.set_external_pending_root(dependent.clone(), trace.is_external_pending())?;
        if old_dependencies == overlay_dependencies {
            self.set_overlay_dependencies(dependent.clone(), overlay_dependencies)?;
            return Ok(GridOverlayDependencyUpdate::unchanged(
                dependent,
                old_dependencies,
            ));
        }
        self.set_overlay_dependencies(dependent.clone(), overlay_dependencies.clone())?;
        Ok(GridOverlayDependencyUpdate::identity_changed(
            dependent,
            old_dependencies,
            overlay_dependencies,
        ))
    }

    pub fn refresh_overlay_spill_blocker_dependency(
        &mut self,
        dependent: ExcelGridCellAddress,
        current_spill_blocker_extent: Option<GridRect>,
    ) -> Result<GridOverlayDependencyUpdate, GridRefError> {
        let old_dependencies = self
            .semantic_dependencies_for_layer(GridDependencyLayer::CalcOverlay, &dependent)
            .into_iter()
            .collect::<BTreeSet<_>>();
        let mut new_dependencies = old_dependencies
            .iter()
            .filter(|dependency| !matches!(dependency, GridDependency::SpillBlocker(_)))
            .cloned()
            .collect::<BTreeSet<_>>();
        if let Some(extent) = current_spill_blocker_extent {
            new_dependencies.insert(GridDependency::SpillBlocker(
                GridSpillBlockerDependency::extent(extent),
            ));
        }
        if old_dependencies == new_dependencies {
            self.set_overlay_dependencies(dependent.clone(), new_dependencies)?;
            return Ok(GridOverlayDependencyUpdate::unchanged(
                dependent,
                old_dependencies,
            ));
        }
        self.set_overlay_dependencies(dependent.clone(), new_dependencies.clone())?;
        Ok(GridOverlayDependencyUpdate::changed(
            dependent,
            old_dependencies,
            new_dependencies,
        ))
    }
}

#[must_use]
pub(super) fn grid_structural_dependencies_for_formula<P>(
    formula: &GridFormulaCell,
    address: &ExcelGridCellAddress,
    profile: &dyn ReferenceBindProfile,
    bounds: ExcelGridBounds,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    let bound = bind_grid_formula_for_transform(formula, address, profile, bounds);
    grid_structural_dependencies_for_bound_formula(&bound, bounds, provider)
}

#[must_use]
pub(super) fn grid_structural_dependencies_for_bound_formula<P>(
    bound: &BoundFormula,
    bounds: ExcelGridBounds,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    let mut dependencies =
        grid_value_structural_dependencies_for_bound_expr(&bound.root, bounds, provider);
    dependencies.extend(grid_reference_metadata_dependencies_for_bound_expr(
        &bound.root,
        bounds,
        provider,
    ));
    dependencies.extend(grid_spill_fact_dependencies_for_bound_expr(
        &bound.root,
        provider,
    ));
    dependencies.extend(grid_axis_visibility_dependencies_for_bound_expr(
        &bound.root,
        bounds,
        provider,
    ));
    dependencies
}

fn grid_value_structural_dependencies_for_bound_expr<P>(
    expr: &BoundExpr,
    bounds: ExcelGridBounds,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    let mut dependencies = BTreeSet::new();
    collect_grid_value_structural_dependencies_for_bound_expr(
        expr,
        bounds,
        provider,
        &mut dependencies,
    );
    dependencies
}

fn collect_grid_value_structural_dependencies_for_bound_expr<P>(
    expr: &BoundExpr,
    bounds: ExcelGridBounds,
    provider: &P,
    dependencies: &mut BTreeSet<GridDependency>,
) where
    P: GridRuntimeTraceReferenceResolver,
{
    match expr {
        BoundExpr::ArrayLiteral(rows) => {
            for child in rows.iter().flatten() {
                collect_grid_value_structural_dependencies_for_bound_expr(
                    child,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Binary { left, right, .. } => {
            collect_grid_value_structural_dependencies_for_bound_expr(
                left,
                bounds,
                provider,
                dependencies,
            );
            collect_grid_value_structural_dependencies_for_bound_expr(
                right,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            collect_grid_value_structural_dependencies_for_bound_expr(
                expr,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
            ..
        } => {
            if grid_function_uses_reference_metadata(function_name) {
                return;
            }
            for arg in args {
                collect_grid_value_structural_dependencies_for_bound_expr(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Invocation { callee, args } => {
            collect_grid_value_structural_dependencies_for_bound_expr(
                callee,
                bounds,
                provider,
                dependencies,
            );
            for arg in args {
                collect_grid_value_structural_dependencies_for_bound_expr(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Reference(reference) => {
            dependencies.extend(grid_structural_dependencies_for_reference_expr(
                reference, bounds, provider,
            ));
        }
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_) => {}
    }
}

fn grid_reference_metadata_dependencies_for_bound_expr<P>(
    expr: &BoundExpr,
    bounds: ExcelGridBounds,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    let mut dependencies = BTreeSet::new();
    collect_grid_reference_metadata_dependencies_for_bound_expr(
        expr,
        bounds,
        provider,
        &mut dependencies,
    );
    dependencies
}

fn collect_grid_reference_metadata_dependencies_for_bound_expr<P>(
    expr: &BoundExpr,
    bounds: ExcelGridBounds,
    provider: &P,
    dependencies: &mut BTreeSet<GridDependency>,
) where
    P: GridRuntimeTraceReferenceResolver,
{
    match expr {
        BoundExpr::ArrayLiteral(rows) => {
            for child in rows.iter().flatten() {
                collect_grid_reference_metadata_dependencies_for_bound_expr(
                    child,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Binary { left, right, .. } => {
            collect_grid_reference_metadata_dependencies_for_bound_expr(
                left,
                bounds,
                provider,
                dependencies,
            );
            collect_grid_reference_metadata_dependencies_for_bound_expr(
                right,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            collect_grid_reference_metadata_dependencies_for_bound_expr(
                expr,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
            ..
        } => {
            if grid_function_uses_reference_metadata(function_name) {
                for arg in args {
                    dependencies.extend(grid_structural_dependencies_for_metadata_expr(
                        arg, bounds, provider,
                    ));
                }
                return;
            }
            for arg in args {
                collect_grid_reference_metadata_dependencies_for_bound_expr(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Invocation { callee, args } => {
            collect_grid_reference_metadata_dependencies_for_bound_expr(
                callee,
                bounds,
                provider,
                dependencies,
            );
            for arg in args {
                collect_grid_reference_metadata_dependencies_for_bound_expr(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Reference(_)
        | BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_) => {}
    }
}

fn grid_structural_dependencies_for_metadata_expr<P>(
    expr: &BoundExpr,
    bounds: ExcelGridBounds,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    let mut dependencies = BTreeSet::new();
    collect_grid_structural_dependencies_for_metadata_expr(
        expr,
        bounds,
        provider,
        &mut dependencies,
    );
    dependencies
}

fn collect_grid_structural_dependencies_for_metadata_expr<P>(
    expr: &BoundExpr,
    bounds: ExcelGridBounds,
    provider: &P,
    dependencies: &mut BTreeSet<GridDependency>,
) where
    P: GridRuntimeTraceReferenceResolver,
{
    match expr {
        BoundExpr::Reference(reference) => {
            dependencies.extend(
                grid_structural_dependencies_for_reference_expr(reference, bounds, provider)
                    .into_iter()
                    .map(|dependency| GridDependency::ReferenceMetadata(Box::new(dependency))),
            );
        }
        BoundExpr::ArrayLiteral(rows) => {
            for child in rows.iter().flatten() {
                collect_grid_structural_dependencies_for_metadata_expr(
                    child,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Binary { left, right, .. } => {
            collect_grid_structural_dependencies_for_metadata_expr(
                left,
                bounds,
                provider,
                dependencies,
            );
            collect_grid_structural_dependencies_for_metadata_expr(
                right,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            collect_grid_structural_dependencies_for_metadata_expr(
                expr,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::FunctionCall { args, .. } => {
            for arg in args {
                collect_grid_structural_dependencies_for_metadata_expr(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Invocation { callee, args } => {
            collect_grid_structural_dependencies_for_metadata_expr(
                callee,
                bounds,
                provider,
                dependencies,
            );
            for arg in args {
                collect_grid_structural_dependencies_for_metadata_expr(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_) => {}
    }
}

fn grid_structural_dependencies_for_reference_expr<P>(
    reference: &ReferenceExpr,
    bounds: ExcelGridBounds,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    match reference {
        ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record)) => {
            grid_structural_dependencies_for_profile_record(record, bounds, provider)
        }
        ReferenceExpr::Range { .. } => grid_extent_for_reference_expr(reference, bounds, provider)
            .map(grid_dependency_for_rect)
            .into_iter()
            .collect(),
        ReferenceExpr::Spill { anchor } => {
            grid_structural_dependencies_for_reference_expr(anchor, bounds, provider)
        }
        ReferenceExpr::Union { left, right } | ReferenceExpr::Intersection { left, right } => {
            let mut dependencies =
                grid_structural_dependencies_for_reference_expr(left, bounds, provider);
            dependencies.extend(grid_structural_dependencies_for_reference_expr(
                right, bounds, provider,
            ));
            dependencies
        }
        // W062 R4.12: a 3D sheet-span reference (`Sheet1:Sheet3!A1`) is a raw
        // `SheetSpan3D` atom in the bound tree — OxFml's shared grammar produces
        // it directly, never through the profile's `bind_*` path — so it does
        // NOT arrive as a `ProfileSymbolic` record. Emit its single stored
        // `SheetSpan` edge here (endpoints + sheet-agnostic target), never a
        // materialized per-sheet fan. The per-sheet fan is the workbook layer's
        // closure-time interval probe against the current sheet order.
        ReferenceExpr::Atom(NormalizedReference::SheetSpan3D(span)) => {
            let mut dependencies = BTreeSet::new();
            dependencies.insert(GridDependency::SheetSpan(GridSheetSpanDependency::new(
                span.workbook_id.clone(),
                span.start_sheet.clone(),
                span.end_sheet.clone(),
                span.target.clone(),
            )));
            dependencies
        }
        ReferenceExpr::Atom(_) => BTreeSet::new(),
    }
}

fn grid_function_uses_reference_metadata(function_name: &str) -> bool {
    matches!(
        function_name.to_ascii_uppercase().as_str(),
        "ROW" | "COLUMN" | "ROWS" | "COLUMNS"
    )
}

/// Functions whose OxFunc implementation dynamically realizes a grid
/// reference at calc time (calls back into the reference-system provider's
/// `resolve_text`/`transform_reference`/`enumerate_values`/`dereference`
/// seam rather than reading a statically-bound reference). This is the same
/// name-over-bound-tree classification pattern as
/// `grid_function_uses_reference_metadata` above — semantic structure, not
/// text sniffing. See F1.
fn grid_function_may_realize_runtime_reference(function_name: &str) -> bool {
    matches!(
        function_name.to_ascii_uppercase().as_str(),
        "INDIRECT" | "OFFSET"
    )
}

/// F1: classify whether EVERY runtime-realizing call
/// (`grid_function_may_realize_runtime_reference`) in `formula` is consumed
/// SOLELY as an argument to a metadata function
/// (`grid_function_uses_reference_metadata`), e.g. `ROWS(INDIRECT("B1:B2"))`.
///
/// Conservative by construction: a formula with no runtime-realizing calls
/// at all returns `false` (nothing to reclassify — keep ordinary value
/// edges), and a formula that consumes a runtime-realizing call's result as
/// a VALUE anywhere (even alongside a metadata consumption elsewhere, e.g.
/// `ROWS(INDIRECT("B1:B2"))+B1`) also returns `false`, so mixed
/// metadata+value formulas keep real value edges and a genuine cycle
/// through the value read still reports.
///
/// The long-term precise fix is per-call-site consumption intent surfaced
/// directly from OxFml (which argument position of which function actually
/// triggered a given realized dependency), rather than this formula-wide
/// bound-tree approximation.
#[must_use]
pub(super) fn grid_formula_runtime_realized_dependencies_are_metadata_only(
    bound: &BoundFormula,
) -> bool {
    let mut found_runtime_realizing_call = false;
    let mut found_value_consumed_runtime_realizing_call = false;
    collect_grid_runtime_realizing_call_consumption(
        &bound.root,
        false,
        &mut found_runtime_realizing_call,
        &mut found_value_consumed_runtime_realizing_call,
    );
    found_runtime_realizing_call && !found_value_consumed_runtime_realizing_call
}

fn collect_grid_runtime_realizing_call_consumption(
    expr: &BoundExpr,
    in_metadata_context: bool,
    found_runtime_realizing_call: &mut bool,
    found_value_consumed_runtime_realizing_call: &mut bool,
) {
    match expr {
        BoundExpr::ArrayLiteral(rows) => {
            for child in rows.iter().flatten() {
                collect_grid_runtime_realizing_call_consumption(
                    child,
                    in_metadata_context,
                    found_runtime_realizing_call,
                    found_value_consumed_runtime_realizing_call,
                );
            }
        }
        BoundExpr::Binary { left, right, .. } => {
            collect_grid_runtime_realizing_call_consumption(
                left,
                in_metadata_context,
                found_runtime_realizing_call,
                found_value_consumed_runtime_realizing_call,
            );
            collect_grid_runtime_realizing_call_consumption(
                right,
                in_metadata_context,
                found_runtime_realizing_call,
                found_value_consumed_runtime_realizing_call,
            );
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            collect_grid_runtime_realizing_call_consumption(
                expr,
                in_metadata_context,
                found_runtime_realizing_call,
                found_value_consumed_runtime_realizing_call,
            );
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
            ..
        } => {
            let is_runtime_realizing = grid_function_may_realize_runtime_reference(function_name);
            if is_runtime_realizing {
                *found_runtime_realizing_call = true;
                if !in_metadata_context {
                    *found_value_consumed_runtime_realizing_call = true;
                }
            }
            let is_metadata = grid_function_uses_reference_metadata(function_name);
            for arg in args {
                collect_grid_runtime_realizing_call_consumption(
                    arg,
                    // A runtime-realizing call's OWN arguments (e.g. the
                    // text argument to `INDIRECT`) are not themselves the
                    // realized reference, so they do not inherit the
                    // enclosing metadata context; but they also cannot
                    // contain a runtime-realizing call whose realized
                    // reference matters here, so this only affects nested
                    // runtime-realizing calls, which stay conservative via
                    // `is_metadata` short-circuiting to `false` in that case.
                    is_metadata,
                    found_runtime_realizing_call,
                    found_value_consumed_runtime_realizing_call,
                );
            }
        }
        BoundExpr::Invocation { callee, args } => {
            collect_grid_runtime_realizing_call_consumption(
                callee,
                false,
                found_runtime_realizing_call,
                found_value_consumed_runtime_realizing_call,
            );
            for arg in args {
                collect_grid_runtime_realizing_call_consumption(
                    arg,
                    false,
                    found_runtime_realizing_call,
                    found_value_consumed_runtime_realizing_call,
                );
            }
        }
        BoundExpr::Reference(_)
        | BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_) => {}
    }
}

fn grid_structural_dependencies_for_profile_record<P>(
    record: &ProfileReferenceRecord,
    bounds: ExcelGridBounds,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    if record.profile_id != EXCEL_GRID_PROFILE_ID {
        return BTreeSet::new();
    }

    let decoded = decode_excel_grid_reference_payload(&record.profile_payload);
    let Some(reference) = excel_grid_reference_like_from_profile_record(record) else {
        return decoded
            .and_then(|reference| grid_structural_dependency_for_decoded_only(reference, bounds))
            .into_iter()
            .collect();
    };
    let resolved_dependencies = provider.runtime_dependencies_for_reference(&reference);

    match decoded {
        Some(ExcelGridReference::WholeRow {
            start_row, end_row, ..
        }) => grid_axis_value_dependency_for_resolved_rows(&resolved_dependencies, bounds)
            .or_else(|| grid_axis_value_dependency_for_rows(start_row, end_row, bounds))
            .map(GridDependency::AxisValue)
            .into_iter()
            .collect(),
        Some(ExcelGridReference::WholeColumn {
            start_col, end_col, ..
        }) => grid_axis_value_dependency_for_resolved_columns(&resolved_dependencies, bounds)
            .or_else(|| grid_axis_value_dependency_for_columns(start_col, end_col, bounds))
            .map(GridDependency::AxisValue)
            .into_iter()
            .collect(),
        Some(ExcelGridReference::Name {
            workbook_id,
            sheet_id,
            name,
            ..
        }) => match provider.runtime_name_dependency_resolution_for_scope(
            &workbook_id,
            &sheet_id,
            &name,
        ) {
            // A local structured reference with no explicit table name
            // (`=SUM([Amount])`, `=[@Amount]*2` inside a table) binds down
            // OxFml's defined-name atom path; the caller-local
            // table-column fallback is not a real namespace entry, so this
            // must install a Table/TableIdentity edge keyed to the OWNING
            // TABLE, not a candidate Name edge keyed to a name text that
            // does not exist. See D4.
            GridNameDependencyScopeResolution::CallerLocalTableColumn(table_name) => {
                grid_named_structural_dependency(&table_name, bounds, resolved_dependencies)
            }
            GridNameDependencyScopeResolution::Name(name_key) => {
                grid_name_structural_dependencies(vec![name_key], resolved_dependencies, bounds)
            }
            GridNameDependencyScopeResolution::Unresolved => grid_name_structural_dependencies(
                provider.runtime_name_dependency_keys_for_scope(&workbook_id, &sheet_id, &name),
                resolved_dependencies,
                bounds,
            ),
        },
        Some(ExcelGridReference::StructuredReference { source_text, .. }) => {
            let Some(table_name) = explicit_table_name_from_structured_reference(&source_text)
            else {
                return resolved_dependencies;
            };
            grid_named_structural_dependency(table_name, bounds, resolved_dependencies)
        }
        Some(ExcelGridReference::SpillAnchor {
            workbook_id,
            sheet_id,
            anchor_key,
            source_text,
            ..
        }) => {
            let mut dependencies = resolved_dependencies;
            if let Some(anchor) = grid_spill_anchor_address_from_payload(
                &workbook_id,
                &sheet_id,
                &anchor_key,
                &source_text,
            ) {
                dependencies.insert(GridDependency::SpillFact(GridSpillDependency::anchor(
                    anchor,
                )));
            }
            dependencies
        }
        // W062 R4.12: a 3D sheet-span reference (`Sheet1:Sheet3!A1`) stores
        // exactly ONE dependency edge — never a materialized per-sheet fan.
        // `resolved_dependencies` is empty for a span (it resolves to no single
        // rect), so this arm installs the stored `SheetSpan` edge carrying the
        // authored endpoints + sheet-agnostic target. The per-sheet fan is a
        // closure-time expansion against the current sheet order (D2 §4.2 / D3
        // §2.3): the workbook coordinator's derived span-interval index probes
        // this edge, never a stored member set.
        Some(ExcelGridReference::SheetSpan {
            workbook_id,
            start_sheet,
            end_sheet,
            target,
            ..
        }) => {
            let mut dependencies = resolved_dependencies;
            dependencies.insert(GridDependency::SheetSpan(GridSheetSpanDependency::new(
                workbook_id,
                start_sheet,
                end_sheet,
                target,
            )));
            dependencies
        }
        _ => resolved_dependencies,
    }
}

fn grid_axis_value_dependency_for_resolved_rows(
    dependencies: &BTreeSet<GridDependency>,
    bounds: ExcelGridBounds,
) -> Option<GridAxisValueDependency> {
    let extent = grid_extent_for_dependencies(dependencies, bounds)?;
    (extent.left_col == 1 && extent.right_col == bounds.max_cols).then_some(
        GridAxisValueDependency::rows(extent.top_row, extent.bottom_row),
    )
}

fn grid_axis_value_dependency_for_resolved_columns(
    dependencies: &BTreeSet<GridDependency>,
    bounds: ExcelGridBounds,
) -> Option<GridAxisValueDependency> {
    let extent = grid_extent_for_dependencies(dependencies, bounds)?;
    (extent.top_row == 1 && extent.bottom_row == bounds.max_rows).then_some(
        GridAxisValueDependency::columns(extent.left_col, extent.right_col),
    )
}

fn grid_axis_value_dependency_for_rows(
    start_row: ExcelGridAxisRef,
    end_row: ExcelGridAxisRef,
    bounds: ExcelGridBounds,
) -> Option<GridAxisValueDependency> {
    let start = grid_absolute_axis_index(start_row)?;
    let end = grid_absolute_axis_index(end_row)?;
    let first = start.min(end);
    let last = start.max(end);
    (bounds.contains_row(first) && bounds.contains_row(last))
        .then_some(GridAxisValueDependency::rows(first, last))
}

fn grid_axis_value_dependency_for_columns(
    start_col: ExcelGridAxisRef,
    end_col: ExcelGridAxisRef,
    bounds: ExcelGridBounds,
) -> Option<GridAxisValueDependency> {
    let start = grid_absolute_axis_index(start_col)?;
    let end = grid_absolute_axis_index(end_col)?;
    let first = start.min(end);
    let last = start.max(end);
    (bounds.contains_col(first) && bounds.contains_col(last))
        .then_some(GridAxisValueDependency::columns(first, last))
}

fn grid_absolute_axis_index(axis_ref: ExcelGridAxisRef) -> Option<u32> {
    match axis_ref {
        ExcelGridAxisRef::Absolute(index) => Some(index),
        ExcelGridAxisRef::Relative(_) => None,
    }
}

fn grid_named_structural_dependency(
    name: &str,
    bounds: ExcelGridBounds,
    resolved_dependencies: BTreeSet<GridDependency>,
) -> BTreeSet<GridDependency> {
    let Some(extent) = resolved_dependencies
        .iter()
        .find_map(|dependency| grid_dependency_extent(dependency, bounds))
    else {
        return GridTableIdentityDependency::new(name, bounds)
            .map(GridDependency::TableIdentity)
            .into_iter()
            .chain(resolved_dependencies)
            .collect();
    };
    GridTableDependency::new(name, extent, bounds)
        .map(GridDependency::Table)
        .into_iter()
        .collect()
}

fn grid_name_structural_dependencies(
    name_keys: Vec<String>,
    resolved_dependencies: BTreeSet<GridDependency>,
    bounds: ExcelGridBounds,
) -> BTreeSet<GridDependency> {
    let Some(extent) = resolved_dependencies
        .iter()
        .find_map(|dependency| grid_dependency_extent(dependency, bounds))
    else {
        return name_keys
            .into_iter()
            .map(GridNameIdentityDependency::from_key)
            .map(GridDependency::NameIdentity)
            .chain(resolved_dependencies)
            .collect();
    };
    let Some(name_key) = name_keys.into_iter().next() else {
        return resolved_dependencies;
    };
    [GridDependency::Name(GridNameDependency::from_key(
        name_key, extent,
    ))]
    .into_iter()
    .collect()
}

fn grid_structural_dependency_for_decoded_only(
    reference: ExcelGridReference,
    _bounds: ExcelGridBounds,
) -> Option<GridDependency> {
    match reference {
        ExcelGridReference::SpillAnchor {
            workbook_id,
            sheet_id,
            anchor_key,
            source_text,
            ..
        } => grid_spill_anchor_address_from_payload(
            &workbook_id,
            &sheet_id,
            &anchor_key,
            &source_text,
        )
        .map(GridSpillDependency::anchor)
        .map(GridDependency::SpillFact),
        _ => None,
    }
}

/// F2: does `formula`'s bound expression tree contain a `#` spill-anchor
/// dereference (`ReferenceExpr::Spill { .. }`) anywhere? This walks the
/// BOUND EXPRESSION TREE the OxFml binder produces for postfix `#`
/// (semantic structure, not text), which is the doctrine-safe replacement
/// for the removed `formula.source_text.contains('#')` sniff. Note this is
/// deliberately NOT the same check as decoding a normalized reference's
/// payload to `ExcelGridReference::SpillAnchor`: the binder wraps an
/// ordinary already-bound `Cell`/`Name`/etc. reference in
/// `ReferenceExpr::Spill`, it never produces a `SpillAnchor`-payload
/// profile record, so that decode never matches real `#` syntax. See F2.
#[must_use]
pub(super) fn grid_bound_formula_references_spill_anchor(bound: &BoundFormula) -> bool {
    grid_bound_expr_references_spill_anchor(&bound.root)
}

fn grid_bound_expr_references_spill_anchor(expr: &BoundExpr) -> bool {
    match expr {
        BoundExpr::ArrayLiteral(rows) => rows
            .iter()
            .flatten()
            .any(grid_bound_expr_references_spill_anchor),
        BoundExpr::Binary { left, right, .. } => {
            grid_bound_expr_references_spill_anchor(left)
                || grid_bound_expr_references_spill_anchor(right)
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            grid_bound_expr_references_spill_anchor(expr)
        }
        BoundExpr::FunctionCall { args, .. } => {
            args.iter().any(grid_bound_expr_references_spill_anchor)
        }
        BoundExpr::Invocation { callee, args } => {
            grid_bound_expr_references_spill_anchor(callee)
                || args.iter().any(grid_bound_expr_references_spill_anchor)
        }
        BoundExpr::Reference(reference) => grid_reference_expr_is_or_contains_spill(reference),
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_) => false,
    }
}

fn grid_reference_expr_is_or_contains_spill(reference: &ReferenceExpr) -> bool {
    match reference {
        ReferenceExpr::Spill { .. } => true,
        ReferenceExpr::Range { start, end }
        | ReferenceExpr::Union {
            left: start,
            right: end,
        }
        | ReferenceExpr::Intersection {
            left: start,
            right: end,
        } => {
            grid_reference_expr_is_or_contains_spill(start)
                || grid_reference_expr_is_or_contains_spill(end)
        }
        ReferenceExpr::Atom(_) => false,
    }
}

fn grid_spill_fact_dependencies_for_bound_expr<P>(
    expr: &BoundExpr,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    let mut dependencies = BTreeSet::new();
    collect_grid_spill_fact_dependencies_for_bound_expr(expr, provider, &mut dependencies);
    dependencies
}

fn collect_grid_spill_fact_dependencies_for_bound_expr<P>(
    expr: &BoundExpr,
    provider: &P,
    dependencies: &mut BTreeSet<GridDependency>,
) where
    P: GridRuntimeTraceReferenceResolver,
{
    match expr {
        BoundExpr::ArrayLiteral(rows) => {
            for child in rows.iter().flatten() {
                collect_grid_spill_fact_dependencies_for_bound_expr(child, provider, dependencies);
            }
        }
        BoundExpr::Binary { left, right, .. } => {
            collect_grid_spill_fact_dependencies_for_bound_expr(left, provider, dependencies);
            collect_grid_spill_fact_dependencies_for_bound_expr(right, provider, dependencies);
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            collect_grid_spill_fact_dependencies_for_bound_expr(expr, provider, dependencies);
        }
        BoundExpr::FunctionCall { args, .. } => {
            for arg in args {
                collect_grid_spill_fact_dependencies_for_bound_expr(arg, provider, dependencies);
            }
        }
        BoundExpr::Invocation { callee, args } => {
            collect_grid_spill_fact_dependencies_for_bound_expr(callee, provider, dependencies);
            for arg in args {
                collect_grid_spill_fact_dependencies_for_bound_expr(arg, provider, dependencies);
            }
        }
        BoundExpr::Reference(reference) => {
            collect_grid_spill_fact_dependencies_for_reference_expr(
                reference,
                provider,
                dependencies,
            );
        }
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_) => {}
    }
}

fn collect_grid_spill_fact_dependencies_for_reference_expr<P>(
    reference: &ReferenceExpr,
    provider: &P,
    dependencies: &mut BTreeSet<GridDependency>,
) where
    P: GridRuntimeTraceReferenceResolver,
{
    match reference {
        ReferenceExpr::Spill { anchor } => {
            if let Some(dependency) = grid_spill_fact_dependency_for_anchor(anchor, provider) {
                dependencies.insert(dependency);
            }
            collect_grid_spill_fact_dependencies_for_reference_expr(anchor, provider, dependencies);
        }
        ReferenceExpr::Range { start, end }
        | ReferenceExpr::Union {
            left: start,
            right: end,
        }
        | ReferenceExpr::Intersection {
            left: start,
            right: end,
        } => {
            collect_grid_spill_fact_dependencies_for_reference_expr(start, provider, dependencies);
            collect_grid_spill_fact_dependencies_for_reference_expr(end, provider, dependencies);
        }
        ReferenceExpr::Atom(_) => {}
    }
}

fn grid_spill_fact_dependency_for_anchor<P>(
    anchor: &ReferenceExpr,
    provider: &P,
) -> Option<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    let ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record)) = anchor else {
        return None;
    };
    if record.profile_id != EXCEL_GRID_PROFILE_ID {
        return None;
    }
    let reference = excel_grid_reference_like_from_profile_record(record)?;
    let dependencies = provider.runtime_dependencies_for_reference(&reference);
    grid_single_cell_dependency(dependencies)
        .map(GridSpillDependency::anchor)
        .map(GridDependency::SpillFact)
}

/// The spill's full current realized extent (via the provider's
/// spill-anchor rect lookup), not just the anchor cell. Mirrors
/// `grid_spill_fact_dependency_for_anchor`'s anchor-record extraction but
/// builds a `SpillAnchor`-kind `ReferenceLike` (target `"{anchor}#"`)
/// instead of resolving the anchor cell alone, so
/// `ReferenceSystemProvider::resolved_rects_for_reference` follows the
/// spill-rect path (`ExcelGridReferenceSystemProvider::spill_rect`) and
/// returns the whole spilled rectangle. G5: hidden-row-sensitive aggregates
/// (SUBTOTAL/AGGREGATE) over a spill argument need this full extent for
/// their AxisVisibility dependency — the anchor's 1x1 extent alone would
/// not re-dirty on a hidden-row change elsewhere in the spilled range.
fn grid_spill_extent_for_anchor<P>(
    anchor: &ReferenceExpr,
    bounds: ExcelGridBounds,
    provider: &P,
) -> Option<GridRect>
where
    P: GridRuntimeTraceReferenceResolver,
{
    let ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record)) = anchor else {
        return None;
    };
    if record.profile_id != EXCEL_GRID_PROFILE_ID {
        return None;
    }
    let anchor_reference = excel_grid_reference_like_from_profile_record(record)?;
    let spill_reference = ReferenceLike::new(
        oxfunc_core::value::ReferenceKind::SpillAnchor,
        format!("{}#", anchor_reference.target()),
    );
    let dependencies = provider.runtime_dependencies_for_reference(&spill_reference);
    grid_extent_for_dependencies(&dependencies, bounds)
}

fn grid_single_cell_dependency(
    dependencies: BTreeSet<GridDependency>,
) -> Option<ExcelGridCellAddress> {
    let mut cells = dependencies
        .into_iter()
        .filter_map(|dependency| match dependency {
            GridDependency::Cell(address) => Some(address),
            GridDependency::Range(rect) if rect.row_count() == 1 && rect.col_count() == 1 => {
                Some(ExcelGridCellAddress::new(
                    rect.workbook_id,
                    rect.sheet_id,
                    rect.top_row,
                    rect.left_col,
                ))
            }
            _ => None,
        });
    let cell = cells.next()?;
    cells.next().is_none().then_some(cell)
}

fn grid_axis_visibility_dependencies_for_bound_expr<P>(
    expr: &BoundExpr,
    bounds: ExcelGridBounds,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    let mut dependencies = BTreeSet::new();
    collect_grid_axis_visibility_dependencies_for_bound_expr(
        expr,
        bounds,
        provider,
        &mut dependencies,
    );
    dependencies
}

fn collect_grid_axis_visibility_dependencies_for_bound_expr<P>(
    expr: &BoundExpr,
    bounds: ExcelGridBounds,
    provider: &P,
    dependencies: &mut BTreeSet<GridDependency>,
) where
    P: GridRuntimeTraceReferenceResolver,
{
    match expr {
        BoundExpr::ArrayLiteral(rows) => {
            for child in rows.iter().flatten() {
                collect_grid_axis_visibility_dependencies_for_bound_expr(
                    child,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Binary { left, right, .. } => {
            collect_grid_axis_visibility_dependencies_for_bound_expr(
                left,
                bounds,
                provider,
                dependencies,
            );
            collect_grid_axis_visibility_dependencies_for_bound_expr(
                right,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            collect_grid_axis_visibility_dependencies_for_bound_expr(
                expr,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
        } => {
            if grid_function_uses_axis_visibility(function_name) {
                for arg in args {
                    collect_grid_axis_visibility_dependencies_for_references(
                        arg,
                        bounds,
                        provider,
                        dependencies,
                    );
                }
            }
            for arg in args {
                collect_grid_axis_visibility_dependencies_for_bound_expr(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Invocation { callee, args } => {
            collect_grid_axis_visibility_dependencies_for_bound_expr(
                callee,
                bounds,
                provider,
                dependencies,
            );
            for arg in args {
                collect_grid_axis_visibility_dependencies_for_bound_expr(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Reference(_)
        | BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_) => {}
    }
}

fn collect_grid_axis_visibility_dependencies_for_references<P>(
    expr: &BoundExpr,
    bounds: ExcelGridBounds,
    provider: &P,
    dependencies: &mut BTreeSet<GridDependency>,
) where
    P: GridRuntimeTraceReferenceResolver,
{
    match expr {
        BoundExpr::ArrayLiteral(rows) => {
            for child in rows.iter().flatten() {
                collect_grid_axis_visibility_dependencies_for_references(
                    child,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Binary { left, right, .. } => {
            collect_grid_axis_visibility_dependencies_for_references(
                left,
                bounds,
                provider,
                dependencies,
            );
            collect_grid_axis_visibility_dependencies_for_references(
                right,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            collect_grid_axis_visibility_dependencies_for_references(
                expr,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::FunctionCall { args, .. } => {
            for arg in args {
                collect_grid_axis_visibility_dependencies_for_references(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Invocation { callee, args } => {
            collect_grid_axis_visibility_dependencies_for_references(
                callee,
                bounds,
                provider,
                dependencies,
            );
            for arg in args {
                collect_grid_axis_visibility_dependencies_for_references(
                    arg,
                    bounds,
                    provider,
                    dependencies,
                );
            }
        }
        BoundExpr::Reference(reference) => {
            collect_grid_axis_visibility_dependencies_for_reference_expr(
                reference,
                bounds,
                provider,
                dependencies,
            );
        }
        BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_) => {}
    }
}

fn collect_grid_axis_visibility_dependencies_for_reference_expr<P>(
    reference: &ReferenceExpr,
    bounds: ExcelGridBounds,
    provider: &P,
    dependencies: &mut BTreeSet<GridDependency>,
) where
    P: GridRuntimeTraceReferenceResolver,
{
    if let Some(extent) = grid_extent_for_reference_expr(reference, bounds, provider) {
        dependencies.insert(GridDependency::AxisVisibility(
            GridAxisVisibilityDependency::rows(extent.top_row, extent.bottom_row),
        ));
        return;
    }

    match reference {
        ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record)) => {
            dependencies.extend(grid_axis_visibility_dependencies_for_profile_record(
                record, bounds, provider,
            ));
        }
        ReferenceExpr::Spill { anchor } => {
            collect_grid_axis_visibility_dependencies_for_reference_expr(
                anchor,
                bounds,
                provider,
                dependencies,
            );
        }
        ReferenceExpr::Range { start, end }
        | ReferenceExpr::Union {
            left: start,
            right: end,
        }
        | ReferenceExpr::Intersection {
            left: start,
            right: end,
        } => {
            collect_grid_axis_visibility_dependencies_for_reference_expr(
                start,
                bounds,
                provider,
                dependencies,
            );
            collect_grid_axis_visibility_dependencies_for_reference_expr(
                end,
                bounds,
                provider,
                dependencies,
            );
        }
        ReferenceExpr::Atom(_) => {}
    }
}

fn grid_extent_for_reference_expr<P>(
    reference: &ReferenceExpr,
    bounds: ExcelGridBounds,
    provider: &P,
) -> Option<GridRect>
where
    P: GridRuntimeTraceReferenceResolver,
{
    match reference {
        ReferenceExpr::Atom(NormalizedReference::ProfileSymbolic(record)) => {
            grid_extent_for_profile_record(record, bounds, provider)
        }
        ReferenceExpr::Range { start, end } => {
            let start = grid_extent_for_reference_expr(start, bounds, provider)?;
            let end = grid_extent_for_reference_expr(end, bounds, provider)?;
            grid_rect_union(&start, &end, bounds)
        }
        // G5: resolve the spill reference to its full CURRENT realized
        // extent (via the provider's spill-rect lookup), not the anchor
        // cell's 1x1 extent. Falls back to the anchor's own extent when the
        // spill cannot be resolved as a spill-anchor reference (e.g. the
        // anchor's display text does not round-trip through the provider's
        // textual spill-anchor path — see the F5 comment in
        // reference_engine.rs), so a still-usable (if narrower) extent is
        // preferred over none.
        ReferenceExpr::Spill { anchor } => grid_spill_extent_for_anchor(anchor, bounds, provider)
            .or_else(|| grid_extent_for_reference_expr(anchor, bounds, provider)),
        ReferenceExpr::Atom(_)
        | ReferenceExpr::Union { .. }
        | ReferenceExpr::Intersection { .. } => None,
    }
}

fn grid_extent_for_profile_record<P>(
    record: &ProfileReferenceRecord,
    bounds: ExcelGridBounds,
    provider: &P,
) -> Option<GridRect>
where
    P: GridRuntimeTraceReferenceResolver,
{
    if record.profile_id != EXCEL_GRID_PROFILE_ID {
        return None;
    }
    let reference = excel_grid_reference_like_from_profile_record(record)?;
    let dependencies = provider.runtime_dependencies_for_reference(&reference);
    grid_extent_for_dependencies(&dependencies, bounds)
}

fn grid_axis_visibility_dependencies_for_profile_record<P>(
    record: &ProfileReferenceRecord,
    bounds: ExcelGridBounds,
    provider: &P,
) -> BTreeSet<GridDependency>
where
    P: GridRuntimeTraceReferenceResolver,
{
    if record.profile_id != EXCEL_GRID_PROFILE_ID {
        return BTreeSet::new();
    }
    let resolved_dependencies = excel_grid_reference_like_from_profile_record(record)
        .map(|reference| provider.runtime_dependencies_for_reference(&reference))
        .unwrap_or_default();
    let decoded = decode_excel_grid_reference_payload(&record.profile_payload);
    let mut dependencies: BTreeSet<GridDependency> = match decoded {
        Some(ExcelGridReference::Cell { row, .. }) => {
            grid_axis_visibility_dependency_for_rows(row, row, bounds)
                .map(GridDependency::AxisVisibility)
                .into_iter()
                .collect()
        }
        Some(ExcelGridReference::Area {
            start_row, end_row, ..
        }) => grid_axis_visibility_dependency_for_rows(start_row, end_row, bounds)
            .map(GridDependency::AxisVisibility)
            .into_iter()
            .collect(),
        Some(ExcelGridReference::WholeRow {
            start_row, end_row, ..
        }) => grid_axis_visibility_dependency_for_rows(start_row, end_row, bounds)
            .map(GridDependency::AxisVisibility)
            .into_iter()
            .collect(),
        Some(ExcelGridReference::WholeColumn { .. }) => [GridDependency::AxisVisibility(
            GridAxisVisibilityDependency::rows(1, bounds.max_rows),
        )]
        .into_iter()
        .collect(),
        _ => BTreeSet::new(),
    };
    dependencies.extend(grid_axis_visibility_dependencies_for_resolved_dependencies(
        &resolved_dependencies,
        bounds,
    ));
    dependencies
}

fn grid_axis_visibility_dependencies_for_resolved_dependencies(
    resolved_dependencies: &BTreeSet<GridDependency>,
    bounds: ExcelGridBounds,
) -> BTreeSet<GridDependency> {
    resolved_dependencies
        .iter()
        .filter_map(|dependency| {
            let extent = grid_dependency_extent(dependency, bounds)?;
            Some(GridDependency::AxisVisibility(
                GridAxisVisibilityDependency::rows(extent.top_row, extent.bottom_row),
            ))
        })
        .collect()
}

fn grid_axis_visibility_dependency_for_rows(
    start_row: ExcelGridAxisRef,
    end_row: ExcelGridAxisRef,
    bounds: ExcelGridBounds,
) -> Option<GridAxisVisibilityDependency> {
    let start = grid_absolute_axis_index(start_row)?;
    let end = grid_absolute_axis_index(end_row)?;
    let first = start.min(end);
    let last = start.max(end);
    (bounds.contains_row(first) && bounds.contains_row(last))
        .then_some(GridAxisVisibilityDependency::rows(first, last))
}

fn grid_function_uses_axis_visibility(function_name: &str) -> bool {
    function_name.eq_ignore_ascii_case("SUBTOTAL")
        || function_name.eq_ignore_ascii_case("AGGREGATE")
}

/// G5(b): whether `bound`'s tree contains a hidden-row-sensitive aggregate
/// call (SUBTOTAL/AGGREGATE) anywhere, regardless of whether its arguments
/// are statically resolvable references. The structural feeder
/// (`grid_axis_visibility_dependencies_for_bound_expr`) only installs
/// AxisVisibility dependencies for arguments it can walk as reference
/// expressions in the bound tree; a text-realized argument (e.g.
/// `SUBTOTAL(109,INDIRECT(C1))`) contributes nothing there because
/// `INDIRECT`'s own argument (`C1`) is not the realized target. This
/// predicate is checked once per evaluation so the runtime-trace feeder
/// (`grid_axis_visibility_overlay_dependencies_from_trace`) knows to derive
/// AxisVisibility overlay dependencies from whatever the trace actually
/// realized at runtime.
#[must_use]
pub(super) fn grid_bound_formula_contains_hidden_sensitive_function(bound: &BoundFormula) -> bool {
    grid_bound_expr_contains_hidden_sensitive_function(&bound.root)
}

fn grid_bound_expr_contains_hidden_sensitive_function(expr: &BoundExpr) -> bool {
    match expr {
        BoundExpr::ArrayLiteral(rows) => rows
            .iter()
            .flatten()
            .any(grid_bound_expr_contains_hidden_sensitive_function),
        BoundExpr::Binary { left, right, .. } => {
            grid_bound_expr_contains_hidden_sensitive_function(left)
                || grid_bound_expr_contains_hidden_sensitive_function(right)
        }
        BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
            grid_bound_expr_contains_hidden_sensitive_function(expr)
        }
        BoundExpr::FunctionCall {
            function_name,
            args,
        } => {
            grid_function_uses_axis_visibility(function_name)
                || args
                    .iter()
                    .any(grid_bound_expr_contains_hidden_sensitive_function)
        }
        BoundExpr::Invocation { callee, args } => {
            grid_bound_expr_contains_hidden_sensitive_function(callee)
                || args
                    .iter()
                    .any(grid_bound_expr_contains_hidden_sensitive_function)
        }
        BoundExpr::Reference(_)
        | BoundExpr::NumberLiteral(_)
        | BoundExpr::StringLiteral(_)
        | BoundExpr::LogicalLiteral(_)
        | BoundExpr::OmittedArgument
        | BoundExpr::HelperParameterName(_)
        | BoundExpr::HelperOptionalParameterName(_)
        | BoundExpr::HostReference(_)
        | BoundExpr::HostStructuralSelector(_)
        | BoundExpr::HostReferenceCollection(_) => false,
    }
}

/// G5(b): derive AxisVisibility overlay dependencies from `realized_dependencies`'
/// Cell/Range members. Called at overlay-replacement time (after evaluation,
/// once the trace's runtime-realized dependencies are known) when the
/// formula's bound tree contains a hidden-row-sensitive aggregate call, so a
/// text-realized target (`INDIRECT`/`OFFSET`) that resolved to a Cell or
/// Range dependency gets the same hidden-row dirtying coverage a
/// statically-resolvable argument gets from the structural feeder.
#[must_use]
pub(super) fn grid_axis_visibility_overlay_dependencies_from_trace(
    trace: &GridRuntimeDependencyTrace,
    bounds: ExcelGridBounds,
) -> BTreeSet<GridDependency> {
    trace
        .realized_dependencies
        .iter()
        .filter(|dependency| {
            matches!(
                dependency,
                GridDependency::Cell(_) | GridDependency::Range(_)
            )
        })
        .filter_map(|dependency| {
            let extent = grid_dependency_extent(dependency, bounds)?;
            Some(GridDependency::AxisVisibility(
                GridAxisVisibilityDependency::rows(extent.top_row, extent.bottom_row),
            ))
        })
        .collect()
}

pub(super) fn grid_extent_for_dependencies(
    dependencies: &BTreeSet<GridDependency>,
    bounds: ExcelGridBounds,
) -> Option<GridRect> {
    let mut extents = dependencies
        .iter()
        .filter_map(|dependency| grid_dependency_extent(dependency, bounds));
    let first = extents.next()?;
    extents.try_fold(first, |acc, extent| grid_rect_union(&acc, &extent, bounds))
}

fn grid_rect_union(left: &GridRect, right: &GridRect, bounds: ExcelGridBounds) -> Option<GridRect> {
    if left.workbook_id != right.workbook_id || left.sheet_id != right.sheet_id {
        return None;
    }
    GridRect::new(
        left.workbook_id.clone(),
        left.sheet_id.clone(),
        left.top_row.min(right.top_row),
        left.left_col.min(right.left_col),
        left.bottom_row.max(right.bottom_row),
        left.right_col.max(right.right_col),
        bounds,
    )
    .ok()
}

pub(super) fn grid_dependency_extent(
    dependency: &GridDependency,
    bounds: ExcelGridBounds,
) -> Option<GridRect> {
    match dependency {
        GridDependency::Cell(address) => GridRect::new(
            address.workbook_id.clone(),
            address.sheet_id.clone(),
            address.row,
            address.col,
            address.row,
            address.col,
            bounds,
        )
        .ok(),
        GridDependency::Range(rect) => Some(rect.clone()),
        GridDependency::Name(dependency) => Some(dependency.extent.clone()),
        GridDependency::NameIdentity(_) => None,
        GridDependency::Table(dependency) => Some(dependency.extent.clone()),
        GridDependency::TableIdentity(_) => None,
        GridDependency::ReferenceMetadata(_) => None,
        _ => None,
    }
}

fn explicit_table_name_from_structured_reference(source_text: &str) -> Option<&str> {
    let local = source_text
        .rsplit_once('!')
        .map_or(source_text, |(_, tail)| tail);
    let bracket = local.find('[')?;
    let table_name = local[..bracket].trim();
    (!table_name.is_empty()).then_some(table_name)
}

fn grid_spill_anchor_address_from_payload(
    workbook_id: &str,
    sheet_id: &str,
    anchor_key: &str,
    source_text: &str,
) -> Option<ExcelGridCellAddress> {
    grid_spill_anchor_address_from_text(workbook_id, sheet_id, anchor_key)
        .or_else(|| grid_spill_anchor_address_from_text(workbook_id, sheet_id, source_text))
}

fn grid_spill_anchor_address_from_text(
    workbook_id: &str,
    sheet_id: &str,
    source_text: &str,
) -> Option<ExcelGridCellAddress> {
    let target = source_text
        .trim()
        .strip_suffix('#')
        .unwrap_or(source_text)
        .trim();
    let target = target
        .rsplit_once('!')
        .map_or(target, |(_, local)| local)
        .trim();
    let (row, col) = grid_a1_cell_address_from_text(target)?;
    Some(ExcelGridCellAddress::new(
        workbook_id.to_string(),
        sheet_id.to_string(),
        row,
        col,
    ))
}

fn grid_a1_cell_address_from_text(text: &str) -> Option<(u32, u32)> {
    let text = text.trim();
    let mut chars = text.char_indices().peekable();
    if matches!(chars.peek(), Some((_, '$'))) {
        chars.next();
    }
    let col_start = chars.peek().map(|(index, _)| *index)?;
    let mut col_end = col_start;
    while let Some((index, ch)) = chars.peek().copied() {
        if !ch.is_ascii_alphabetic() {
            break;
        }
        col_end = index + ch.len_utf8();
        chars.next();
    }
    if col_end == col_start {
        return None;
    }
    if matches!(chars.peek(), Some((_, '$'))) {
        chars.next();
    }
    let row_start = chars.peek().map(|(index, _)| *index)?;
    let mut row_end = row_start;
    while let Some((index, ch)) = chars.peek().copied() {
        if !ch.is_ascii_digit() {
            break;
        }
        row_end = index + ch.len_utf8();
        chars.next();
    }
    if row_end == row_start || chars.peek().is_some() {
        return None;
    }
    let col = grid_column_to_index(&text[col_start..col_end])?;
    let row = text[row_start..row_end].parse::<u32>().ok()?;
    Some((row, col))
}

fn grid_column_to_index(text: &str) -> Option<u32> {
    let mut index = 0_u32;
    for ch in text.chars() {
        let upper = ch.to_ascii_uppercase();
        if !upper.is_ascii_uppercase() {
            return None;
        }
        index = index.checked_mul(26)?;
        index = index.checked_add(u32::from(upper) - u32::from('A') + 1)?;
    }
    (index > 0).then_some(index)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridRuntimeResolutionEffect {
    Dereferenced {
        dependencies: BTreeSet<GridDependency>,
    },
    Enumerated {
        dependencies: BTreeSet<GridDependency>,
    },
    TextResolved {
        text: String,
        dependencies: BTreeSet<GridDependency>,
    },
    Transformed {
        dependencies: BTreeSet<GridDependency>,
    },
    Composed {
        dependencies: BTreeSet<GridDependency>,
    },
}

pub(super) trait GridRuntimeTraceReferenceResolver: ReferenceSystemProvider {
    fn runtime_trace_bounds(&self) -> ExcelGridBounds;

    fn runtime_dependencies_for_reference(
        &self,
        reference: &ReferenceLike,
    ) -> BTreeSet<GridDependency>;

    fn runtime_name_dependency_keys_for_scope(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        name: &str,
    ) -> Vec<String>;

    fn runtime_name_dependency_keys_for_text(&self, text: &str) -> Vec<String>;

    /// Discriminated counterpart of
    /// [`Self::runtime_name_dependency_keys_for_scope`] that reports whether
    /// `name` resolved as a real defined-name namespace entry or only as a
    /// caller-local table column (a local structured reference such as
    /// `[Amount]` that bound down the Name path). See D4.
    fn runtime_name_dependency_resolution_for_scope(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        name: &str,
    ) -> GridNameDependencyScopeResolution;

    /// Discriminated counterpart of
    /// [`Self::runtime_name_dependency_keys_for_text`], used by runtime text
    /// resolution (`INDIRECT`) so a caller-local table-column resolution
    /// records a Table/TableIdentity dependency instead of a phantom Name
    /// identity. See D4.
    fn runtime_name_dependency_resolution_for_text(
        &self,
        text: &str,
    ) -> GridNameDependencyScopeResolution;
}

impl GridRuntimeTraceReferenceResolver for ExcelGridReferenceSystemProvider<'_> {
    fn runtime_trace_bounds(&self) -> ExcelGridBounds {
        self.bounds()
    }

    fn runtime_dependencies_for_reference(
        &self,
        reference: &ReferenceLike,
    ) -> BTreeSet<GridDependency> {
        self.resolved_rects_for_reference(reference)
            .map(grid_dependencies_for_rects)
            .unwrap_or_default()
    }

    fn runtime_name_dependency_keys_for_scope(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        name: &str,
    ) -> Vec<String> {
        self.defined_name_dependency_key_for_scope(workbook_id, sheet_id, name)
            .map(|key| vec![key])
            .unwrap_or_else(|| {
                self.defined_name_candidate_dependency_keys_for_scope(workbook_id, sheet_id, name)
            })
    }

    fn runtime_name_dependency_keys_for_text(&self, text: &str) -> Vec<String> {
        let (sheet_id, name) = split_provider_text_sheet_qualifier(text, self.sheet_id());
        self.runtime_name_dependency_keys_for_scope(self.workbook_id(), sheet_id, name)
    }

    fn runtime_name_dependency_resolution_for_scope(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        name: &str,
    ) -> GridNameDependencyScopeResolution {
        self.defined_name_dependency_resolution_for_scope(workbook_id, sheet_id, name)
    }

    fn runtime_name_dependency_resolution_for_text(
        &self,
        text: &str,
    ) -> GridNameDependencyScopeResolution {
        let (sheet_id, name) = split_provider_text_sheet_qualifier(text, self.sheet_id());
        self.runtime_name_dependency_resolution_for_scope(self.workbook_id(), sheet_id, name)
    }
}

impl GridRuntimeTraceReferenceResolver for GridOptimizedReferenceSystemProvider<'_> {
    fn runtime_trace_bounds(&self) -> ExcelGridBounds {
        self.bounds()
    }

    fn runtime_dependencies_for_reference(
        &self,
        reference: &ReferenceLike,
    ) -> BTreeSet<GridDependency> {
        self.resolved_rects_for_reference(reference)
            .map(grid_dependencies_for_rects)
            .unwrap_or_default()
    }

    fn runtime_name_dependency_keys_for_scope(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        name: &str,
    ) -> Vec<String> {
        self.defined_name_dependency_key_for_scope(workbook_id, sheet_id, name)
            .map(|key| vec![key])
            .unwrap_or_else(|| {
                self.defined_name_candidate_dependency_keys_for_scope(workbook_id, sheet_id, name)
            })
    }

    fn runtime_name_dependency_keys_for_text(&self, text: &str) -> Vec<String> {
        let (sheet_id, name) = split_provider_text_sheet_qualifier(text, self.sheet_id());
        self.runtime_name_dependency_keys_for_scope(self.workbook_id(), sheet_id, name)
    }

    fn runtime_name_dependency_resolution_for_scope(
        &self,
        workbook_id: &str,
        sheet_id: &str,
        name: &str,
    ) -> GridNameDependencyScopeResolution {
        self.defined_name_dependency_resolution_for_scope(workbook_id, sheet_id, name)
    }

    fn runtime_name_dependency_resolution_for_text(
        &self,
        text: &str,
    ) -> GridNameDependencyScopeResolution {
        let (sheet_id, name) = split_provider_text_sheet_qualifier(text, self.sheet_id());
        self.runtime_name_dependency_resolution_for_scope(self.workbook_id(), sheet_id, name)
    }
}

/// W062 R4.14 (D3 §10 constraint 1, "per-evaluation trace buffers"): the trace
/// is accumulated as a *value-returned owned buffer*, not through a borrowing
/// interior-mutability cell.
///
/// The upstream `ReferenceSystemProvider` trait (OxFunc `resolver.rs`) takes
/// `&self` on every callback and OxFml's evaluator holds the provider as a
/// shared `&dyn ReferenceSystemProvider`, so the accumulator cannot be threaded
/// as `&mut` through the trait and cannot be reconstructed after the call
/// (`resolution_effects` order is load-bearing — `host_info.rs` reads the
/// LITERAL LAST effect). The scope-preserving form (no trait/API break) is a
/// `Cell<GridRuntimeDependencyTrace>`: each record step `take`s the owned buffer
/// out, mutates the value, and `set`s the owned value back — the trace is only
/// ever moved by value, never aliased by a live borrow. This eliminates the
/// named `RefCell` blocker (`runtime_trace.rs:2111`) and its reentrant-borrow
/// panic surface.
///
/// Concurrency disposition (D3 §10 constraints 1 & 4): the provider is
/// constructed per cell evaluation (`new(inner)` / `finish()` bracketing) and is
/// never shared across nodes or threads. `Cell<GridRuntimeDependencyTrace>` is
/// `Send` (`GridRuntimeDependencyTrace: Send`) and the accumulation is
/// evaluation-local — a future staged-concurrency executor (W053) evaluates
/// distinct cells on distinct threads, each with its own provider and its own
/// buffer, so no interior-mutable state is ever reachable across an evaluation
/// step. `Cell` (unlike `RefCell`) is `!Sync` but panic-free, which is exactly
/// the single-threaded-per-node contract the sequential worklist already holds.
pub(super) struct GridTracingReferenceSystemProvider<'a, P> {
    inner: &'a P,
    trace: Cell<GridRuntimeDependencyTrace>,
}

impl<'a, P> GridTracingReferenceSystemProvider<'a, P>
where
    P: GridRuntimeTraceReferenceResolver,
{
    #[must_use]
    pub fn new(inner: &'a P) -> Self {
        Self {
            inner,
            trace: Cell::new(GridRuntimeDependencyTrace::default()),
        }
    }

    #[must_use]
    pub fn finish(self) -> GridRuntimeDependencyTrace {
        self.trace.into_inner()
    }

    /// Mutate the owned trace buffer by value: take it out of the `Cell`,
    /// apply `mutate`, and set the owned value back. No borrow of the buffer
    /// escapes this call, so the `&self` trait callbacks accumulate without any
    /// aliasing interior mutability.
    fn with_trace(&self, mutate: impl FnOnce(&mut GridRuntimeDependencyTrace)) {
        let mut trace = self.trace.take();
        mutate(&mut trace);
        self.trace.set(trace);
    }

    fn record(
        &self,
        dependencies: BTreeSet<GridDependency>,
        effect: impl FnOnce(BTreeSet<GridDependency>) -> GridRuntimeResolutionEffect,
    ) {
        if dependencies.is_empty() {
            return;
        }
        self.with_trace(|trace| {
            trace
                .realized_dependencies
                .extend(dependencies.iter().cloned());
            trace.resolution_effects.push(effect(dependencies));
        });
    }

    /// Like [`Self::record`], but always pushes the resolution effect even
    /// when `dependencies` is empty. Used by the failed-`resolve_text` path:
    /// a text resolution that fails against genuinely invalid reference text
    /// (not a name/table candidate) has no identity dependencies to carry,
    /// but the LITERAL LAST resolution effect must still reflect that the
    /// terminal step was a (failed) text resolution — not silently vanish
    /// and leave an earlier selector-cell `Dereferenced` effect as the
    /// apparent last effect. See `dynamic_defined_name_extent_from_trace`
    /// (host_info.rs), which relies on the last effect's discriminant to
    /// decide realized-vs-unresolved.
    fn record_terminal(
        &self,
        dependencies: BTreeSet<GridDependency>,
        effect: impl FnOnce(BTreeSet<GridDependency>) -> GridRuntimeResolutionEffect,
    ) {
        self.with_trace(|trace| {
            trace
                .realized_dependencies
                .extend(dependencies.iter().cloned());
            trace.resolution_effects.push(effect(dependencies));
        });
    }

    fn dependencies_for_reference(&self, reference: &ReferenceLike) -> BTreeSet<GridDependency> {
        self.inner.runtime_dependencies_for_reference(reference)
    }

    fn identity_dependencies_for_text(
        &self,
        text: &str,
        resolved_dependencies: &BTreeSet<GridDependency>,
    ) -> BTreeSet<GridDependency> {
        runtime_identity_dependencies_for_text(
            text,
            resolved_dependencies,
            self.inner.runtime_trace_bounds(),
            self.inner.runtime_name_dependency_keys_for_text(text),
            self.inner.runtime_name_dependency_resolution_for_text(text),
        )
    }
}

impl<P> ReferenceSystemProvider for GridTracingReferenceSystemProvider<'_, P>
where
    P: GridRuntimeTraceReferenceResolver,
{
    fn capabilities(&self) -> ReferenceSystemCapabilities {
        self.inner.capabilities()
    }

    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<CalcValue, ReferenceResolutionError> {
        let result = self.inner.dereference(request);
        if result.is_ok() {
            let dependencies = self.dependencies_for_reference(&request.reference);
            self.record(dependencies, |dependencies| {
                GridRuntimeResolutionEffect::Dereferenced { dependencies }
            });
        }
        result
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        let result = self.inner.enumerate_values(request);
        if matches!(result, Ok(Some(_))) {
            let dependencies = self.dependencies_for_reference(&request.reference);
            self.record(dependencies, |dependencies| {
                GridRuntimeResolutionEffect::Enumerated { dependencies }
            });
        }
        result
    }

    fn resolve_text(
        &self,
        request: &ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        let result = self.inner.resolve_text(request);
        if let Ok(reference) = &result {
            let resolved_dependencies = self.dependencies_for_reference(reference);
            let mut dependencies = resolved_dependencies.clone();
            dependencies
                .extend(self.identity_dependencies_for_text(&request.text, &resolved_dependencies));
            let text = request.text.clone();
            self.record(dependencies, |dependencies| {
                GridRuntimeResolutionEffect::TextResolved { text, dependencies }
            });
        } else {
            let dependencies = runtime_name_identity_dependency_for_text(
                &request.text,
                self.inner.runtime_trace_bounds(),
                self.inner
                    .runtime_name_dependency_keys_for_text(&request.text),
                self.inner
                    .runtime_name_dependency_resolution_for_text(&request.text),
            );
            let text = request.text.clone();
            // E3 fix: record the terminal failed text-resolution effect even
            // when `dependencies` is empty (invalid reference text that is
            // not a name/table candidate carries no identity dependencies).
            // Otherwise the trace's last effect stays an earlier selector
            // `Dereferenced` effect (e.g. reading C1's own value before
            // resolving its text), and `dynamic_defined_name_extent_from_trace`
            // would wrongly realize the selector cell as the name's extent
            // instead of reporting unresolved.
            self.record_terminal(dependencies, |dependencies| {
                GridRuntimeResolutionEffect::TextResolved { text, dependencies }
            });
        }
        result
    }

    fn facts(
        &self,
        request: &ReferenceFactsRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError> {
        self.inner.facts(request)
    }

    fn transform_reference(
        &self,
        request: &EvalTransformRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        let result = self.inner.transform_reference(request);
        if let Ok(reference) = &result {
            let dependencies = self.dependencies_for_reference(reference);
            self.record(dependencies, |dependencies| {
                GridRuntimeResolutionEffect::Transformed { dependencies }
            });
        }
        result
    }

    fn compose_references(
        &self,
        request: &ReferenceComposeRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        let result = self.inner.compose_references(request);
        if let Ok(reference) = &result {
            let dependencies = self.dependencies_for_reference(reference);
            self.record(dependencies, |dependencies| {
                GridRuntimeResolutionEffect::Composed { dependencies }
            });
        }
        result
    }

    fn caller_context(&self) -> Option<CallerContext> {
        self.inner.caller_context()
    }
}

fn grid_dependencies_for_rects(rects: Vec<GridRect>) -> BTreeSet<GridDependency> {
    rects.into_iter().map(grid_dependency_for_rect).collect()
}

fn grid_dependency_for_rect(rect: GridRect) -> GridDependency {
    if rect.row_count() == 1 && rect.col_count() == 1 {
        return GridDependency::Cell(ExcelGridCellAddress::new(
            rect.workbook_id,
            rect.sheet_id,
            rect.top_row,
            rect.left_col,
        ));
    }
    GridDependency::Range(rect)
}

fn grid_dependency_covers(structural: &GridDependency, candidate: &GridDependency) -> bool {
    match (structural, candidate) {
        (GridDependency::Cell(lhs), GridDependency::Cell(rhs)) => lhs == rhs,
        (GridDependency::Range(lhs), GridDependency::Cell(rhs)) => lhs.contains(rhs),
        (GridDependency::Range(lhs), GridDependency::Range(rhs)) => {
            grid_rect_contains_rect(lhs, rhs)
        }
        (GridDependency::Name(lhs), GridDependency::Cell(rhs)) => lhs.extent.contains(rhs),
        (GridDependency::Name(lhs), GridDependency::Range(rhs)) => {
            grid_rect_contains_rect(&lhs.extent, rhs)
        }
        (GridDependency::NameIdentity(lhs), GridDependency::NameIdentity(rhs)) => lhs == rhs,
        (GridDependency::Table(lhs), GridDependency::Cell(rhs)) => lhs.extent.contains(rhs),
        (GridDependency::Table(lhs), GridDependency::Range(rhs)) => {
            grid_rect_contains_rect(&lhs.extent, rhs)
        }
        (GridDependency::TableIdentity(lhs), GridDependency::TableIdentity(rhs)) => lhs == rhs,
        // G6: without these arms, a whole-axis formula's runtime
        // enumeration (which installs the same span as a
        // `GridDependency::AxisValue` structural dependency) never gets
        // filtered as structurally known by
        // `overlay_dependencies_excluding_structural`, so it reinstalls its
        // runtime enumeration as a permanent (benign but redundant) overlay
        // `Range` edge on every evaluation.
        (GridDependency::AxisValue(lhs), GridDependency::Cell(rhs)) => {
            axis_value_dependency_contains_address(lhs, rhs)
        }
        (GridDependency::AxisValue(lhs), GridDependency::Range(rhs)) => {
            axis_value_dependency_contains_rect(lhs, rhs)
        }
        (GridDependency::ReferenceMetadata(lhs), GridDependency::ReferenceMetadata(rhs)) => {
            grid_dependency_covers(lhs, rhs)
        }
        _ => structural == candidate,
    }
}

fn grid_rect_contains_rect(lhs: &GridRect, rhs: &GridRect) -> bool {
    lhs.workbook_id == rhs.workbook_id
        && lhs.sheet_id == rhs.sheet_id
        && lhs.top_row <= rhs.top_row
        && lhs.left_col <= rhs.left_col
        && lhs.bottom_row >= rhs.bottom_row
        && lhs.right_col >= rhs.right_col
}
