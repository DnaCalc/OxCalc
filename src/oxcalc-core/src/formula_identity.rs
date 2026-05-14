#![forbid(unsafe_code)]

//! Formula plan-template identity keys derived from OxFml public artifacts.

use std::collections::BTreeMap;
use std::fmt::{Display, Write};

use oxfml_core::binding::{BoundExpr, BoundFormula, NormalizedReference, ReferenceExpr};
use oxfml_core::semantics::{FunctionAvailabilitySummary, FunctionPlanBinding, SemanticPlan};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShapeKey(String);

impl ShapeKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for ShapeKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DispatchSkeletonKey(String);

impl DispatchSkeletonKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for DispatchSkeletonKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PlanTemplateKey(String);

impl PlanTemplateKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for PlanTemplateKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedFormulaIdentityKeys {
    pub shape_key: ShapeKey,
    pub dispatch_skeleton_key: DispatchSkeletonKey,
    pub plan_template_key: PlanTemplateKey,
}

#[must_use]
pub fn derive_prepared_formula_identity_keys(
    bound_formula: &BoundFormula,
    semantic_plan: &SemanticPlan,
) -> PreparedFormulaIdentityKeys {
    let shape_input = shape_key_input(bound_formula);
    let shape_key = ShapeKey(fingerprint("shape", &shape_input));

    let dispatch_input = dispatch_skeleton_key_input(&shape_key, semantic_plan);
    let dispatch_skeleton_key =
        DispatchSkeletonKey(fingerprint("dispatch_skeleton", &dispatch_input));

    let plan_input = plan_template_key_input(&dispatch_skeleton_key, semantic_plan);
    let plan_template_key = PlanTemplateKey(fingerprint("plan_template", &plan_input));

    PreparedFormulaIdentityKeys {
        shape_key,
        dispatch_skeleton_key,
        plan_template_key,
    }
}

fn shape_key_input(bound_formula: &BoundFormula) -> String {
    let mut state = ShapeCanonicalizer::default();
    let mut input = String::from("oxcalc.shape.v1;");
    write!(
        input,
        "root_grouped={};",
        bound_formula.root_expression_is_grouped
    )
    .expect("write to string");
    state.push_expr_shape(&bound_formula.root, &mut input);
    input
}

fn dispatch_skeleton_key_input(shape_key: &ShapeKey, semantic_plan: &SemanticPlan) -> String {
    let mut input = String::from("oxcalc.dispatch_skeleton.v1;");
    write!(
        input,
        "shape_key={};oxfunc_catalog_identity={};library_context_snapshot_ref={:?};",
        shape_key.as_str(),
        semantic_plan.oxfunc_catalog_identity,
        semantic_plan.library_context_snapshot_ref
    )
    .expect("write to string");
    for binding in &semantic_plan.function_bindings {
        push_function_binding(binding, &mut input);
    }
    for summary in &semantic_plan.availability_summaries {
        push_availability_summary(summary, &mut input);
    }
    input
}

fn plan_template_key_input(
    dispatch_skeleton_key: &DispatchSkeletonKey,
    semantic_plan: &SemanticPlan,
) -> String {
    let mut input = String::from("oxcalc.plan_template.v1;");
    write!(
        input,
        "dispatch_skeleton_key={};locale_profile={:?};date_system={:?};format_profile={:?};",
        dispatch_skeleton_key.as_str(),
        semantic_plan.locale_profile,
        semantic_plan.date_system,
        semantic_plan.format_profile
    )
    .expect("write to string");
    write!(
        input,
        "evaluation_requirements={:?};execution_profile={:?};helper_profile={:?};capability_requirements={:?};",
        semantic_plan.evaluation_requirements,
        semantic_plan.execution_profile,
        semantic_plan.helper_profile,
        semantic_plan.capability_requirements
    )
    .expect("write to string");
    for diagnostic in &semantic_plan.diagnostics {
        write!(
            input,
            "semantic_diagnostic(code={};function={:?};worksheet_error={:?});",
            diagnostic.code, diagnostic.function_name, diagnostic.worksheet_error_class
        )
        .expect("write to string");
    }
    input
}

fn push_function_binding(binding: &FunctionPlanBinding, input: &mut String) {
    write!(
        input,
        "function_binding(name={};id={};argc={};arity={}..{};arg_prep={:?};fec={:?};surface_fec={:?};host={:?};thread={:?};volatility={:?};determinism={:?});",
        binding.function_name.to_ascii_uppercase(),
        binding.function_id,
        binding.arg_count,
        binding.arity_min,
        binding.arity_max,
        binding.arg_preparation_profile,
        binding.fec_dependency_profile,
        binding.surface_fec_dependency_profile,
        binding.host_interaction,
        binding.thread_safety,
        binding.volatility,
        binding.determinism
    )
    .expect("write to string");
}

