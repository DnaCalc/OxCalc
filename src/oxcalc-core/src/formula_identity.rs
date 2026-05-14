#![forbid(unsafe_code)]

//! Formula plan-template identity keys derived from OxFml public artifacts.

use std::collections::BTreeMap;
use std::fmt::{Display, Write};

use oxfml_core::binding::{BoundExpr, BoundFormula, NormalizedReference, ReferenceExpr};
use oxfml_core::semantics::{FunctionAvailabilitySummary, FunctionPlanBinding, SemanticPlan};
use oxfunc_core::function::ArgPreparationProfile;

use crate::rich_value_capability::RichValueCapabilitySet;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTemplate {
    pub shape_key: ShapeKey,
    pub dispatch_skeleton_key: DispatchSkeletonKey,
    pub plan_template_key: PlanTemplateKey,
    pub holes: Vec<PlanTemplateHole>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTemplateHole {
    pub hole_id: String,
    pub ordinal: usize,
    pub path: String,
    pub kind: PlanTemplateHoleKind,
}

impl PlanTemplateHole {
    #[must_use]
    pub fn stable_key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.ordinal,
            self.hole_id,
            self.path,
            self.kind.stable_key()
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PlanTemplateHoleKind {
    ValueHole {
        value_class_bound: ValueClassBound,
    },
    RefOrValueHole {
        ref_observability: RefObservability,
    },
    CallableHole {
        callable_signature: CallableSignature,
    },
    ShapeSensitiveHole {
        extent_class: ExtentClass,
    },
    SparseRangeHole {
        extent_class: ExtentClass,
        cardinality_class: CardinalityClass,
    },
    RichValueHole {
        required_capability_set: RichValueCapabilitySet,
    },
}

impl PlanTemplateHoleKind {
    #[must_use]
    pub fn stable_key(&self) -> String {
        match self {
            Self::ValueHole { value_class_bound } => {
                format!("ValueHole({})", value_class_bound.stable_key())
            }
            Self::RefOrValueHole { ref_observability } => {
                format!("RefOrValueHole({})", ref_observability.stable_key())
            }
            Self::CallableHole { callable_signature } => {
                format!("CallableHole({})", callable_signature.stable_key())
            }
            Self::ShapeSensitiveHole { extent_class } => {
                format!("ShapeSensitiveHole({})", extent_class.stable_key())
            }
            Self::SparseRangeHole {
                extent_class,
                cardinality_class,
            } => format!(
                "SparseRangeHole({},{})",
                extent_class.stable_key(),
                cardinality_class.stable_key()
            ),
            Self::RichValueHole {
                required_capability_set,
            } => format!("RichValueHole({})", required_capability_set.stable_key()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ValueClassBound {
    Any,
}

impl ValueClassBound {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::Any => "AnyValue",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RefObservability {
    ReferenceIdentityVisible,
}

impl RefObservability {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::ReferenceIdentityVisible => "ReferenceIdentityVisible",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CallableSignature {
    AnyCallable,
}

impl CallableSignature {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::AnyCallable => "AnyCallable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExtentClass {
    AnyExtent,
}

impl ExtentClass {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::AnyExtent => "AnyExtent",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CardinalityClass {
    AnyCardinality,
}

impl CardinalityClass {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::AnyCardinality => "AnyCardinality",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HoleBindings {
    pub binding_fingerprint: String,
    pub bindings: Vec<PlanTemplateHoleBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanTemplateHoleBinding {
    pub hole_id: String,
    pub payload: HoleBindingPayload,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HoleBindingPayload {
    NumberLiteral(String),
    TextLiteral(String),
    LogicalLiteral(bool),
    OmittedArgument,
    Reference(String),
    HelperParameterName(String),
    HelperOptionalParameterName(String),
}

impl HoleBindingPayload {
    #[must_use]
    pub fn stable_key(&self) -> String {
        match self {
            Self::NumberLiteral(value) => format!("number:{value}"),
            Self::TextLiteral(value) => format!("text:{value}"),
            Self::LogicalLiteral(value) => format!("logical:{value}"),
            Self::OmittedArgument => "omitted".to_string(),
            Self::Reference(value) => format!("reference:{value}"),
            Self::HelperParameterName(value) => format!("helper_parameter:{value}"),
            Self::HelperOptionalParameterName(value) => {
                format!("helper_optional_parameter:{value}")
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedCallable {
    pub prepared_callable_key: String,
    pub plan_template: PlanTemplate,
    pub hole_bindings: HoleBindings,
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

    let template_holes = collect_template_holes(bound_formula, semantic_plan);
    let plan_input =
        plan_template_key_input(&dispatch_skeleton_key, semantic_plan, &template_holes);
    let plan_template_key = PlanTemplateKey(fingerprint("plan_template", &plan_input));

    PreparedFormulaIdentityKeys {
        shape_key,
        dispatch_skeleton_key,
        plan_template_key,
    }
}

#[must_use]
pub fn derive_prepared_callable(
    bound_formula: &BoundFormula,
    semantic_plan: &SemanticPlan,
) -> PreparedCallable {
    let identity_keys = derive_prepared_formula_identity_keys(bound_formula, semantic_plan);
    let collected_holes = collect_template_holes_and_bindings(&bound_formula.root, semantic_plan);
    let holes = collected_holes
        .iter()
        .map(|collected| collected.hole.clone())
        .collect::<Vec<_>>();
    let bindings = collected_holes
        .into_iter()
        .map(|collected| collected.binding)
        .collect::<Vec<_>>();
    let binding_fingerprint = fingerprint("hole_bindings", &hole_binding_key_input(&bindings));
    let hole_bindings = HoleBindings {
        binding_fingerprint,
        bindings,
    };
    let plan_template = PlanTemplate {
        shape_key: identity_keys.shape_key,
        dispatch_skeleton_key: identity_keys.dispatch_skeleton_key,
        plan_template_key: identity_keys.plan_template_key,
        holes,
    };
    let prepared_callable_key = fingerprint(
        "prepared_callable",
        &format!(
            "plan_template_key={};hole_binding_fingerprint={};",
            plan_template.plan_template_key, hole_bindings.binding_fingerprint
        ),
    );

    PreparedCallable {
        prepared_callable_key,
        plan_template,
        hole_bindings,
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
    template_holes: &[PlanTemplateHole],
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
    for hole in template_holes {
        write!(input, "template_hole({});", hole.stable_key()).expect("write to string");
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

#[derive(Debug, Clone, PartialEq, Eq)]
struct CollectedTemplateHole {
    hole: PlanTemplateHole,
    binding: PlanTemplateHoleBinding,
}

struct HoleCollector<'a> {
    function_bindings: &'a [FunctionPlanBinding],
    function_binding_cursor: usize,
    collected: Vec<CollectedTemplateHole>,
}

impl<'a> HoleCollector<'a> {
    fn new(function_bindings: &'a [FunctionPlanBinding]) -> Self {
        Self {
            function_bindings,
            function_binding_cursor: 0,
            collected: Vec::new(),
        }
    }

    fn push_hole(&mut self, path: String, kind: PlanTemplateHoleKind, payload: HoleBindingPayload) {
        let ordinal = self.collected.len();
        let hole_id = format!("hole:{ordinal}");
        self.collected.push(CollectedTemplateHole {
            hole: PlanTemplateHole {
                hole_id: hole_id.clone(),
                ordinal,
                path,
                kind,
            },
            binding: PlanTemplateHoleBinding { hole_id, payload },
        });
    }

    fn collect_expr(&mut self, expr: &BoundExpr, path: String, policy: HoleTemplatePolicy) {
        match expr {
            BoundExpr::NumberLiteral(value) => self.push_hole(
                path,
                policy.default_hole_kind(),
                HoleBindingPayload::NumberLiteral(value.clone()),
            ),
            BoundExpr::StringLiteral(value) => self.push_hole(
                path,
                policy.default_hole_kind(),
                HoleBindingPayload::TextLiteral(value.clone()),
            ),
            BoundExpr::LogicalLiteral(value) => self.push_hole(
                path,
                policy.default_hole_kind(),
                HoleBindingPayload::LogicalLiteral(*value),
            ),
            BoundExpr::ArrayLiteral(rows) => {
                for (row_index, row) in rows.iter().enumerate() {
                    for (column_index, value) in row.iter().enumerate() {
                        self.collect_expr(
                            value,
                            format!("{path}.row{row_index}.col{column_index}"),
                            policy,
                        );
                    }
                }
            }
            BoundExpr::OmittedArgument => self.push_hole(
                path,
                policy.default_hole_kind(),
                HoleBindingPayload::OmittedArgument,
            ),
            BoundExpr::HelperParameterName(name) => self.push_hole(
                path,
                policy.default_hole_kind(),
                HoleBindingPayload::HelperParameterName(name.clone()),
            ),
            BoundExpr::HelperOptionalParameterName(name) => self.push_hole(
                path,
                policy.default_hole_kind(),
                HoleBindingPayload::HelperOptionalParameterName(name.clone()),
            ),
            BoundExpr::Binary { left, right, .. } => {
                self.collect_expr(left, format!("{path}.left"), policy);
                self.collect_expr(right, format!("{path}.right"), policy);
            }
            BoundExpr::Unary { expr, .. } | BoundExpr::ImplicitIntersection(expr) => {
                self.collect_expr(expr, format!("{path}.expr"), policy);
            }
            BoundExpr::FunctionCall {
                function_name,
                args,
            } => {
                let arg_policy = self
                    .function_argument_policy(function_name, args.len())
                    .unwrap_or_default();
                for (index, arg) in args.iter().enumerate() {
                    self.collect_expr(arg, format!("{path}.arg{index}"), arg_policy);
                }
            }
            BoundExpr::Invocation { callee, args } => {
                self.push_hole(
                    format!("{path}.callee"),
                    PlanTemplateHoleKind::CallableHole {
                        callable_signature: CallableSignature::AnyCallable,
                    },
                    HoleBindingPayload::Reference(format!("{callee:?}")),
                );
                for (index, arg) in args.iter().enumerate() {
                    self.collect_expr(arg, format!("{path}.arg{index}"), policy);
                }
            }
            BoundExpr::Reference(reference) => self.collect_reference(reference, path, policy),
        }
    }

    fn function_argument_policy(
        &mut self,
        function_name: &str,
        arg_count: usize,
    ) -> Option<HoleTemplatePolicy> {
        let function_name = function_name.to_ascii_uppercase();
        let mut skipped = 0;
        for binding in self.function_bindings[self.function_binding_cursor..].iter() {
            skipped += 1;
            if binding.function_name.to_ascii_uppercase() == function_name
                && binding.arg_count == arg_count
            {
                self.function_binding_cursor += skipped;
                return Some(HoleTemplatePolicy::from(binding.arg_preparation_profile));
            }
        }

        None
    }

    fn collect_reference(
        &mut self,
        reference: &ReferenceExpr,
        path: String,
        policy: HoleTemplatePolicy,
    ) {
        match reference {
            ReferenceExpr::Atom(normalized) => self.push_hole(
                path,
                policy.default_hole_kind(),
                HoleBindingPayload::Reference(format!("{normalized:?}")),
            ),
            ReferenceExpr::Range { start, end } => {
                self.collect_reference(start, format!("{path}.range_start"), policy);
                self.collect_reference(end, format!("{path}.range_end"), policy);
            }
            ReferenceExpr::Union { left, right } => {
                self.collect_reference(left, format!("{path}.union_left"), policy);
                self.collect_reference(right, format!("{path}.union_right"), policy);
            }
            ReferenceExpr::Intersection { left, right } => {
                self.collect_reference(left, format!("{path}.intersection_left"), policy);
                self.collect_reference(right, format!("{path}.intersection_right"), policy);
            }
            ReferenceExpr::Spill { anchor } => {
                self.collect_reference(anchor, format!("{path}.spill_anchor"), policy);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum HoleTemplatePolicy {
    #[default]
    WideValue,
    ReferenceVisible,
}

impl From<ArgPreparationProfile> for HoleTemplatePolicy {
    fn from(profile: ArgPreparationProfile) -> Self {
        match profile {
            ArgPreparationProfile::ValuesOnlyPreAdapter => Self::WideValue,
            ArgPreparationProfile::RefsVisibleInAdapter => Self::ReferenceVisible,
        }
    }
}

impl HoleTemplatePolicy {
    fn default_hole_kind(self) -> PlanTemplateHoleKind {
        match self {
            Self::WideValue => PlanTemplateHoleKind::ValueHole {
                value_class_bound: ValueClassBound::Any,
            },
            Self::ReferenceVisible => PlanTemplateHoleKind::RefOrValueHole {
                ref_observability: RefObservability::ReferenceIdentityVisible,
            },
        }
    }
}

fn collect_template_holes_and_bindings(
    root: &BoundExpr,
    semantic_plan: &SemanticPlan,
) -> Vec<CollectedTemplateHole> {
    let mut collector = HoleCollector::new(&semantic_plan.function_bindings);
    collector.collect_expr(root, "root".to_string(), HoleTemplatePolicy::default());
    collector.collected
}

fn collect_template_holes(
    bound_formula: &BoundFormula,
    semantic_plan: &SemanticPlan,
) -> Vec<PlanTemplateHole> {
    collect_template_holes_and_bindings(&bound_formula.root, semantic_plan)
        .into_iter()
        .map(|collected| collected.hole)
        .collect()
}

fn hole_binding_key_input(bindings: &[PlanTemplateHoleBinding]) -> String {
    let mut input = String::from("oxcalc.hole_bindings.v1;");
    for binding in bindings {
        write!(
            input,
            "{}={};",
            binding.hole_id,
            binding.payload.stable_key()
        )
        .expect("write to string");
    }
    input
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
    use std::fs;
    use std::path::PathBuf;

    use oxfml_core::binding::{BindContext, BindRequest, bind_formula};
    use oxfml_core::red::project_red_view;
    use oxfml_core::source::{FormulaSourceRecord, StructureContextVersion};
    use oxfml_core::syntax::parser::{ParseRequest, parse_formula};
    use oxfml_core::{CompileSemanticPlanRequest, compile_semantic_plan};
    use serde_json::json;

    use crate::rich_value_capability::{
        RichValueCapability, RichValueCapabilitySet, w050_initial_capability_examples,
        w050_initial_required_capability_set_example,
    };

    use super::*;

    fn compile_formula_for(
        formula_stable_id: &str,
        source_text: &str,
    ) -> (BoundFormula, SemanticPlan) {
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

        (bind_result.bound_formula, semantic_plan)
    }

    fn prepared_callable_for(formula_stable_id: &str, source_text: &str) -> PreparedCallable {
        let (bound_formula, semantic_plan) = compile_formula_for(formula_stable_id, source_text);
        derive_prepared_callable(&bound_formula, &semantic_plan)
    }

    fn identity_for(formula_stable_id: &str, source_text: &str) -> PreparedFormulaIdentityKeys {
        let prepared_callable = prepared_callable_for(formula_stable_id, source_text);
        PreparedFormulaIdentityKeys {
            shape_key: prepared_callable.plan_template.shape_key,
            dispatch_skeleton_key: prepared_callable.plan_template.dispatch_skeleton_key,
            plan_template_key: prepared_callable.plan_template.plan_template_key,
        }
    }

    fn g2_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../docs/test-runs/core-engine/w050-g2-rich-value-hole-capability-requirements-001",
        )
    }

    fn indexable_only_capability_set() -> RichValueCapabilitySet {
        let indexable = w050_initial_capability_examples()
            .into_iter()
            .find(|capability| matches!(capability, RichValueCapability::Indexable { .. }))
            .expect("G1 vocabulary should include Indexable");
        RichValueCapabilitySet::new([indexable])
    }

    fn rich_value_hole_for(required_capability_set: RichValueCapabilitySet) -> PlanTemplateHole {
        PlanTemplateHole {
            hole_id: "hole:rich".to_string(),
            ordinal: 0,
            path: "root.arg0".to_string(),
            kind: PlanTemplateHoleKind::RichValueHole {
                required_capability_set,
            },
        }
    }

    fn rich_value_plan_template_material(
        required_capability_set: RichValueCapabilitySet,
    ) -> String {
        let (bound_formula, semantic_plan) = compile_formula_for("formula:rich-key", "=SUM(A1,2)");
        let identity_keys = derive_prepared_formula_identity_keys(&bound_formula, &semantic_plan);
        let rich_hole = rich_value_hole_for(required_capability_set);

        plan_template_key_input(
            &identity_keys.dispatch_skeleton_key,
            &semantic_plan,
            &[rich_hole],
        )
    }

    fn rich_value_hole_artifact_json() -> serde_json::Value {
        let full_required = w050_initial_required_capability_set_example();
        let indexable_only = indexable_only_capability_set();
        let full_material = rich_value_plan_template_material(full_required.clone());
        let indexable_material = rich_value_plan_template_material(indexable_only.clone());

        json!({
            "run_id": "w050-g2-rich-value-hole-capability-requirements-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core rich_value_hole -- --nocapture",
            "rich_value_hole": {
                "variant": "RichValueHole(required_capability_set)",
                "full_required_capability_set_key": full_required.stable_key(),
                "indexable_only_capability_set_key": indexable_only.stable_key(),
                "stable_key_includes_required_capability_set": PlanTemplateHoleKind::RichValueHole {
                    required_capability_set: full_required.clone()
                }.stable_key()
            },
            "plan_template_identity": {
                "capability_set_participates_in_template_key_material": full_material != indexable_material,
                "full_material_contains_full_required_set": full_material.contains(&full_required.stable_key()),
                "indexable_material_contains_indexable_required_set": indexable_material.contains(&indexable_only.stable_key())
            },
            "scope_boundary": {
                "current_v1_production_paths_emit_rich_holes": false,
                "concrete_rich_kernel_claim": false
            }
        })
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

    #[test]
    fn current_v1_does_not_fold_constants_into_plan_template_identity() {
        let expression = identity_for("formula:expression", "=2+3*4");
        let already_folded = identity_for("formula:already_folded", "=14");

        assert_ne!(expression.shape_key, already_folded.shape_key);
        assert_ne!(
            expression.plan_template_key,
            already_folded.plan_template_key
        );
    }

    #[test]
    fn hole_taxonomy_stable_keys_are_deterministic() {
        let cases = [
            (
                PlanTemplateHoleKind::ValueHole {
                    value_class_bound: ValueClassBound::Any,
                },
                "ValueHole(AnyValue)",
            ),
            (
                PlanTemplateHoleKind::RefOrValueHole {
                    ref_observability: RefObservability::ReferenceIdentityVisible,
                },
                "RefOrValueHole(ReferenceIdentityVisible)",
            ),
            (
                PlanTemplateHoleKind::CallableHole {
                    callable_signature: CallableSignature::AnyCallable,
                },
                "CallableHole(AnyCallable)",
            ),
            (
                PlanTemplateHoleKind::ShapeSensitiveHole {
                    extent_class: ExtentClass::AnyExtent,
                },
                "ShapeSensitiveHole(AnyExtent)",
            ),
            (
                PlanTemplateHoleKind::SparseRangeHole {
                    extent_class: ExtentClass::AnyExtent,
                    cardinality_class: CardinalityClass::AnyCardinality,
                },
                "SparseRangeHole(AnyExtent,AnyCardinality)",
            ),
        ];

        for (kind, stable_key) in cases {
            assert_eq!(kind.stable_key(), stable_key);
        }

        assert_eq!(
            PlanTemplateHoleKind::RichValueHole {
                required_capability_set: indexable_only_capability_set(),
            }
            .stable_key(),
            "RichValueHole(Indexable(rank=2,index_type=GridCoordinate,element_value_class=AnyValue))"
        );
    }

    #[test]
    fn rich_value_hole_capability_set_participates_in_plan_template_key_material() {
        let full_required = w050_initial_required_capability_set_example();
        let indexable_only = indexable_only_capability_set();
        let full_material = rich_value_plan_template_material(full_required.clone());
        let indexable_material = rich_value_plan_template_material(indexable_only.clone());

        assert_ne!(full_material, indexable_material);
        assert!(full_material.contains(&full_required.stable_key()));
        assert!(indexable_material.contains(&indexable_only.stable_key()));
        assert!(full_material.contains("RichValueHole("));
        assert!(indexable_material.contains("RichValueHole("));
    }

    #[test]
    fn prepared_callable_separates_template_from_hole_bindings() {
        let left = prepared_callable_for("formula:left", "=SUM(A1,2)");
        let right = prepared_callable_for("formula:right", "=SUM(B7,99)");

        assert_eq!(
            left.plan_template.plan_template_key,
            right.plan_template.plan_template_key
        );
        assert_eq!(left.plan_template.holes, right.plan_template.holes);
        assert_eq!(
            left.plan_template
                .holes
                .iter()
                .map(|hole| hole.kind.clone())
                .collect::<Vec<_>>(),
            vec![
                PlanTemplateHoleKind::ValueHole {
                    value_class_bound: ValueClassBound::Any,
                },
                PlanTemplateHoleKind::ValueHole {
                    value_class_bound: ValueClassBound::Any,
                },
            ]
        );
        assert_ne!(
            left.hole_bindings.binding_fingerprint,
            right.hole_bindings.binding_fingerprint
        );
        assert_ne!(
            left.hole_bindings.bindings[0].payload,
            right.hole_bindings.bindings[0].payload
        );
        assert_ne!(
            left.hole_bindings.bindings[1].payload,
            right.hole_bindings.bindings[1].payload
        );
        assert_ne!(left.prepared_callable_key, right.prepared_callable_key);
    }

    #[test]
    fn prepared_callable_template_changes_when_dispatch_changes() {
        let sum = prepared_callable_for("formula:sum", "=SUM(A1,2)");
        let max = prepared_callable_for("formula:max", "=MAX(A1,2)");

        assert_eq!(sum.plan_template.shape_key, max.plan_template.shape_key);
        assert_ne!(
            sum.plan_template.dispatch_skeleton_key,
            max.plan_template.dispatch_skeleton_key
        );
        assert_ne!(
            sum.plan_template.plan_template_key,
            max.plan_template.plan_template_key
        );
        assert_eq!(
            sum.hole_bindings.binding_fingerprint,
            max.hole_bindings.binding_fingerprint
        );
    }

    #[test]
    fn refs_visible_function_uses_ref_or_value_template_holes() {
        let prepared_callable = prepared_callable_for("formula:rows", "=ROWS(A1:A3)");

        assert!(!prepared_callable.plan_template.holes.is_empty());
        assert!(
            prepared_callable
                .plan_template
                .holes
                .iter()
                .all(|hole| matches!(
                    hole.kind,
                    PlanTemplateHoleKind::RefOrValueHole {
                        ref_observability: RefObservability::ReferenceIdentityVisible
                    }
                ))
        );
    }

    #[test]
    fn current_v1_paths_do_not_emit_sparse_or_rich_holes() {
        let prepared_callable = prepared_callable_for("formula:sum_range", "=SUM(A1:A3)");

        assert!(!prepared_callable.plan_template.holes.is_empty());
        assert!(
            prepared_callable
                .plan_template
                .holes
                .iter()
                .all(|hole| !matches!(
                    hole.kind,
                    PlanTemplateHoleKind::SparseRangeHole { .. }
                        | PlanTemplateHoleKind::RichValueHole { .. }
                ))
        );
    }

    #[test]
    fn rich_value_hole_checked_artifact_matches_runtime_identity_surface() {
        let artifact_path = g2_artifact_root().join("run_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("G2 run artifact should be checked in"),
        )
        .expect("G2 run artifact should be valid JSON");

        assert_eq!(artifact, rich_value_hole_artifact_json());
    }
}
