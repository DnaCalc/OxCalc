//! The optimized engine's repeated-R1C1 fast-path evaluator: the compiled
//! plan / expression / operation representation (direct scalar refs, bounded
//! binary and comparison expressions, range aggregates, IF and IFERROR) and
//! the per-template formula plan cache. The recalc compiles a cell's
//! R1C1-relative normal form into one of these plans and evaluates it
//! without the general OxFml stack. Internal to the machine; shares the
//! machine's types via `use super::*`. Plan structs are pub(super) so the
//! recalc (in the parent) can compile cells into them.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1BinaryPlan {
    pub(super) left: Box<GridOptimizedR1C1ScalarExpression>,
    pub(super) op: GridOptimizedR1C1BinaryOp,
    pub(super) right: Box<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1BinaryPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let left = self.left.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        let right = self.right.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        self.op.apply(left, right)
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let left = self.left.value_for_single_cell(address, valuation)?;
        let right = self.right.value_for_single_cell(address, valuation)?;
        self.op.apply(left, right)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridOptimizedR1C1RangeAggregatePlan {
    pub(super) function: GridOptimizedR1C1RangeAggregateFunction,
    pub(super) start: GridOptimizedR1C1Ref,
    pub(super) end: GridOptimizedR1C1Ref,
}

impl GridOptimizedR1C1RangeAggregatePlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let (start_row, start_col) = self.start.resolve(row, col)?;
        let (end_row, end_col) = self.end.resolve(row, col)?;
        aggregate_optimized_r1c1_rect(
            self.function,
            start_row.min(end_row),
            start_col.min(end_col),
            start_row.max(end_row),
            start_col.max(end_col),
            Some(region),
            row_major_formula_values,
            valuation,
        )
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let (start_row, start_col) = self.start.resolve(address.row, address.col)?;
        let (end_row, end_col) = self.end.resolve(address.row, address.col)?;
        aggregate_optimized_r1c1_rect(
            self.function,
            start_row.min(end_row),
            start_col.min(end_col),
            start_row.max(end_row),
            start_col.max(end_col),
            None,
            &[],
            valuation,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1IfPlan {
    pub(super) condition: GridOptimizedR1C1LogicalExpression,
    pub(super) when_true: GridOptimizedR1C1ScalarExpression,
    pub(super) when_false: GridOptimizedR1C1ScalarExpression,
}

impl GridOptimizedR1C1IfPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let condition = self.condition.evaluate_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        match condition {
            GridOptimizedR1C1ConditionValue::Logical(true) => {
                self.when_true.evaluate_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )
            }
            GridOptimizedR1C1ConditionValue::Logical(false) => {
                self.when_false.evaluate_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )
            }
            GridOptimizedR1C1ConditionValue::Error(error) => Some(CalcValue::error(error)),
        }
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let condition = self.condition.evaluate_single_cell(address, valuation)?;
        match condition {
            GridOptimizedR1C1ConditionValue::Logical(true) => {
                self.when_true.evaluate_single_cell(address, valuation)
            }
            GridOptimizedR1C1ConditionValue::Logical(false) => {
                self.when_false.evaluate_single_cell(address, valuation)
            }
            GridOptimizedR1C1ConditionValue::Error(error) => Some(CalcValue::error(error)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1IfErrorPlan {
    pub(super) value: GridOptimizedR1C1ScalarExpression,
    pub(super) fallback: GridOptimizedR1C1ScalarExpression,
}

impl GridOptimizedR1C1IfErrorPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let value = self.value.evaluate_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        if matches!(value.core, CoreValue::Error(_)) {
            self.fallback.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )
        } else {
            Some(value)
        }
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let value = self.value.evaluate_single_cell(address, valuation)?;
        if matches!(value.core, CoreValue::Error(_)) {
            self.fallback.evaluate_single_cell(address, valuation)
        } else {
            Some(value)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1ComparisonPlan {
    pub(super) comparison: GridOptimizedR1C1Comparison,
}

impl GridOptimizedR1C1ComparisonPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        self.comparison
            .evaluate_repeated_region_cell(row, col, region, row_major_formula_values, valuation)?
            .into_calc_value()
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        self.comparison
            .evaluate_single_cell(address, valuation)?
            .into_calc_value()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1LogicalFunctionPlan {
    pub(super) function: GridOptimizedR1C1LogicalFunction,
    pub(super) arguments: Vec<GridOptimizedR1C1LogicalExpression>,
}

impl GridOptimizedR1C1LogicalFunctionPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        self.condition_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?
        .into_calc_value()
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        self.condition_for_single_cell(address, valuation)?
            .into_calc_value()
    }

    pub(super) fn condition_for_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        let arguments = self
            .arguments
            .iter()
            .map(|argument| {
                argument.evaluate_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )
            })
            .collect::<Option<Vec<_>>>()?;
        Some(self.function.apply(&arguments))
    }

    pub(super) fn condition_for_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        let arguments = self
            .arguments
            .iter()
            .map(|argument| argument.evaluate_single_cell(address, valuation))
            .collect::<Option<Vec<_>>>()?;
        Some(self.function.apply(&arguments))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridOptimizedR1C1LogicalExpression {
    Comparison(GridOptimizedR1C1Comparison),
    Function(Box<GridOptimizedR1C1LogicalFunctionPlan>),
}

impl GridOptimizedR1C1LogicalExpression {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        match self {
            Self::Comparison(comparison) => comparison.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Function(plan) => plan.condition_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
        }
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        match self {
            Self::Comparison(comparison) => comparison.evaluate_single_cell(address, valuation),
            Self::Function(plan) => plan.condition_for_single_cell(address, valuation),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridOptimizedR1C1LogicalFunction {
    And,
    Or,
    Not,
}

impl GridOptimizedR1C1LogicalFunction {
    pub(super) const ALL: [Self; 3] = [Self::And, Self::Or, Self::Not];

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::And => "AND",
            Self::Or => "OR",
            Self::Not => "NOT",
        }
    }