fn push_availability_summary(summary: &FunctionAvailabilitySummary, input: &mut String) {
    write!(
        input,
        "availability(surface={};canonical={:?};stable={:?};parse={:?};semantic={:?};runtime={:?};post={:?};source={:?};special={:?};admission={:?};prep_owner={:?};runtime_boundary={:?};contract={:?});",
        summary.surface_name.to_ascii_uppercase(),
        summary.canonical_id,
        summary.surface_stable_id,
        summary.parse_bind_state,
        summary.semantic_plan_state,
        summary.runtime_capability_state,
        summary.post_dispatch_state,
        summary.registration_source_kind,
        summary.special_interface_kind,
        summary.admission_interface_kind,
        summary.preparation_owner,
        summary.runtime_boundary_kind,
        summary.interface_contract_ref
    )
    .expect("write to string");
}

#[derive(Default)]
struct ShapeCanonicalizer {
    helper_slots: BTreeMap<String, usize>,
}

impl ShapeCanonicalizer {
    fn push_expr_shape(&mut self, expr: &BoundExpr, input: &mut String) {
        match expr {
            BoundExpr::NumberLiteral(_) => input.push_str("number_hole;"),
            BoundExpr::StringLiteral(_) => input.push_str("text_hole;"),
            BoundExpr::LogicalLiteral(_) => input.push_str("logical_hole;"),
            BoundExpr::ArrayLiteral(rows) => {
                write!(input, "array(rows={};", rows.len()).expect("write to string");
                for row in rows {
                    write!(input, "row(cols={};", row.len()).expect("write to string");
                    for value in row {
                        self.push_expr_shape(value, input);
                    }
                    input.push_str(");");
                }
                input.push_str(");");
            }
            BoundExpr::OmittedArgument => input.push_str("omitted_hole;"),
            BoundExpr::HelperParameterName(name) => {
                let slot = self.helper_slot(name);
                write!(input, "helper_parameter(slot={slot});").expect("write to string");
            }
            BoundExpr::HelperOptionalParameterName(name) => {
                let slot = self.helper_slot(name);
                write!(input, "helper_optional_parameter(slot={slot});").expect("write to string");
            }
            BoundExpr::Binary { op, left, right } => {
                write!(input, "binary(op={op:?};").expect("write to string");
                self.push_expr_shape(left, input);
                self.push_expr_shape(right, input);
                input.push_str(");");
            }
            BoundExpr::Unary { op, expr } => {
                write!(input, "unary(op={op:?};").expect("write to string");
                self.push_expr_shape(expr, input);
                input.push_str(");");
            }
            BoundExpr::FunctionCall {
                function_name,
                args,
            } => {
                write!(
                    input,
                    "function_call(posture={};argc={};",
                    function_shape_posture(function_name),
                    args.len()
                )
                .expect("write to string");
                for arg in args {
                    self.push_expr_shape(arg, input);
                }
                input.push_str(");");
            }
            BoundExpr::Invocation { callee, args } => {
                write!(input, "invocation(argc={};", args.len()).expect("write to string");
                self.push_expr_shape(callee, input);
                for arg in args {
                    self.push_expr_shape(arg, input);
                }
                input.push_str(");");
            }
            BoundExpr::Reference(reference) => push_reference_shape(reference, input),
            BoundExpr::ImplicitIntersection(expr) => {
                input.push_str("implicit_intersection(");
                self.push_expr_shape(expr, input);
                input.push_str(");");
            }
        }
    }

    fn helper_slot(&mut self, name: &str) -> usize {
        if let Some(slot) = self.helper_slots.get(name) {
            return *slot;
        }
        let slot = self.helper_slots.len();
        self.helper_slots.insert(name.to_string(), slot);
        slot
    }
}

fn function_shape_posture(function_name: &str) -> &'static str {
    match function_name.to_ascii_uppercase().as_str() {
        "IF" | "IFS" | "CHOOSE" | "SWITCH" => "branch_lazy",
        "IFERROR" | "IFNA" => "fallback_lazy",
        "LET" => "helper_let",
        "LAMBDA" => "lambda_literal",
        _ => "eager_or_unknown",
    }
}

fn push_reference_shape(reference: &ReferenceExpr, input: &mut String) {
    match reference {
        ReferenceExpr::Atom(normalized) => push_normalized_reference_shape(normalized, input),
        ReferenceExpr::Range { start, end } => {
            input.push_str("reference_range(");
            push_reference_shape(start, input);
            push_reference_shape(end, input);
            input.push_str(");");
        }
        ReferenceExpr::Union { left, right } => {
            input.push_str("reference_union(");
            push_reference_shape(left, input);
            push_reference_shape(right, input);
            input.push_str(");");
        }
        ReferenceExpr::Intersection { left, right } => {
            input.push_str("reference_intersection(");
            push_reference_shape(left, input);
            push_reference_shape(right, input);
            input.push_str(");");
        }
        ReferenceExpr::Spill { anchor } => {
            input.push_str("reference_spill(");
            push_reference_shape(anchor, input);
            input.push_str(");");
        }
    }
}

fn push_normalized_reference_shape(reference: &NormalizedReference, input: &mut String) {
    match reference {
        NormalizedReference::Cell(cell) => write!(
            input,
            "reference_cell(row_abs={};col_abs={};caller_anchor={});",
            cell.address_mode.row_absolute,
            cell.address_mode.col_absolute,
            cell.caller_anchor_used
        )
        .expect("write to string"),
        NormalizedReference::Area(area) => write!(
            input,
            "reference_area(height={};width={};row_abs={};col_abs={};caller_anchor={});",
            area.height,
            area.width,
            area.address_mode.row_absolute,
            area.address_mode.col_absolute,
            area.caller_anchor_used
        )
        .expect("write to string"),
        NormalizedReference::WholeRow(row) => write!(
            input,
            "reference_whole_row(count={};row_abs={};col_abs={});",
            row.row_count, row.address_mode.row_absolute, row.address_mode.col_absolute
        )
        .expect("write to string"),
        NormalizedReference::WholeColumn(column) => write!(
            input,
            "reference_whole_column(count={};row_abs={};col_abs={});",
            column.col_count, column.address_mode.row_absolute, column.address_mode.col_absolute
        )
        .expect("write to string"),
        NormalizedReference::Name(name) => write!(
            input,
            "reference_name(kind={:?};caller_context_dependent={});",
            name.kind, name.caller_context_dependent
        )
        .expect("write to string"),
        NormalizedReference::External(external) => write!(
            input,
            "reference_external(class={};capability={});",
            external.external_reference_class, external.capability_requirement
        )
        .expect("write to string"),
        NormalizedReference::Structured(structured) => write!(
            input,
            "reference_structured(selector={:?};sections={:?};column_count={};caller_row_sensitive={});",
            structured.selector_kind,
            structured.section_qualifiers,
            structured.selected_column_ids.len(),
            structured.caller_row_sensitive
        )
        .expect("write to string"),
        NormalizedReference::Error(error) => write!(
            input,
            "reference_error(class={});",
            error.error_class
        )
        .expect("write to string"),
    }
}

fn fingerprint(namespace: &str, input: &str) -> String {
    format!("{namespace}:v1:{:016x}", stable_fnv1a64(input.as_bytes()))
}

fn stable_fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use oxfml_core::binding::{BindContext, BindRequest, bind_formula};
    use oxfml_core::red::project_red_view;
    use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
    use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
    use oxfml_core::{CompileSemanticPlanRequest, compile_semantic_plan};

    use super::*;

    fn identity_for(formula_stable_id: &str, source_text: &str) -> PreparedFormulaIdentityKeys {
        let source = FormulaSourceRecord::new(formula_stable_id, 1, source_text);
        let parse = parse_formula(ParseRequest {
            source: source.clone(),
        });
        let red_projection = project_red_view(source.formula_stable_id.clone(), &parse.green_tree);
        let bind_result = bind_formula(BindRequest {
            source,
            green_tree: parse.green_tree,
            red_projection,
            context: BindContext {
                caller_row: 10,
                caller_col: 4,
                structure_context_version: StructureContextVersion("snapshot:test".to_string()),
                ..BindContext::default()
            },
        });
        let semantic_plan = compile_semantic_plan(CompileSemanticPlanRequest {
            bound_formula: bind_result.bound_formula.clone(),
            oxfunc_catalog_identity: "oxfunc:host".to_string(),
            locale_profile: None,
            date_system: None,
            format_profile: None,
            library_context_snapshot: None,
        })
        .semantic_plan;

        derive_prepared_formula_identity_keys(&bind_result.bound_formula, &semantic_plan)
    }

    #[test]
    fn shape_and_template_keys_abstract_literal_and_reference_leaves() {
        let left = identity_for("formula:left", "=SUM(A1,2)");
        let right = identity_for("formula:right", "=SUM(B7,99)");

        assert_eq!(left.shape_key, right.shape_key);
        assert_eq!(left.dispatch_skeleton_key, right.dispatch_skeleton_key);
        assert_eq!(left.plan_template_key, right.plan_template_key);
    }

    #[test]
    fn dispatch_key_changes_when_same_shape_resolves_to_different_function() {
        let sum = identity_for("formula:sum", "=SUM(A1,2)");
        let max = identity_for("formula:max", "=MAX(B7,99)");

        assert_eq!(sum.shape_key, max.shape_key);
        assert_ne!(sum.dispatch_skeleton_key, max.dispatch_skeleton_key);
        assert_ne!(sum.plan_template_key, max.plan_template_key);
    }

    #[test]
    fn shape_key_preserves_lazy_control_posture() {
        let eager = identity_for("formula:eager", "=SUM(A1,2,3)");
        let branch_lazy = identity_for("formula:branch", "=IF(A1,2,3)");

        assert_ne!(eager.shape_key, branch_lazy.shape_key);
    }
}