    pub(super) fn arity_holds(self, len: usize) -> bool {
        match self {
            Self::And | Self::Or => len >= 1,
            Self::Not => len == 1,
        }
    }

    pub(super) fn apply(
        self,
        arguments: &[GridOptimizedR1C1ConditionValue],
    ) -> GridOptimizedR1C1ConditionValue {
        let mut values = Vec::with_capacity(arguments.len());
        for argument in arguments {
            match *argument {
                GridOptimizedR1C1ConditionValue::Logical(value) => values.push(value),
                GridOptimizedR1C1ConditionValue::Error(error) => {
                    return GridOptimizedR1C1ConditionValue::Error(error);
                }
            }
        }
        match self {
            Self::And => {
                GridOptimizedR1C1ConditionValue::Logical(values.into_iter().all(|value| value))
            }
            Self::Or => {
                GridOptimizedR1C1ConditionValue::Logical(values.into_iter().any(|value| value))
            }
            Self::Not => GridOptimizedR1C1ConditionValue::Logical(!values[0]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridOptimizedR1C1ScalarExpression {
    Operand(GridOptimizedR1C1Operand),
    UnaryMinus(GridOptimizedR1C1UnaryMinusPlan),
    Binary(GridOptimizedR1C1BinaryPlan),
    RangeAggregate(GridOptimizedR1C1RangeAggregatePlan),
    ArgumentAggregate(GridOptimizedR1C1ArgumentAggregatePlan),
    ScalarFunction(GridOptimizedR1C1ScalarFunctionPlan),
    ReferenceFunction(GridOptimizedR1C1ReferenceFunctionPlan),
    TextFunction(GridOptimizedR1C1TextFunctionPlan),
    Index(GridOptimizedR1C1IndexPlan),
    Match(GridOptimizedR1C1MatchPlan),
    VLookup(GridOptimizedR1C1VLookupPlan),
    If(Box<GridOptimizedR1C1IfPlan>),
    IfError(Box<GridOptimizedR1C1IfErrorPlan>),
}

impl GridOptimizedR1C1ScalarExpression {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::Operand(operand) => operand.calc_value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::UnaryMinus(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Binary(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::RangeAggregate(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::ArgumentAggregate(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::ScalarFunction(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::ReferenceFunction(plan) => plan.evaluate_repeated_region_cell(row, col),
            Self::TextFunction(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Index(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Match(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::VLookup(plan) => plan.evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::If(plan) => (*plan).evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::IfError(plan) => (*plan).evaluate_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
        }
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::Operand(operand) => operand.calc_value_for_single_cell(address, valuation),
            Self::UnaryMinus(plan) => plan.evaluate_single_cell(address, valuation),
            Self::Binary(plan) => plan.evaluate_single_cell(address, valuation),
            Self::RangeAggregate(plan) => plan.evaluate_single_cell(address, valuation),
            Self::ArgumentAggregate(plan) => plan.evaluate_single_cell(address, valuation),
            Self::ScalarFunction(plan) => plan.evaluate_single_cell(address, valuation),
            Self::ReferenceFunction(plan) => plan.evaluate_single_cell(address),
            Self::TextFunction(plan) => plan.evaluate_single_cell(address, valuation),
            Self::Index(plan) => plan.evaluate_single_cell(address, valuation),
            Self::Match(plan) => plan.evaluate_single_cell(address, valuation),
            Self::VLookup(plan) => plan.evaluate_single_cell(address, valuation),
            Self::If(plan) => (*plan).evaluate_single_cell(address, valuation),
            Self::IfError(plan) => (*plan).evaluate_single_cell(address, valuation),
        }
    }

    pub(super) fn value_for_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1Value> {
        let value = self.evaluate_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        optimized_r1c1_value_from_calc_value(&value)
    }

    pub(super) fn value_for_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1Value> {
        let value = self.evaluate_single_cell(address, valuation)?;
        optimized_r1c1_value_from_calc_value(&value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1UnaryMinusPlan {
    pub(super) value: Box<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1UnaryMinusPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let value = self.value.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        Some(negate_optimized_r1c1_value(value))
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let value = self.value.value_for_single_cell(address, valuation)?;
        Some(negate_optimized_r1c1_value(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1ArgumentAggregatePlan {
    pub(super) function: GridOptimizedR1C1RangeAggregateFunction,
    pub(super) arguments: Vec<GridOptimizedR1C1AggregateArgument>,
}

impl GridOptimizedR1C1ArgumentAggregatePlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let mut state = GridOptimizedR1C1AggregateState::new();
        for argument in &self.arguments {
            match argument.accumulate_repeated_region_cell(
                self.function,
                &mut state,
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )? {
                Ok(()) => {}
                Err(error) => return Some(error),
            }
        }
        Some(state.finish(self.function))
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let mut state = GridOptimizedR1C1AggregateState::new();
        for argument in &self.arguments {
            match argument.accumulate_single_cell(self.function, &mut state, address, valuation)? {
                Ok(()) => {}
                Err(error) => return Some(error),
            }
        }
        Some(state.finish(self.function))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1ScalarFunctionPlan {
    pub(super) function: GridOptimizedR1C1ScalarFunction,
    pub(super) arguments: Vec<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1ScalarFunctionPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let arguments = self
            .arguments
            .iter()
            .map(|argument| {
                argument.value_for_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )
            })
            .collect::<Option<Vec<_>>>()?;
        Some(self.function.apply(&arguments))
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let arguments = self
            .arguments
            .iter()
            .map(|argument| argument.value_for_single_cell(address, valuation))
            .collect::<Option<Vec<_>>>()?;
        Some(self.function.apply(&arguments))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridOptimizedR1C1ReferenceFunctionPlan {
    pub(super) function: GridOptimizedR1C1ReferenceFunction,
    pub(super) argument: GridOptimizedR1C1ReferenceFunctionArgument,
}

impl GridOptimizedR1C1ReferenceFunctionPlan {
    pub(super) fn evaluate_repeated_region_cell(&self, row: u32, col: u32) -> Option<CalcValue> {
        self.function.apply(self.argument.resolve(row, col)?)
    }

    pub(super) fn evaluate_single_cell(&self, address: &ExcelGridCellAddress) -> Option<CalcValue> {
        self.function
            .apply(self.argument.resolve(address.row, address.col)?)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1IndexPlan {
    pub(super) start: GridOptimizedR1C1Ref,
    pub(super) end: GridOptimizedR1C1Ref,
    pub(super) row_index: Box<GridOptimizedR1C1ScalarExpression>,
    pub(super) col_index: Box<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1IndexPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let range = self.resolve_range(row, col)?;
        let row_index =
            match optimized_r1c1_index_from_value(self.row_index.value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )?) {
                Ok(index) => index,
                Err(error) => return Some(error),
            };
        let col_index =
            match optimized_r1c1_index_from_value(self.col_index.value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )?) {
                Ok(index) => index,
                Err(error) => return Some(error),
            };
        self.value_at_indexed_range(
            range,
            row_index,
            col_index,
            Some(region),
            row_major_formula_values,
            valuation,
        )
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let range = self.resolve_range(address.row, address.col)?;
        let row_index = match optimized_r1c1_index_from_value(
            self.row_index.value_for_single_cell(address, valuation)?,
        ) {
            Ok(index) => index,
            Err(error) => return Some(error),
        };
        let col_index = match optimized_r1c1_index_from_value(
            self.col_index.value_for_single_cell(address, valuation)?,
        ) {
            Ok(index) => index,
            Err(error) => return Some(error),
        };
        self.value_at_indexed_range(range, row_index, col_index, None, &[], valuation)
    }

    pub(super) fn resolve_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
    ) -> Option<GridOptimizedR1C1ResolvedRef> {
        let (start_row, start_col) = self.start.resolve(anchor_row, anchor_col)?;
        let (end_row, end_col) = self.end.resolve(anchor_row, anchor_col)?;
        Some(GridOptimizedR1C1ResolvedRef {
            top_row: start_row.min(end_row),
            left_col: start_col.min(end_col),
            bottom_row: start_row.max(end_row),
            right_col: start_col.max(end_col),
        })
    }

    pub(super) fn value_at_indexed_range(
        &self,
        range: GridOptimizedR1C1ResolvedRef,
        row_index: usize,
        col_index: usize,
        region: Option<&GridRepeatedFormulaRegion>,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let row_offset = u32::try_from(row_index.checked_sub(1)?).ok()?;
        let col_offset = u32::try_from(col_index.checked_sub(1)?).ok()?;
        let target_row = range.top_row.checked_add(row_offset)?;
        let target_col = range.left_col.checked_add(col_offset)?;
        if target_row > range.bottom_row || target_col > range.right_col {
            return Some(CalcValue::error(WorksheetErrorCode::Ref));
        }
        optimized_r1c1_calc_value_for_cell(
            target_row,
            target_col,
            region,
            row_major_formula_values,
            valuation,
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1MatchPlan {
    pub(super) lookup: Box<GridOptimizedR1C1ScalarExpression>,
    pub(super) start: GridOptimizedR1C1Ref,
    pub(super) end: GridOptimizedR1C1Ref,
}

impl GridOptimizedR1C1MatchPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup = self.lookup.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        self.match_in_range(
            row,
            col,
            lookup,
            Some(region),
            row_major_formula_values,
            valuation,
        )
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup = self.lookup.value_for_single_cell(address, valuation)?;
        self.match_in_range(address.row, address.col, lookup, None, &[], valuation)
    }

    pub(super) fn resolve_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
    ) -> Option<GridOptimizedR1C1ResolvedRef> {
        let (start_row, start_col) = self.start.resolve(anchor_row, anchor_col)?;
        let (end_row, end_col) = self.end.resolve(anchor_row, anchor_col)?;
        Some(GridOptimizedR1C1ResolvedRef {
            top_row: start_row.min(end_row),
            left_col: start_col.min(end_col),
            bottom_row: start_row.max(end_row),
            right_col: start_col.max(end_col),
        })
    }

    pub(super) fn match_in_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
        lookup: GridOptimizedR1C1Value,
        region: Option<&GridRepeatedFormulaRegion>,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup_number = match lookup {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => return Some(CalcValue::error(error)),
        };
        let range = self.resolve_range(anchor_row, anchor_col)?;
        let row_count = range.bottom_row - range.top_row + 1;
        let col_count = range.right_col - range.left_col + 1;
        if row_count > 1 && col_count > 1 {
            return Some(CalcValue::error(WorksheetErrorCode::Value));
        }

        let mut position = 1_u64;
        for row in range.top_row..=range.bottom_row {
            for col in range.left_col..=range.right_col {
                let candidate = optimized_r1c1_value_for_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )?;
                match candidate {
                    GridOptimizedR1C1Value::Number(candidate) if candidate == lookup_number => {
                        return Some(CalcValue::number(position as f64));
                    }
                    GridOptimizedR1C1Value::Number(_) => {}
                    GridOptimizedR1C1Value::Error(error) => {
                        return Some(CalcValue::error(error));
                    }
                }
                position = position.saturating_add(1);
            }
        }
        Some(CalcValue::error(WorksheetErrorCode::NA))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1VLookupPlan {
    pub(super) lookup: Box<GridOptimizedR1C1ScalarExpression>,
    pub(super) start: GridOptimizedR1C1Ref,
    pub(super) end: GridOptimizedR1C1Ref,
    pub(super) col_index: Box<GridOptimizedR1C1ScalarExpression>,
}

impl GridOptimizedR1C1VLookupPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup = self.lookup.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        let col_index =
            match optimized_r1c1_index_from_value(self.col_index.value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )?) {
                Ok(index) => index,
                Err(error) => return Some(error),
            };
        self.lookup_in_range(
            row,
            col,
            lookup,
            col_index,
            Some(region),
            row_major_formula_values,
            valuation,
        )
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup = self.lookup.value_for_single_cell(address, valuation)?;
        let col_index = match optimized_r1c1_index_from_value(
            self.col_index.value_for_single_cell(address, valuation)?,
        ) {
            Ok(index) => index,
            Err(error) => return Some(error),
        };
        self.lookup_in_range(
            address.row,
            address.col,
            lookup,
            col_index,
            None,
            &[],
            valuation,
        )
    }

    pub(super) fn resolve_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
    ) -> Option<GridOptimizedR1C1ResolvedRef> {
        let (start_row, start_col) = self.start.resolve(anchor_row, anchor_col)?;
        let (end_row, end_col) = self.end.resolve(anchor_row, anchor_col)?;
        Some(GridOptimizedR1C1ResolvedRef {
            top_row: start_row.min(end_row),
            left_col: start_col.min(end_col),
            bottom_row: start_row.max(end_row),
            right_col: start_col.max(end_col),
        })
    }

    pub(super) fn lookup_in_range(
        &self,
        anchor_row: u32,
        anchor_col: u32,
        lookup: GridOptimizedR1C1Value,
        col_index: usize,
        region: Option<&GridRepeatedFormulaRegion>,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let lookup_number = match lookup {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => return Some(CalcValue::error(error)),
        };
        let range = self.resolve_range(anchor_row, anchor_col)?;
        let col_offset = u32::try_from(col_index.checked_sub(1)?).ok()?;
        let value_col = range.left_col.checked_add(col_offset)?;
        if value_col > range.right_col {
            return Some(CalcValue::error(WorksheetErrorCode::Ref));
        }

        for row in range.top_row..=range.bottom_row {
            let candidate = optimized_r1c1_value_for_cell(
                row,
                range.left_col,
                region,
                row_major_formula_values,
                valuation,
            )?;
            match candidate {
                GridOptimizedR1C1Value::Number(candidate) if candidate == lookup_number => {
                    return optimized_r1c1_calc_value_for_cell(
                        row,
                        value_col,
                        region,
                        row_major_formula_values,
                        valuation,
                    );
                }
                GridOptimizedR1C1Value::Number(_) => {}
                GridOptimizedR1C1Value::Error(error) => {
                    return Some(CalcValue::error(error));
                }
            }
        }
        Some(CalcValue::error(WorksheetErrorCode::NA))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridOptimizedR1C1TextFunctionPlan {
    Len {
        text: GridOptimizedR1C1Ref,
    },
    Left {
        text: GridOptimizedR1C1Ref,
        count: Box<GridOptimizedR1C1ScalarExpression>,
    },
    Right {
        text: GridOptimizedR1C1Ref,
        count: Box<GridOptimizedR1C1ScalarExpression>,
    },
    Concat {
        texts: Vec<GridOptimizedR1C1Ref>,
    },
}

impl GridOptimizedR1C1TextFunctionPlan {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::Len { text } => {
                let text = self.text_for_repeated_region_cell(
                    *text,
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )?;
                Some(match text {
                    Ok(text) => CalcValue::number(text.len_utf16_code_units() as f64),
                    Err(error) => error,
                })
            }
            Self::Left { text, count } => self.text_slice_for_repeated_region_cell(
                *text,
                count,
                GridOptimizedR1C1TextSliceSide::Left,
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Right { text, count } => self.text_slice_for_repeated_region_cell(
                *text,
                count,
                GridOptimizedR1C1TextSliceSide::Right,
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            ),
            Self::Concat { texts } => {
                let mut units = Vec::new();
                for text_ref in texts {
                    match self.text_for_repeated_region_cell(
                        *text_ref,
                        row,
                        col,
                        region,
                        row_major_formula_values,
                        valuation,
                    )? {
                        Ok(text) => units.extend_from_slice(text.utf16_code_units()),
                        Err(error) => return Some(error),
                    }
                }
                Some(CalcValue::text(ExcelText::from_utf16_code_units(units)))
            }
        }
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        match self {
            Self::Len { text } => {
                let text = self.text_for_single_cell(*text, address, valuation)?;
                Some(match text {
                    Ok(text) => CalcValue::number(text.len_utf16_code_units() as f64),
                    Err(error) => error,
                })
            }
            Self::Left { text, count } => self.text_slice_for_single_cell(
                *text,
                count,
                GridOptimizedR1C1TextSliceSide::Left,
                address,
                valuation,
            ),
            Self::Right { text, count } => self.text_slice_for_single_cell(
                *text,
                count,
                GridOptimizedR1C1TextSliceSide::Right,
                address,
                valuation,
            ),
            Self::Concat { texts } => {
                let mut units = Vec::new();
                for text_ref in texts {
                    match self.text_for_single_cell(*text_ref, address, valuation)? {
                        Ok(text) => units.extend_from_slice(text.utf16_code_units()),
                        Err(error) => return Some(error),
                    }
                }
                Some(CalcValue::text(ExcelText::from_utf16_code_units(units)))
            }
        }
    }

    pub(super) fn text_slice_for_repeated_region_cell(
        &self,
        text_ref: GridOptimizedR1C1Ref,
        count: &GridOptimizedR1C1ScalarExpression,
        side: GridOptimizedR1C1TextSliceSide,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let text = match self.text_for_repeated_region_cell(
            text_ref,
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )? {
            Ok(text) => text,
            Err(error) => return Some(error),
        };
        let count =
            match optimized_r1c1_text_count_from_value(count.value_for_repeated_region_cell(
                row,
                col,
                region,
                row_major_formula_values,
                valuation,
            )?) {
                Ok(count) => count,
                Err(error) => return Some(error),
            };
        Some(optimized_r1c1_text_slice(text, count, side))
    }

    pub(super) fn text_slice_for_single_cell(
        &self,
        text_ref: GridOptimizedR1C1Ref,
        count: &GridOptimizedR1C1ScalarExpression,
        side: GridOptimizedR1C1TextSliceSide,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        let text = match self.text_for_single_cell(text_ref, address, valuation)? {
            Ok(text) => text,
            Err(error) => return Some(error),
        };
        let count = match optimized_r1c1_text_count_from_value(
            count.value_for_single_cell(address, valuation)?,
        ) {
            Ok(count) => count,
            Err(error) => return Some(error),
        };
        Some(optimized_r1c1_text_slice(text, count, side))
    }

    pub(super) fn text_for_repeated_region_cell(
        &self,
        text_ref: GridOptimizedR1C1Ref,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<Result<ExcelText, CalcValue>> {
        let (target_row, target_col) = text_ref.resolve(row, col)?;
        optimized_r1c1_calc_value_for_cell(
            target_row,
            target_col,
            Some(region),
            row_major_formula_values,
            valuation,
        )
        .map(optimized_r1c1_text_from_calc_value)
    }

    pub(super) fn text_for_single_cell(
        &self,
        text_ref: GridOptimizedR1C1Ref,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<Result<ExcelText, CalcValue>> {
        let (target_row, target_col) = text_ref.resolve(address.row, address.col)?;
        optimized_r1c1_calc_value_for_cell(target_row, target_col, None, &[], valuation)
            .map(optimized_r1c1_text_from_calc_value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GridOptimizedR1C1TextSliceSide {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GridOptimizedR1C1ReferenceFunctionArgument {
    CurrentCell,
    Ref(GridOptimizedR1C1Ref),
    Range {
        start: GridOptimizedR1C1Ref,
        end: GridOptimizedR1C1Ref,
    },
}

impl GridOptimizedR1C1ReferenceFunctionArgument {
    pub(super) fn resolve(
        self,
        anchor_row: u32,
        anchor_col: u32,
    ) -> Option<GridOptimizedR1C1ResolvedRef> {
        match self {
            Self::CurrentCell => Some(GridOptimizedR1C1ResolvedRef {
                top_row: anchor_row,
                left_col: anchor_col,
                bottom_row: anchor_row,
                right_col: anchor_col,
            }),
            Self::Ref(reference) => {
                let (row, col) = reference.resolve(anchor_row, anchor_col)?;
                Some(GridOptimizedR1C1ResolvedRef {
                    top_row: row,
                    left_col: col,
                    bottom_row: row,
                    right_col: col,
                })
            }
            Self::Range { start, end } => {
                let (start_row, start_col) = start.resolve(anchor_row, anchor_col)?;
                let (end_row, end_col) = end.resolve(anchor_row, anchor_col)?;
                Some(GridOptimizedR1C1ResolvedRef {
                    top_row: start_row.min(end_row),
                    left_col: start_col.min(end_col),
                    bottom_row: start_row.max(end_row),
                    right_col: start_col.max(end_col),
                })
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GridOptimizedR1C1ResolvedRef {
    pub(super) top_row: u32,
    pub(super) left_col: u32,
    pub(super) bottom_row: u32,
    pub(super) right_col: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GridOptimizedR1C1ReferenceFunction {
    Row,
    Column,
    Rows,
    Columns,
}

impl GridOptimizedR1C1ReferenceFunction {
    pub(super) const ALL: [Self; 4] = [Self::Row, Self::Column, Self::Rows, Self::Columns];

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Row => "ROW",
            Self::Column => "COLUMN",
            Self::Rows => "ROWS",
            Self::Columns => "COLUMNS",
        }
    }

    pub(super) const fn allows_current_cell_argument(self) -> bool {
        matches!(self, Self::Row | Self::Column)
    }

    pub(super) fn apply(self, reference: GridOptimizedR1C1ResolvedRef) -> Option<CalcValue> {
        let value = match self {
            Self::Row => f64::from(reference.top_row),
            Self::Column => f64::from(reference.left_col),
            Self::Rows => f64::from(
                reference
                    .bottom_row
                    .checked_sub(reference.top_row)?
                    .saturating_add(1),
            ),
            Self::Columns => f64::from(
                reference
                    .right_col
                    .checked_sub(reference.left_col)?
                    .saturating_add(1),
            ),
        };
        Some(CalcValue::number(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GridOptimizedR1C1ScalarFunction {
    Abs,
    Sqrt,
    Power,
    Mod,
    Round,
    RoundUp,
    RoundDown,
    Int,
    Trunc,
    Exp,
    Ln,
    Log10,
    Log,
    Sin,
    Cos,
    Tan,
    Radians,
    Degrees,
    Pi,
}

impl GridOptimizedR1C1ScalarFunction {
    pub(super) const ALL: [Self; 19] = [
        Self::Abs,
        Self::Sqrt,
        Self::Power,
        Self::Mod,
        Self::Round,
        Self::RoundUp,
        Self::RoundDown,
        Self::Int,
        Self::Trunc,
        Self::Exp,
        Self::Ln,
        Self::Log10,
        Self::Log,
        Self::Sin,
        Self::Cos,
        Self::Tan,
        Self::Radians,
        Self::Degrees,
        Self::Pi,
    ];

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Abs => "ABS",
            Self::Sqrt => "SQRT",
            Self::Power => "POWER",
            Self::Mod => "MOD",
            Self::Round => "ROUND",
            Self::RoundUp => "ROUNDUP",
            Self::RoundDown => "ROUNDDOWN",
            Self::Int => "INT",
            Self::Trunc => "TRUNC",
            Self::Exp => "EXP",
            Self::Ln => "LN",
            Self::Log10 => "LOG10",
            Self::Log => "LOG",
            Self::Sin => "SIN",
            Self::Cos => "COS",
            Self::Tan => "TAN",
            Self::Radians => "RADIANS",
            Self::Degrees => "DEGREES",
            Self::Pi => "PI",
        }
    }

    pub(super) const fn arity_holds(self, arity: usize) -> bool {
        match self {
            Self::Abs
            | Self::Sqrt
            | Self::Int
            | Self::Exp
            | Self::Ln
            | Self::Log10
            | Self::Sin
            | Self::Cos
            | Self::Tan
            | Self::Radians
            | Self::Degrees => arity == 1,
            Self::Power | Self::Mod | Self::Round | Self::RoundUp | Self::RoundDown => arity == 2,
            Self::Trunc => arity == 1 || arity == 2,
            Self::Log => arity == 1 || arity == 2,
            Self::Pi => arity == 0,
        }
    }

    pub(super) fn apply(self, arguments: &[GridOptimizedR1C1Value]) -> CalcValue {
        let number_at = |index: usize| match arguments.get(index).copied() {
            Some(GridOptimizedR1C1Value::Number(number)) => Ok(number),
            Some(GridOptimizedR1C1Value::Error(error)) => Err(CalcValue::error(error)),
            None => Err(CalcValue::error(WorksheetErrorCode::Value)),
        };
        let result = match self {
            Self::Abs => match number_at(0) {
                Ok(number) => number.abs(),
                Err(error) => return error,
            },
            Self::Sqrt => match number_at(0) {
                Ok(number) if number >= 0.0 => number.sqrt(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Power => {
                let base = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let exponent = match number_at(1) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                base.powf(exponent)
            }
            Self::Mod => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let divisor = match number_at(1) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                if divisor == 0.0 {
                    return CalcValue::error(WorksheetErrorCode::Div0);
                }
                let result = number - divisor * (number / divisor).floor();
                if result == 0.0 { 0.0 } else { result }
            }
            Self::Round | Self::RoundUp | Self::RoundDown => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let digits = match number_at(1) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                return self.apply_rounding(number, digits);
            }
            Self::Int => match number_at(0) {
                Ok(number) => {
                    if number.is_finite() {
                        let result = number.floor();
                        if result == 0.0 { 0.0 } else { result }
                    } else {
                        return CalcValue::error(WorksheetErrorCode::Num);
                    }
                }
                Err(error) => return error,
            },
            Self::Trunc => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let digits = match arguments.get(1).copied() {
                    Some(GridOptimizedR1C1Value::Number(number)) => number,
                    Some(GridOptimizedR1C1Value::Error(error)) => {
                        return CalcValue::error(error);
                    }
                    None => 0.0,
                };
                return Self::RoundDown.apply_rounding(number, digits);
            }
            Self::Exp => match number_at(0) {
                Ok(number) if number.is_finite() => number.exp(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Ln => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                return Self::apply_logarithm(number, std::f64::consts::E);
            }
            Self::Log10 => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                return Self::apply_logarithm(number, 10.0);
            }
            Self::Log => {
                let number = match number_at(0) {
                    Ok(number) => number,
                    Err(error) => return error,
                };
                let base = match arguments.get(1).copied() {
                    Some(GridOptimizedR1C1Value::Number(number)) => number,
                    Some(GridOptimizedR1C1Value::Error(error)) => {
                        return CalcValue::error(error);
                    }
                    None => 10.0,
                };
                return Self::apply_logarithm(number, base);
            }
            Self::Sin => match number_at(0) {
                Ok(number) if number.is_finite() => number.sin(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Cos => match number_at(0) {
                Ok(number) if number.is_finite() => number.cos(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Tan => match number_at(0) {
                Ok(number) if number.is_finite() => number.tan(),
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Radians => match number_at(0) {
                Ok(number) if number.is_finite() => number * std::f64::consts::PI / 180.0,
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Degrees => match number_at(0) {
                Ok(number) if number.is_finite() => number * 180.0 / std::f64::consts::PI,
                Ok(_) => return CalcValue::error(WorksheetErrorCode::Num),
                Err(error) => return error,
            },
            Self::Pi => std::f64::consts::PI,
        };
        if result.is_finite() {
            CalcValue::number(result)
        } else {
            CalcValue::error(WorksheetErrorCode::Num)
        }
    }

    pub(super) fn apply_rounding(self, number: f64, digits: f64) -> CalcValue {
        if !number.is_finite() || !digits.is_finite() {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let digits = digits.trunc();
        if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&digits) {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let digits = digits as i32;
        let exponent = digits.unsigned_abs();
        if exponent > 308 {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let scale = 10_f64.powi(i32::try_from(exponent).unwrap_or(308));
        if !scale.is_finite() {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let scaled = if digits >= 0 {
            number * scale
        } else {
            number / scale
        };
        if !scaled.is_finite() {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        let rounded = match self {
            Self::Round => scaled.round(),
            Self::RoundUp if scaled.is_sign_negative() => scaled.floor(),
            Self::RoundUp => scaled.ceil(),
            Self::RoundDown => scaled.trunc(),
            Self::Abs
            | Self::Sqrt
            | Self::Power
            | Self::Mod
            | Self::Int
            | Self::Trunc
            | Self::Exp
            | Self::Ln
            | Self::Log10
            | Self::Log
            | Self::Sin
            | Self::Cos
            | Self::Tan
            | Self::Radians
            | Self::Degrees
            | Self::Pi => return CalcValue::error(WorksheetErrorCode::Value),
        };
        let result = if digits >= 0 {
            rounded / scale
        } else {
            rounded * scale
        };
        if !result.is_finite() {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        if result == 0.0 {
            CalcValue::number(0.0)
        } else {
            CalcValue::number(result)
        }
    }

    pub(super) fn apply_logarithm(number: f64, base: f64) -> CalcValue {
        if !number.is_finite() || !base.is_finite() || number <= 0.0 || base <= 0.0 {
            return CalcValue::error(WorksheetErrorCode::Num);
        }
        if base == 1.0 {
            return CalcValue::error(WorksheetErrorCode::Div0);
        }
        let result = if base == std::f64::consts::E {
            number.ln()
        } else if base == 10.0 {
            number.log10()
        } else if base == 2.0 {
            number.log2()
        } else {
            number.ln() / base.ln()
        };
        if result.is_finite() {
            CalcValue::number(result)
        } else {
            CalcValue::error(WorksheetErrorCode::Num)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum GridOptimizedR1C1AggregateArgument {
    Scalar(GridOptimizedR1C1ScalarExpression),
    Range {
        start: GridOptimizedR1C1Ref,
        end: GridOptimizedR1C1Ref,
    },
}

impl GridOptimizedR1C1AggregateArgument {
    pub(super) fn accumulate_repeated_region_cell(
        &self,
        function: GridOptimizedR1C1RangeAggregateFunction,
        state: &mut GridOptimizedR1C1AggregateState,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<Result<(), CalcValue>> {
        match self {
            Self::Scalar(expression) => Some(state.accumulate(
                function,
                expression.value_for_repeated_region_cell(
                    row,
                    col,
                    region,
                    row_major_formula_values,
                    valuation,
                )?,
            )),
            Self::Range { start, end } => {
                let (start_row, start_col) = start.resolve(row, col)?;
                let (end_row, end_col) = end.resolve(row, col)?;
                accumulate_optimized_r1c1_rect(
                    function,
                    start_row.min(end_row),
                    start_col.min(end_col),
                    start_row.max(end_row),
                    start_col.max(end_col),
                    Some(region),
                    row_major_formula_values,
                    valuation,
                    state,
                )
            }
        }
    }

    pub(super) fn accumulate_single_cell(
        &self,
        function: GridOptimizedR1C1RangeAggregateFunction,
        state: &mut GridOptimizedR1C1AggregateState,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<Result<(), CalcValue>> {
        match self {
            Self::Scalar(expression) => Some(state.accumulate(
                function,
                expression.value_for_single_cell(address, valuation)?,
            )),
            Self::Range { start, end } => {
                let (start_row, start_col) = start.resolve(address.row, address.col)?;
                let (end_row, end_col) = end.resolve(address.row, address.col)?;
                accumulate_optimized_r1c1_rect(
                    function,
                    start_row.min(end_row),
                    start_col.min(end_col),
                    start_row.max(end_row),
                    start_col.max(end_col),
                    None,
                    &[],
                    valuation,
                    state,
                )
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOptimizedR1C1Comparison {
    pub(super) left: GridOptimizedR1C1ScalarExpression,
    pub(super) op: GridOptimizedR1C1ComparisonOp,
    pub(super) right: GridOptimizedR1C1ScalarExpression,
}

impl GridOptimizedR1C1Comparison {
    pub(super) fn evaluate_repeated_region_cell(
        &self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        let left = self.left.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        let right = self.right.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?;
        Some(self.op.apply(left, right))
    }

    pub(super) fn evaluate_single_cell(
        &self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1ConditionValue> {
        let left = self.left.value_for_single_cell(address, valuation)?;
        let right = self.right.value_for_single_cell(address, valuation)?;
        Some(self.op.apply(left, right))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridOptimizedR1C1ComparisonOp {
    LessThan,
    LessThanOrEqual,
    Equal,
    NotEqual,
    GreaterThanOrEqual,
    GreaterThan,
}

impl GridOptimizedR1C1ComparisonOp {
    pub(super) fn apply(
        self,
        left: GridOptimizedR1C1Value,
        right: GridOptimizedR1C1Value,
    ) -> GridOptimizedR1C1ConditionValue {
        let left = match left {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => {
                return GridOptimizedR1C1ConditionValue::Error(error);
            }
        };
        let right = match right {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => {
                return GridOptimizedR1C1ConditionValue::Error(error);
            }
        };
        let result = match self {
            Self::LessThan => left < right,
            Self::LessThanOrEqual => left <= right,
            Self::Equal => left == right,
            Self::NotEqual => left != right,
            Self::GreaterThanOrEqual => left >= right,
            Self::GreaterThan => left > right,
        };
        GridOptimizedR1C1ConditionValue::Logical(result)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GridOptimizedR1C1ConditionValue {
    Logical(bool),
    Error(WorksheetErrorCode),
}

impl GridOptimizedR1C1ConditionValue {
    pub(super) fn into_calc_value(self) -> Option<CalcValue> {
        match self {
            Self::Logical(value) => Some(CalcValue::logical(value)),
            Self::Error(error) => Some(CalcValue::error(error)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GridOptimizedR1C1RangeAggregateFunction {
    Sum,
    Count,
    Product,
    Average,
    Min,
    Max,
    SumSq,
}

impl GridOptimizedR1C1RangeAggregateFunction {
    pub(super) const ALL: [Self; 7] = [
        Self::Sum,
        Self::Count,
        Self::Product,
        Self::Average,
        Self::Min,
        Self::Max,
        Self::SumSq,
    ];

    pub(super) const fn name(self) -> &'static str {
        match self {
            Self::Sum => "SUM",
            Self::Count => "COUNT",
            Self::Product => "PRODUCT",
            Self::Average => "AVERAGE",
            Self::Min => "MIN",
            Self::Max => "MAX",
            Self::SumSq => "SUMSQ",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridOptimizedR1C1BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl GridOptimizedR1C1BinaryOp {
    #[must_use]
    pub(super) fn apply(
        self,
        left: GridOptimizedR1C1Value,
        right: GridOptimizedR1C1Value,
    ) -> Option<CalcValue> {
        let left = match left {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => return Some(CalcValue::error(error)),
        };
        let right = match right {
            GridOptimizedR1C1Value::Number(number) => number,
            GridOptimizedR1C1Value::Error(error) => return Some(CalcValue::error(error)),
        };
        match self {
            Self::Add => Some(CalcValue::number(left + right)),
            Self::Subtract => Some(CalcValue::number(left - right)),
            Self::Multiply => Some(CalcValue::number(left * right)),
            Self::Divide if right == 0.0 => Some(CalcValue::error(WorksheetErrorCode::Div0)),
            Self::Divide => Some(CalcValue::number(left / right)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum GridOptimizedR1C1Value {
    Number(f64),
    Error(WorksheetErrorCode),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridOptimizedR1C1Ref {
    pub(super) row: GridOptimizedR1C1AxisRef,
    pub(super) col: GridOptimizedR1C1AxisRef,
}

impl GridOptimizedR1C1Ref {
    pub(super) fn resolve(self, anchor_row: u32, anchor_col: u32) -> Option<(u32, u32)> {
        Some((self.row.resolve(anchor_row)?, self.col.resolve(anchor_col)?))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum GridOptimizedR1C1AxisRef {
    Relative(i32),
    Absolute(u32),
}

impl GridOptimizedR1C1AxisRef {
    pub(super) fn resolve(self, anchor: u32) -> Option<u32> {
        match self {
            Self::Relative(delta) => add_i32_to_u32(anchor, delta),
            Self::Absolute(value) => Some(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridOptimizedR1C1Operand {
    Ref(GridOptimizedR1C1Ref),
    Number(GridOptimizedR1C1NumberLiteral),
}

impl GridOptimizedR1C1Operand {
    pub(super) fn value_for_repeated_region_cell(
        self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1Value> {
        match self {
            Self::Number(value) => Some(GridOptimizedR1C1Value::Number(value.as_f64())),
            Self::Ref(reference) => {
                let (target_row, target_col) = reference.resolve(row, col)?;
                optimized_r1c1_value_for_cell(
                    target_row,
                    target_col,
                    Some(region),
                    row_major_formula_values,
                    valuation,
                )
            }
        }
    }

    pub(super) fn value_for_single_cell(
        self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<GridOptimizedR1C1Value> {
        match self {
            Self::Number(value) => Some(GridOptimizedR1C1Value::Number(value.as_f64())),
            Self::Ref(reference) => {
                let (target_row, target_col) = reference.resolve(address.row, address.col)?;
                optimized_r1c1_value_for_cell(target_row, target_col, None, &[], valuation)
            }
        }
    }

    pub(super) fn calc_value_for_repeated_region_cell(
        self,
        row: u32,
        col: u32,
        region: &GridRepeatedFormulaRegion,
        row_major_formula_values: &[CalcValue],
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        optimized_r1c1_calc_value_from_value(self.value_for_repeated_region_cell(
            row,
            col,
            region,
            row_major_formula_values,
            valuation,
        )?)
    }

    pub(super) fn calc_value_for_single_cell(
        self,
        address: &ExcelGridCellAddress,
        valuation: &GridOptimizedValuation,
    ) -> Option<CalcValue> {
        optimized_r1c1_calc_value_from_value(self.value_for_single_cell(address, valuation)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridOptimizedR1C1NumberLiteral {
    pub(super) bits: u64,
}

impl GridOptimizedR1C1NumberLiteral {
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        let normalized = if value == 0.0 { 0.0 } else { value };
        Some(Self {
            bits: normalized.to_bits(),
        })
    }

    #[must_use]
    pub const fn as_f64(self) -> f64 {
        f64::from_bits(self.bits)
    }
}

pub(super) fn r1c1_number_literal_expression(
    value: f64,
) -> Option<GridOptimizedR1C1ScalarExpression> {
    Some(GridOptimizedR1C1ScalarExpression::Operand(
        GridOptimizedR1C1Operand::Number(GridOptimizedR1C1NumberLiteral::new(value)?),
    ))
}

/// A content fingerprint of a compiled plan's *input*, used only as a
/// collision guard behind the `normal_form_key` cache key.
///
/// W062 R3.10 (template-identity-keyed plan sharing): the fingerprint carries
/// the **normalized R1C1 expression** — the exact text `compile` consumes —
/// not the raw authored `source_text`. Two placements whose dollar-form,
/// whitespace, or case differ but that share one R1C1 normal form
/// (`=RC[-1]*2` vs `= rc[-1] * 2`) normalize identically, so they share one
/// compiled plan instead of each forcing a recompile under a raw-text gate.
/// The key stays `normal_form_key` (genuine caller-independent
/// `FormulaTemplateIdentity`); this fingerprint only ensures a mis-minted key
/// cannot serve a plan compiled from genuinely different input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GridOptimizedFormulaPlanFingerprint {
    pub(super) normalized_expression: String,
    pub(super) source_channel: FormulaChannelKind,
}

impl GridOptimizedFormulaPlanFingerprint {
    pub(super) fn from_formula(formula: &GridFormulaCell) -> Self {
        // Normalize to the plan-input form (whitespace-stripped, upper-cased,
        // `=`-stripped). Fall back to the trimmed source text when the formula
        // is not an `=`-prefixed expression, so the fingerprint remains a total
        // function without pretending unlike inputs are alike.
        let normalized_expression = normalized_r1c1_expression(&formula.source_text)
            .unwrap_or_else(|| formula.source_text.trim().to_ascii_uppercase());
        Self {
            normalized_expression,
            source_channel: formula.source_channel,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct GridOptimizedCompiledFormulaPlanEntry {
    pub(super) fingerprint: GridOptimizedFormulaPlanFingerprint,
    pub(super) plan: GridOptimizedCompiledFormulaPlan,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridOptimizedFormulaPlanCache {
    pub(super) templates: BTreeSet<String>,
    pub(super) compiled_plans: BTreeMap<String, GridOptimizedCompiledFormulaPlanEntry>,
}

impl GridOptimizedFormulaPlanCache {
    #[must_use]
    pub fn cached_template_count(&self) -> usize {
        self.templates.len()
    }

    #[must_use]
    pub fn cached_compiled_plan_count(&self) -> usize {
        self.compiled_plans.len()
    }

    #[must_use]
    pub fn contains_template(&self, normal_form_key: &str) -> bool {
        self.templates.contains(normal_form_key)
    }

    #[must_use]
    pub fn contains_compiled_plan(&self, normal_form_key: &str) -> bool {
        self.compiled_plans.contains_key(normal_form_key)
    }

    #[must_use]
    pub(super) fn compiled_plan_for_formula(
        &self,
        formula: &GridFormulaCell,
    ) -> Option<GridOptimizedCompiledFormulaPlan> {
        let fingerprint = GridOptimizedFormulaPlanFingerprint::from_formula(formula);
        self.compiled_plans
            .get(&formula.normal_form_key)
            .filter(|entry| entry.fingerprint == fingerprint)
            .map(|entry| entry.plan.clone())
            .or_else(|| GridOptimizedCompiledFormulaPlan::compile(formula))
    }

    pub(super) fn prune_to_templates(&mut self, active_templates: &BTreeSet<String>) {
        self.templates
            .retain(|template| active_templates.contains(template));
        self.compiled_plans
            .retain(|template, _| active_templates.contains(template));
    }
}
