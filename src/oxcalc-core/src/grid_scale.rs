#![forbid(unsafe_code)]

//! Procedural scale/performance runner for the W061 strict Excel grid lane.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use oxfunc_core::value::{CalcValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode};
use serde_json::{Value, json};
use thiserror::Error;

use crate::coordinator::calc_value_display_text;
use crate::excel_grid_reference::{
    ExcelGridBounds, ExcelGridCellAddress, ExcelGridReferenceSystemProvider, ExcelGridResolvedRect,
};
use crate::grid_reference_machine::{
    GridAuthoredCell, GridAxisEdit, GridAxisProps, GridAxisVisibilityDependency, GridDependency,
    GridEngineMode, GridFormulaCell, GridInvalidationRef, GridOptimizedCellSource,
    GridOptimizedSheet, GridOptimizedStructuralEditReport, GridOptimizedValuation, GridRect,
    GridRefError, GridSpillDependency, GridSpillEpochLedger, GridSpillFact,
};
use oxfml_core::source::FormulaChannelKind;

const GRID_SCALE_RUN_SUMMARY_SCHEMA_V1: &str = "oxcalc.grid.scale_run_summary.v1";
const GRID_SCALE_COUNTER_SUMMARY_SCHEMA_V1: &str = "oxcalc.grid.scale_counter_summary.v1";
const GRID_SCALE_REGISTER_ASSERTIONS_SCHEMA_V1: &str = "oxcalc.grid.scale_register_assertions.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridScaleProfile {
    SparseWholeColumn,
    FullColumn1M,
    SparseSingletons,
    ZigZag1M,
    DenseValues,
    RepeatedR1C1,
    FillDownR1C1,
    PascalR1C1,
    Boring1Mx10,
    DirectR1C1,
    UnaryR1C1,
    ArgumentAggregateR1C1,
    MathFunctionR1C1,
    ModFunctionR1C1,
    RoundingFunctionR1C1,
    IntegerFunctionR1C1,
    LogFunctionR1C1,
    TrigFunctionR1C1,
    AngleFunctionR1C1,
    ReferenceFunctionR1C1,
    LogicalFunctionR1C1,
    IfLogicalR1C1,
    TwoLeftR1C1,
    AbsoluteR1C1,
    DivisionR1C1,
    DecimalLiteralR1C1,
    RecursiveBinaryR1C1,
    IfR1C1,
    IfBranchR1C1,
    NestedIfR1C1,
    IfErrorR1C1,
    ComparisonR1C1,
    ComparisonExpressionR1C1,
    ComparisonIfErrorR1C1,
    SumRowR1C1,
    SumSqRowR1C1,
    CountRowR1C1,
    ProductRowR1C1,
    AverageRowR1C1,
    MinMaxRowR1C1,
    SumWindowR1C1,
    DivisionErrorR1C1,
    DivisionErrorPropagationR1C1,
    AggregateErrorR1C1,
    TextFunctionR1C1,
    IndexFunctionR1C1,
    MatchFunctionR1C1,
    VLookupFunctionR1C1,
    InsertStorm1M,
    PublicationDelta1M,
    TileStream64K,
    Viewport64K,
    CowRetention1M,
    PlanCacheRounds1M,
    RangeInvalidation1M,
    RangeQuery1M,
    SumPyramid1M,
    DirtyRect1M,
    HideStorm1M,
    SpillAnchor1M,
    SpillBlockage1M,
    AggregateContext1M,
    SpillEpoch1M,
    FilterSpill1M,
    SequenceSpill1M,
}

impl GridScaleProfile {
    #[must_use]
    pub fn parse(input: &str) -> Option<Self> {
        match input.to_ascii_lowercase().as_str() {
            "sparse-whole-column" | "full-column" | "p20" => Some(Self::SparseWholeColumn),
            "full-column-1m" | "dense-full-column" | "p20-full-column" => Some(Self::FullColumn1M),
            "sparse-singletons" | "p10-sparse" => Some(Self::SparseSingletons),
            "zig-zag" | "zig-zag-1m" | "p10-zig-zag" => Some(Self::ZigZag1M),
            "dense-values" | "dense" => Some(Self::DenseValues),
            "repeated-r1c1" | "r1c1" | "p11" => Some(Self::RepeatedR1C1),
            "fill-down-r1c1" | "fill-down" | "fill-down-1m" | "p11-fill-down" => {
                Some(Self::FillDownR1C1)
            }
            "pascal-r1c1-1m" | "pascal-r1c1" | "r1c1-pascal" | "p11-pascal" => {
                Some(Self::PascalR1C1)
            }
            "boring-1mx10" | "boring" | "p10-boring" => Some(Self::Boring1Mx10),
            "direct-r1c1-1m" | "direct-r1c1" | "copy-r1c1" | "r1c1-direct" => {
                Some(Self::DirectR1C1)
            }
            "unary-r1c1-1m" | "unary-r1c1" | "negate-r1c1" | "r1c1-unary" => Some(Self::UnaryR1C1),
            "argument-aggregate-r1c1-1m"
            | "argument-aggregate-r1c1"
            | "arg-aggregate-r1c1-1m"
            | "aggregate-args-r1c1" => Some(Self::ArgumentAggregateR1C1),
            "math-function-r1c1-1m"
            | "math-function-r1c1"
            | "scalar-function-r1c1"
            | "r1c1-math-function" => Some(Self::MathFunctionR1C1),
            "mod-function-r1c1-1m" | "mod-function-r1c1" | "mod-r1c1-1m" | "r1c1-mod-function" => {
                Some(Self::ModFunctionR1C1)
            }
            "rounding-function-r1c1-1m"
            | "rounding-function-r1c1"
            | "round-r1c1-1m"
            | "r1c1-rounding-function" => Some(Self::RoundingFunctionR1C1),
            "integer-function-r1c1-1m"
            | "integer-function-r1c1"
            | "int-trunc-r1c1-1m"
            | "r1c1-integer-function" => Some(Self::IntegerFunctionR1C1),
            "log-function-r1c1-1m"
            | "log-function-r1c1"
            | "exp-log-r1c1-1m"
            | "r1c1-log-function" => Some(Self::LogFunctionR1C1),
            "trig-function-r1c1-1m"
            | "trig-function-r1c1"
            | "sin-cos-tan-r1c1-1m"
            | "r1c1-trig-function" => Some(Self::TrigFunctionR1C1),
            "angle-function-r1c1-1m"
            | "angle-function-r1c1"
            | "radians-degrees-r1c1-1m"
            | "r1c1-angle-function" => Some(Self::AngleFunctionR1C1),
            "reference-function-r1c1-1m"
            | "reference-function-r1c1"
            | "row-column-r1c1-1m"
            | "r1c1-reference-function" => Some(Self::ReferenceFunctionR1C1),
            "logical-function-r1c1-1m"
            | "logical-function-r1c1"
            | "logical-r1c1-1m"
            | "r1c1-logical-function" => Some(Self::LogicalFunctionR1C1),
            "if-logical-r1c1-1m" | "if-logical-r1c1" | "logical-if-r1c1-1m" | "r1c1-if-logical" => {
                Some(Self::IfLogicalR1C1)
            }
            "two-left-r1c1-1m" | "two-left-r1c1" | "r1c1-two-left" => Some(Self::TwoLeftR1C1),
            "absolute-r1c1-1m" | "absolute-r1c1" | "r1c1-absolute" => Some(Self::AbsoluteR1C1),
            "division-r1c1-1m" | "division-r1c1" | "r1c1-division" => Some(Self::DivisionR1C1),
            "decimal-r1c1-1m" | "decimal-r1c1" | "r1c1-decimal" => Some(Self::DecimalLiteralR1C1),
            "recursive-binary-r1c1-1m"
            | "recursive-binary-r1c1"
            | "precedence-r1c1-1m"
            | "r1c1-recursive-binary" => Some(Self::RecursiveBinaryR1C1),
            "if-r1c1-1m" | "if-r1c1" | "r1c1-if" => Some(Self::IfR1C1),
            "if-branch-r1c1-1m" | "if-branch-r1c1" | "r1c1-if-branch" => Some(Self::IfBranchR1C1),
            "nested-if-r1c1-1m" | "nested-if-r1c1" | "r1c1-nested-if" => Some(Self::NestedIfR1C1),
            "iferror-r1c1-1m" | "iferror-r1c1" | "r1c1-iferror" => Some(Self::IfErrorR1C1),
            "comparison-r1c1-1m" | "comparison-r1c1" | "r1c1-comparison" => {
                Some(Self::ComparisonR1C1)
            }
            "comparison-expression-r1c1-1m"
            | "comparison-expression-r1c1"
            | "r1c1-comparison-expression" => Some(Self::ComparisonExpressionR1C1),
            "comparison-iferror-r1c1-1m"
            | "comparison-iferror-r1c1"
            | "r1c1-comparison-iferror" => Some(Self::ComparisonIfErrorR1C1),
            "sum-row-r1c1-1m" | "sum-row-r1c1" | "row-sum-r1c1" | "r1c1-row-sum" => {
                Some(Self::SumRowR1C1)
            }
            "sumsq-row-r1c1-1m" | "sumsq-row-r1c1" | "row-sumsq-r1c1" | "r1c1-row-sumsq" => {
                Some(Self::SumSqRowR1C1)
            }
            "count-row-r1c1-1m" | "count-row-r1c1" | "row-count-r1c1" | "r1c1-row-count" => {
                Some(Self::CountRowR1C1)
            }
            "product-row-r1c1-1m"
            | "product-row-r1c1"
            | "row-product-r1c1"
            | "r1c1-row-product" => Some(Self::ProductRowR1C1),
            "average-row-r1c1-1m"
            | "average-row-r1c1"
            | "row-average-r1c1"
            | "r1c1-row-average" => Some(Self::AverageRowR1C1),
            "min-max-row-r1c1-1m"
            | "min-max-row-r1c1"
            | "row-min-max-r1c1"
            | "r1c1-row-min-max" => Some(Self::MinMaxRowR1C1),
            "sum-window-r1c1-1m" | "sum-window-r1c1" | "window-sum-r1c1" | "r1c1-window-sum" => {
                Some(Self::SumWindowR1C1)
            }
            "division-error-r1c1-1m" | "division-error-r1c1" | "r1c1-division-error" => {
                Some(Self::DivisionErrorR1C1)
            }
            "division-error-propagation-r1c1-1m"
            | "division-error-propagation-r1c1"
            | "r1c1-division-error-propagation" => Some(Self::DivisionErrorPropagationR1C1),
            "aggregate-error-r1c1-1m"
            | "aggregate-error-r1c1"
            | "range-error-r1c1-1m"
            | "r1c1-aggregate-error" => Some(Self::AggregateErrorR1C1),
            "text-function-r1c1-1m"
            | "text-function-r1c1"
            | "text-r1c1-1m"
            | "r1c1-text-function" => Some(Self::TextFunctionR1C1),
            "index-function-r1c1-1m"
            | "index-function-r1c1"
            | "index-r1c1-1m"
            | "r1c1-index-function" => Some(Self::IndexFunctionR1C1),
            "match-function-r1c1-1m"
            | "match-function-r1c1"
            | "match-r1c1-1m"
            | "r1c1-match-function" => Some(Self::MatchFunctionR1C1),
            "vlookup-function-r1c1-1m"
            | "vlookup-function-r1c1"
            | "vlookup-r1c1-1m"
            | "r1c1-vlookup-function" => Some(Self::VLookupFunctionR1C1),
            "insert-storm-1m" | "insert-storm" | "edit-storm" | "p17" => Some(Self::InsertStorm1M),
            "publication-delta-1m" | "publication-delta" | "publish-delta" | "p22" => {
                Some(Self::PublicationDelta1M)
            }
            "tile-stream-64k" | "tile-stream" | "doom-320x200" | "p15" => Some(Self::TileStream64K),
            "viewport-64k-of-1m" | "viewport-64k" | "visible-first" | "p16" => {
                Some(Self::Viewport64K)
            }
            "cow-retention-1m" | "cow-retention" | "retention-1m" | "p21" => {
                Some(Self::CowRetention1M)
            }
            "plan-cache-rounds-1m"
            | "plan-cache-rounds"
            | "persistent-plan-cache"
            | "p14-rounds" => Some(Self::PlanCacheRounds1M),
            "range-invalidation-1m" | "range-invalidation" | "p13" => {
                Some(Self::RangeInvalidation1M)
            }
            "range-query-1m" | "range-query" | "p12" => Some(Self::RangeQuery1M),
            "sum-pyramid-1m" | "sum-pyramid" | "p12-pyramid" | "p13-pyramid" => {
                Some(Self::SumPyramid1M)
            }
            "dirty-rect-1m" | "dirty-rect" | "dirty-rectangle" | "p12-rect" => {
                Some(Self::DirtyRect1M)
            }
            "hide-storm-1m" | "hide-storm" | "p24" => Some(Self::HideStorm1M),
            "spill-anchor-1m" | "spill-anchor" | "p25" => Some(Self::SpillAnchor1M),
            "spill-blockage-1m" | "spill-blockage" | "p26" => Some(Self::SpillBlockage1M),
            "aggregate-context-1m" | "aggregate-context" | "p28" => Some(Self::AggregateContext1M),
            "spill-epoch-1m" | "spill-epoch" | "p27" => Some(Self::SpillEpoch1M),
            "filter-spill-1m" | "filter-spill" | "respill-clear-1m" | "respill-clear" | "p23" => {
                Some(Self::FilterSpill1M)
            }
            "sequence-spill-1m" | "sequence-spill" | "dynamic-spill-1m" | "dynamic-spill" => {
                Some(Self::SequenceSpill1M)
            }
            _ => None,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SparseWholeColumn => "sparse-whole-column",
            Self::FullColumn1M => "full-column-1m",
            Self::SparseSingletons => "sparse-singletons",
            Self::ZigZag1M => "zig-zag-1m",
            Self::DenseValues => "dense-values",
            Self::RepeatedR1C1 => "repeated-r1c1",
            Self::FillDownR1C1 => "fill-down-r1c1",
            Self::PascalR1C1 => "pascal-r1c1-1m",
            Self::Boring1Mx10 => "boring-1mx10",
            Self::DirectR1C1 => "direct-r1c1-1m",
            Self::UnaryR1C1 => "unary-r1c1-1m",
            Self::ArgumentAggregateR1C1 => "argument-aggregate-r1c1-1m",
            Self::MathFunctionR1C1 => "math-function-r1c1-1m",
            Self::ModFunctionR1C1 => "mod-function-r1c1-1m",
            Self::RoundingFunctionR1C1 => "rounding-function-r1c1-1m",
            Self::IntegerFunctionR1C1 => "integer-function-r1c1-1m",
            Self::LogFunctionR1C1 => "log-function-r1c1-1m",
            Self::TrigFunctionR1C1 => "trig-function-r1c1-1m",
            Self::AngleFunctionR1C1 => "angle-function-r1c1-1m",
            Self::ReferenceFunctionR1C1 => "reference-function-r1c1-1m",
            Self::LogicalFunctionR1C1 => "logical-function-r1c1-1m",
            Self::IfLogicalR1C1 => "if-logical-r1c1-1m",
            Self::TwoLeftR1C1 => "two-left-r1c1-1m",
            Self::AbsoluteR1C1 => "absolute-r1c1-1m",
            Self::DivisionR1C1 => "division-r1c1-1m",
            Self::DecimalLiteralR1C1 => "decimal-r1c1-1m",
            Self::RecursiveBinaryR1C1 => "recursive-binary-r1c1-1m",
            Self::IfR1C1 => "if-r1c1-1m",
            Self::IfBranchR1C1 => "if-branch-r1c1-1m",
            Self::NestedIfR1C1 => "nested-if-r1c1-1m",
            Self::IfErrorR1C1 => "iferror-r1c1-1m",
            Self::ComparisonR1C1 => "comparison-r1c1-1m",
            Self::ComparisonExpressionR1C1 => "comparison-expression-r1c1-1m",
            Self::ComparisonIfErrorR1C1 => "comparison-iferror-r1c1-1m",
            Self::SumRowR1C1 => "sum-row-r1c1-1m",
            Self::SumSqRowR1C1 => "sumsq-row-r1c1-1m",
            Self::CountRowR1C1 => "count-row-r1c1-1m",
            Self::ProductRowR1C1 => "product-row-r1c1-1m",
            Self::AverageRowR1C1 => "average-row-r1c1-1m",
            Self::MinMaxRowR1C1 => "min-max-row-r1c1-1m",
            Self::SumWindowR1C1 => "sum-window-r1c1-1m",
            Self::DivisionErrorR1C1 => "division-error-r1c1-1m",
            Self::DivisionErrorPropagationR1C1 => "division-error-propagation-r1c1-1m",
            Self::AggregateErrorR1C1 => "aggregate-error-r1c1-1m",
            Self::TextFunctionR1C1 => "text-function-r1c1-1m",
            Self::IndexFunctionR1C1 => "index-function-r1c1-1m",
            Self::MatchFunctionR1C1 => "match-function-r1c1-1m",
            Self::VLookupFunctionR1C1 => "vlookup-function-r1c1-1m",
            Self::InsertStorm1M => "insert-storm-1m",
            Self::PublicationDelta1M => "publication-delta-1m",
            Self::TileStream64K => "tile-stream-64k",
            Self::Viewport64K => "viewport-64k-of-1m",
            Self::CowRetention1M => "cow-retention-1m",
            Self::PlanCacheRounds1M => "plan-cache-rounds-1m",
            Self::RangeInvalidation1M => "range-invalidation-1m",
            Self::RangeQuery1M => "range-query-1m",
            Self::SumPyramid1M => "sum-pyramid-1m",
            Self::DirtyRect1M => "dirty-rect-1m",
            Self::HideStorm1M => "hide-storm-1m",
            Self::SpillAnchor1M => "spill-anchor-1m",
            Self::SpillBlockage1M => "spill-blockage-1m",
            Self::AggregateContext1M => "aggregate-context-1m",
            Self::SpillEpoch1M => "spill-epoch-1m",
            Self::FilterSpill1M => "filter-spill-1m",
            Self::SequenceSpill1M => "sequence-spill-1m",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridScaleOptions {
    pub run_id: String,
    pub profile: GridScaleProfile,
    pub rows: u32,
    pub cols: u32,
}

impl GridScaleOptions {
    #[must_use]
    pub fn default_for(profile: GridScaleProfile, run_id: impl Into<String>) -> Self {
        match profile {
            GridScaleProfile::SparseWholeColumn => Self {
                run_id: run_id.into(),
                profile,
                rows: ExcelGridBounds::strict_excel().max_rows,
                cols: 2,
            },
            GridScaleProfile::FullColumn1M => Self {
                run_id: run_id.into(),
                profile,
                rows: ExcelGridBounds::strict_excel().max_rows,
                cols: 2,
            },
            GridScaleProfile::SparseSingletons => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 16,
            },
            GridScaleProfile::ZigZag1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: ExcelGridBounds::strict_excel().max_cols,
            },
            GridScaleProfile::DenseValues => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000,
                cols: 10,
            },
            GridScaleProfile::RepeatedR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000,
                cols: 2,
            },
            GridScaleProfile::FillDownR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 1,
            },
            GridScaleProfile::PascalR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 8,
            },
            GridScaleProfile::Boring1Mx10 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 10,
            },
            GridScaleProfile::DirectR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::UnaryR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::ArgumentAggregateR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 8,
            },
            GridScaleProfile::MathFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::ModFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::RoundingFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::IntegerFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::LogFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 6,
            },
            GridScaleProfile::TrigFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::AngleFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 6,
            },
            GridScaleProfile::ReferenceFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 6,
            },
            GridScaleProfile::LogicalFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::IfLogicalR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::TwoLeftR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 10,
            },
            GridScaleProfile::AbsoluteR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::DivisionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::DecimalLiteralR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::RecursiveBinaryR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::IfR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::IfBranchR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::NestedIfR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::IfErrorR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::ComparisonR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::ComparisonExpressionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::ComparisonIfErrorR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::SumRowR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::SumSqRowR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::CountRowR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::ProductRowR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::AverageRowR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::MinMaxRowR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::SumWindowR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::DivisionErrorR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::DivisionErrorPropagationR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::AggregateErrorR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::TextFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::IndexFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 6,
            },
            GridScaleProfile::MatchFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 6,
            },
            GridScaleProfile::VLookupFunctionR1C1 => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 7,
            },
            GridScaleProfile::InsertStorm1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 10,
            },
            GridScaleProfile::PublicationDelta1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::TileStream64K => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 320,
            },
            GridScaleProfile::Viewport64K => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 10,
            },
            GridScaleProfile::CowRetention1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 10,
            },
            GridScaleProfile::PlanCacheRounds1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 10,
            },
            GridScaleProfile::RangeInvalidation1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::RangeQuery1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::SumPyramid1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 6,
            },
            GridScaleProfile::DirtyRect1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::HideStorm1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 3,
            },
            GridScaleProfile::SpillAnchor1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::SpillBlockage1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::AggregateContext1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 2,
            },
            GridScaleProfile::SpillEpoch1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 4,
            },
            GridScaleProfile::FilterSpill1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 5,
            },
            GridScaleProfile::SequenceSpill1M => Self {
                run_id: run_id.into(),
                profile,
                rows: 1_000_000,
                cols: 1,
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridScaleRunSummary {
    pub run_id: String,
    pub profile: String,
    pub artifact_root: String,
    pub rows: u32,
    pub cols: u32,
    pub register_assertion_count: usize,
    pub failed_register_assertion_count: usize,
}

#[derive(Debug, Error)]
pub enum GridScaleRunnerError {
    #[error("invalid grid scale options: {detail}")]
    InvalidOptions { detail: String },
    #[error(transparent)]
    Grid(#[from] GridRefError),
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
}

#[derive(Debug, Clone, Default)]
pub struct GridScaleRunner;

impl GridScaleRunner {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        repo_root: &Path,
        options: GridScaleOptions,
    ) -> Result<GridScaleRunSummary, GridScaleRunnerError> {
        validate_grid_scale_options(&options)?;

        let artifact_root = repo_root.join(format!(
            "docs/test-runs/core-engine/grid-scale/{}",
            options.run_id
        ));
        let relative_artifact_root = relative_artifact_path([
            "docs",
            "test-runs",
            "core-engine",
            "grid-scale",
            options.run_id.as_str(),
        ]);

        if artifact_root.exists() {
            fs::remove_dir_all(&artifact_root).map_err(|source| {
                GridScaleRunnerError::RemoveDirectory {
                    path: artifact_root.display().to_string(),
                    source,
                }
            })?;
        }
        create_directory(&artifact_root)?;

        let execution = execute_grid_scale_model(&options, &relative_artifact_root)?;
        write_json(
            &artifact_root.join("run_summary.json"),
            &execution.run_summary_json,
        )?;
        write_json(
            &artifact_root.join("counter_summary.json"),
            &execution.counter_summary_json,
        )?;
        write_json(
            &artifact_root.join("register_assertions.json"),
            &execution.register_assertions_json,
        )?;

        Ok(GridScaleRunSummary {
            artifact_root: artifact_root.display().to_string(),
            ..execution.summary
        })
    }
}

#[derive(Debug)]
struct GridScaleExecution {
    summary: GridScaleRunSummary,
    run_summary_json: Value,
    counter_summary_json: Value,
    register_assertions_json: Value,
}

fn execute_grid_scale_model(
    options: &GridScaleOptions,
    relative_artifact_root: &str,
) -> Result<GridScaleExecution, GridScaleRunnerError> {
    let profile_json = match options.profile {
        GridScaleProfile::SparseWholeColumn => sparse_whole_column_scale(options)?,
        GridScaleProfile::FullColumn1M => full_column_1m_scale(options)?,
        GridScaleProfile::SparseSingletons => sparse_singletons_scale(options)?,
        GridScaleProfile::ZigZag1M => zig_zag_1m_scale(options)?,
        GridScaleProfile::DenseValues => dense_values_scale(options)?,
        GridScaleProfile::RepeatedR1C1 => repeated_r1c1_scale(options)?,
        GridScaleProfile::FillDownR1C1 => fill_down_r1c1_scale(options)?,
        GridScaleProfile::PascalR1C1 => pascal_r1c1_scale(options)?,
        GridScaleProfile::Boring1Mx10 => boring_1mx10_scale(options)?,
        GridScaleProfile::DirectR1C1 => direct_r1c1_1m_scale(options)?,
        GridScaleProfile::UnaryR1C1 => unary_r1c1_1m_scale(options)?,
        GridScaleProfile::ArgumentAggregateR1C1 => argument_aggregate_r1c1_1m_scale(options)?,
        GridScaleProfile::MathFunctionR1C1 => math_function_r1c1_1m_scale(options)?,
        GridScaleProfile::ModFunctionR1C1 => mod_function_r1c1_1m_scale(options)?,
        GridScaleProfile::RoundingFunctionR1C1 => rounding_function_r1c1_1m_scale(options)?,
        GridScaleProfile::IntegerFunctionR1C1 => integer_function_r1c1_1m_scale(options)?,
        GridScaleProfile::LogFunctionR1C1 => log_function_r1c1_1m_scale(options)?,
        GridScaleProfile::TrigFunctionR1C1 => trig_function_r1c1_1m_scale(options)?,
        GridScaleProfile::AngleFunctionR1C1 => angle_function_r1c1_1m_scale(options)?,
        GridScaleProfile::ReferenceFunctionR1C1 => reference_function_r1c1_1m_scale(options)?,
        GridScaleProfile::LogicalFunctionR1C1 => logical_function_r1c1_1m_scale(options)?,
        GridScaleProfile::IfLogicalR1C1 => if_logical_r1c1_1m_scale(options)?,
        GridScaleProfile::TwoLeftR1C1 => two_left_r1c1_1m_scale(options)?,
        GridScaleProfile::AbsoluteR1C1 => absolute_r1c1_1m_scale(options)?,
        GridScaleProfile::DivisionR1C1 => division_r1c1_1m_scale(options)?,
        GridScaleProfile::DecimalLiteralR1C1 => decimal_r1c1_1m_scale(options)?,
        GridScaleProfile::RecursiveBinaryR1C1 => recursive_binary_r1c1_1m_scale(options)?,
        GridScaleProfile::IfR1C1 => if_r1c1_1m_scale(options)?,
        GridScaleProfile::IfBranchR1C1 => if_branch_r1c1_1m_scale(options)?,
        GridScaleProfile::NestedIfR1C1 => nested_if_r1c1_1m_scale(options)?,
        GridScaleProfile::IfErrorR1C1 => iferror_r1c1_1m_scale(options)?,
        GridScaleProfile::ComparisonR1C1 => comparison_r1c1_1m_scale(options)?,
        GridScaleProfile::ComparisonExpressionR1C1 => comparison_expression_r1c1_1m_scale(options)?,
        GridScaleProfile::ComparisonIfErrorR1C1 => comparison_iferror_r1c1_1m_scale(options)?,
        GridScaleProfile::SumRowR1C1 => sum_row_r1c1_1m_scale(options)?,
        GridScaleProfile::SumSqRowR1C1 => sumsq_row_r1c1_1m_scale(options)?,
        GridScaleProfile::CountRowR1C1 => count_row_r1c1_1m_scale(options)?,
        GridScaleProfile::ProductRowR1C1 => product_row_r1c1_1m_scale(options)?,
        GridScaleProfile::AverageRowR1C1 => average_row_r1c1_1m_scale(options)?,
        GridScaleProfile::MinMaxRowR1C1 => min_max_row_r1c1_1m_scale(options)?,
        GridScaleProfile::SumWindowR1C1 => sum_window_r1c1_1m_scale(options)?,
        GridScaleProfile::DivisionErrorR1C1 => division_error_r1c1_1m_scale(options)?,
        GridScaleProfile::DivisionErrorPropagationR1C1 => {
            division_error_propagation_r1c1_1m_scale(options)?
        }
        GridScaleProfile::AggregateErrorR1C1 => aggregate_error_r1c1_1m_scale(options)?,
        GridScaleProfile::TextFunctionR1C1 => text_function_r1c1_1m_scale(options)?,
        GridScaleProfile::IndexFunctionR1C1 => index_function_r1c1_1m_scale(options)?,
        GridScaleProfile::MatchFunctionR1C1 => match_function_r1c1_1m_scale(options)?,
        GridScaleProfile::VLookupFunctionR1C1 => vlookup_function_r1c1_1m_scale(options)?,
        GridScaleProfile::InsertStorm1M => insert_storm_1m_scale(options)?,
        GridScaleProfile::PublicationDelta1M => publication_delta_1m_scale(options)?,
        GridScaleProfile::TileStream64K => tile_stream_64k_scale(options)?,
        GridScaleProfile::Viewport64K => viewport_64k_of_1m_scale(options)?,
        GridScaleProfile::CowRetention1M => cow_retention_1m_scale(options)?,
        GridScaleProfile::PlanCacheRounds1M => plan_cache_rounds_1m_scale(options)?,
        GridScaleProfile::RangeInvalidation1M => range_invalidation_1m_scale(options)?,
        GridScaleProfile::RangeQuery1M => range_query_1m_scale(options)?,
        GridScaleProfile::SumPyramid1M => sum_pyramid_1m_scale(options)?,
        GridScaleProfile::DirtyRect1M => dirty_rect_1m_scale(options)?,
        GridScaleProfile::HideStorm1M => hide_storm_1m_scale(options)?,
        GridScaleProfile::SpillAnchor1M => spill_anchor_1m_scale(options)?,
        GridScaleProfile::SpillBlockage1M => spill_blockage_1m_scale(options)?,
        GridScaleProfile::AggregateContext1M => aggregate_context_1m_scale(options)?,
        GridScaleProfile::SpillEpoch1M => spill_epoch_1m_scale(options)?,
        GridScaleProfile::FilterSpill1M => filter_spill_1m_scale(options)?,
        GridScaleProfile::SequenceSpill1M => sequence_spill_1m_scale(options)?,
    };
    let assertions = profile_json["register_assertions"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let failed_register_assertion_count = assertions
        .iter()
        .filter(|assertion| assertion["passed"] != true)
        .count();
    let counter_summary_json = json!({
        "schema_version": GRID_SCALE_COUNTER_SUMMARY_SCHEMA_V1,
        "profile": options.profile.as_str(),
        "rows": options.rows,
        "cols": options.cols,
        "counters": profile_json["counters"].clone()
    });
    let register_assertions_json = json!({
        "schema_version": GRID_SCALE_REGISTER_ASSERTIONS_SCHEMA_V1,
        "profile": options.profile.as_str(),
        "assertions": assertions
    });
    let summary = GridScaleRunSummary {
        run_id: options.run_id.clone(),
        profile: options.profile.as_str().to_string(),
        artifact_root: relative_artifact_root.to_string(),
        rows: options.rows,
        cols: options.cols,
        register_assertion_count: register_assertions_json["assertions"]
            .as_array()
            .map_or(0, Vec::len),
        failed_register_assertion_count,
    };
    let run_summary_json = json!({
        "schema_version": GRID_SCALE_RUN_SUMMARY_SCHEMA_V1,
        "run_id": options.run_id,
        "profile": options.profile.as_str(),
        "artifact_root": relative_artifact_root,
        "rows": options.rows,
        "cols": options.cols,
        "register_assertion_count": summary.register_assertion_count,
        "failed_register_assertion_count": failed_register_assertion_count,
        "matched": failed_register_assertion_count == 0,
        "counter_summary_path": format!("{relative_artifact_root}/counter_summary.json"),
        "register_assertions_path": format!("{relative_artifact_root}/register_assertions.json")
    });
    Ok(GridScaleExecution {
        summary,
        run_summary_json,
        counter_summary_json,
        register_assertions_json,
    })
}

fn sparse_whole_column_scale(_options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = ExcelGridBounds::strict_excel();
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let middle_row = bounds.max_rows / 2;
    sheet.set_literal(address(1, 1), CalcValue::number(5.0))?;
    sheet.set_literal(address(middle_row, 1), CalcValue::number(7.0))?;
    sheet.set_literal(address(bounds.max_rows, 1), CalcValue::number(11.0))?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=SUM(A:A)", "excel.grid.v1:sum-whole-column:C1"),
    )?;

    let report =
        sheet.run_engine_mode_with_oxfml(GridEngineMode::Optimized, [address(1, 2)], 100_000)?;
    let p20_reports =
        sheet.optimized_formula_reference_enumeration_reports(&address(1, 2), 100_000)?;
    let Some(p20) = p20_reports.first() else {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sparse whole-column scale produced no P-20 enumeration report".to_string(),
        });
    };
    let computed_sum = report
        .optimized
        .as_ref()
        .and_then(|run| run.readout.first())
        .map(|readout| readout.computed.clone())
        .unwrap_or_else(CalcValue::empty);
    let byte_report = sheet.storage_byte_report();

    Ok(json!({
        "counters": {
            "declared_cell_count": p20.declared_cell_count,
            "defined_cell_count": p20.defined_cell_count,
            "slots_visited": p20.slots_visited(),
            "sparse_value_cells_visited": p20.sparse_value_cells_visited,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "sparse_point_bytes": byte_report.sparse_point_bytes,
            "sparse_point_bytes_per_cell_micros": byte_report.sparse_point_bytes_per_cell_micros(),
            "grid_cell_capacity": byte_report.grid_cell_capacity,
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "computed_sum": calc_value_display_text(&computed_sum)
        },
        "register_assertions": [
            register_assertion(
                "P-10",
                "blank cells require zero authored storage bytes",
                byte_report.p10_blank_cells_zero_bytes_holds()
            ),
            register_assertion(
                "P-20",
                "strict A:A provider enumeration visits occupied slots only",
                p20.p20_occupied_slots_holds()
                    && p20.declared_cell_count == usize::try_from(bounds.max_rows).unwrap()
                    && p20.defined_cell_count == 3
                    && p20.slots_visited() == 3
            )
        ]
    }))
}

fn full_column_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "full-column-1m requires at least 2 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row))?;
    sheet.set_formula(
        address(1, 2),
        GridFormulaCell::new("=SUM(A:A)", "excel.grid.v1:sum-whole-column:C1"),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let p20_reports = sheet
        .optimized_formula_reference_enumeration_reports(&address(1, 2), materialization_limit)?;
    let Some(p20) = p20_reports.first() else {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "full-column-1m scale produced no P-20 enumeration report".to_string(),
        });
    };
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let computed_sum = valuation.read_cell(&address(1, 2)).computed;
    let expected_sum =
        u64::from(options.rows).saturating_mul(u64::from(options.rows).saturating_add(1)) / 2;
    let expected_sum_display = expected_sum.to_string();

    Ok(json!({
        "counters": {
            "declared_cell_count": p20.declared_cell_count,
            "defined_cell_count": p20.defined_cell_count,
            "slots_visited": p20.slots_visited(),
            "dense_value_cells_visited": p20.dense_value_cells_visited,
            "sparse_value_cells_visited": p20.sparse_value_cells_visited,
            "compact_regions_intersected": p20.compact_regions_intersected,
            "dense_value_regions": stats.dense_value_regions,
            "dense_value_cells": stats.dense_value_cells,
            "dense_numeric_packed_cells": byte_report.dense_numeric_packed_cells,
            "sparse_point_cells": stats.sparse_point_cells,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "dense_value_region_bytes": byte_report.dense_value_region_bytes,
            "dense_bytes_per_cell_micros": byte_report.dense_bytes_per_cell_micros(),
            "authored_bytes_per_cell_micros": byte_report.authored_bytes_per_cell_micros(),
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "occupied_cells": recalc.occupied_cells,
            "literal_cells": recalc.literal_cells,
            "formula_cells": recalc.formula_cells,
            "formula_evaluations": recalc.formula_evaluations,
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "computed_sum": calc_value_display_text(&computed_sum),
            "expected_sum": expected_sum_display.clone(),
            "warm_cells_visited": warm.cells_visited,
            "warm_formula_evaluations": warm.formula_evaluations
        },
        "register_assertions": [
            register_assertion(
                "P-00",
                "full-column-1M recalc visits each occupied cell once",
                recalc.p00_primary_exact_once_holds()
                    && recalc.literal_cells == u64::from(options.rows)
                    && recalc.formula_cells == 1
            ),
            register_assertion(
                "P-10",
                "full-column-1M dense numeric authored values stay within 17 B/cell and blanks cost zero bytes",
                byte_report.p10_dense_value_budget_holds()
                    && byte_report.p10_blank_cells_zero_bytes_holds()
            ),
            register_assertion(
                "P-19",
                "unchanged full-column-1M sheet hits warm no-op cache",
                warm.p19_warm_noop_holds()
            ),
            register_assertion(
                "P-20",
                "full-column-1M SUM(A:A) visits occupied dense slots only",
                p20.p20_occupied_slots_holds()
                    && p20.declared_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && p20.defined_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && p20.slots_visited() == u64::from(options.rows)
                    && p20.dense_value_cells_visited == u64::from(options.rows)
                    && p20.sparse_value_cells_visited == 0
                    && p20.compact_regions_intersected == 1
            ),
            register_assertion(
                "GRID-FULL-COLUMN-1M",
                "full-column-1M keeps the occupied column dense and computes the expected aggregate",
                stats.dense_value_regions == 1
                    && stats.dense_value_cells == u64::from(options.rows)
                    && byte_report.dense_numeric_packed_cells == u64::from(options.rows)
                    && stats.sparse_point_cells == 1
                    && calc_value_display_text(&computed_sum) == expected_sum_display
            )
        ]
    }))
}

fn sparse_singletons_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    for row in 1..=options.rows {
        let col = ((row - 1) % options.cols) + 1;
        sheet.set_literal(address(row, col), CalcValue::number(f64::from(row)))?;
    }
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let first_value = sheet
        .authored_cell_at(&address(1, 1))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));
    let middle_row = (options.rows / 2).max(1);
    let middle_col = ((middle_row - 1) % options.cols) + 1;
    let middle_value = sheet
        .authored_cell_at(&address(middle_row, middle_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));
    let last_col = ((options.rows - 1) % options.cols) + 1;
    let last_value = sheet
        .authored_cell_at(&address(options.rows, last_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));

    Ok(json!({
        "counters": {
            "sparse_point_cells": stats.sparse_point_cells,
            "authored_cells_upper_bound": stats.authored_cells_upper_bound,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "sparse_point_bytes": byte_report.sparse_point_bytes,
            "sparse_point_bytes_per_cell_micros": byte_report.sparse_point_bytes_per_cell_micros(),
            "grid_cell_capacity": byte_report.grid_cell_capacity,
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "sample_first_value": authored_cell_display_text(&first_value),
            "sample_middle_value": authored_cell_display_text(&middle_value),
            "sample_last_value": authored_cell_display_text(&last_value)
        },
        "register_assertions": [
            register_assertion(
                "P-10",
                "sparse numeric singletons stay within 85 B/cell and blanks cost zero bytes",
                byte_report.p10_sparse_singleton_budget_holds()
                    && byte_report.p10_blank_cells_zero_bytes_holds()
                    && stats.sparse_point_cells == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && stats.authored_cells_upper_bound == u64::from(options.rows)
            )
        ]
    }))
}

fn zig_zag_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    for row in 1..=options.rows {
        let col = ((row - 1) % options.cols) + 1;
        sheet.set_literal(address(row, col), CalcValue::number(f64::from(row)))?;
    }
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let middle_row = (options.rows / 2).max(1);
    let first_col = zig_zag_col(1, options.cols);
    let middle_col = zig_zag_col(middle_row, options.cols);
    let last_col = zig_zag_col(options.rows, options.cols);
    let first_value = sheet
        .authored_cell_at(&address(1, first_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));
    let middle_value = sheet
        .authored_cell_at(&address(middle_row, middle_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));
    let last_value = sheet
        .authored_cell_at(&address(options.rows, last_col))
        .and_then(|readout| readout.authored)
        .unwrap_or_else(|| GridAuthoredCell::Literal(CalcValue::empty()));

    Ok(json!({
        "counters": {
            "sparse_point_cells": stats.sparse_point_cells,
            "authored_cells_upper_bound": stats.authored_cells_upper_bound,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "sparse_point_bytes": byte_report.sparse_point_bytes,
            "sparse_point_bytes_per_cell_micros": byte_report.sparse_point_bytes_per_cell_micros(),
            "grid_cell_capacity": byte_report.grid_cell_capacity,
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "columns_spanned": options.cols,
            "sample_first_col": first_col,
            "sample_middle_col": middle_col,
            "sample_last_col": last_col,
            "sample_first_value": authored_cell_display_text(&first_value),
            "sample_middle_value": authored_cell_display_text(&middle_value),
            "sample_last_value": authored_cell_display_text(&last_value),
            "partition_sparse_point_cells": partition.sparse_point_cells,
            "partition_dense_value_regions": partition.dense_value_regions,
            "partition_repeated_formula_regions": partition.repeated_formula_regions,
            "partition_dense_value_pair_checks": partition.dense_value_pair_checks,
            "partition_repeated_formula_pair_checks": partition.repeated_formula_pair_checks,
            "partition_dense_value_overlap_count": partition.dense_value_overlap_count,
            "partition_repeated_formula_overlap_count": partition.repeated_formula_overlap_count,
            "partition_max_parallelism_bound": partition.max_parallelism_bound
        },
        "register_assertions": [
            register_assertion(
                "P-10",
                "zig-zag-1M sparse diagonal singletons stay within 85 B/cell and blanks cost zero bytes",
                byte_report.p10_sparse_singleton_budget_holds()
                    && byte_report.p10_blank_cells_zero_bytes_holds()
                    && stats.sparse_point_cells == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && stats.authored_cells_upper_bound == u64::from(options.rows)
            ),
            register_assertion(
                "P-18",
                "zig-zag-1M sparse singleton partition witness is valid and records parallelism bound",
                partition.p18_partition_witness_holds()
                    && partition.sparse_point_cells == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && partition.max_parallelism_bound == u64::from(options.rows)
            ),
            register_assertion(
                "GRID-ZIG-ZAG-1M",
                "zig-zag-1M spans the configured column width with one sparse point per row",
                options.cols == bounds.max_cols
                    && stats.sparse_point_cells == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && authored_cell_display_text(&first_value) == "1"
                    && authored_cell_display_text(&middle_value) == middle_row.to_string()
                    && authored_cell_display_text(&last_value) == options.rows.to_string()
            )
        ]
    }))
}

fn dense_values_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        options.cols,
        bounds,
    )?;
    sheet.put_dense_literal_region_with(rect, |address| {
        CalcValue::number(f64::from(address.row) + f64::from(address.col) / 1000.0)
    })?;
    let (valuation, recalc) = sheet.recalculate_mark_all_dirty_compact_with_oxfml(100_000)?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let expected_cells = u64::from(options.rows) * u64::from(options.cols);

    Ok(json!({
        "counters": {
            "dense_value_regions": stats.dense_value_regions,
            "dense_value_cells": stats.dense_value_cells,
            "dense_numeric_packed_cells": byte_report.dense_numeric_packed_cells,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "dense_value_region_bytes": byte_report.dense_value_region_bytes,
            "dense_bytes_per_cell_micros": byte_report.dense_bytes_per_cell_micros(),
            "authored_bytes_per_cell_micros": byte_report.authored_bytes_per_cell_micros(),
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "sparse_point_cells": stats.sparse_point_cells,
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "literal_cells": recalc.literal_cells,
            "occupied_cells": recalc.occupied_cells
        },
        "register_assertions": [
            register_assertion("P-00", "dense value recalc visits each occupied value once", recalc.p00_primary_exact_once_holds()),
            register_assertion("P-10", "dense numeric authored values stay within 17 B/cell and blanks cost zero bytes", byte_report.p10_dense_value_budget_holds() && byte_report.p10_blank_cells_zero_bytes_holds()),
            register_assertion(
                "GRID-DENSE-VALUES-REGION",
                "dense values remain region-backed without sparse point expansion",
                stats.dense_value_regions == 1
                    && stats.dense_value_cells == expected_cells
                    && byte_report.dense_numeric_packed_cells == expected_cells
                    && stats.sparse_point_cells == 0
                    && recalc.computed_dense_value_regions == 1
                    && valuation.sparse_computed_cells() == 0
            )
        ]
    }))
}

fn repeated_r1c1_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_literal_region_with(dense_rect, |address| {
        CalcValue::number(f64::from(address.row))
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(100_000)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let last_value = valuation.read_cell(&address(options.rows, 2)).computed;

    Ok(json!({
        "counters": {
            "repeated_formula_regions": stats.repeated_formula_regions,
            "repeated_formula_cells": stats.repeated_formula_cells,
            "distinct_repeated_formula_templates": stats.distinct_repeated_formula_templates,
            "dense_numeric_packed_cells": byte_report.dense_numeric_packed_cells,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "dense_value_region_bytes": byte_report.dense_value_region_bytes,
            "repeated_formula_region_bytes": byte_report.repeated_formula_region_bytes,
            "dense_bytes_per_cell_micros": byte_report.dense_bytes_per_cell_micros(),
            "repeated_formula_bytes_per_cell_micros": byte_report.repeated_formula_bytes_per_cell_micros(),
            "authored_bytes_per_cell_micros": byte_report.authored_bytes_per_cell_micros(),
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "formula_templates_prepared": recalc.formula_templates_prepared,
            "distinct_formula_templates": recalc.distinct_formula_templates,
            "formula_cells": recalc.formula_cells,
            "formula_plan_cache_lookups": recalc.formula_plan_cache_lookups(),
            "formula_plan_cache_hits": recalc.formula_plan_cache_hits,
            "formula_plan_cache_misses": recalc.formula_plan_cache_misses,
            "formula_plan_cache_hit_rate_micros": recalc.formula_plan_cache_hit_rate_micros(),
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "last_formula_value": calc_value_display_text(&last_value),
            "warm_cells_visited": warm.cells_visited,
            "warm_formula_evaluations": warm.formula_evaluations
        },
        "register_assertions": [
            register_assertion("P-00", "repeated R1C1 recalc visits each occupied cell once", recalc.p00_primary_exact_once_holds()),
            register_assertion("P-10", "repeated R1C1 formulas share authored bytes and dense inputs stay packed", byte_report.p10_dense_value_budget_holds() && byte_report.p10_repeated_formula_budget_holds() && byte_report.p10_blank_cells_zero_bytes_holds()),
            register_assertion("P-11", "repeated R1C1 prepares one template for the region", recalc.p11_template_prepare_once_holds() && recalc.formula_templates_prepared == 1),
            register_assertion(
                "P-14",
                "repeated R1C1 formula plan cache misses once and hits for the remaining formula cells",
                recalc.p14_plan_cache_hit_floor_holds()
                    && recalc.formula_plan_cache_misses == 1
                    && recalc.formula_plan_cache_hits == recalc.formula_cells.saturating_sub(1)
            ),
            register_assertion("P-19", "unchanged repeated R1C1 sheet hits warm no-op cache", warm.p19_warm_noop_holds())
        ]
    }))
}

fn fill_down_r1c1_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "fill-down-r1c1 requires at least 2 rows".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.set_literal(address(1, 1), CalcValue::number(1.0))?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        2,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=R[-1]C+1", "excel.grid.v1:r1c1-template:R[-1]C+1")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let first_value = valuation.read_cell(&address(1, 1)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let last_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let expected_formula_cells = u64::from(options.rows - 1);

    Ok(json!({
        "counters": {
            "repeated_formula_regions": stats.repeated_formula_regions,
            "repeated_formula_cells": stats.repeated_formula_cells,
            "distinct_repeated_formula_templates": stats.distinct_repeated_formula_templates,
            "sparse_point_cells": stats.sparse_point_cells,
            "authored_storage_bytes": byte_report.authored_storage_bytes,
            "sparse_point_bytes": byte_report.sparse_point_bytes,
            "repeated_formula_region_bytes": byte_report.repeated_formula_region_bytes,
            "repeated_formula_bytes_per_cell_micros": byte_report.repeated_formula_bytes_per_cell_micros(),
            "authored_bytes_per_cell_micros": byte_report.authored_bytes_per_cell_micros(),
            "blank_cells": byte_report.blank_cells,
            "blank_cell_bytes": byte_report.blank_cell_bytes,
            "formula_templates_prepared": recalc.formula_templates_prepared,
            "distinct_formula_templates": recalc.distinct_formula_templates,
            "formula_cells": recalc.formula_cells,
            "formula_evaluations": recalc.formula_evaluations,
            "formula_plan_cache_lookups": recalc.formula_plan_cache_lookups(),
            "formula_plan_cache_hits": recalc.formula_plan_cache_hits,
            "formula_plan_cache_misses": recalc.formula_plan_cache_misses,
            "formula_plan_cache_hit_rate_micros": recalc.formula_plan_cache_hit_rate_micros(),
            "last_formula_value": calc_value_display_text(&last_value),
            "middle_formula_value": calc_value_display_text(&middle_value),
            "first_value": calc_value_display_text(&first_value),
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "warm_cells_visited": warm.cells_visited,
            "warm_formula_evaluations": warm.formula_evaluations
        },
        "register_assertions": [
            register_assertion("P-00", "fill-down R1C1 recalc visits each occupied cell once", recalc.p00_primary_exact_once_holds()),
            register_assertion("P-10", "fill-down R1C1 formulas share authored bytes and blanks cost zero bytes", byte_report.p10_repeated_formula_budget_holds() && byte_report.p10_blank_cells_zero_bytes_holds()),
            register_assertion(
                "P-11",
                "fill-down R1C1 prepares one template for the repeated region",
                recalc.p11_template_prepare_once_holds()
                    && recalc.formula_templates_prepared == 1
                    && stats.repeated_formula_cells == expected_formula_cells
            ),
            register_assertion(
                "P-14",
                "fill-down R1C1 formula plan cache misses once and hits for the remaining formula cells",
                recalc.p14_plan_cache_hit_floor_holds()
                    && recalc.formula_plan_cache_misses == 1
                    && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
            ),
            register_assertion("P-19", "unchanged fill-down R1C1 sheet hits warm no-op cache", warm.p19_warm_noop_holds()),
            register_assertion(
                "GRID-FILL-DOWN-R1C1",
                "fill-down R1C1 produces the expected first/middle/last values",
                calc_value_display_text(&first_value) == "1"
                    && calc_value_display_text(&middle_value) == middle_row.to_string()
                    && calc_value_display_text(&last_value) == options.rows.to_string()
                    && recalc.computed_dense_value_regions == 1
                    && valuation.sparse_computed_cells() == 1
            )
        ]
    }))
}

fn pascal_r1c1_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "pascal-r1c1-1m requires at least 2 rows".to_string(),
        });
    }
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "pascal-r1c1-1m requires at least 2 columns".to_string(),
        });
    }

    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let left_boundary = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(left_boundary, |_address| 1.0)?;
    for col in 2..=options.cols {
        sheet.set_literal(address(1, col), CalcValue::number(1.0))?;
    }

    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        2,
        2,
        options.rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=RC[-1]+R[-1]C",
            "excel.grid.v1:r1c1-template:RC[-1]+R[-1]C",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_formula_cells =
        u64::from(options.rows.saturating_sub(1)).saturating_mul(u64::from(options.cols - 1));
    let expected_occupied_cells = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let expected_sparse_boundary_cells = usize::try_from(options.cols - 1).unwrap_or(usize::MAX);
    let middle_row = (options.rows / 2).max(2);
    let first_formula_value = valuation.read_cell(&address(2, 2)).computed;
    let middle_formula_value = valuation
        .read_cell(&address(middle_row, options.cols))
        .computed;
    let last_formula_value = valuation
        .read_cell(&address(options.rows, options.cols))
        .computed;
    let expected_first_formula = CalcValue::number(pascal_r1c1_expected_value(2, 2));
    let expected_middle_formula =
        CalcValue::number(pascal_r1c1_expected_value(middle_row, options.cols));
    let expected_last_formula =
        CalcValue::number(pascal_r1c1_expected_value(options.rows, options.cols));

    let counters = json_object([
        ("boundary_dense_value_cells", json!(stats.dense_value_cells)),
        (
            "boundary_sparse_point_cells",
            json!(stats.sparse_point_cells),
        ),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "distinct_repeated_formula_templates",
            json!(stats.distinct_repeated_formula_templates),
        ),
        (
            "authored_cells_upper_bound",
            json!(stats.authored_cells_upper_bound),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        ("sparse_point_bytes", json!(byte_report.sparse_point_bytes)),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
        ),
        (
            "sparse_point_bytes_per_cell_micros",
            json!(byte_report.sparse_point_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        ("blank_cells", json!(byte_report.blank_cells)),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "formula_plan_cache_hit_rate_micros",
            json!(recalc.formula_plan_cache_hit_rate_micros()),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        ("sample_middle_row", json!(middle_row)),
        (
            "sample_first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "sample_middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "sample_last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_first_formula_value",
            json!(calc_value_display_text(&expected_first_formula)),
        ),
        (
            "expected_middle_formula_value",
            json!(calc_value_display_text(&expected_middle_formula)),
        ),
        (
            "expected_last_formula_value",
            json!(calc_value_display_text(&expected_last_formula)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_sparse_point_cells",
            json!(partition.sparse_point_cells),
        ),
        (
            "partition_dense_value_regions",
            json!(partition.dense_value_regions),
        ),
        (
            "partition_repeated_formula_regions",
            json!(partition.repeated_formula_regions),
        ),
        (
            "partition_dense_value_pair_checks",
            json!(partition.dense_value_pair_checks),
        ),
        (
            "partition_repeated_formula_pair_checks",
            json!(partition.repeated_formula_pair_checks),
        ),
        (
            "partition_dense_value_overlap_count",
            json!(partition.dense_value_overlap_count),
        ),
        (
            "partition_repeated_formula_overlap_count",
            json!(partition.repeated_formula_overlap_count),
        ),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "pascal-r1c1-1M recalc visits each occupied boundary/formula cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "pascal-r1c1-1M keeps dense boundary, sparse boundary, repeated formulas, and blanks within byte floors",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_sparse_singleton_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "pascal-r1c1-1M prepares one R1C1 template for the 2D repeated formula region",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && stats.repeated_formula_cells == expected_formula_cells
        ),
        register_assertion(
            "P-14",
            "pascal-r1c1-1M formula plan cache misses once and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
        ),
        register_assertion(
            "P-18",
            "pascal-r1c1-1M compact boundary and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "P-19",
            "unchanged pascal-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "GRID-PASCAL-R1C1-1M",
            "pascal-r1c1-1M publishes a two-dimensional R1C1 recurrence as dense output with expected sampled values",
            stats.dense_value_cells == u64::from(options.rows)
                && stats.sparse_point_cells == expected_sparse_boundary_cells
                && stats.repeated_formula_regions == 1
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == expected_sparse_boundary_cells
                && first_formula_value == expected_first_formula
                && middle_formula_value == expected_middle_formula
                && last_formula_value == expected_last_formula
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn boring_1mx10_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "boring-1mx10 requires at least 2 columns".to_string(),
        });
    }
    let formula_cols = if options.cols >= 10 {
        2
    } else {
        (options.cols / 2).max(1)
    };
    let dense_cols = options.cols - formula_cols;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        dense_cols,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        dense_cols + 1,
        options.rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows) * u64::from(dense_cols);
    let expected_formula_cells = u64::from(options.rows) * u64::from(formula_cols);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_dense_value = valuation.read_cell(&address(1, 1)).computed;
    let first_formula_value = valuation.read_cell(&address(1, dense_cols + 1)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation
        .read_cell(&address(middle_row, options.cols))
        .computed;
    let last_formula_value = valuation
        .read_cell(&address(options.rows, options.cols))
        .computed;
    let expected_last_formula = (f64::from(options.rows) * 1000.0 + f64::from(dense_cols))
        * 2_f64.powi(formula_cols as i32);
    let expected_last_formula_display = integer_display(expected_last_formula);

    let counters = json_object([
        ("dense_columns", json!(dense_cols)),
        ("formula_columns", json!(formula_cols)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "distinct_repeated_formula_templates",
            json!(stats.distinct_repeated_formula_templates),
        ),
        (
            "dense_numeric_packed_cells",
            json!(byte_report.dense_numeric_packed_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        ("blank_cells", json!(byte_report.blank_cells)),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "formula_plan_cache_hit_rate_micros",
            json!(recalc.formula_plan_cache_hit_rate_micros()),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_dense_value",
            json!(calc_value_display_text(&first_dense_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_sparse_point_cells",
            json!(partition.sparse_point_cells),
        ),
        (
            "partition_dense_value_regions",
            json!(partition.dense_value_regions),
        ),
        (
            "partition_repeated_formula_regions",
            json!(partition.repeated_formula_regions),
        ),
        (
            "partition_dense_value_pair_checks",
            json!(partition.dense_value_pair_checks),
        ),
        (
            "partition_repeated_formula_pair_checks",
            json!(partition.repeated_formula_pair_checks),
        ),
        (
            "partition_dense_value_overlap_count",
            json!(partition.dense_value_overlap_count),
        ),
        (
            "partition_repeated_formula_overlap_count",
            json!(partition.repeated_formula_overlap_count),
        ),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "boring-1Mx10 recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "boring-1Mx10 dense values and repeated formulas stay within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "boring-1Mx10 prepares one R1C1 template for the repeated formula block",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && stats.repeated_formula_cells == expected_formula_cells
        ),
        register_assertion(
            "P-14",
            "boring-1Mx10 formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
        ),
        register_assertion(
            "P-19",
            "unchanged boring-1Mx10 sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "boring-1Mx10 compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
                && partition.max_parallelism_bound == 2
        ),
        register_assertion(
            "GRID-BORING-1MX10",
            "boring-1Mx10 keeps authored values/formulas compact and produces expected sampled values",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_dense_value) == "1001"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn direct_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "direct-r1c1-1m requires at least 3 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 10.0)?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        GridFormulaCell::new("=RC[-1]", "excel.grid.v1:r1c1-template:RC[-1]")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=(RC[-2])", "excel.grid.v1:r1c1-template:(RC[-2])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(2);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_value = |row: u32| CalcValue::number(f64::from(row) * 10.0);
    let middle_row = (options.rows / 2).max(1);
    let first_direct_value = valuation.read_cell(&address(1, 2)).computed;
    let first_parenthesized_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_direct_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_parenthesized_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let last_direct_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_parenthesized_value = valuation.read_cell(&address(options.rows, 3)).computed;

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(2)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_direct_value",
            json!(calc_value_display_text(&first_direct_value)),
        ),
        (
            "first_parenthesized_value",
            json!(calc_value_display_text(&first_parenthesized_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_direct_value",
            json!(calc_value_display_text(&middle_direct_value)),
        ),
        (
            "middle_parenthesized_value",
            json!(calc_value_display_text(&middle_parenthesized_value)),
        ),
        (
            "last_direct_value",
            json!(calc_value_display_text(&last_direct_value)),
        ),
        (
            "last_parenthesized_value",
            json!(calc_value_display_text(&last_parenthesized_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "direct-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "direct-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "direct-r1c1-1M prepares two direct scalar R1C1 templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 2
                && recalc.compiled_formula_plans_cached == 2
        ),
        register_assertion(
            "P-14",
            "direct-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 2
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(2)
                && recalc.compiled_formula_plan_cache_misses == 2
        ),
        register_assertion(
            "P-19",
            "unchanged direct-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "direct-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 2
        ),
        register_assertion(
            "GRID-DIRECT-R1C1-1M",
            "direct-r1c1-1M publishes dense output for direct scalar and parenthesized direct scalar R1C1 formulas",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 2
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 3
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_direct_value == expected_value(1)
                && first_parenthesized_value == expected_value(1)
                && middle_direct_value == expected_value(middle_row)
                && middle_parenthesized_value == expected_value(middle_row)
                && last_direct_value == expected_value(options.rows)
                && last_parenthesized_value == expected_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn unary_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "unary-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 10.0)?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        GridFormulaCell::new("=-RC[-1]", "excel.grid.v1:r1c1-template:-RC[-1]")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=-(RC[-2]+5)", "excel.grid.v1:r1c1-template:-(RC[-2]+5)")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new("=-RC[-3]*2+1", "excel.grid.v1:r1c1-template:-RC[-3]*2+1")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let direct_value = |row: u32| CalcValue::number(-(f64::from(row) * 10.0));
    let parenthesized_value = |row: u32| CalcValue::number(-(f64::from(row) * 10.0 + 5.0));
    let arithmetic_value = |row: u32| CalcValue::number(-(f64::from(row) * 10.0) * 2.0 + 1.0);
    let middle_row = (options.rows / 2).max(1);
    let first_direct_value = valuation.read_cell(&address(1, 2)).computed;
    let first_parenthesized_value = valuation.read_cell(&address(1, 3)).computed;
    let first_arithmetic_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_direct_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_parenthesized_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_arithmetic_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_direct_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_parenthesized_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_arithmetic_value = valuation.read_cell(&address(options.rows, 4)).computed;

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_direct_value",
            json!(calc_value_display_text(&first_direct_value)),
        ),
        (
            "first_parenthesized_value",
            json!(calc_value_display_text(&first_parenthesized_value)),
        ),
        (
            "first_arithmetic_value",
            json!(calc_value_display_text(&first_arithmetic_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_direct_value",
            json!(calc_value_display_text(&middle_direct_value)),
        ),
        (
            "middle_parenthesized_value",
            json!(calc_value_display_text(&middle_parenthesized_value)),
        ),
        (
            "middle_arithmetic_value",
            json!(calc_value_display_text(&middle_arithmetic_value)),
        ),
        (
            "last_direct_value",
            json!(calc_value_display_text(&last_direct_value)),
        ),
        (
            "last_parenthesized_value",
            json!(calc_value_display_text(&last_parenthesized_value)),
        ),
        (
            "last_arithmetic_value",
            json!(calc_value_display_text(&last_arithmetic_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "unary-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "unary-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "unary-r1c1-1M prepares three unary scalar R1C1 templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "unary-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged unary-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "unary-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-UNARY-R1C1-1M",
            "unary-r1c1-1M publishes dense output for direct unary, parenthesized unary, and unary arithmetic R1C1 formulas",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_direct_value == direct_value(1)
                && first_parenthesized_value == parenthesized_value(1)
                && first_arithmetic_value == arithmetic_value(1)
                && middle_direct_value == direct_value(middle_row)
                && middle_parenthesized_value == parenthesized_value(middle_row)
                && middle_arithmetic_value == arithmetic_value(middle_row)
                && last_direct_value == direct_value(options.rows)
                && last_parenthesized_value == parenthesized_value(options.rows)
                && last_arithmetic_value == arithmetic_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn argument_aggregate_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 8 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "argument-aggregate-r1c1-1m requires at least 8 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row) * 10.0,
        2 => f64::from(address.row),
        _ => unreachable!("dense input region has two columns"),
    })?;
    for (col, source, normal_form) in [
        (
            3,
            "=SUM(RC[-2],RC[-1],5)",
            "excel.grid.v1:r1c1-template:SUM(RC[-2],RC[-1],5)",
        ),
        (
            4,
            "=COUNT(RC[-3],RC[-2],5)",
            "excel.grid.v1:r1c1-template:COUNT(RC[-3],RC[-2],5)",
        ),
        (
            5,
            "=PRODUCT(RC[-4],RC[-3],2)",
            "excel.grid.v1:r1c1-template:PRODUCT(RC[-4],RC[-3],2)",
        ),
        (
            6,
            "=AVERAGE(RC[-5],RC[-4],5)",
            "excel.grid.v1:r1c1-template:AVERAGE(RC[-5],RC[-4],5)",
        ),
        (
            7,
            "=MIN(RC[-6],RC[-5],5)",
            "excel.grid.v1:r1c1-template:MIN(RC[-6],RC[-5],5)",
        ),
        (
            8,
            "=MAX(RC[-7],RC[-6],5)",
            "excel.grid.v1:r1c1-template:MAX(RC[-7],RC[-6],5)",
        ),
    ] {
        sheet.put_repeated_formula_region(
            GridRect::new(
                "book:grid-scale",
                "sheet:grid-scale",
                1,
                col,
                options.rows,
                col,
                bounds,
            )?,
            GridFormulaCell::new(source, normal_form)
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
        )?;
    }

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(6);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let sum_value = |row: u32| CalcValue::number(f64::from(row) * 11.0 + 5.0);
    let count_value = |_row: u32| CalcValue::number(3.0);
    let product_value = |row: u32| {
        let row = f64::from(row);
        CalcValue::number(row * 10.0 * row * 2.0)
    };
    let average_value = |row: u32| CalcValue::number((f64::from(row) * 11.0 + 5.0) / 3.0);
    let min_value = |row: u32| CalcValue::number(f64::from(row).min(5.0));
    let max_value = |row: u32| CalcValue::number(f64::from(row) * 10.0);
    let middle_row = (options.rows / 2).max(1);
    let first_sum_value = valuation.read_cell(&address(1, 3)).computed;
    let first_count_value = valuation.read_cell(&address(1, 4)).computed;
    let first_product_value = valuation.read_cell(&address(1, 5)).computed;
    let first_average_value = valuation.read_cell(&address(1, 6)).computed;
    let first_min_value = valuation.read_cell(&address(1, 7)).computed;
    let first_max_value = valuation.read_cell(&address(1, 8)).computed;
    let middle_sum_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_product_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let middle_average_value = valuation.read_cell(&address(middle_row, 6)).computed;
    let last_sum_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_product_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let last_average_value = valuation.read_cell(&address(options.rows, 6)).computed;
    let last_min_value = valuation.read_cell(&address(options.rows, 7)).computed;
    let last_max_value = valuation.read_cell(&address(options.rows, 8)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(6)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_sum_value",
            json!(calc_value_display_text(&first_sum_value)),
        ),
        (
            "first_count_value",
            json!(calc_value_display_text(&first_count_value)),
        ),
        (
            "first_product_value",
            json!(calc_value_display_text(&first_product_value)),
        ),
        (
            "first_average_value",
            json!(calc_value_display_text(&first_average_value)),
        ),
        (
            "first_min_value",
            json!(calc_value_display_text(&first_min_value)),
        ),
        (
            "first_max_value",
            json!(calc_value_display_text(&first_max_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_sum_value",
            json!(calc_value_display_text(&middle_sum_value)),
        ),
        (
            "middle_product_value",
            json!(calc_value_display_text(&middle_product_value)),
        ),
        (
            "middle_average_value",
            json!(calc_value_display_text(&middle_average_value)),
        ),
        (
            "last_sum_value",
            json!(calc_value_display_text(&last_sum_value)),
        ),
        (
            "last_product_value",
            json!(calc_value_display_text(&last_product_value)),
        ),
        (
            "last_average_value",
            json!(calc_value_display_text(&last_average_value)),
        ),
        (
            "last_min_value",
            json!(calc_value_display_text(&last_min_value)),
        ),
        (
            "last_max_value",
            json!(calc_value_display_text(&last_max_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "argument-aggregate-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "argument-aggregate-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "argument-aggregate-r1c1-1M prepares six aggregate argument templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 6
                && recalc.compiled_formula_plans_cached == 6
        ),
        register_assertion(
            "P-14",
            "argument-aggregate-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 6
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(6)
                && recalc.compiled_formula_plan_cache_misses == 6
        ),
        register_assertion(
            "P-19",
            "unchanged argument-aggregate-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "argument-aggregate-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 6
        ),
        register_assertion(
            "GRID-ARGUMENT-AGGREGATE-R1C1-1M",
            "argument-aggregate-r1c1-1M publishes dense output for SUM/COUNT/PRODUCT/AVERAGE/MIN/MAX argument lists",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 6
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 7
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_sum_value == sum_value(1)
                && first_count_value == count_value(1)
                && first_product_value == product_value(1)
                && first_average_value == average_value(1)
                && first_min_value == min_value(1)
                && first_max_value == max_value(1)
                && middle_sum_value == sum_value(middle_row)
                && middle_product_value == product_value(middle_row)
                && middle_average_value == average_value(middle_row)
                && last_sum_value == sum_value(options.rows)
                && last_product_value == product_value(options.rows)
                && last_average_value == average_value(options.rows)
                && last_min_value == min_value(options.rows)
                && last_max_value == max_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn math_function_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "math-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 if address.row % 2 == 0 => f64::from(address.row),
        1 => -f64::from(address.row),
        2 => {
            let row = f64::from(address.row);
            row * row
        }
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=ABS(RC[-2])", "excel.grid.v1:r1c1-template:ABS(RC[-2])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new("=SQRT(RC[-2])", "excel.grid.v1:r1c1-template:SQRT(RC[-2])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=POWER(ABS(RC[-4]),2)",
            "excel.grid.v1:r1c1-template:POWER(ABS(RC[-4]),2)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_abs = |row: u32| CalcValue::number(f64::from(row));
    let expected_sqrt = |row: u32| CalcValue::number(f64::from(row));
    let expected_power = |row: u32| {
        let row = f64::from(row);
        CalcValue::number(row * row)
    };
    let middle_row = (options.rows / 2).max(1);
    let first_abs_value = valuation.read_cell(&address(1, 3)).computed;
    let first_sqrt_value = valuation.read_cell(&address(1, 4)).computed;
    let first_power_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_abs_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_sqrt_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_power_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_abs_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_sqrt_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_power_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_abs_value",
            json!(calc_value_display_text(&first_abs_value)),
        ),
        (
            "first_sqrt_value",
            json!(calc_value_display_text(&first_sqrt_value)),
        ),
        (
            "first_power_value",
            json!(calc_value_display_text(&first_power_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_abs_value",
            json!(calc_value_display_text(&middle_abs_value)),
        ),
        (
            "middle_sqrt_value",
            json!(calc_value_display_text(&middle_sqrt_value)),
        ),
        (
            "middle_power_value",
            json!(calc_value_display_text(&middle_power_value)),
        ),
        (
            "last_abs_value",
            json!(calc_value_display_text(&last_abs_value)),
        ),
        (
            "last_sqrt_value",
            json!(calc_value_display_text(&last_sqrt_value)),
        ),
        (
            "last_power_value",
            json!(calc_value_display_text(&last_power_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "math-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "math-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "math-function-r1c1-1M prepares three scalar math function templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "math-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged math-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "math-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-MATH-FUNCTION-R1C1-1M",
            "math-function-r1c1-1M publishes dense output for ABS/SQRT/POWER scalar functions",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_abs_value == expected_abs(1)
                && first_sqrt_value == expected_sqrt(1)
                && first_power_value == expected_power(1)
                && middle_abs_value == expected_abs(middle_row)
                && middle_sqrt_value == expected_sqrt(middle_row)
                && middle_power_value == expected_power(middle_row)
                && last_abs_value == expected_abs(options.rows)
                && last_sqrt_value == expected_sqrt(options.rows)
                && last_power_value == expected_power(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn mod_function_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "mod-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row),
        2 => 7.0,
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=MOD(RC[-2],RC[-1])",
            "excel.grid.v1:r1c1-template:MOD(RC[-2],RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)",
            "excel.grid.v1:r1c1-template:IF(MOD(RC[-3],2)=0,RC[-3]/2,RC[-3]*3)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=MOD(POWER(RC[-4],2),RC[-3])",
            "excel.grid.v1:r1c1-template:MOD(POWER(RC[-4],2),RC[-3])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_mod = |row: u32| CalcValue::number(f64::from(row % 7));
    let expected_if = |row: u32| {
        if row % 2 == 0 {
            CalcValue::number(f64::from(row) / 2.0)
        } else {
            CalcValue::number(f64::from(row) * 3.0)
        }
    };
    let expected_power_mod = |row: u32| {
        let remainder = row % 7;
        CalcValue::number(f64::from((remainder * remainder) % 7))
    };
    let middle_row = (options.rows / 2).max(1);
    let first_mod_value = valuation.read_cell(&address(1, 3)).computed;
    let first_if_value = valuation.read_cell(&address(1, 4)).computed;
    let first_power_mod_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_mod_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_if_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_power_mod_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_mod_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_if_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_power_mod_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_mod_value",
            json!(calc_value_display_text(&first_mod_value)),
        ),
        (
            "first_if_value",
            json!(calc_value_display_text(&first_if_value)),
        ),
        (
            "first_power_mod_value",
            json!(calc_value_display_text(&first_power_mod_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_mod_value",
            json!(calc_value_display_text(&middle_mod_value)),
        ),
        (
            "middle_if_value",
            json!(calc_value_display_text(&middle_if_value)),
        ),
        (
            "middle_power_mod_value",
            json!(calc_value_display_text(&middle_power_mod_value)),
        ),
        (
            "last_mod_value",
            json!(calc_value_display_text(&last_mod_value)),
        ),
        (
            "last_if_value",
            json!(calc_value_display_text(&last_if_value)),
        ),
        (
            "last_power_mod_value",
            json!(calc_value_display_text(&last_power_mod_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "mod-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "mod-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "mod-function-r1c1-1M prepares three MOD/scalar function templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "mod-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged mod-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "mod-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-MOD-FUNCTION-R1C1-1M",
            "mod-function-r1c1-1M publishes dense numeric output for MOD and MOD-driven IF templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_mod_value == expected_mod(1)
                && first_if_value == expected_if(1)
                && first_power_mod_value == expected_power_mod(1)
                && middle_mod_value == expected_mod(middle_row)
                && middle_if_value == expected_if(middle_row)
                && middle_power_mod_value == expected_power_mod(middle_row)
                && last_mod_value == expected_mod(options.rows)
                && last_if_value == expected_if(options.rows)
                && last_power_mod_value == expected_power_mod(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn rounding_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "rounding-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row) + 0.5,
        2 => 0.0,
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=ROUND(RC[-2],RC[-1])",
            "excel.grid.v1:r1c1-template:ROUND(RC[-2],RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=ROUNDUP(RC[-3],RC[-2])",
            "excel.grid.v1:r1c1-template:ROUNDUP(RC[-3],RC[-2])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=ROUNDDOWN(RC[-4],RC[-3])",
            "excel.grid.v1:r1c1-template:ROUNDDOWN(RC[-4],RC[-3])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_round = |row: u32| CalcValue::number(f64::from(row) + 1.0);
    let expected_roundup = expected_round;
    let expected_rounddown = |row: u32| CalcValue::number(f64::from(row));
    let middle_row = (options.rows / 2).max(1);
    let first_round_value = valuation.read_cell(&address(1, 3)).computed;
    let first_roundup_value = valuation.read_cell(&address(1, 4)).computed;
    let first_rounddown_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_round_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_roundup_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_rounddown_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_round_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_roundup_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_rounddown_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_round_value",
            json!(calc_value_display_text(&first_round_value)),
        ),
        (
            "first_roundup_value",
            json!(calc_value_display_text(&first_roundup_value)),
        ),
        (
            "first_rounddown_value",
            json!(calc_value_display_text(&first_rounddown_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_round_value",
            json!(calc_value_display_text(&middle_round_value)),
        ),
        (
            "middle_roundup_value",
            json!(calc_value_display_text(&middle_roundup_value)),
        ),
        (
            "middle_rounddown_value",
            json!(calc_value_display_text(&middle_rounddown_value)),
        ),
        (
            "last_round_value",
            json!(calc_value_display_text(&last_round_value)),
        ),
        (
            "last_roundup_value",
            json!(calc_value_display_text(&last_roundup_value)),
        ),
        (
            "last_rounddown_value",
            json!(calc_value_display_text(&last_rounddown_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "rounding-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "rounding-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "rounding-function-r1c1-1M prepares three ROUND-family templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "rounding-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged rounding-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "rounding-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-ROUNDING-FUNCTION-R1C1-1M",
            "rounding-function-r1c1-1M publishes dense numeric output for ROUND/ROUNDUP/ROUNDDOWN templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_round_value == expected_round(1)
                && first_roundup_value == expected_roundup(1)
                && first_rounddown_value == expected_rounddown(1)
                && middle_round_value == expected_round(middle_row)
                && middle_roundup_value == expected_roundup(middle_row)
                && middle_rounddown_value == expected_rounddown(middle_row)
                && last_round_value == expected_round(options.rows)
                && last_roundup_value == expected_roundup(options.rows)
                && last_rounddown_value == expected_rounddown(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn integer_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "integer-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row) + 0.9,
        2 => -1.0,
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=INT(RC[-2])", "excel.grid.v1:r1c1-template:INT(RC[-2])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=TRUNC(RC[-3])",
            "excel.grid.v1:r1c1-template:TRUNC(RC[-3])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=TRUNC(RC[-4],RC[-3])",
            "excel.grid.v1:r1c1-template:TRUNC(RC[-4],RC[-3])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_int = |row: u32| CalcValue::number(f64::from(row));
    let expected_trunc = expected_int;
    let expected_trunc_tens = |row: u32| CalcValue::number(f64::from((row / 10) * 10));
    let middle_row = (options.rows / 2).max(1);
    let first_int_value = valuation.read_cell(&address(1, 3)).computed;
    let first_trunc_value = valuation.read_cell(&address(1, 4)).computed;
    let first_trunc_tens_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_int_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_trunc_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_trunc_tens_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_int_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_trunc_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_trunc_tens_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_int_value",
            json!(calc_value_display_text(&first_int_value)),
        ),
        (
            "first_trunc_value",
            json!(calc_value_display_text(&first_trunc_value)),
        ),
        (
            "first_trunc_tens_value",
            json!(calc_value_display_text(&first_trunc_tens_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_int_value",
            json!(calc_value_display_text(&middle_int_value)),
        ),
        (
            "middle_trunc_value",
            json!(calc_value_display_text(&middle_trunc_value)),
        ),
        (
            "middle_trunc_tens_value",
            json!(calc_value_display_text(&middle_trunc_tens_value)),
        ),
        (
            "last_int_value",
            json!(calc_value_display_text(&last_int_value)),
        ),
        (
            "last_trunc_value",
            json!(calc_value_display_text(&last_trunc_value)),
        ),
        (
            "last_trunc_tens_value",
            json!(calc_value_display_text(&last_trunc_tens_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "integer-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "integer-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "integer-function-r1c1-1M prepares three INT/TRUNC templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "integer-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged integer-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "integer-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-INTEGER-FUNCTION-R1C1-1M",
            "integer-function-r1c1-1M publishes dense numeric output for INT and one/two-arg TRUNC templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_int_value == expected_int(1)
                && first_trunc_value == expected_trunc(1)
                && first_trunc_tens_value == expected_trunc_tens(1)
                && middle_int_value == expected_int(middle_row)
                && middle_trunc_value == expected_trunc(middle_row)
                && middle_trunc_tens_value == expected_trunc_tens(middle_row)
                && last_int_value == expected_int(options.rows)
                && last_trunc_value == expected_trunc(options.rows)
                && last_trunc_tens_value == expected_trunc_tens(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn log_function_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "log-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => 1.0,
        2 => 0.0,
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=EXP(RC[-1])", "excel.grid.v1:r1c1-template:EXP(RC[-1])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new("=LN(RC[-3])", "excel.grid.v1:r1c1-template:LN(RC[-3])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=LOG10(RC[-4]*100)",
            "excel.grid.v1:r1c1-template:LOG10(RC[-4]*100)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            6,
            options.rows,
            6,
            bounds,
        )?,
        GridFormulaCell::new(
            "=LOG(RC[-5]*100,10)",
            "excel.grid.v1:r1c1-template:LOG(RC[-5]*100,10)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(4);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_exp = CalcValue::number(1.0);
    let expected_ln = CalcValue::number(0.0);
    let expected_log10 = CalcValue::number(2.0);
    let expected_log = CalcValue::number(2.0);
    let middle_row = (options.rows / 2).max(1);
    let first_exp_value = valuation.read_cell(&address(1, 3)).computed;
    let first_ln_value = valuation.read_cell(&address(1, 4)).computed;
    let first_log10_value = valuation.read_cell(&address(1, 5)).computed;
    let first_log_value = valuation.read_cell(&address(1, 6)).computed;
    let middle_exp_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_ln_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_log10_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let middle_log_value = valuation.read_cell(&address(middle_row, 6)).computed;
    let last_exp_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_ln_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_log10_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let last_log_value = valuation.read_cell(&address(options.rows, 6)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(4)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_exp_value",
            json!(calc_value_display_text(&first_exp_value)),
        ),
        (
            "first_ln_value",
            json!(calc_value_display_text(&first_ln_value)),
        ),
        (
            "first_log10_value",
            json!(calc_value_display_text(&first_log10_value)),
        ),
        (
            "first_log_value",
            json!(calc_value_display_text(&first_log_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_exp_value",
            json!(calc_value_display_text(&middle_exp_value)),
        ),
        (
            "middle_ln_value",
            json!(calc_value_display_text(&middle_ln_value)),
        ),
        (
            "middle_log10_value",
            json!(calc_value_display_text(&middle_log10_value)),
        ),
        (
            "middle_log_value",
            json!(calc_value_display_text(&middle_log_value)),
        ),
        (
            "last_exp_value",
            json!(calc_value_display_text(&last_exp_value)),
        ),
        (
            "last_ln_value",
            json!(calc_value_display_text(&last_ln_value)),
        ),
        (
            "last_log10_value",
            json!(calc_value_display_text(&last_log10_value)),
        ),
        (
            "last_log_value",
            json!(calc_value_display_text(&last_log_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "log-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "log-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "log-function-r1c1-1M prepares four EXP/LN/LOG templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "log-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged log-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "log-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-LOG-FUNCTION-R1C1-1M",
            "log-function-r1c1-1M publishes dense numeric output for EXP/LN/LOG10/LOG templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 5
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_exp_value == expected_exp
                && first_ln_value == expected_ln
                && first_log10_value == expected_log10
                && first_log_value == expected_log
                && middle_exp_value == expected_exp
                && middle_ln_value == expected_ln
                && middle_log10_value == expected_log10
                && middle_log_value == expected_log
                && last_exp_value == expected_exp
                && last_ln_value == expected_ln
                && last_log10_value == expected_log10
                && last_log_value == expected_log
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn trig_function_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "trig-function-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |_| 0.0)?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        GridFormulaCell::new("=SIN(RC[-1])", "excel.grid.v1:r1c1-template:SIN(RC[-1])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new("=COS(RC[-2])", "excel.grid.v1:r1c1-template:COS(RC[-2])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new("=TAN(RC[-3])", "excel.grid.v1:r1c1-template:TAN(RC[-3])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_sin = CalcValue::number(0.0);
    let expected_cos = CalcValue::number(1.0);
    let expected_tan = CalcValue::number(0.0);
    let middle_row = (options.rows / 2).max(1);
    let first_sin_value = valuation.read_cell(&address(1, 2)).computed;
    let first_cos_value = valuation.read_cell(&address(1, 3)).computed;
    let first_tan_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_sin_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_cos_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_tan_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_sin_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_cos_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_tan_value = valuation.read_cell(&address(options.rows, 4)).computed;

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_sin_value",
            json!(calc_value_display_text(&first_sin_value)),
        ),
        (
            "first_cos_value",
            json!(calc_value_display_text(&first_cos_value)),
        ),
        (
            "first_tan_value",
            json!(calc_value_display_text(&first_tan_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_sin_value",
            json!(calc_value_display_text(&middle_sin_value)),
        ),
        (
            "middle_cos_value",
            json!(calc_value_display_text(&middle_cos_value)),
        ),
        (
            "middle_tan_value",
            json!(calc_value_display_text(&middle_tan_value)),
        ),
        (
            "last_sin_value",
            json!(calc_value_display_text(&last_sin_value)),
        ),
        (
            "last_cos_value",
            json!(calc_value_display_text(&last_cos_value)),
        ),
        (
            "last_tan_value",
            json!(calc_value_display_text(&last_tan_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "trig-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "trig-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "trig-function-r1c1-1M prepares three SIN/COS/TAN templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "trig-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged trig-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "trig-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-TRIG-FUNCTION-R1C1-1M",
            "trig-function-r1c1-1M publishes dense numeric output for SIN/COS/TAN templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_sin_value == expected_sin
                && first_cos_value == expected_cos
                && first_tan_value == expected_tan
                && middle_sin_value == expected_sin
                && middle_cos_value == expected_cos
                && middle_tan_value == expected_tan
                && last_sin_value == expected_sin
                && last_cos_value == expected_cos
                && last_tan_value == expected_tan
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn angle_function_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "angle-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |_| 0.0)?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=RADIANS(RC[-2])",
            "excel.grid.v1:r1c1-template:RADIANS(RC[-2])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=DEGREES(RC[-2])",
            "excel.grid.v1:r1c1-template:DEGREES(RC[-2])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=SIN(RADIANS(RC[-4]))",
            "excel.grid.v1:r1c1-template:SIN(RADIANS(RC[-4]))",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            6,
            options.rows,
            6,
            bounds,
        )?,
        GridFormulaCell::new("=PI()", "excel.grid.v1:r1c1-template:PI()")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(4);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_radians = CalcValue::number(0.0);
    let expected_degrees = CalcValue::number(0.0);
    let expected_sin_degrees = CalcValue::number(0.0);
    let expected_pi = CalcValue::number(std::f64::consts::PI);
    let middle_row = (options.rows / 2).max(1);
    let first_radians_value = valuation.read_cell(&address(1, 3)).computed;
    let first_degrees_value = valuation.read_cell(&address(1, 4)).computed;
    let first_sin_degrees_value = valuation.read_cell(&address(1, 5)).computed;
    let first_pi_value = valuation.read_cell(&address(1, 6)).computed;
    let middle_radians_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_degrees_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_sin_degrees_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let middle_pi_value = valuation.read_cell(&address(middle_row, 6)).computed;
    let last_radians_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_degrees_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_sin_degrees_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let last_pi_value = valuation.read_cell(&address(options.rows, 6)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(4)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_radians_value",
            json!(calc_value_display_text(&first_radians_value)),
        ),
        (
            "first_degrees_value",
            json!(calc_value_display_text(&first_degrees_value)),
        ),
        (
            "first_sin_degrees_value",
            json!(calc_value_display_text(&first_sin_degrees_value)),
        ),
        (
            "first_pi_value",
            json!(calc_value_display_text(&first_pi_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_radians_value",
            json!(calc_value_display_text(&middle_radians_value)),
        ),
        (
            "middle_degrees_value",
            json!(calc_value_display_text(&middle_degrees_value)),
        ),
        (
            "middle_sin_degrees_value",
            json!(calc_value_display_text(&middle_sin_degrees_value)),
        ),
        (
            "middle_pi_value",
            json!(calc_value_display_text(&middle_pi_value)),
        ),
        (
            "last_radians_value",
            json!(calc_value_display_text(&last_radians_value)),
        ),
        (
            "last_degrees_value",
            json!(calc_value_display_text(&last_degrees_value)),
        ),
        (
            "last_sin_degrees_value",
            json!(calc_value_display_text(&last_sin_degrees_value)),
        ),
        (
            "last_pi_value",
            json!(calc_value_display_text(&last_pi_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "angle-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "angle-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "angle-function-r1c1-1M prepares four RADIANS/DEGREES/PI templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "angle-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged angle-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "angle-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-ANGLE-FUNCTION-R1C1-1M",
            "angle-function-r1c1-1M publishes dense numeric output for RADIANS/DEGREES/PI templates",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 5
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_radians_value == expected_radians
                && first_degrees_value == expected_degrees
                && first_sin_degrees_value == expected_sin_degrees
                && first_pi_value == expected_pi
                && middle_radians_value == expected_radians
                && middle_degrees_value == expected_degrees
                && middle_sin_degrees_value == expected_sin_degrees
                && middle_pi_value == expected_pi
                && last_radians_value == expected_radians
                && last_degrees_value == expected_degrees
                && last_sin_degrees_value == expected_sin_degrees
                && last_pi_value == expected_pi
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn reference_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "reference-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    for (col, source, normal_form) in [
        (1, "=ROW()", "excel.grid.v1:r1c1-template:ROW()"),
        (2, "=COLUMN()", "excel.grid.v1:r1c1-template:COLUMN()"),
        (3, "=ROW(RC[-2])", "excel.grid.v1:r1c1-template:ROW(RC[-2])"),
        (
            4,
            "=COLUMN(RC[-3])",
            "excel.grid.v1:r1c1-template:COLUMN(RC[-3])",
        ),
        (
            5,
            "=ROWS(R1C1:R3C1)",
            "excel.grid.v1:r1c1-template:ROWS(R1C1:R3C1)",
        ),
        (
            6,
            "=COLUMNS(RC[-5]:RC[-3])",
            "excel.grid.v1:r1c1-template:COLUMNS(RC[-5]:RC[-3])",
        ),
    ] {
        sheet.put_repeated_formula_region(
            GridRect::new(
                "book:grid-scale",
                "sheet:grid-scale",
                1,
                col,
                options.rows,
                col,
                bounds,
            )?,
            GridFormulaCell::new(source, normal_form)
                .with_source_channel(FormulaChannelKind::WorksheetR1C1),
        )?;
    }

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = 0_u64;
    let expected_formula_cells = u64::from(options.rows).saturating_mul(6);
    let expected_occupied_cells = expected_formula_cells;
    let middle_row = (options.rows / 2).max(1);
    let first_row_value = valuation.read_cell(&address(1, 1)).computed;
    let first_column_value = valuation.read_cell(&address(1, 2)).computed;
    let first_ref_row_value = valuation.read_cell(&address(1, 3)).computed;
    let first_ref_column_value = valuation.read_cell(&address(1, 4)).computed;
    let first_rows_value = valuation.read_cell(&address(1, 5)).computed;
    let first_columns_value = valuation.read_cell(&address(1, 6)).computed;
    let middle_row_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_ref_row_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let last_row_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_ref_row_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_rows_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let last_columns_value = valuation.read_cell(&address(options.rows, 6)).computed;
    let expected_first_row = CalcValue::number(1.0);
    let expected_middle_row = CalcValue::number(f64::from(middle_row));
    let expected_last_row = CalcValue::number(f64::from(options.rows));
    let expected_current_column = CalcValue::number(2.0);
    let expected_reference_column = CalcValue::number(1.0);
    let expected_rows = CalcValue::number(3.0);
    let expected_columns = CalcValue::number(3.0);

    let counters = json_object([
        ("dense_columns", json!(0)),
        ("formula_columns", json!(6)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_row_value",
            json!(calc_value_display_text(&first_row_value)),
        ),
        (
            "first_column_value",
            json!(calc_value_display_text(&first_column_value)),
        ),
        (
            "first_ref_row_value",
            json!(calc_value_display_text(&first_ref_row_value)),
        ),
        (
            "first_ref_column_value",
            json!(calc_value_display_text(&first_ref_column_value)),
        ),
        (
            "first_rows_value",
            json!(calc_value_display_text(&first_rows_value)),
        ),
        (
            "first_columns_value",
            json!(calc_value_display_text(&first_columns_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_row_value",
            json!(calc_value_display_text(&middle_row_value)),
        ),
        (
            "middle_ref_row_value",
            json!(calc_value_display_text(&middle_ref_row_value)),
        ),
        (
            "last_row_value",
            json!(calc_value_display_text(&last_row_value)),
        ),
        (
            "last_ref_row_value",
            json!(calc_value_display_text(&last_ref_row_value)),
        ),
        (
            "last_rows_value",
            json!(calc_value_display_text(&last_rows_value)),
        ),
        (
            "last_columns_value",
            json!(calc_value_display_text(&last_columns_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "reference-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "reference-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "reference-function-r1c1-1M prepares six ROW/COLUMN reference-function templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 6
                && recalc.compiled_formula_plans_cached == 6
        ),
        register_assertion(
            "P-14",
            "reference-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 6
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(6)
                && recalc.compiled_formula_plan_cache_misses == 6
        ),
        register_assertion(
            "P-19",
            "unchanged reference-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "reference-function-r1c1-1M compact repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 0
                && partition.repeated_formula_regions == 6
        ),
        register_assertion(
            "GRID-REFERENCE-FUNCTION-R1C1-1M",
            "reference-function-r1c1-1M publishes dense numeric output for ROW/COLUMN/ROWS/COLUMNS without dereferencing values",
            stats.dense_value_regions == 0
                && stats.repeated_formula_regions == 6
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 6
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_row_value == expected_first_row
                && first_column_value == expected_current_column
                && first_ref_row_value == expected_first_row
                && first_ref_column_value == expected_reference_column
                && first_rows_value == expected_rows
                && first_columns_value == expected_columns
                && middle_row_value == expected_middle_row
                && middle_ref_row_value == expected_middle_row
                && last_row_value == expected_last_row
                && last_ref_row_value == expected_last_row
                && last_rows_value == expected_rows
                && last_columns_value == expected_columns
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn logical_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "logical-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 if address.row % 2 == 1 => f64::from(address.row),
        1 => -f64::from(address.row),
        2 if address.row % 3 == 0 => f64::from(address.row),
        2 => -f64::from(address.row),
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=AND(RC[-2]>0,RC[-1]>0)",
            "excel.grid.v1:r1c1-template:AND(RC[-2]>0,RC[-1]>0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=OR(RC[-3]>0,RC[-2]>0)",
            "excel.grid.v1:r1c1-template:OR(RC[-3]>0,RC[-2]>0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=NOT(AND(RC[-4]>0,RC[-3]>0))",
            "excel.grid.v1:r1c1-template:NOT(AND(RC[-4]>0,RC[-3]>0))",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_and = |row: u32| CalcValue::logical(row % 2 == 1 && row % 3 == 0);
    let expected_or = |row: u32| CalcValue::logical(row % 2 == 1 || row % 3 == 0);
    let expected_not = |row: u32| CalcValue::logical(!(row % 2 == 1 && row % 3 == 0));
    let middle_row = (options.rows / 2).max(1);
    let true_sample_row = if options.rows >= 3 { 3 } else { 1 };
    let first_and_value = valuation.read_cell(&address(1, 3)).computed;
    let first_or_value = valuation.read_cell(&address(1, 4)).computed;
    let first_not_value = valuation.read_cell(&address(1, 5)).computed;
    let true_sample_and_value = valuation.read_cell(&address(true_sample_row, 3)).computed;
    let true_sample_or_value = valuation.read_cell(&address(true_sample_row, 4)).computed;
    let true_sample_not_value = valuation.read_cell(&address(true_sample_row, 5)).computed;
    let middle_and_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_or_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_not_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_and_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_or_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_not_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_and_value",
            json!(calc_value_display_text(&first_and_value)),
        ),
        (
            "first_or_value",
            json!(calc_value_display_text(&first_or_value)),
        ),
        (
            "first_not_value",
            json!(calc_value_display_text(&first_not_value)),
        ),
        ("true_sample_row", json!(true_sample_row)),
        (
            "true_sample_and_value",
            json!(calc_value_display_text(&true_sample_and_value)),
        ),
        (
            "true_sample_or_value",
            json!(calc_value_display_text(&true_sample_or_value)),
        ),
        (
            "true_sample_not_value",
            json!(calc_value_display_text(&true_sample_not_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_and_value",
            json!(calc_value_display_text(&middle_and_value)),
        ),
        (
            "middle_or_value",
            json!(calc_value_display_text(&middle_or_value)),
        ),
        (
            "middle_not_value",
            json!(calc_value_display_text(&middle_not_value)),
        ),
        (
            "last_and_value",
            json!(calc_value_display_text(&last_and_value)),
        ),
        (
            "last_or_value",
            json!(calc_value_display_text(&last_or_value)),
        ),
        (
            "last_not_value",
            json!(calc_value_display_text(&last_not_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "logical-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "logical-function-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "logical-function-r1c1-1M prepares three logical function templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "logical-function-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged logical-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "logical-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-LOGICAL-FUNCTION-R1C1-1M",
            "logical-function-r1c1-1M publishes dense logical output for AND/OR/NOT over R1C1 comparisons",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_dense_cells
                && valuation.dense_computed_logical_packed_cells() == expected_formula_cells
                && valuation.sparse_computed_cells() == 0
                && first_and_value == expected_and(1)
                && first_or_value == expected_or(1)
                && first_not_value == expected_not(1)
                && true_sample_and_value == expected_and(true_sample_row)
                && true_sample_or_value == expected_or(true_sample_row)
                && true_sample_not_value == expected_not(true_sample_row)
                && middle_and_value == expected_and(middle_row)
                && middle_or_value == expected_or(middle_row)
                && middle_not_value == expected_not(middle_row)
                && last_and_value == expected_and(options.rows)
                && last_or_value == expected_or(options.rows)
                && last_not_value == expected_not(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn if_logical_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "if-logical-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 if address.row % 2 == 1 => f64::from(address.row),
        1 => -f64::from(address.row),
        2 if address.row % 3 == 0 => f64::from(address.row),
        2 => -f64::from(address.row),
        _ => unreachable!("dense input region has two columns"),
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)",
            "excel.grid.v1:r1c1-template:IF(AND(RC[-2]>0,RC[-1]>0),RC[-2]+RC[-1],0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)",
            "excel.grid.v1:r1c1-template:IF(OR(RC[-3]>0,RC[-2]>0),RC[-3]-RC[-2],0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)",
            "excel.grid.v1:r1c1-template:IF(NOT(AND(RC[-4]>0,RC[-3]>0)),ABS(RC[-4]),0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let input_one = |row: u32| {
        if row % 2 == 1 {
            f64::from(row)
        } else {
            -f64::from(row)
        }
    };
    let input_two = |row: u32| {
        if row % 3 == 0 {
            f64::from(row)
        } else {
            -f64::from(row)
        }
    };
    let expected_and_if = |row: u32| {
        if row % 2 == 1 && row % 3 == 0 {
            CalcValue::number(input_one(row) + input_two(row))
        } else {
            CalcValue::number(0.0)
        }
    };
    let expected_or_if = |row: u32| {
        if row % 2 == 1 || row % 3 == 0 {
            CalcValue::number(input_one(row) - input_two(row))
        } else {
            CalcValue::number(0.0)
        }
    };
    let expected_not_if = |row: u32| {
        if !(row % 2 == 1 && row % 3 == 0) {
            CalcValue::number(input_one(row).abs())
        } else {
            CalcValue::number(0.0)
        }
    };
    let middle_row = (options.rows / 2).max(1);
    let true_sample_row = if options.rows >= 3 { 3 } else { 1 };
    let first_and_if_value = valuation.read_cell(&address(1, 3)).computed;
    let first_or_if_value = valuation.read_cell(&address(1, 4)).computed;
    let first_not_if_value = valuation.read_cell(&address(1, 5)).computed;
    let true_sample_and_if_value = valuation.read_cell(&address(true_sample_row, 3)).computed;
    let true_sample_or_if_value = valuation.read_cell(&address(true_sample_row, 4)).computed;
    let true_sample_not_if_value = valuation.read_cell(&address(true_sample_row, 5)).computed;
    let middle_and_if_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_or_if_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_not_if_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_and_if_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_or_if_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_not_if_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_and_if_value",
            json!(calc_value_display_text(&first_and_if_value)),
        ),
        (
            "first_or_if_value",
            json!(calc_value_display_text(&first_or_if_value)),
        ),
        (
            "first_not_if_value",
            json!(calc_value_display_text(&first_not_if_value)),
        ),
        ("true_sample_row", json!(true_sample_row)),
        (
            "true_sample_and_if_value",
            json!(calc_value_display_text(&true_sample_and_if_value)),
        ),
        (
            "true_sample_or_if_value",
            json!(calc_value_display_text(&true_sample_or_if_value)),
        ),
        (
            "true_sample_not_if_value",
            json!(calc_value_display_text(&true_sample_not_if_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_and_if_value",
            json!(calc_value_display_text(&middle_and_if_value)),
        ),
        (
            "middle_or_if_value",
            json!(calc_value_display_text(&middle_or_if_value)),
        ),
        (
            "middle_not_if_value",
            json!(calc_value_display_text(&middle_not_if_value)),
        ),
        (
            "last_and_if_value",
            json!(calc_value_display_text(&last_and_if_value)),
        ),
        (
            "last_or_if_value",
            json!(calc_value_display_text(&last_or_if_value)),
        ),
        (
            "last_not_if_value",
            json!(calc_value_display_text(&last_not_if_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "if-logical-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "if-logical-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "if-logical-r1c1-1M prepares three IF logical-condition templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "if-logical-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged if-logical-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "if-logical-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-IF-LOGICAL-R1C1-1M",
            "if-logical-r1c1-1M publishes dense numeric IF output for AND/OR/NOT R1C1 conditions",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && first_and_if_value == expected_and_if(1)
                && first_or_if_value == expected_or_if(1)
                && first_not_if_value == expected_not_if(1)
                && true_sample_and_if_value == expected_and_if(true_sample_row)
                && true_sample_or_if_value == expected_or_if(true_sample_row)
                && true_sample_not_if_value == expected_not_if(true_sample_row)
                && middle_and_if_value == expected_and_if(middle_row)
                && middle_or_if_value == expected_or_if(middle_row)
                && middle_not_if_value == expected_not_if(middle_row)
                && last_and_if_value == expected_and_if(options.rows)
                && last_or_if_value == expected_or_if(options.rows)
                && last_not_if_value == expected_not_if(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn two_left_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "two-left-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let formula_cols = 2_u32;
    let dense_cols = options.cols - formula_cols;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        dense_cols,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        dense_cols + 1,
        options.rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=RC[-2]+RC[-1]",
            "excel.grid.v1:r1c1-template:RC[-2]+RC[-1]",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows) * u64::from(dense_cols);
    let expected_formula_cells = u64::from(options.rows) * u64::from(formula_cols);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, dense_cols + 1)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation
        .read_cell(&address(middle_row, options.cols))
        .computed;
    let last_formula_value = valuation
        .read_cell(&address(options.rows, options.cols))
        .computed;
    let expected_first_formula =
        (1_000.0 + f64::from(dense_cols - 1)) + (1_000.0 + f64::from(dense_cols));
    let expected_last_formula = f64::from(options.rows) * 3000.0 + f64::from(dense_cols * 3 - 1);
    let expected_last_formula_display = integer_display(expected_last_formula);

    let counters = json_object([
        ("dense_columns", json!(dense_cols)),
        ("formula_columns", json!(formula_cols)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "two-left-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "two-left-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "two-left-r1c1-1M prepares one R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "two-left-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged two-left-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "two-left-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-TWO-LEFT-R1C1-1M",
            "two-left-r1c1-1M publishes dense formula output for RC[-2]+RC[-1]",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value)
                    == integer_display(expected_first_formula)
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn absolute_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "absolute-r1c1-1m requires at least 3 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]+R1C1", "excel.grid.v1:r1c1-template:RC[-1]+R1C1")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;
    let expected_middle_formula_display = integer_display(f64::from(middle_row) * 2.0 + 1.0);
    let expected_last_formula_display = integer_display(f64::from(options.rows) * 2.0 + 1.0);

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "expected_middle_formula_value",
            json!(expected_middle_formula_display.clone()),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "absolute-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "absolute-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "absolute-r1c1-1M prepares one mixed absolute/relative R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "absolute-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged absolute-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "absolute-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-ABSOLUTE-R1C1-1M",
            "absolute-r1c1-1M publishes dense formula output for RC[-1]+R1C1",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "3"
                && calc_value_display_text(&middle_formula_value)
                    == expected_middle_formula_display
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn division_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "division-r1c1-1m requires at least 2 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 2.0)?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]/2", "excel.grid.v1:r1c1-template:RC[-1]/2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let expected_last_formula_display = integer_display(f64::from(options.rows));

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "division-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "division-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "division-r1c1-1M prepares one R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "division-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged division-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "division-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-DIVISION-R1C1-1M",
            "division-r1c1-1M publishes dense formula output for RC[-1]/2",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "1"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn decimal_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "decimal-r1c1-1m requires at least 2 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 2.0)?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*0.5", "excel.grid.v1:r1c1-template:RC[-1]*0.5")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let expected_last_formula_display = integer_display(f64::from(options.rows));

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "decimal-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "decimal-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "decimal-r1c1-1M prepares one R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "decimal-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged decimal-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "decimal-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-DECIMAL-R1C1-1M",
            "decimal-r1c1-1M publishes dense formula output for RC[-1]*0.5",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "1"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn recursive_binary_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "recursive-binary-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row),
        2 => f64::from(address.row) * 10.0,
        _ => 2.0,
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=RC[-3]+RC[-2]*RC[-1]",
            "excel.grid.v1:r1c1-template:RC[-3]+RC[-2]*RC[-1]",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=(RC[-4]+RC[-3])*RC[-2]",
            "excel.grid.v1:r1c1-template:(RC[-4]+RC[-3])*RC[-2]",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(2);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let precedence_value = |row: u32| CalcValue::number(f64::from(row) * 21.0);
    let parenthesized_value = |row: u32| CalcValue::number(f64::from(row) * 22.0);
    let middle_row = (options.rows / 2).max(1);
    let first_precedence_value = valuation.read_cell(&address(1, 4)).computed;
    let first_parenthesized_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_precedence_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_parenthesized_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_precedence_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_parenthesized_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(2)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_precedence_value",
            json!(calc_value_display_text(&first_precedence_value)),
        ),
        (
            "first_parenthesized_value",
            json!(calc_value_display_text(&first_parenthesized_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_precedence_value",
            json!(calc_value_display_text(&middle_precedence_value)),
        ),
        (
            "middle_parenthesized_value",
            json!(calc_value_display_text(&middle_parenthesized_value)),
        ),
        (
            "last_precedence_value",
            json!(calc_value_display_text(&last_precedence_value)),
        ),
        (
            "last_parenthesized_value",
            json!(calc_value_display_text(&last_parenthesized_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "recursive-binary-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "recursive-binary-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "recursive-binary-r1c1-1M prepares two recursive R1C1 binary templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 2
                && recalc.compiled_formula_plans_cached == 2
        ),
        register_assertion(
            "P-14",
            "recursive-binary-r1c1-1M formula plan cache misses once per template and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 2
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(2)
                && recalc.compiled_formula_plan_cache_misses == 2
        ),
        register_assertion(
            "P-19",
            "unchanged recursive-binary-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "recursive-binary-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 2
        ),
        register_assertion(
            "GRID-RECURSIVE-BINARY-R1C1-1M",
            "recursive-binary-r1c1-1M publishes dense output for precedence and parenthesized arithmetic expressions",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 2
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 3
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_precedence_value == precedence_value(1)
                && first_parenthesized_value == parenthesized_value(1)
                && middle_precedence_value == precedence_value(middle_row)
                && middle_parenthesized_value == parenthesized_value(middle_row)
                && last_precedence_value == precedence_value(options.rows)
                && last_parenthesized_value == parenthesized_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn if_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "if-r1c1-1m requires at least 2 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.row % 2 == 0 {
            -f64::from(address.row)
        } else {
            f64::from(address.row)
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=IF(RC[-1]>0,RC[-1],0)",
            "excel.grid.v1:r1c1-template:IF(RC[-1]>0,RC[-1],0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 2)).computed;
    let last_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_input_value",
            json!(calc_value_display_text(&first_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_input_value",
            json!(calc_value_display_text(&middle_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_input_value",
            json!(calc_value_display_text(&last_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "if-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "if-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "if-r1c1-1M prepares one R1C1 IF template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "if-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged if-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "if-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-IF-R1C1-1M",
            "if-r1c1-1M publishes dense formula output for IF(RC[-1]>0,RC[-1],0)",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && first_input_value == CalcValue::number(1.0)
                && first_formula_value == CalcValue::number(1.0)
                && positive_tail_formula_value == CalcValue::number(f64::from(positive_tail_row))
                && last_formula_value
                    == if options.rows % 2 == 0 {
                        CalcValue::number(0.0)
                    } else {
                        CalcValue::number(f64::from(options.rows))
                    }
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn if_branch_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "if-branch-r1c1-1m requires at least 2 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.row % 2 == 0 {
            -f64::from(address.row)
        } else {
            f64::from(address.row)
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=IF(RC[-1]>0,RC[-1]*2,RC[-1]/2)",
            "excel.grid.v1:r1c1-template:IF(RC[-1]>0,RC[-1]*2,RC[-1]/2)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| {
        let input = if row % 2 == 0 {
            -f64::from(row)
        } else {
            f64::from(row)
        };
        if input > 0.0 {
            CalcValue::number(input * 2.0)
        } else {
            CalcValue::number(input / 2.0)
        }
    };
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 2)).computed;
    let last_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_input_value",
            json!(calc_value_display_text(&first_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_input_value",
            json!(calc_value_display_text(&middle_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_input_value",
            json!(calc_value_display_text(&last_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "if-branch-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "if-branch-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "if-branch-r1c1-1M prepares one R1C1 IF branch-expression template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "if-branch-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged if-branch-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "if-branch-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-IF-BRANCH-R1C1-1M",
            "if-branch-r1c1-1M publishes dense formula output for IF branches with scalar arithmetic",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_formula_value == expected_formula_value(1)
                && middle_formula_value == expected_formula_value(middle_row)
                && positive_tail_formula_value == expected_formula_value(positive_tail_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn nested_if_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "nested-if-r1c1-1m requires at least 3 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| match address.col {
        1 => f64::from(address.row),
        _ if address.row % 2 == 0 => 1.0,
        _ => -1.0,
    })?;
    let threshold = (options.rows / 2).max(1);
    let formula_text = format!(
        "=IF(RC[-2]>{threshold},IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+1,RC[-2]-1))"
    );
    let normal_form_key = format!(
        "excel.grid.v1:r1c1-template:IF(RC[-2]>{threshold},IF(RC[-1]>0,RC[-2]*2,RC[-2]*3),IF(RC[-1]>0,RC[-2]+1,RC[-2]-1))"
    );
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(formula_text, normal_form_key)
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| {
        let primary = f64::from(row);
        let selector = if row % 2 == 0 { 1.0 } else { -1.0 };
        if primary > f64::from(threshold) {
            if selector > 0.0 {
                CalcValue::number(primary * 2.0)
            } else {
                CalcValue::number(primary * 3.0)
            }
        } else if selector > 0.0 {
            CalcValue::number(primary + 1.0)
        } else {
            CalcValue::number(primary - 1.0)
        }
    };
    let middle_row = (options.rows / 2).max(1);
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let after_threshold_row = threshold.saturating_add(1).min(options.rows).max(1);
    let after_threshold_formula_value = valuation
        .read_cell(&address(after_threshold_row, 3))
        .computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        ("threshold_row", json!(threshold)),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("after_threshold_row", json!(after_threshold_row)),
        (
            "after_threshold_formula_value",
            json!(calc_value_display_text(&after_threshold_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "nested-if-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "nested-if-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "nested-if-r1c1-1M prepares one nested R1C1 IF template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "nested-if-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged nested-if-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "nested-if-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-NESTED-IF-R1C1-1M",
            "nested-if-r1c1-1M publishes dense output for nested scalar IF branches",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_formula_value == expected_formula_value(1)
                && middle_formula_value == expected_formula_value(middle_row)
                && after_threshold_formula_value == expected_formula_value(after_threshold_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn iferror_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "iferror-r1c1-1m requires at least 3 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.col == 1 {
            f64::from(address.row) * 2.0
        } else if address.row % 2 == 0 {
            0.0
        } else {
            2.0
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=IFERROR(RC[-2]/RC[-1],0)",
            "excel.grid.v1:r1c1-template:IFERROR(RC[-2]/RC[-1],0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_denominator_value = valuation.read_cell(&address(1, 2)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_denominator_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 3)).computed;
    let last_denominator_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_denominator_value",
            json!(calc_value_display_text(&first_denominator_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_denominator_value",
            json!(calc_value_display_text(&middle_denominator_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_denominator_value",
            json!(calc_value_display_text(&last_denominator_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "iferror-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "iferror-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "iferror-r1c1-1M prepares one R1C1 IFERROR template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "iferror-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged iferror-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "iferror-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-IFERROR-R1C1-1M",
            "iferror-r1c1-1M publishes dense formula output for IFERROR(RC[-2]/RC[-1],0)",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && first_denominator_value == CalcValue::number(2.0)
                && first_formula_value == CalcValue::number(1.0)
                && middle_denominator_value
                    == if middle_row % 2 == 0 {
                        CalcValue::number(0.0)
                    } else {
                        CalcValue::number(2.0)
                    }
                && middle_formula_value
                    == if middle_row % 2 == 0 {
                        CalcValue::number(0.0)
                    } else {
                        CalcValue::number(f64::from(middle_row))
                    }
                && positive_tail_formula_value == CalcValue::number(f64::from(positive_tail_row))
                && last_formula_value
                    == if options.rows % 2 == 0 {
                        CalcValue::number(0.0)
                    } else {
                        CalcValue::number(f64::from(options.rows))
                    }
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn comparison_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "comparison-r1c1-1m requires at least 2 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.row % 2 == 0 {
            -f64::from(address.row)
        } else {
            f64::from(address.row)
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]>0", "excel.grid.v1:r1c1-template:RC[-1]>0")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 2)).computed;
    let last_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_input_value",
            json!(calc_value_display_text(&first_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_input_value",
            json!(calc_value_display_text(&middle_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_input_value",
            json!(calc_value_display_text(&last_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "comparison-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "comparison-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "comparison-r1c1-1M prepares one R1C1 comparison template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "comparison-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged comparison-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "comparison-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-COMPARISON-R1C1-1M",
            "comparison-r1c1-1M publishes dense logical formula output for RC[-1]>0",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_dense_cells
                && valuation.dense_computed_logical_packed_cells() == expected_formula_cells
                && valuation.sparse_computed_cells() == 0
                && first_input_value == CalcValue::number(1.0)
                && first_formula_value == CalcValue::logical(true)
                && middle_formula_value == CalcValue::logical(middle_row % 2 == 1)
                && positive_tail_formula_value == CalcValue::logical(true)
                && last_formula_value == CalcValue::logical(options.rows % 2 == 1)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn comparison_expression_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "comparison-expression-r1c1-1m requires at least 3 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.col == 1 {
            if address.row % 2 == 0 {
                -f64::from(address.row)
            } else {
                f64::from(address.row)
            }
        } else {
            f64::from(address.row)
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=RC[-2]*2>RC[-1]+1",
            "excel.grid.v1:r1c1-template:RC[-2]*2>RC[-1]+1",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| {
        let left = if row % 2 == 0 {
            -f64::from(row)
        } else {
            f64::from(row)
        };
        CalcValue::logical(left * 2.0 > f64::from(row) + 1.0)
    };
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_left_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_right_input_value = valuation.read_cell(&address(1, 2)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_left_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_right_input_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 3)).computed;
    let last_left_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_right_input_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_left_input_value",
            json!(calc_value_display_text(&first_left_input_value)),
        ),
        (
            "first_right_input_value",
            json!(calc_value_display_text(&first_right_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_left_input_value",
            json!(calc_value_display_text(&middle_left_input_value)),
        ),
        (
            "middle_right_input_value",
            json!(calc_value_display_text(&middle_right_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_left_input_value",
            json!(calc_value_display_text(&last_left_input_value)),
        ),
        (
            "last_right_input_value",
            json!(calc_value_display_text(&last_right_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "comparison-expression-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "comparison-expression-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "comparison-expression-r1c1-1M prepares one scalar-expression comparison template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "comparison-expression-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged comparison-expression-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "comparison-expression-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-COMPARISON-EXPRESSION-R1C1-1M",
            "comparison-expression-r1c1-1M publishes dense logical output for scalar-expression comparison operands",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_dense_cells
                && valuation.dense_computed_logical_packed_cells() == expected_formula_cells
                && valuation.sparse_computed_cells() == 0
                && first_left_input_value == CalcValue::number(1.0)
                && first_right_input_value == CalcValue::number(1.0)
                && first_formula_value == expected_formula_value(1)
                && middle_formula_value == expected_formula_value(middle_row)
                && positive_tail_formula_value == expected_formula_value(positive_tail_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn comparison_iferror_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "comparison-iferror-r1c1-1m requires at least 3 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        if address.col == 1 {
            if address.row % 2 == 0 {
                -f64::from(address.row)
            } else {
                f64::from(address.row)
            }
        } else if address.row % 2 == 0 {
            0.0
        } else {
            1.0
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=IFERROR(RC[-2]/RC[-1],0)>0",
            "excel.grid.v1:r1c1-template:IFERROR(RC[-2]/RC[-1],0)>0",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| CalcValue::logical(row % 2 == 1);
    let middle_row = (options.rows / 2).max(1);
    let positive_tail_row = if options.rows % 2 == 1 {
        options.rows
    } else {
        options.rows.saturating_sub(1).max(1)
    };
    let first_left_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_right_input_value = valuation.read_cell(&address(1, 2)).computed;
    let first_formula_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_left_input_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let middle_right_input_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let middle_formula_value = valuation.read_cell(&address(middle_row, 3)).computed;
    let positive_tail_formula_value = valuation.read_cell(&address(positive_tail_row, 3)).computed;
    let last_left_input_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let last_right_input_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 3)).computed;

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_left_input_value",
            json!(calc_value_display_text(&first_left_input_value)),
        ),
        (
            "first_right_input_value",
            json!(calc_value_display_text(&first_right_input_value)),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_left_input_value",
            json!(calc_value_display_text(&middle_left_input_value)),
        ),
        (
            "middle_right_input_value",
            json!(calc_value_display_text(&middle_right_input_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        ("positive_tail_row", json!(positive_tail_row)),
        (
            "positive_tail_formula_value",
            json!(calc_value_display_text(&positive_tail_formula_value)),
        ),
        (
            "last_left_input_value",
            json!(calc_value_display_text(&last_left_input_value)),
        ),
        (
            "last_right_input_value",
            json!(calc_value_display_text(&last_right_input_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "comparison-iferror-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "comparison-iferror-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "comparison-iferror-r1c1-1M prepares one nested IFERROR comparison template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "comparison-iferror-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged comparison-iferror-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "comparison-iferror-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-COMPARISON-IFERROR-R1C1-1M",
            "comparison-iferror-r1c1-1M publishes dense logical output for nested IFERROR comparison operands",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_dense_cells
                && valuation.dense_computed_logical_packed_cells() == expected_formula_cells
                && valuation.sparse_computed_cells() == 0
                && first_formula_value == expected_formula_value(1)
                && middle_formula_value == expected_formula_value(middle_row)
                && positive_tail_formula_value == expected_formula_value(positive_tail_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn sum_row_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sum-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=SUM(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:SUM(RC[-3]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let expected_last_formula_display = integer_display(f64::from(options.rows) * 6.0);

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "sum-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "sum-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "sum-row-r1c1-1M prepares one R1C1 SUM range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "sum-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged sum-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "sum-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-SUM-ROW-R1C1-1M",
            "sum-row-r1c1-1M publishes dense formula output for SUM(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "6"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn sumsq_row_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sumsq-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=SUMSQ(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:SUMSQ(RC[-3]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_formula_value = |row: u32| {
        let row = f64::from(row);
        CalcValue::number(14.0 * row * row)
    };
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let expected_last_formula_display =
        integer_display(14.0 * f64::from(options.rows) * f64::from(options.rows));

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "sumsq-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "sumsq-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "sumsq-row-r1c1-1M prepares one R1C1 SUMSQ range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "sumsq-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged sumsq-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "sumsq-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-SUMSQ-ROW-R1C1-1M",
            "sumsq-row-r1c1-1M publishes dense formula output for SUMSQ(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_occupied_cells
                && valuation.sparse_computed_cells() == 0
                && first_formula_value == expected_formula_value(1)
                && middle_formula_value == expected_formula_value(middle_row)
                && last_formula_value == expected_formula_value(options.rows)
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn count_row_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "count-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=COUNT(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:COUNT(RC[-3]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        ("expected_formula_value", json!("3")),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "count-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "count-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "count-row-r1c1-1M prepares one R1C1 COUNT range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "count-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged count-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "count-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-COUNT-ROW-R1C1-1M",
            "count-row-r1c1-1M publishes dense formula output for COUNT(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "3"
                && calc_value_display_text(&middle_formula_value) == "3"
                && calc_value_display_text(&last_formula_value) == "3"
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn product_row_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "product-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.col))?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=PRODUCT(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:PRODUCT(RC[-3]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        ("expected_formula_value", json!("6")),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "product-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "product-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "product-row-r1c1-1M prepares one R1C1 PRODUCT range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "product-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged product-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "product-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-PRODUCT-ROW-R1C1-1M",
            "product-row-r1c1-1M publishes dense formula output for PRODUCT(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "6"
                && calc_value_display_text(&middle_formula_value) == "6"
                && calc_value_display_text(&last_formula_value) == "6"
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn average_row_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "average-row-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=AVERAGE(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:AVERAGE(RC[-3]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let expected_last_formula_display = integer_display(f64::from(options.rows) * 2.0);

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "average-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "average-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "average-row-r1c1-1M prepares one R1C1 AVERAGE range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "average-row-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged average-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "average-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-AVERAGE-ROW-R1C1-1M",
            "average-row-r1c1-1M publishes dense formula output for AVERAGE(RC[-3]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "2"
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn min_max_row_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "min-max-row-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * f64::from(address.col)
    })?;
    let min_formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        min_formula_rect,
        GridFormulaCell::new(
            "=MIN(RC[-3]:RC[-1])",
            "excel.grid.v1:r1c1-template:MIN(RC[-3]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let max_formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        5,
        options.rows,
        5,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        max_formula_rect,
        GridFormulaCell::new(
            "=MAX(RC[-4]:RC[-2])",
            "excel.grid.v1:r1c1-template:MAX(RC[-4]:RC[-2])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(2);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_min_value = valuation.read_cell(&address(1, 4)).computed;
    let first_max_value = valuation.read_cell(&address(1, 5)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_min_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let middle_max_value = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_min_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let last_max_value = valuation.read_cell(&address(options.rows, 5)).computed;
    let expected_last_min_display = integer_display(f64::from(options.rows));
    let expected_last_max_display = integer_display(f64::from(options.rows) * 3.0);

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(2)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_min_formula_value",
            json!(calc_value_display_text(&first_min_value)),
        ),
        (
            "first_max_formula_value",
            json!(calc_value_display_text(&first_max_value)),
        ),
        (
            "middle_min_formula_value",
            json!(calc_value_display_text(&middle_min_value)),
        ),
        (
            "middle_max_formula_value",
            json!(calc_value_display_text(&middle_max_value)),
        ),
        (
            "last_min_formula_value",
            json!(calc_value_display_text(&last_min_value)),
        ),
        (
            "last_max_formula_value",
            json!(calc_value_display_text(&last_max_value)),
        ),
        (
            "expected_last_min_formula_value",
            json!(expected_last_min_display.clone()),
        ),
        (
            "expected_last_max_formula_value",
            json!(expected_last_max_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "min-max-row-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "min-max-row-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "min-max-row-r1c1-1M prepares two R1C1 MIN/MAX range templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 2
                && recalc.compiled_formula_plans_cached == 2
        ),
        register_assertion(
            "P-14",
            "min-max-row-r1c1-1M formula plan cache misses twice and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 2
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(2)
                && recalc.compiled_formula_plan_cache_misses == 2
        ),
        register_assertion(
            "P-19",
            "unchanged min-max-row-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "min-max-row-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 2
        ),
        register_assertion(
            "GRID-MIN-MAX-ROW-R1C1-1M",
            "min-max-row-r1c1-1M publishes dense formula output for MIN/MAX row ranges",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 2
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 3
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_min_value) == "1"
                && calc_value_display_text(&first_max_value) == "3"
                && calc_value_display_text(&last_min_value) == expected_last_min_display
                && calc_value_display_text(&last_max_value) == expected_last_max_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn sum_window_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 3 || options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sum-window-r1c1-1m requires at least 3 rows and 2 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row))?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        3,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new(
            "=SUM(R[-2]C[-1]:RC[-1])",
            "excel.grid.v1:r1c1-template:SUM(R[-2]C[-1]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows.saturating_sub(2));
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(3, 2)).computed;
    let middle_row = (options.rows / 2).max(3);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let expected_middle_formula_display = integer_display(f64::from(middle_row) * 3.0 - 3.0);
    let expected_last_formula_display = integer_display(f64::from(options.rows) * 3.0 - 3.0);

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "expected_middle_formula_value",
            json!(expected_middle_formula_display.clone()),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_last_formula_value",
            json!(expected_last_formula_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "sum-window-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "sum-window-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "sum-window-r1c1-1M prepares one R1C1 SUM range template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "sum-window-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged sum-window-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "sum-window-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-SUM-WINDOW-R1C1-1M",
            "sum-window-r1c1-1M publishes dense formula output for SUM(R[-2]C[-1]:RC[-1])",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == "6"
                && calc_value_display_text(&middle_formula_value)
                    == expected_middle_formula_display
                && calc_value_display_text(&last_formula_value) == expected_last_formula_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn division_error_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "division-error-r1c1-1m requires at least 2 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 2.0)?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]/0", "excel.grid.v1:r1c1-template:RC[-1]/0")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_formula_value = valuation.read_cell(&address(1, 2)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_formula_value = valuation.read_cell(&address(middle_row, 2)).computed;
    let last_formula_value = valuation.read_cell(&address(options.rows, 2)).computed;
    let expected_error_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Div0));

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(1)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_formula_value",
            json!(calc_value_display_text(&first_formula_value)),
        ),
        (
            "middle_formula_value",
            json!(calc_value_display_text(&middle_formula_value)),
        ),
        (
            "last_formula_value",
            json!(calc_value_display_text(&last_formula_value)),
        ),
        (
            "expected_formula_error",
            json!(expected_error_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "division-error-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "division-error-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "division-error-r1c1-1M prepares one R1C1 template",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 1
                && recalc.compiled_formula_plans_cached == 1
        ),
        register_assertion(
            "P-14",
            "division-error-r1c1-1M formula plan cache misses once and hits for the remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 1
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(1)
                && recalc.compiled_formula_plan_cache_misses == 1
        ),
        register_assertion(
            "P-19",
            "unchanged division-error-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "division-error-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 1
        ),
        register_assertion(
            "GRID-DIVISION-ERROR-R1C1-1M",
            "division-error-r1c1-1M publishes dense formula output for RC[-1]/0",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 1
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 2
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_formula_value) == expected_error_display
                && calc_value_display_text(&middle_formula_value) == expected_error_display
                && calc_value_display_text(&last_formula_value) == expected_error_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn division_error_propagation_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "division-error-propagation-r1c1-1m requires at least 3 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 2.0)?;
    let division_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        division_rect,
        GridFormulaCell::new("=RC[-1]/0", "excel.grid.v1:r1c1-template:RC[-1]/0")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let propagation_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        propagation_rect,
        GridFormulaCell::new("=RC[-1]+1", "excel.grid.v1:r1c1-template:RC[-1]+1")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(2);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let first_direct_error = valuation.read_cell(&address(1, 2)).computed;
    let first_propagated_error = valuation.read_cell(&address(1, 3)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_propagated_error = valuation.read_cell(&address(middle_row, 3)).computed;
    let last_propagated_error = valuation.read_cell(&address(options.rows, 3)).computed;
    let expected_error_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Div0));

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(2)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_direct_error_value",
            json!(calc_value_display_text(&first_direct_error)),
        ),
        (
            "first_propagated_error_value",
            json!(calc_value_display_text(&first_propagated_error)),
        ),
        (
            "middle_propagated_error_value",
            json!(calc_value_display_text(&middle_propagated_error)),
        ),
        (
            "last_propagated_error_value",
            json!(calc_value_display_text(&last_propagated_error)),
        ),
        (
            "expected_formula_error",
            json!(expected_error_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "division-error-propagation-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "division-error-propagation-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "division-error-propagation-r1c1-1M prepares two R1C1 templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 2
                && recalc.compiled_formula_plans_cached == 2
        ),
        register_assertion(
            "P-14",
            "division-error-propagation-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 2
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(2)
                && recalc.compiled_formula_plan_cache_misses == 2
        ),
        register_assertion(
            "P-19",
            "unchanged division-error-propagation-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "division-error-propagation-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 2
        ),
        register_assertion(
            "GRID-DIVISION-ERROR-PROPAGATION-R1C1-1M",
            "division-error-propagation-r1c1-1M keeps direct and propagated Div0 outputs dense",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 2
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 3
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_direct_error) == expected_error_display
                && calc_value_display_text(&first_propagated_error) == expected_error_display
                && calc_value_display_text(&middle_propagated_error) == expected_error_display
                && calc_value_display_text(&last_propagated_error) == expected_error_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn aggregate_error_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "aggregate-error-r1c1-1m requires at least 4 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| f64::from(address.row) * 2.0)?;
    let direct_error_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        direct_error_rect,
        GridFormulaCell::new("=RC[-1]/0", "excel.grid.v1:r1c1-template:RC[-1]/0")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let aggregate_error_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        aggregate_error_rect,
        GridFormulaCell::new(
            "=SUM(RC[-2]:RC[-1])",
            "excel.grid.v1:r1c1-template:SUM(RC[-2]:RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let recovered_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        options.rows,
        4,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        recovered_rect,
        GridFormulaCell::new(
            "=IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])",
            "excel.grid.v1:r1c1-template:IFERROR(SUM(RC[-3]:RC[-2]),RC[-3])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_numeric_cells = u64::from(options.rows).saturating_mul(2);
    let first_direct_error = valuation.read_cell(&address(1, 2)).computed;
    let first_aggregate_error = valuation.read_cell(&address(1, 3)).computed;
    let first_recovered_value = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_aggregate_error = valuation.read_cell(&address(middle_row, 3)).computed;
    let middle_recovered_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_aggregate_error = valuation.read_cell(&address(options.rows, 3)).computed;
    let last_recovered_value = valuation.read_cell(&address(options.rows, 4)).computed;
    let expected_error_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Div0));
    let expected_middle_recovered_display =
        calc_value_display_text(&CalcValue::number(f64::from(middle_row) * 2.0));
    let expected_last_recovered_display =
        calc_value_display_text(&CalcValue::number(f64::from(options.rows) * 2.0));

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_direct_error_value",
            json!(calc_value_display_text(&first_direct_error)),
        ),
        (
            "first_aggregate_error_value",
            json!(calc_value_display_text(&first_aggregate_error)),
        ),
        (
            "middle_aggregate_error_value",
            json!(calc_value_display_text(&middle_aggregate_error)),
        ),
        (
            "last_aggregate_error_value",
            json!(calc_value_display_text(&last_aggregate_error)),
        ),
        (
            "first_recovered_value",
            json!(calc_value_display_text(&first_recovered_value)),
        ),
        (
            "middle_recovered_value",
            json!(calc_value_display_text(&middle_recovered_value)),
        ),
        (
            "last_recovered_value",
            json!(calc_value_display_text(&last_recovered_value)),
        ),
        (
            "expected_formula_error",
            json!(expected_error_display.clone()),
        ),
        (
            "expected_middle_recovered_value",
            json!(expected_middle_recovered_display.clone()),
        ),
        (
            "expected_last_recovered_value",
            json!(expected_last_recovered_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "aggregate-error-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "aggregate-error-r1c1-1M stays within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
                && byte_report.authored_bytes_per_cell_micros() <= 17_000_000
        ),
        register_assertion(
            "P-11",
            "aggregate-error-r1c1-1M prepares three R1C1 templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "aggregate-error-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged aggregate-error-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "aggregate-error-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-AGGREGATE-ERROR-R1C1-1M",
            "aggregate-error-r1c1-1M propagates range-aggregate errors and recovers with IFERROR without sparse output",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_numeric_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_direct_error) == expected_error_display
                && calc_value_display_text(&first_aggregate_error) == expected_error_display
                && calc_value_display_text(&middle_aggregate_error) == expected_error_display
                && calc_value_display_text(&last_aggregate_error) == expected_error_display
                && calc_value_display_text(&first_recovered_value) == "2"
                && calc_value_display_text(&middle_recovered_value)
                    == expected_middle_recovered_display
                && calc_value_display_text(&last_recovered_value)
                    == expected_last_recovered_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn text_function_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "text-function-r1c1-1m requires at least 5 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_literal_region_with(dense_rect, |_address| {
        CalcValue::text(ExcelText::from_interop_assignment("RowGrid"))
    })?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        GridFormulaCell::new("=LEN(RC[-1])", "excel.grid.v1:r1c1-template:LEN(RC[-1])")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=LEFT(RC[-2],3)",
            "excel.grid.v1:r1c1-template:LEFT(RC[-2],3)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=RIGHT(RC[-3],4)",
            "excel.grid.v1:r1c1-template:RIGHT(RC[-3],4)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=CONCAT(RC[-2],RC[-1])",
            "excel.grid.v1:r1c1-template:CONCAT(RC[-2],RC[-1])",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(4);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_numeric_cells = u64::from(options.rows);
    let first_input_value = valuation.read_cell(&address(1, 1)).computed;
    let first_len_value = valuation.read_cell(&address(1, 2)).computed;
    let first_left_value = valuation.read_cell(&address(1, 3)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_right_value = valuation.read_cell(&address(middle_row, 4)).computed;
    let last_concat_value = valuation.read_cell(&address(options.rows, 5)).computed;

    let counters = json_object([
        ("dense_columns", json!(1)),
        ("formula_columns", json!(4)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_input_value",
            json!(calc_value_display_text(&first_input_value)),
        ),
        (
            "first_len_value",
            json!(calc_value_display_text(&first_len_value)),
        ),
        (
            "first_left_value",
            json!(calc_value_display_text(&first_left_value)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_right_value",
            json!(calc_value_display_text(&middle_right_value)),
        ),
        (
            "last_concat_value",
            json!(calc_value_display_text(&last_concat_value)),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "text-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "text-function-r1c1-1M keeps uniform dense text, repeated formulas, and blanks within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "text-function-r1c1-1M prepares four R1C1 text templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "text-function-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged text-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "text-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-TEXT-FUNCTION-R1C1-1M",
            "text-function-r1c1-1M evaluates LEN/LEFT/RIGHT/CONCAT over R1C1 refs as dense output without sparse fallback",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 5
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_numeric_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_input_value) == "RowGrid"
                && calc_value_display_text(&first_len_value) == "7"
                && calc_value_display_text(&first_left_value) == "Row"
                && calc_value_display_text(&middle_right_value) == "Grid"
                && calc_value_display_text(&last_concat_value) == "RowGrid"
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn index_function_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "index-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.put_dense_number_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            1,
            options.rows,
            1,
            bounds,
        )?,
        |address| f64::from(address.row) * 10.0,
    )?;
    sheet.put_dense_literal_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        |_address| CalcValue::text(ExcelText::from_interop_assignment("Index")),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        GridFormulaCell::new(
            "=INDEX(RC[-2]:RC[-1],1,1)",
            "excel.grid.v1:r1c1-template:INDEX(RC[-2]:RC[-1],1,1)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=INDEX(RC[-3]:RC[-2],1,2)",
            "excel.grid.v1:r1c1-template:INDEX(RC[-3]:RC[-2],1,2)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=INDEX(R1C1:RC1,ROW(),1)",
            "excel.grid.v1:r1c1-template:INDEX(R1C1:RC1,ROW(),1)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            6,
            options.rows,
            6,
            bounds,
        )?,
        GridFormulaCell::new(
            "=INDEX(RC[-5]:RC[-4],2,1)",
            "excel.grid.v1:r1c1-template:INDEX(RC[-5]:RC[-4],2,1)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(2);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(4);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_numeric_cells = u64::from(options.rows).saturating_mul(3);
    let first_numeric_lookup = valuation.read_cell(&address(1, 3)).computed;
    let first_text_lookup = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_dynamic_lookup = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_dynamic_lookup = valuation.read_cell(&address(options.rows, 5)).computed;
    let first_ref_error = valuation.read_cell(&address(1, 6)).computed;
    let expected_ref_display = calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Ref));
    let expected_middle_dynamic_display =
        calc_value_display_text(&CalcValue::number(f64::from(middle_row) * 10.0));
    let expected_last_dynamic_display =
        calc_value_display_text(&CalcValue::number(f64::from(options.rows) * 10.0));

    let counters = json_object([
        ("dense_columns", json!(2)),
        ("formula_columns", json!(4)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_numeric_lookup_value",
            json!(calc_value_display_text(&first_numeric_lookup)),
        ),
        (
            "first_text_lookup_value",
            json!(calc_value_display_text(&first_text_lookup)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_dynamic_lookup_value",
            json!(calc_value_display_text(&middle_dynamic_lookup)),
        ),
        (
            "last_dynamic_lookup_value",
            json!(calc_value_display_text(&last_dynamic_lookup)),
        ),
        (
            "first_ref_error_value",
            json!(calc_value_display_text(&first_ref_error)),
        ),
        (
            "expected_middle_dynamic_lookup_value",
            json!(expected_middle_dynamic_display.clone()),
        ),
        (
            "expected_last_dynamic_lookup_value",
            json!(expected_last_dynamic_display.clone()),
        ),
        ("expected_ref_error", json!(expected_ref_display.clone())),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "index-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "index-function-r1c1-1M keeps dense values, repeated formulas, and blanks within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "index-function-r1c1-1M prepares four R1C1 INDEX templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "index-function-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged index-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "index-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 2
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-INDEX-FUNCTION-R1C1-1M",
            "index-function-r1c1-1M evaluates INDEX over R1C1 ranges as dense numeric, text, and #REF! output without sparse fallback",
            stats.dense_value_regions == 2
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 6
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_numeric_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_numeric_lookup) == "10"
                && calc_value_display_text(&first_text_lookup) == "Index"
                && calc_value_display_text(&middle_dynamic_lookup)
                    == expected_middle_dynamic_display
                && calc_value_display_text(&last_dynamic_lookup) == expected_last_dynamic_display
                && calc_value_display_text(&first_ref_error) == expected_ref_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn match_function_r1c1_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "match-function-r1c1-1m requires at least 6 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.put_dense_number_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            1,
            options.rows,
            3,
            bounds,
        )?,
        |address| f64::from(address.row) * 10.0 + f64::from(address.col) - 1.0,
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=MATCH(RC[-2],RC[-3]:RC[-1],0)",
            "excel.grid.v1:r1c1-template:MATCH(RC[-2],RC[-3]:RC[-1],0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))",
            "excel.grid.v1:r1c1-template:INDEX(RC[-4]:RC[-2],1,MATCH(RC[-3],RC[-4]:RC[-2],0))",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            6,
            options.rows,
            6,
            bounds,
        )?,
        GridFormulaCell::new(
            "=MATCH(999999999,RC[-5]:RC[-3],0)",
            "excel.grid.v1:r1c1-template:MATCH(999999999,RC[-5]:RC[-3],0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(3);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_numeric_cells = u64::from(options.rows).saturating_mul(5);
    let first_match_position = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_index_match = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_no_match = valuation.read_cell(&address(options.rows, 6)).computed;
    let expected_middle_index_display =
        calc_value_display_text(&CalcValue::number(f64::from(middle_row) * 10.0 + 1.0));
    let expected_no_match_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::NA));

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(3)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_match_position_value",
            json!(calc_value_display_text(&first_match_position)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_index_match_value",
            json!(calc_value_display_text(&middle_index_match)),
        ),
        (
            "last_no_match_value",
            json!(calc_value_display_text(&last_no_match)),
        ),
        (
            "expected_middle_index_match_value",
            json!(expected_middle_index_display.clone()),
        ),
        (
            "expected_no_match",
            json!(expected_no_match_display.clone()),
        ),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "match-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "match-function-r1c1-1M keeps dense values, repeated formulas, and blanks within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "match-function-r1c1-1M prepares three R1C1 MATCH/INDEX templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 3
                && recalc.compiled_formula_plans_cached == 3
        ),
        register_assertion(
            "P-14",
            "match-function-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 3
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(3)
                && recalc.compiled_formula_plan_cache_misses == 3
        ),
        register_assertion(
            "P-19",
            "unchanged match-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "match-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 1
                && partition.repeated_formula_regions == 3
        ),
        register_assertion(
            "GRID-MATCH-FUNCTION-R1C1-1M",
            "match-function-r1c1-1M evaluates exact MATCH and nested INDEX/MATCH over R1C1 ranges as dense numeric and #N/A output without sparse fallback",
            stats.dense_value_regions == 1
                && stats.repeated_formula_regions == 3
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 4
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_numeric_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_match_position) == "2"
                && calc_value_display_text(&middle_index_match) == expected_middle_index_display
                && calc_value_display_text(&last_no_match) == expected_no_match_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn vlookup_function_r1c1_1m_scale(
    options: &GridScaleOptions,
) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 7 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "vlookup-function-r1c1-1m requires at least 7 columns".to_string(),
        });
    }
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.put_dense_number_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            1,
            options.rows,
            1,
            bounds,
        )?,
        |address| f64::from(address.row) * 10.0,
    )?;
    sheet.put_dense_literal_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            2,
            options.rows,
            2,
            bounds,
        )?,
        |_address| CalcValue::text(ExcelText::from_interop_assignment("Lookup")),
    )?;
    sheet.put_dense_number_region_with(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            3,
            options.rows,
            3,
            bounds,
        )?,
        |address| f64::from(address.row) * 100.0,
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            4,
            options.rows,
            4,
            bounds,
        )?,
        GridFormulaCell::new(
            "=VLOOKUP(RC[-3],RC[-3]:RC[-1],2,FALSE)",
            "excel.grid.v1:r1c1-template:VLOOKUP(RC[-3],RC[-3]:RC[-1],2,FALSE)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            5,
            options.rows,
            5,
            bounds,
        )?,
        GridFormulaCell::new(
            "=VLOOKUP(RC[-4],RC[-4]:RC[-2],3,0)",
            "excel.grid.v1:r1c1-template:VLOOKUP(RC[-4],RC[-4]:RC[-2],3,0)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            6,
            options.rows,
            6,
            bounds,
        )?,
        GridFormulaCell::new(
            "=VLOOKUP(999999999,RC[-5]:RC[-3],2,FALSE)",
            "excel.grid.v1:r1c1-template:VLOOKUP(999999999,RC[-5]:RC[-3],2,FALSE)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    sheet.put_repeated_formula_region(
        GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            1,
            7,
            options.rows,
            7,
            bounds,
        )?,
        GridFormulaCell::new(
            "=VLOOKUP(RC[-6],RC[-6]:RC[-4],4,FALSE)",
            "excel.grid.v1:r1c1-template:VLOOKUP(RC[-6],RC[-6]:RC[-4],4,FALSE)",
        )
        .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, recalc, cache) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml_cached(materialization_limit)?;
    let (_, warm) = sheet
        .recalculate_warm_noop_compact_with_oxfml(&cache)
        .ok_or(GridScaleRunnerError::Grid(
            GridRefError::OptimizedWarmNoOpCacheStale,
        ))?;
    let stats = sheet.storage_stats();
    let byte_report = sheet.storage_byte_report();
    let partition = sheet.partition_witness_report();
    let expected_dense_cells = u64::from(options.rows).saturating_mul(3);
    let expected_formula_cells = u64::from(options.rows).saturating_mul(4);
    let expected_occupied_cells = expected_dense_cells.saturating_add(expected_formula_cells);
    let expected_numeric_cells = u64::from(options.rows).saturating_mul(3);
    let first_text_lookup = valuation.read_cell(&address(1, 4)).computed;
    let middle_row = (options.rows / 2).max(1);
    let middle_numeric_lookup = valuation.read_cell(&address(middle_row, 5)).computed;
    let last_no_match = valuation.read_cell(&address(options.rows, 6)).computed;
    let first_ref_error = valuation.read_cell(&address(1, 7)).computed;
    let expected_middle_numeric_display =
        calc_value_display_text(&CalcValue::number(f64::from(middle_row) * 100.0));
    let expected_no_match_display =
        calc_value_display_text(&CalcValue::error(WorksheetErrorCode::NA));
    let expected_ref_display = calc_value_display_text(&CalcValue::error(WorksheetErrorCode::Ref));

    let counters = json_object([
        ("dense_columns", json!(3)),
        ("formula_columns", json!(4)),
        ("dense_value_regions", json!(stats.dense_value_regions)),
        ("dense_value_cells", json!(stats.dense_value_cells)),
        (
            "repeated_formula_regions",
            json!(stats.repeated_formula_regions),
        ),
        (
            "repeated_formula_cells",
            json!(stats.repeated_formula_cells),
        ),
        (
            "authored_storage_bytes",
            json!(byte_report.authored_storage_bytes),
        ),
        (
            "dense_value_region_bytes",
            json!(byte_report.dense_value_region_bytes),
        ),
        (
            "repeated_formula_region_bytes",
            json!(byte_report.repeated_formula_region_bytes),
        ),
        (
            "authored_bytes_per_cell_micros",
            json!(byte_report.authored_bytes_per_cell_micros()),
        ),
        (
            "dense_bytes_per_cell_micros",
            json!(byte_report.dense_bytes_per_cell_micros()),
        ),
        (
            "repeated_formula_bytes_per_cell_micros",
            json!(byte_report.repeated_formula_bytes_per_cell_micros()),
        ),
        ("blank_cell_bytes", json!(byte_report.blank_cell_bytes)),
        ("occupied_cells", json!(recalc.occupied_cells)),
        ("literal_cells", json!(recalc.literal_cells)),
        ("formula_cells", json!(recalc.formula_cells)),
        ("formula_evaluations", json!(recalc.formula_evaluations)),
        (
            "formula_templates_prepared",
            json!(recalc.formula_templates_prepared),
        ),
        (
            "distinct_formula_templates",
            json!(recalc.distinct_formula_templates),
        ),
        (
            "formula_plan_cache_lookups",
            json!(recalc.formula_plan_cache_lookups()),
        ),
        (
            "formula_plan_cache_hits",
            json!(recalc.formula_plan_cache_hits),
        ),
        (
            "formula_plan_cache_misses",
            json!(recalc.formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plan_cache_misses",
            json!(recalc.compiled_formula_plan_cache_misses),
        ),
        (
            "compiled_formula_plans_cached",
            json!(recalc.compiled_formula_plans_cached),
        ),
        (
            "computed_dense_value_regions",
            json!(recalc.computed_dense_value_regions),
        ),
        (
            "computed_dense_cells",
            json!(valuation.dense_computed_cells()),
        ),
        (
            "computed_dense_numeric_packed_cells",
            json!(valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "computed_dense_logical_packed_cells",
            json!(valuation.dense_computed_logical_packed_cells()),
        ),
        (
            "computed_sparse_cells",
            json!(valuation.sparse_computed_cells()),
        ),
        (
            "first_text_lookup_value",
            json!(calc_value_display_text(&first_text_lookup)),
        ),
        ("middle_row", json!(middle_row)),
        (
            "middle_numeric_lookup_value",
            json!(calc_value_display_text(&middle_numeric_lookup)),
        ),
        (
            "last_no_match_value",
            json!(calc_value_display_text(&last_no_match)),
        ),
        (
            "first_ref_error_value",
            json!(calc_value_display_text(&first_ref_error)),
        ),
        (
            "expected_middle_numeric_lookup_value",
            json!(expected_middle_numeric_display.clone()),
        ),
        (
            "expected_no_match",
            json!(expected_no_match_display.clone()),
        ),
        ("expected_ref_error", json!(expected_ref_display.clone())),
        ("warm_cells_visited", json!(warm.cells_visited)),
        ("warm_formula_evaluations", json!(warm.formula_evaluations)),
        (
            "partition_max_parallelism_bound",
            json!(partition.max_parallelism_bound),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-00",
            "vlookup-function-r1c1-1M recalc visits each occupied cell once",
            recalc.p00_primary_exact_once_holds()
                && recalc.occupied_cells == expected_occupied_cells
        ),
        register_assertion(
            "P-10",
            "vlookup-function-r1c1-1M keeps dense values, repeated formulas, and blanks within the authored byte floor",
            byte_report.p10_dense_value_budget_holds()
                && byte_report.p10_repeated_formula_budget_holds()
                && byte_report.p10_blank_cells_zero_bytes_holds()
        ),
        register_assertion(
            "P-11",
            "vlookup-function-r1c1-1M prepares four R1C1 exact VLOOKUP templates",
            recalc.p11_template_prepare_once_holds()
                && recalc.formula_templates_prepared == 4
                && recalc.compiled_formula_plans_cached == 4
        ),
        register_assertion(
            "P-14",
            "vlookup-function-r1c1-1M formula plan cache misses once per template and hits for remaining formula cells",
            recalc.p14_plan_cache_hit_floor_holds()
                && recalc.formula_plan_cache_misses == 4
                && recalc.formula_plan_cache_hits == expected_formula_cells.saturating_sub(4)
                && recalc.compiled_formula_plan_cache_misses == 4
        ),
        register_assertion(
            "P-19",
            "unchanged vlookup-function-r1c1-1M sheet hits warm no-op cache",
            warm.p19_warm_noop_holds()
        ),
        register_assertion(
            "P-18",
            "vlookup-function-r1c1-1M compact dense and repeated-formula regions have a valid partition witness",
            partition.p18_partition_witness_holds()
                && partition.dense_value_regions == 3
                && partition.repeated_formula_regions == 4
        ),
        register_assertion(
            "GRID-VLOOKUP-FUNCTION-R1C1-1M",
            "vlookup-function-r1c1-1M evaluates exact VLOOKUP over R1C1 ranges as dense text, numeric, #N/A, and #REF! output without sparse fallback",
            stats.dense_value_regions == 3
                && stats.repeated_formula_regions == 4
                && stats.dense_value_cells == expected_dense_cells
                && stats.repeated_formula_cells == expected_formula_cells
                && recalc.literal_cells == expected_dense_cells
                && recalc.formula_cells == expected_formula_cells
                && recalc.computed_dense_value_regions == 7
                && valuation.dense_computed_cells() == expected_occupied_cells
                && valuation.dense_computed_numeric_packed_cells() == expected_numeric_cells
                && valuation.dense_computed_logical_packed_cells() == 0
                && valuation.sparse_computed_cells() == 0
                && calc_value_display_text(&first_text_lookup) == "Lookup"
                && calc_value_display_text(&middle_numeric_lookup)
                    == expected_middle_numeric_display
                && calc_value_display_text(&last_no_match) == expected_no_match_display
                && calc_value_display_text(&first_ref_error) == expected_ref_display
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn plan_cache_rounds_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "plan-cache-rounds-1m requires at least 2 columns".to_string(),
        });
    }
    let formula_cols = if options.cols >= 10 {
        2
    } else {
        (options.cols / 2).max(1)
    };
    let dense_cols = options.cols - formula_cols;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        dense_cols,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        dense_cols + 1,
        options.rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let rounds = 3_usize;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let cache_report = sheet.persistent_formula_plan_cache_report(rounds, materialization_limit)?;
    let expected_formula_cells = u64::from(options.rows) * u64::from(formula_cols);
    let expected_total_lookups = expected_formula_cells.saturating_mul(rounds as u64);
    let expected_total_hits = expected_total_lookups.saturating_sub(1);
    let round_reports = cache_report
        .round_reports
        .iter()
        .map(|round| {
            json!({
                "round_index": round.round_index,
                "formula_cells": round.formula_cells,
                "distinct_formula_templates": round.distinct_formula_templates,
                "formula_plan_cache_lookups": round.formula_plan_cache_lookups(),
                "formula_plan_cache_hits": round.formula_plan_cache_hits,
                "formula_plan_cache_misses": round.formula_plan_cache_misses,
                "formula_plan_cache_hit_rate_micros": round.formula_plan_cache_hit_rate_micros(),
                "compiled_formula_plan_cache_hits": round.compiled_formula_plan_cache_hits,
                "compiled_formula_plan_cache_misses": round.compiled_formula_plan_cache_misses,
                "cached_template_count_after_round": round.cached_template_count_after_round,
                "cached_compiled_plan_count_after_round": round.cached_compiled_plan_count_after_round,
            })
        })
        .collect::<Vec<_>>();
    let first_round_hits = cache_report
        .round_reports
        .first()
        .map_or(0, |round| round.formula_plan_cache_hits);
    let second_round_misses = cache_report
        .round_reports
        .get(1)
        .map_or(0, |round| round.formula_plan_cache_misses);
    let third_round_misses = cache_report
        .round_reports
        .get(2)
        .map_or(0, |round| round.formula_plan_cache_misses);
    let first_round_compiled_plan_misses = cache_report
        .round_reports
        .first()
        .map_or(0, |round| round.compiled_formula_plan_cache_misses);
    let later_round_compiled_plan_misses = cache_report
        .round_reports
        .iter()
        .skip(1)
        .map(|round| round.compiled_formula_plan_cache_misses)
        .sum::<u64>();

    let counters = json_object([
        ("rounds", json!(cache_report.rounds)),
        ("dense_columns", json!(dense_cols)),
        ("formula_columns", json!(formula_cols)),
        (
            "formula_cells_per_round",
            json!(cache_report.formula_cells_per_round),
        ),
        (
            "distinct_formula_templates",
            json!(cache_report.distinct_formula_templates),
        ),
        (
            "cached_template_count",
            json!(cache_report.cached_template_count),
        ),
        (
            "cached_compiled_plan_count",
            json!(cache_report.cached_compiled_plan_count),
        ),
        ("first_round_misses", json!(cache_report.first_round_misses)),
        ("later_round_misses", json!(cache_report.later_round_misses)),
        ("total_lookups", json!(cache_report.total_lookups())),
        ("total_hits", json!(cache_report.total_hits)),
        ("total_misses", json!(cache_report.total_misses)),
        (
            "total_compiled_plan_hits",
            json!(cache_report.total_compiled_plan_hits),
        ),
        (
            "total_compiled_plan_misses",
            json!(cache_report.total_compiled_plan_misses),
        ),
        (
            "first_round_compiled_plan_misses",
            json!(first_round_compiled_plan_misses),
        ),
        (
            "later_round_compiled_plan_misses",
            json!(later_round_compiled_plan_misses),
        ),
        ("hit_rate_micros", json!(cache_report.hit_rate_micros())),
        ("first_round_hits", json!(first_round_hits)),
        ("second_round_misses", json!(second_round_misses)),
        ("third_round_misses", json!(third_round_misses)),
        ("round_reports", json!(round_reports)),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-14",
            "persistent formula plan cache misses only during the first dirty recalc round",
            cache_report.p14_persistent_plan_cache_holds()
                && cache_report.formula_cells_per_round == expected_formula_cells
                && cache_report.total_hits == expected_total_hits
                && cache_report.total_misses == 1
        ),
        register_assertion(
            "GRID-PLAN-CACHE-ROUNDS-1M",
            "plan-cache-rounds-1M preserves one cached R1C1 template across three dirty recalc rounds",
            cache_report.rounds == rounds
                && cache_report.distinct_formula_templates == 1
                && cache_report.cached_template_count == 1
                && cache_report.cached_compiled_plan_count == 1
                && cache_report.first_round_misses == 1
                && cache_report.later_round_misses == 0
                && cache_report.total_compiled_plan_misses == 1
                && cache_report.total_compiled_plan_hits == rounds.saturating_sub(1) as u64
                && cache_report.total_lookups() == expected_total_lookups
        )
    ]);

    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn insert_storm_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 16 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "insert-storm-1m requires at least 16 rows".to_string(),
        });
    }
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "insert-storm-1m requires at least 2 columns".to_string(),
        });
    }

    let formula_cols = if options.cols >= 10 {
        2
    } else {
        (options.cols / 2).max(1)
    };
    let dense_cols = options.cols - formula_cols;
    let occupied_rows = options.rows - 8;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        occupied_rows,
        dense_cols,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        dense_cols + 1,
        occupied_rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let initial_stats = sheet.storage_stats();
    let initial_byte_report = sheet.storage_byte_report();
    let edit_rows = insert_storm_rows(occupied_rows);
    let row_edit_pairs = edit_rows.len();
    let mut edit_reports = Vec::with_capacity(edit_rows.len() * 2);
    for row in edit_rows {
        edit_reports.push(sheet.apply_axis_edit(GridAxisEdit::insert_rows(row, 1))?);
        edit_reports.push(sheet.apply_axis_edit(GridAxisEdit::delete_rows(row + 2, 1))?);
    }
    let final_stats = sheet.storage_stats();
    let final_byte_report = sheet.storage_byte_report();
    let storm = InsertStormReport::from_reports(&edit_reports);
    let initial_authored_cells = initial_stats
        .dense_value_cells
        .saturating_add(initial_stats.repeated_formula_cells);
    let final_authored_cells = final_stats
        .dense_value_cells
        .saturating_add(final_stats.repeated_formula_cells);
    let expected_deleted_cells =
        u64::try_from(row_edit_pairs).unwrap_or(u64::MAX) * u64::from(options.cols);
    let naive_cell_rewrite_floor = initial_authored_cells
        .saturating_mul(u64::try_from(edit_reports.len()).unwrap_or(u64::MAX));
    let compact_metadata_touch_ratio_micros = micros_ratio(
        storm.compact_region_metadata_touches,
        naive_cell_rewrite_floor,
    );

    Ok(json!({
        "counters": {
            "occupied_rows_before": occupied_rows,
            "dense_columns": dense_cols,
            "formula_columns": formula_cols,
            "row_edit_pairs": row_edit_pairs,
            "edits_applied": edit_reports.len(),
            "dense_regions_initial": initial_stats.dense_value_regions,
            "repeated_formula_regions_initial": initial_stats.repeated_formula_regions,
            "dense_regions_final": final_stats.dense_value_regions,
            "repeated_formula_regions_final": final_stats.repeated_formula_regions,
            "max_dense_regions_after_edit": storm.max_dense_regions_after,
            "max_repeated_formula_regions_after_edit": storm.max_repeated_formula_regions_after,
            "dense_region_metadata_visits": storm.dense_region_metadata_visits,
            "repeated_formula_region_metadata_visits": storm.repeated_formula_region_metadata_visits,
            "compact_region_metadata_touches": storm.compact_region_metadata_touches,
            "naive_cell_rewrite_floor": naive_cell_rewrite_floor,
            "compact_metadata_touch_ratio_micros": compact_metadata_touch_ratio_micros,
            "dense_value_cells_initial": initial_stats.dense_value_cells,
            "repeated_formula_cells_initial": initial_stats.repeated_formula_cells,
            "dense_value_cells_final": final_stats.dense_value_cells,
            "repeated_formula_cells_final": final_stats.repeated_formula_cells,
            "authored_cells_initial": initial_authored_cells,
            "authored_cells_final": final_authored_cells,
            "expected_deleted_cells": expected_deleted_cells,
            "actual_deleted_cells": initial_authored_cells.saturating_sub(final_authored_cells),
            "sparse_point_cells_final": final_stats.sparse_point_cells,
            "dense_value_regions_dropped": storm.dense_value_regions_dropped,
            "repeated_formula_regions_dropped": storm.repeated_formula_regions_dropped,
            "repeated_formula_segments_transformed": storm.repeated_formula_segments_transformed,
            "repeated_formula_reference_transforms": storm.repeated_formula_reference_transforms,
            "authored_storage_bytes_initial": initial_byte_report.authored_storage_bytes,
            "authored_storage_bytes_final": final_byte_report.authored_storage_bytes,
            "blank_cell_bytes_final": final_byte_report.blank_cell_bytes
        },
        "register_assertions": [
            register_assertion(
                "P-10",
                "insert-storm compact storage remains region-backed and blanks cost zero bytes",
                final_byte_report.p10_dense_value_budget_holds()
                    && final_byte_report.p10_repeated_formula_budget_holds()
                    && final_byte_report.p10_blank_cells_zero_bytes_holds()
                    && final_stats.sparse_point_cells == 0
            ),
            register_assertion(
                "P-17",
                "row insert/delete storm touches compact region metadata rather than rewriting all authored cells",
                storm.compact_region_metadata_touches
                    <= u64::try_from(edit_reports.len()).unwrap_or(u64::MAX).saturating_mul(16)
                    && storm.compact_region_metadata_touches < naive_cell_rewrite_floor
                    && compact_metadata_touch_ratio_micros <= 20_000
            ),
            register_assertion(
                "GRID-INSERT-STORM-1M",
                "insert/delete row storm preserves compact dense and repeated formula regions with only deleted rows removed",
                final_stats.sparse_point_cells == 0
                    && initial_stats.dense_value_regions == 1
                    && initial_stats.repeated_formula_regions == 1
                    && final_stats.dense_value_regions <= edit_reports.len() + 1
                    && final_stats.repeated_formula_regions <= edit_reports.len() + 1
                    && initial_authored_cells.saturating_sub(final_authored_cells) == expected_deleted_cells
            )
        ]
    }))
}

fn cow_retention_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 16 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "cow-retention-1m requires at least 16 rows".to_string(),
        });
    }
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "cow-retention-1m requires at least 2 columns".to_string(),
        });
    }

    let formula_cols = if options.cols >= 10 {
        2
    } else {
        (options.cols / 2).max(1)
    };
    let dense_cols = options.cols - formula_cols;
    let occupied_rows = options.rows - 8;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        occupied_rows,
        dense_cols,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        dense_cols + 1,
        occupied_rows,
        options.cols,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let initial_stats = sheet.storage_stats();
    let initial_byte_report = sheet.storage_byte_report();
    let edit_rows = insert_storm_rows(occupied_rows);
    let mut retained_roots = vec![sheet.clone()];
    let mut edit_reports = Vec::with_capacity(edit_rows.len() * 2);
    for row in edit_rows {
        edit_reports.push(sheet.apply_axis_edit(GridAxisEdit::insert_rows(row, 1))?);
        retained_roots.push(sheet.clone());
        edit_reports.push(sheet.apply_axis_edit(GridAxisEdit::delete_rows(row + 2, 1))?);
        retained_roots.push(sheet.clone());
    }
    let final_stats = sheet.storage_stats();
    let final_byte_report = sheet.storage_byte_report();
    let retention = GridOptimizedSheet::cow_retention_report(&retained_roots);
    let storm = InsertStormReport::from_reports(&edit_reports);
    let initial_authored_cells = initial_stats
        .dense_value_cells
        .saturating_add(initial_stats.repeated_formula_cells);
    let final_authored_cells = final_stats
        .dense_value_cells
        .saturating_add(final_stats.repeated_formula_cells);
    let expected_deleted_cells =
        u64::try_from(edit_rows.len()).unwrap_or(u64::MAX) * u64::from(options.cols);
    let naive_cell_rewrite_floor = initial_authored_cells
        .saturating_mul(u64::try_from(edit_reports.len()).unwrap_or(u64::MAX));

    Ok(json!({
        "counters": {
            "occupied_rows_before": occupied_rows,
            "dense_columns": dense_cols,
            "formula_columns": formula_cols,
            "edits_applied": edit_reports.len(),
            "retained_revision_count": retention.retained_revision_count,
            "unique_dense_payloads": retention.unique_dense_payloads,
            "unique_dense_payload_bytes": retention.unique_dense_payload_bytes,
            "dense_region_metadata_bytes_retained": retention.dense_region_metadata_bytes,
            "repeated_formula_region_bytes_retained": retention.repeated_formula_region_bytes,
            "sparse_point_bytes_retained": retention.sparse_point_bytes,
            "sheet_root_metadata_bytes_retained": retention.sheet_root_metadata_bytes,
            "retained_compact_regions": retention.retained_compact_regions,
            "cow_retained_bytes": retention.cow_retained_bytes,
            "naive_full_snapshot_retention_bytes_floor": retention.naive_full_snapshot_retention_bytes_floor,
            "retained_to_naive_ratio_micros": retention.retained_to_naive_ratio_micros,
            "compact_region_metadata_touches": storm.compact_region_metadata_touches,
            "naive_cell_rewrite_floor": naive_cell_rewrite_floor,
            "dense_value_cells_initial": initial_stats.dense_value_cells,
            "repeated_formula_cells_initial": initial_stats.repeated_formula_cells,
            "dense_value_cells_final": final_stats.dense_value_cells,
            "repeated_formula_cells_final": final_stats.repeated_formula_cells,
            "authored_cells_initial": initial_authored_cells,
            "authored_cells_final": final_authored_cells,
            "expected_deleted_cells": expected_deleted_cells,
            "actual_deleted_cells": initial_authored_cells.saturating_sub(final_authored_cells),
            "dense_regions_final": final_stats.dense_value_regions,
            "repeated_formula_regions_final": final_stats.repeated_formula_regions,
            "sparse_point_cells_final": final_stats.sparse_point_cells,
            "authored_storage_bytes_initial": initial_byte_report.authored_storage_bytes,
            "authored_storage_bytes_final": final_byte_report.authored_storage_bytes,
            "blank_cell_bytes_final": final_byte_report.blank_cell_bytes
        },
        "register_assertions": [
            register_assertion(
                "P-21",
                "retained COW roots share dense payload bytes and grow with compact touched regions rather than full snapshots",
                retention.p21_cow_retention_holds()
                    && retention.retained_revision_count == edit_reports.len() + 1
                    && retention.unique_dense_payloads == 1
                    && retention.cow_retained_bytes < retention.naive_full_snapshot_retention_bytes_floor
                    && retention.retained_to_naive_ratio_micros <= 500_000
                    && storm.compact_region_metadata_touches < naive_cell_rewrite_floor
            ),
            register_assertion(
                "GRID-COW-RETENTION-1M",
                "COW retention preserves compact edited roots without sparse materialization or full dense payload duplication",
                final_stats.sparse_point_cells == 0
                    && final_byte_report.p10_blank_cells_zero_bytes_holds()
                    && initial_authored_cells.saturating_sub(final_authored_cells) == expected_deleted_cells
                    && final_stats.dense_value_regions <= edit_reports.len() + 1
                    && final_stats.repeated_formula_regions <= edit_reports.len() + 1
            )
        ]
    }))
}

fn publication_delta_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "publication-delta-1m requires at least 2 columns".to_string(),
        });
    }
    let changed_row = (options.rows / 2).max(1);
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let previous_sheet = publication_delta_sheet(options, bounds, None)?;
    let current_sheet = publication_delta_sheet(options, bounds, Some(changed_row))?;
    let (previous_valuation, previous_recalc) =
        previous_sheet.recalculate_mark_all_dirty_compact_with_oxfml(materialization_limit)?;
    let (current_valuation, current_recalc) =
        current_sheet.recalculate_mark_all_dirty_compact_with_oxfml(materialization_limit)?;
    let delta = current_valuation.publication_delta_report_since(&previous_valuation);
    let publication_entries_total = delta.publication_entries_total();
    let publication_entry_ratio_micros = micros_ratio(
        u64::try_from(publication_entries_total).unwrap_or(u64::MAX),
        delta.naive_current_computed_cell_publication_floor,
    );
    let previous_changed_input = previous_valuation
        .read_cell(&address(changed_row, 1))
        .computed;
    let current_changed_input = current_valuation
        .read_cell(&address(changed_row, 1))
        .computed;
    let previous_changed_formula = previous_valuation
        .read_cell(&address(changed_row, 2))
        .computed;
    let current_changed_formula = current_valuation
        .read_cell(&address(changed_row, 2))
        .computed;

    Ok(json!({
        "counters": {
            "changed_row": changed_row,
            "previous_occupied_cells": previous_recalc.occupied_cells,
            "current_occupied_cells": current_recalc.occupied_cells,
            "previous_formula_cells": previous_recalc.formula_cells,
            "current_formula_cells": current_recalc.formula_cells,
            "previous_computed_dense_value_regions": previous_recalc.computed_dense_value_regions,
            "current_computed_dense_value_regions": current_recalc.computed_dense_value_regions,
            "previous_computed_sparse_cells": previous_valuation.sparse_computed_cells(),
            "current_computed_sparse_cells": current_valuation.sparse_computed_cells(),
            "same_grid_identity": delta.same_grid_identity,
            "previous_sparse_cells": delta.previous_sparse_cells,
            "current_sparse_cells": delta.current_sparse_cells,
            "previous_dense_region_entries": delta.previous_dense_region_entries,
            "current_dense_region_entries": delta.current_dense_region_entries,
            "previous_dense_cells": delta.previous_dense_cells,
            "current_dense_cells": delta.current_dense_cells,
            "previous_spill_fact_entries": delta.previous_spill_fact_entries,
            "current_spill_fact_entries": delta.current_spill_fact_entries,
            "sparse_entries_added": delta.sparse_entries_added,
            "sparse_entries_changed": delta.sparse_entries_changed,
            "sparse_entries_removed": delta.sparse_entries_removed,
            "dense_region_entries_added": delta.dense_region_entries_added,
            "dense_region_entries_changed": delta.dense_region_entries_changed,
            "dense_region_entries_removed": delta.dense_region_entries_removed,
            "dense_region_entries_unchanged": delta.dense_region_entries_unchanged,
            "dense_region_cells_changed": delta.dense_region_cells_changed,
            "spill_fact_entries_added": delta.spill_fact_entries_added,
            "spill_fact_entries_changed": delta.spill_fact_entries_changed,
            "spill_fact_entries_removed": delta.spill_fact_entries_removed,
            "publication_entries_total": publication_entries_total,
            "naive_current_computed_cell_publication_floor": delta.naive_current_computed_cell_publication_floor,
            "naive_full_grid_publication_floor": delta.naive_full_grid_publication_floor,
            "publication_entry_ratio_micros": publication_entry_ratio_micros,
            "previous_changed_input": calc_value_display_text(&previous_changed_input),
            "current_changed_input": calc_value_display_text(&current_changed_input),
            "previous_changed_formula": calc_value_display_text(&previous_changed_formula),
            "current_changed_formula": calc_value_display_text(&current_changed_formula)
        },
        "register_assertions": [
            register_assertion(
                "P-22",
                "one dense input edit plus one dependent repeated-formula output publishes compact region entries rather than all cells",
                delta.same_grid_identity
                    && publication_entries_total == 2
                    && delta.dense_region_entries_changed == 2
                    && delta.sparse_entries_added == 0
                    && delta.sparse_entries_changed == 0
                    && delta.sparse_entries_removed == 0
                    && delta.spill_fact_entries_added == 0
                    && delta.spill_fact_entries_changed == 0
                    && delta.spill_fact_entries_removed == 0
                    && u64::try_from(publication_entries_total).unwrap_or(u64::MAX)
                        < delta.naive_current_computed_cell_publication_floor
                    && publication_entry_ratio_micros <= 1_000
            ),
            register_assertion(
                "GRID-PUBLICATION-DELTA-1M",
                "the publication delta lane changes the edited dense input and dependent repeated-formula output while retaining dense computed storage",
                previous_recalc.computed_dense_value_regions == 2
                    && current_recalc.computed_dense_value_regions == 2
                    && previous_valuation.sparse_computed_cells() == 0
                    && current_valuation.sparse_computed_cells() == 0
                    && previous_changed_input != current_changed_input
                    && previous_changed_formula != current_changed_formula
            )
        ]
    }))
}

fn publication_delta_sheet(
    options: &GridScaleOptions,
    bounds: ExcelGridBounds,
    changed_row: Option<u32>,
) -> Result<GridOptimizedSheet, GridScaleRunnerError> {
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let input_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    sheet.put_dense_number_region_with(input_rect, |address| {
        let base = f64::from(address.row);
        if changed_row == Some(address.row) {
            base + 10_000.0
        } else {
            base
        }
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        2,
        options.rows,
        2,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;
    Ok(sheet)
}

fn tile_stream_64k_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const TILE_ROWS: u32 = 200;
    const TILE_COLS: u32 = 320;
    const UNRELATED_SPARSE_CELLS: u32 = 1_000;
    const MAX_FRAME_BYTES_PER_SUBSCRIBED_CELL: u64 = 64;
    if options.rows < TILE_ROWS.saturating_mul(2) {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "tile-stream-64k requires at least 400 rows".to_string(),
        });
    }
    if options.cols < TILE_COLS {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "tile-stream-64k requires at least 320 columns".to_string(),
        });
    }

    let tile_top = (options.rows / 2).saturating_sub(TILE_ROWS / 2).max(1);
    let tile_bottom = tile_top + TILE_ROWS - 1;
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let tile_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        tile_top,
        1,
        tile_bottom,
        TILE_COLS,
        bounds,
    )?;
    sheet.put_dense_number_region_with(tile_rect.clone(), |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    for offset in 0..UNRELATED_SPARSE_CELLS {
        let row = options.rows.saturating_sub(offset);
        if row < 1 || (tile_top <= row && row <= tile_bottom) {
            continue;
        }
        sheet.set_literal(
            address(row, options.cols),
            CalcValue::number(f64::from(row)),
        )?;
    }

    let materialization_limit = u64::from(TILE_ROWS).saturating_mul(u64::from(TILE_COLS));
    let (valuation, recalc) =
        sheet.recalculate_mark_all_dirty_compact_with_oxfml(materialization_limit)?;
    let report = valuation.tile_snapshot_report(tile_rect)?;
    let full_grid_to_tile_cell_ratio_micros =
        micros_ratio(report.subscribed_cell_count, report.full_grid_cell_floor);

    Ok(json!({
        "counters": {
            "tile_rows": TILE_ROWS,
            "tile_cols": TILE_COLS,
            "tile_top": tile_top,
            "tile_bottom": tile_bottom,
            "tile_subscribed_cells": report.subscribed_cell_count,
            "tile_defined_cells": report.defined_cell_count,
            "tile_blank_cells": report.blank_cell_count,
            "dense_value_cells_visited": report.dense_value_cells_visited,
            "sparse_value_cells_visited": report.sparse_value_cells_visited,
            "compact_regions_intersected": report.compact_regions_intersected,
            "estimated_value_payload_bytes": report.estimated_value_payload_bytes,
            "estimated_frame_bytes": report.estimated_frame_bytes,
            "frame_bytes_per_subscribed_cell_micros": report.frame_bytes_per_subscribed_cell_micros(),
            "max_frame_bytes_per_subscribed_cell": MAX_FRAME_BYTES_PER_SUBSCRIBED_CELL,
            "full_grid_cell_floor": report.full_grid_cell_floor,
            "full_grid_dense_numeric_bytes_floor": report.full_grid_dense_numeric_bytes_floor,
            "full_grid_to_tile_cell_ratio_micros": full_grid_to_tile_cell_ratio_micros,
            "unrelated_sparse_cells": valuation.sparse_computed_cells(),
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "occupied_cells": recalc.occupied_cells
        },
        "register_assertions": [
            register_assertion(
                "P-15",
                "tile-stream-64K frame bytes are bounded by subscribed cells and do not scale with grid capacity or unrelated sparse changes",
                report.p15_tile_streaming_holds(MAX_FRAME_BYTES_PER_SUBSCRIBED_CELL)
                    && report.subscribed_cell_count == u64::from(TILE_ROWS).saturating_mul(u64::from(TILE_COLS))
                    && report.estimated_frame_bytes < report.full_grid_dense_numeric_bytes_floor
                    && valuation.sparse_computed_cells() == usize::try_from(UNRELATED_SPARSE_CELLS).unwrap_or(usize::MAX)
                    && report.sparse_value_cells_visited == 0
            ),
            register_assertion(
                "GRID-TILE-STREAM-64K",
                "a 320x200 tile over a large grid visits the intersecting dense tile region and no unrelated sparse cells",
                report.defined_cell_count == usize::try_from(report.subscribed_cell_count).unwrap_or(usize::MAX)
                    && report.blank_cell_count == 0
                    && report.dense_value_cells_visited == report.subscribed_cell_count
                    && report.compact_regions_intersected == 1
                    && recalc.computed_dense_value_regions == 1
            )
        ]
    }))
}

fn viewport_64k_of_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const VISIBLE_ROWS: u32 = 64_000;
    const DENSE_COLS: u32 = 8;
    const FORMULA_COLS: u32 = 2;
    const VISIBLE_COL: u32 = DENSE_COLS + FORMULA_COLS;
    if options.rows < VISIBLE_ROWS {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "viewport-64k-of-1m requires at least 64,000 rows".to_string(),
        });
    }
    if options.cols < VISIBLE_COL {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "viewport-64k-of-1m requires at least 10 columns".to_string(),
        });
    }

    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let dense_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        DENSE_COLS,
        bounds,
    )?;
    sheet.put_dense_number_region_with(dense_rect, |address| {
        f64::from(address.row) * 1000.0 + f64::from(address.col)
    })?;
    let formula_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        DENSE_COLS + 1,
        options.rows,
        VISIBLE_COL,
        bounds,
    )?;
    sheet.put_repeated_formula_region(
        formula_rect,
        GridFormulaCell::new("=RC[-1]*2", "excel.grid.v1:r1c1-template:RC[-1]*2")
            .with_source_channel(FormulaChannelKind::WorksheetR1C1),
    )?;

    let visible_top = (options.rows / 2).saturating_sub(VISIBLE_ROWS / 2).max(1);
    let visible_bottom = visible_top + VISIBLE_ROWS - 1;
    let visible_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        visible_top,
        VISIBLE_COL,
        visible_bottom,
        VISIBLE_COL,
        bounds,
    )?;
    let materialization_limit = u64::from(VISIBLE_ROWS).saturating_mul(3);
    let (valuation, visible_report) = sheet
        .recalculate_visible_rect_compact_with_oxfml(visible_rect.clone(), materialization_limit)?;
    let snapshot = valuation.tile_snapshot_report(visible_rect.clone())?;

    let middle_row = visible_top + (VISIBLE_ROWS / 2);
    let top_value = valuation
        .read_cell(&address(visible_top, VISIBLE_COL))
        .computed;
    let middle_value = valuation
        .read_cell(&address(middle_row, VISIBLE_COL))
        .computed;
    let bottom_value = valuation
        .read_cell(&address(visible_bottom, VISIBLE_COL))
        .computed;
    let expected_bottom_value = (f64::from(visible_bottom) * 1000.0 + f64::from(DENSE_COLS)) * 4.0;
    let expected_bottom_display = integer_display(expected_bottom_value);

    Ok(json!({
        "counters": {
            "visible_rows": VISIBLE_ROWS,
            "visible_cols": 1,
            "visible_top": visible_top,
            "visible_bottom": visible_bottom,
            "visible_left": VISIBLE_COL,
            "visible_right": VISIBLE_COL,
            "visible_cell_count": visible_report.visible_cell_count,
            "visible_upstream_top": visible_report.upstream_rect.top_row,
            "visible_upstream_bottom": visible_report.upstream_rect.bottom_row,
            "visible_upstream_left": visible_report.upstream_rect.left_col,
            "visible_upstream_right": visible_report.upstream_rect.right_col,
            "visible_upstream_cell_count": visible_report.visible_upstream_cell_count,
            "cells_evaluated_before_visible_complete": visible_report.cells_evaluated_before_visible_complete,
            "formula_evaluations_before_visible_complete": visible_report.formula_evaluations_before_visible_complete,
            "dense_value_cells_projected": visible_report.dense_value_cells_projected,
            "repeated_formula_cells_projected": visible_report.repeated_formula_cells_projected,
            "sparse_point_cells_projected": visible_report.sparse_point_cells_projected,
            "computed_dense_value_regions": visible_report.computed_dense_value_regions,
            "computed_sparse_cells": visible_report.computed_sparse_cells,
            "full_recalc_occupied_cell_floor": visible_report.full_recalc_occupied_cell_floor,
            "full_grid_cell_floor": visible_report.full_grid_cell_floor,
            "visible_eval_to_full_occupied_ratio_micros": visible_report.evaluated_to_full_occupied_ratio_micros(),
            "upstream_to_full_occupied_ratio_micros": micros_ratio(visible_report.visible_upstream_cell_count, visible_report.full_recalc_occupied_cell_floor),
            "snapshot_subscribed_cells": snapshot.subscribed_cell_count,
            "snapshot_defined_cells": snapshot.defined_cell_count,
            "snapshot_dense_value_cells_visited": snapshot.dense_value_cells_visited,
            "snapshot_sparse_value_cells_visited": snapshot.sparse_value_cells_visited,
            "snapshot_estimated_frame_bytes": snapshot.estimated_frame_bytes,
            "top_visible_value": calc_value_display_text(&top_value),
            "middle_visible_value": calc_value_display_text(&middle_value),
            "bottom_visible_value": calc_value_display_text(&bottom_value),
            "expected_bottom_visible_value": expected_bottom_display.clone()
        },
        "register_assertions": [
            register_assertion(
                "P-16",
                "viewport-64K visible-first recalc evaluates only the visible same-row upstream cone before the viewport is clean",
                visible_report.p16_visible_first_holds()
                    && visible_report.visible_cell_count == u64::from(VISIBLE_ROWS)
                    && visible_report.visible_upstream_cell_count == u64::from(VISIBLE_ROWS).saturating_mul(3)
                    && visible_report.cells_evaluated_before_visible_complete == visible_report.visible_upstream_cell_count
                    && visible_report.cells_evaluated_before_visible_complete < visible_report.full_recalc_occupied_cell_floor
                    && visible_report.computed_sparse_cells == 0
            ),
            register_assertion(
                "GRID-VIEWPORT-64K",
                "viewport-64K produces the visible formula column from compact dense and repeated-R1C1 regions",
                visible_report.dense_value_cells_projected == u64::from(VISIBLE_ROWS)
                    && visible_report.repeated_formula_cells_projected == u64::from(VISIBLE_ROWS).saturating_mul(2)
                    && visible_report.formula_evaluations_before_visible_complete == u64::from(VISIBLE_ROWS).saturating_mul(2)
                    && snapshot.subscribed_cell_count == u64::from(VISIBLE_ROWS)
                    && snapshot.defined_cell_count == usize::try_from(VISIBLE_ROWS).unwrap_or(usize::MAX)
                    && snapshot.dense_value_cells_visited == u64::from(VISIBLE_ROWS)
                    && snapshot.sparse_value_cells_visited == 0
                    && calc_value_display_text(&bottom_value) == expected_bottom_display
            )
        ]
    }))
}

fn range_invalidation_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const SCALARIZATION_LIMIT: u64 = 1_024;
    if u64::from(options.rows) <= SCALARIZATION_LIMIT {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "range-invalidation-1m requires rows above the scalarization limit".to_string(),
        });
    }
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "range-invalidation-1m requires at least 3 columns".to_string(),
        });
    }

    let range = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    let dependent = address(1, 2);
    let downstream = address(1, 3);
    let seed = address((options.rows / 2).max(1), 1);
    let mut invalidation =
        GridInvalidationRef::with_scalarization_limit(bounds, SCALARIZATION_LIMIT);
    let installed_range_scalar_edges = invalidation
        .set_cell_dependencies(dependent.clone(), [GridDependency::Range(range.clone())])?;
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(dependent.clone())],
    )?;
    let dirty_closure = invalidation.dirty_closure([seed.clone()]);
    let compressed_range_dependencies_for_dependent = invalidation
        .compressed_range_dependencies_for(&dependent)
        .len();
    let expanded_scalar_edge_floor = range.cell_count();
    let compressed_range_edges = invalidation.compressed_range_edge_count();
    let scalar_edges = invalidation.scalar_edge_count();
    let semantic_dependencies = invalidation.semantic_dependencies_for(&dependent).len();
    let compressed_support_edge_ratio_micros = micros_ratio(
        u64::try_from(compressed_range_edges).unwrap_or(u64::MAX),
        expanded_scalar_edge_floor,
    );

    Ok(json!({
        "counters": {
            "dependency_rows": options.rows,
            "dependency_cols": 1,
            "scalarization_limit": SCALARIZATION_LIMIT,
            "expanded_scalar_edge_floor": expanded_scalar_edge_floor,
            "installed_range_scalar_edges": installed_range_scalar_edges,
            "scalar_edges_total": scalar_edges,
            "compressed_range_edges": compressed_range_edges,
            "compressed_range_dependencies_for_dependent": compressed_range_dependencies_for_dependent,
            "semantic_dependencies_for_dependent": semantic_dependencies,
            "compressed_support_edge_ratio_micros": compressed_support_edge_ratio_micros,
            "dirty_seed_row": seed.row,
            "dirty_closure_size": dirty_closure.len(),
            "dirty_closure_contains_dependent": dirty_closure.contains(&dependent),
            "dirty_closure_contains_downstream": dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-13",
                "finite range invalidation keeps one compressed reverse edge instead of expanding a 1M-row range",
                compressed_range_edges == 1
                    && installed_range_scalar_edges == 0
                    && scalar_edges == 1
                    && compressed_range_dependencies_for_dependent == 1
                    && expanded_scalar_edge_floor == u64::from(options.rows)
                    && compressed_support_edge_ratio_micros < 1_000_000
                    && dirty_closure.contains(&dependent)
                    && dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-RANGE-INVALIDATION-1M",
                "a cell edit inside a compressed 1M-row range reaches the range formula and its downstream dependent",
                dirty_closure.len() == 3
                    && dirty_closure.contains(&seed)
                    && dirty_closure.contains(&dependent)
                    && dirty_closure.contains(&downstream)
            )
        ]
    }))
}

fn range_query_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const SCALARIZATION_LIMIT: u64 = 64;
    const RANGE_COUNT: u32 = 1_000;
    if options.rows < RANGE_COUNT {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "range-query-1m requires at least 1000 rows".to_string(),
        });
    }
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "range-query-1m requires at least 3 columns".to_string(),
        });
    }

    let range_height = options.rows / RANGE_COUNT;
    let covered_rows = range_height * RANGE_COUNT;
    let selected_range_index = RANGE_COUNT / 2;
    let selected_start = selected_range_index * range_height + 1;
    let selected_seed_row = selected_start + (range_height / 2).min(range_height - 1);
    let selected_dependent = address(selected_range_index + 1, 2);
    let downstream = address(1, 3);
    let seed = address(selected_seed_row, 1);

    let mut invalidation =
        GridInvalidationRef::with_scalarization_limit(bounds, SCALARIZATION_LIMIT);
    for range_index in 0..RANGE_COUNT {
        let start_row = range_index * range_height + 1;
        let end_row = start_row + range_height - 1;
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            1,
            end_row,
            1,
            bounds,
        )?;
        invalidation
            .set_cell_dependencies(address(range_index + 1, 2), [GridDependency::Range(range)])?;
    }
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(selected_dependent.clone())],
    )?;

    let query = invalidation.compressed_range_query_report(seed.clone())?;
    let dirty_closure = invalidation.dirty_closure([seed.clone()]);
    let total_compressed_range_edges = invalidation.compressed_range_edge_count();
    let naive_candidate_floor = total_compressed_range_edges;
    let indexed_candidate_ratio_micros = micros_ratio(
        u64::try_from(query.indexed_candidate_count).unwrap_or(u64::MAX),
        u64::try_from(naive_candidate_floor).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "range_count": RANGE_COUNT,
            "range_height": range_height,
            "covered_rows": covered_rows,
            "scalarization_limit": SCALARIZATION_LIMIT,
            "total_compressed_range_edges": total_compressed_range_edges,
            "naive_candidate_floor": naive_candidate_floor,
            "indexed_candidate_count": query.indexed_candidate_count,
            "matched_dependent_count": query.matched_dependent_count,
            "indexed_candidate_ratio_micros": indexed_candidate_ratio_micros,
            "dirty_seed_row": seed.row,
            "dirty_closure_size": dirty_closure.len(),
            "dirty_closure_contains_selected_dependent": dirty_closure.contains(&selected_dependent),
            "dirty_closure_contains_downstream": dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-12",
                "compressed range invalidation uses the block interval index instead of scanning every range edge",
                total_compressed_range_edges == usize::try_from(RANGE_COUNT).unwrap_or(usize::MAX)
                    && query.indexed_candidate_count < naive_candidate_floor
                    && query.indexed_candidate_count <= 4
                    && query.matched_dependent_count == 1
                    && indexed_candidate_ratio_micros <= 4_000
                    && dirty_closure.contains(&selected_dependent)
                    && dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-RANGE-QUERY-1M",
                "a seed inside one of 1000 compressed ranges dirties only the matching range chain",
                dirty_closure.len() == 3
                    && dirty_closure.contains(&seed)
                    && dirty_closure.contains(&selected_dependent)
                    && dirty_closure.contains(&downstream)
            )
        ]
    }))
}

fn sum_pyramid_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const SCALARIZATION_LIMIT: u64 = 4;
    const LEVEL1_COUNT: u32 = 1_000;
    const GROUP_SIZE: u32 = 10;
    const LEVEL2_COUNT: u32 = LEVEL1_COUNT / GROUP_SIZE;
    const LEVEL3_COUNT: u32 = LEVEL2_COUNT / GROUP_SIZE;
    if options.rows < LEVEL1_COUNT * (u32::try_from(SCALARIZATION_LIMIT).unwrap() + 1) {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sum-pyramid-1m requires enough rows for compressed leaf ranges".to_string(),
        });
    }
    if options.cols < 6 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sum-pyramid-1m requires at least 6 columns".to_string(),
        });
    }

    let level1_height = options.rows / LEVEL1_COUNT;
    let covered_rows = level1_height * LEVEL1_COUNT;
    let mut invalidation =
        GridInvalidationRef::with_scalarization_limit(bounds, SCALARIZATION_LIMIT);
    let mut expanded_range_edge_floor = 0_u64;
    let mut installed_range_scalar_edges = 0_usize;

    for level1_index in 0..LEVEL1_COUNT {
        let start_row = sum_pyramid_row_for_level1(level1_index, level1_height);
        let end_row = start_row + level1_height - 1;
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            1,
            end_row,
            1,
            bounds,
        )?;
        expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(range.cell_count());
        installed_range_scalar_edges = installed_range_scalar_edges.saturating_add(
            invalidation
                .set_cell_dependencies(address(start_row, 2), [GridDependency::Range(range)])?,
        );
    }

    for level2_index in 0..LEVEL2_COUNT {
        let first_level1_index = level2_index * GROUP_SIZE;
        let last_level1_index = first_level1_index + GROUP_SIZE - 1;
        let start_row = sum_pyramid_row_for_level1(first_level1_index, level1_height);
        let end_row = sum_pyramid_row_for_level1(last_level1_index, level1_height);
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            2,
            end_row,
            2,
            bounds,
        )?;
        expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(range.cell_count());
        installed_range_scalar_edges = installed_range_scalar_edges.saturating_add(
            invalidation
                .set_cell_dependencies(address(start_row, 3), [GridDependency::Range(range)])?,
        );
    }

    for level3_index in 0..LEVEL3_COUNT {
        let first_level2_index = level3_index * GROUP_SIZE;
        let last_level2_index = first_level2_index + GROUP_SIZE - 1;
        let start_row = sum_pyramid_row_for_level1(first_level2_index * GROUP_SIZE, level1_height);
        let end_row = sum_pyramid_row_for_level1(last_level2_index * GROUP_SIZE, level1_height);
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            3,
            end_row,
            3,
            bounds,
        )?;
        expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(range.cell_count());
        installed_range_scalar_edges = installed_range_scalar_edges.saturating_add(
            invalidation
                .set_cell_dependencies(address(start_row, 4), [GridDependency::Range(range)])?,
        );
    }

    let final_dependent = address(1, 5);
    let final_range_end_row =
        sum_pyramid_row_for_level1((LEVEL3_COUNT - 1) * GROUP_SIZE * GROUP_SIZE, level1_height);
    let final_range = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        4,
        final_range_end_row,
        4,
        bounds,
    )?;
    expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(final_range.cell_count());
    installed_range_scalar_edges =
        installed_range_scalar_edges.saturating_add(invalidation.set_cell_dependencies(
            final_dependent.clone(),
            [GridDependency::Range(final_range)],
        )?);
    let downstream = address(1, 6);
    let downstream_scalar_edges = invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(final_dependent.clone())],
    )?;

    let selected_level1_index = LEVEL1_COUNT / 2;
    let selected_level1_row = sum_pyramid_row_for_level1(selected_level1_index, level1_height);
    let selected_seed_row = selected_level1_row + (level1_height / 2).min(level1_height - 1);
    let seed = address(selected_seed_row, 1);
    let level1_dependent = address(selected_level1_row, 2);
    let selected_level2_index = selected_level1_index / GROUP_SIZE;
    let selected_level2_row =
        sum_pyramid_row_for_level1(selected_level2_index * GROUP_SIZE, level1_height);
    let level2_dependent = address(selected_level2_row, 3);
    let selected_level3_index = selected_level2_index / GROUP_SIZE;
    let selected_level3_row = sum_pyramid_row_for_level1(
        selected_level3_index * GROUP_SIZE * GROUP_SIZE,
        level1_height,
    );
    let level3_dependent = address(selected_level3_row, 4);

    let leaf_query = invalidation.compressed_range_query_report(seed.clone())?;
    let level1_query = invalidation.compressed_range_query_report(level1_dependent.clone())?;
    let level2_query = invalidation.compressed_range_query_report(level2_dependent.clone())?;
    let level3_query = invalidation.compressed_range_query_report(level3_dependent.clone())?;
    let indexed_candidate_sum = leaf_query
        .indexed_candidate_count
        .saturating_add(level1_query.indexed_candidate_count)
        .saturating_add(level2_query.indexed_candidate_count)
        .saturating_add(level3_query.indexed_candidate_count);
    let matched_dependent_sum = leaf_query
        .matched_dependent_count
        .saturating_add(level1_query.matched_dependent_count)
        .saturating_add(level2_query.matched_dependent_count)
        .saturating_add(level3_query.matched_dependent_count);
    let dirty_closure = invalidation.dirty_closure([seed.clone()]);
    let total_compressed_range_edges = invalidation.compressed_range_edge_count();
    let compressed_support_edge_ratio_micros = micros_ratio(
        u64::try_from(total_compressed_range_edges).unwrap_or(u64::MAX),
        expanded_range_edge_floor,
    );
    let indexed_candidate_ratio_micros = micros_ratio(
        u64::try_from(indexed_candidate_sum).unwrap_or(u64::MAX),
        u64::try_from(total_compressed_range_edges).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "level1_count": LEVEL1_COUNT,
            "level2_count": LEVEL2_COUNT,
            "level3_count": LEVEL3_COUNT,
            "level1_height": level1_height,
            "covered_rows": covered_rows,
            "scalarization_limit": SCALARIZATION_LIMIT,
            "expanded_range_edge_floor": expanded_range_edge_floor,
            "installed_range_scalar_edges": installed_range_scalar_edges,
            "downstream_scalar_edges": downstream_scalar_edges,
            "scalar_edges_total": invalidation.scalar_edge_count(),
            "total_compressed_range_edges": total_compressed_range_edges,
            "compressed_support_edge_ratio_micros": compressed_support_edge_ratio_micros,
            "leaf_indexed_candidate_count": leaf_query.indexed_candidate_count,
            "level1_indexed_candidate_count": level1_query.indexed_candidate_count,
            "level2_indexed_candidate_count": level2_query.indexed_candidate_count,
            "level3_indexed_candidate_count": level3_query.indexed_candidate_count,
            "indexed_candidate_sum": indexed_candidate_sum,
            "indexed_candidate_ratio_micros": indexed_candidate_ratio_micros,
            "matched_dependent_sum": matched_dependent_sum,
            "dirty_seed_row": seed.row,
            "dirty_closure_size": dirty_closure.len(),
            "dirty_closure_contains_level1": dirty_closure.contains(&level1_dependent),
            "dirty_closure_contains_level2": dirty_closure.contains(&level2_dependent),
            "dirty_closure_contains_level3": dirty_closure.contains(&level3_dependent),
            "dirty_closure_contains_final": dirty_closure.contains(&final_dependent),
            "dirty_closure_contains_downstream": dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-12",
                "sum-pyramid-1M compressed range queries use the block index across every aggregation level",
                total_compressed_range_edges == usize::try_from(LEVEL1_COUNT + LEVEL2_COUNT + LEVEL3_COUNT + 1).unwrap_or(usize::MAX)
                    && indexed_candidate_sum < total_compressed_range_edges
                    && indexed_candidate_ratio_micros <= 250_000
                    && matched_dependent_sum == 4
                    && dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "P-13",
                "sum-pyramid-1M keeps pyramid range support compressed instead of expanding range cells",
                installed_range_scalar_edges == 0
                    && downstream_scalar_edges == 1
                    && invalidation.scalar_edge_count() == 1
                    && expanded_range_edge_floor > u64::try_from(total_compressed_range_edges).unwrap_or(u64::MAX)
                    && compressed_support_edge_ratio_micros < 1_000_000
            ),
            register_assertion(
                "GRID-SUM-PYRAMID-1M",
                "a leaf edit in the compressed sum pyramid dirties exactly the selected aggregation chain and downstream dependent",
                dirty_closure.len() == 6
                    && dirty_closure.contains(&seed)
                    && dirty_closure.contains(&level1_dependent)
                    && dirty_closure.contains(&level2_dependent)
                    && dirty_closure.contains(&level3_dependent)
                    && dirty_closure.contains(&final_dependent)
                    && dirty_closure.contains(&downstream)
            )
        ]
    }))
}

fn dirty_rect_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const SCALARIZATION_LIMIT: u64 = 4;
    const DEPENDENCY_COUNT: u32 = 1_000;
    if options.rows < DEPENDENCY_COUNT * (u32::try_from(SCALARIZATION_LIMIT).unwrap() + 1) {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "dirty-rect-1m requires enough rows for compressed range dependencies"
                .to_string(),
        });
    }
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "dirty-rect-1m requires at least 4 columns".to_string(),
        });
    }

    let range_height = options.rows / DEPENDENCY_COUNT;
    let selected_range_index = DEPENDENCY_COUNT / 2;
    let selected_start_row = selected_range_index * range_height + 1;
    let selected_scalar_dependency_row =
        selected_start_row + (range_height / 2).min(range_height - 1);
    let mut invalidation =
        GridInvalidationRef::with_scalarization_limit(bounds, SCALARIZATION_LIMIT);
    let mut expanded_range_edge_floor = 0_u64;
    let mut installed_range_scalar_edges = 0_usize;

    for range_index in 0..DEPENDENCY_COUNT {
        let start_row = range_index * range_height + 1;
        let end_row = start_row + range_height - 1;
        let range_dependent = address(range_index + 1, 2);
        let scalar_dependent = address(range_index + 1, 3);
        let scalar_dependency_row = start_row + (range_height / 2).min(range_height - 1);
        let range = GridRect::new(
            "book:grid-scale",
            "sheet:grid-scale",
            start_row,
            1,
            end_row,
            1,
            bounds,
        )?;
        expanded_range_edge_floor = expanded_range_edge_floor.saturating_add(range.cell_count());
        installed_range_scalar_edges = installed_range_scalar_edges.saturating_add(
            invalidation.set_cell_dependencies(range_dependent, [GridDependency::Range(range)])?,
        );
        invalidation.set_cell_dependencies(
            scalar_dependent,
            [GridDependency::Cell(address(scalar_dependency_row, 1))],
        )?;
    }

    let selected_range_dependent = address(selected_range_index + 1, 2);
    let selected_scalar_dependent = address(selected_range_index + 1, 3);
    let downstream = address(1, 4);
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [
            GridDependency::Cell(selected_range_dependent.clone()),
            GridDependency::Cell(selected_scalar_dependent.clone()),
        ],
    )?;

    let dirty_rect_top = selected_scalar_dependency_row.saturating_sub(5).max(1);
    let dirty_rect_bottom = selected_scalar_dependency_row
        .saturating_add(5)
        .min(options.rows);
    let dirty_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        dirty_rect_top,
        1,
        dirty_rect_bottom,
        1,
        bounds,
    )?;
    let report = invalidation.dirty_rect_query_report(dirty_rect)?;
    let indexed_candidate_sum = report
        .indexed_scalar_candidate_count
        .saturating_add(report.indexed_compressed_range_candidate_count);
    let total_edge_count = report
        .total_scalar_edges
        .saturating_add(report.total_compressed_range_edges);
    let indexed_candidate_ratio_micros = micros_ratio(
        u64::try_from(indexed_candidate_sum).unwrap_or(u64::MAX),
        u64::try_from(total_edge_count).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "dependency_count": DEPENDENCY_COUNT,
            "range_height": range_height,
            "dirty_rect_top": dirty_rect_top,
            "dirty_rect_bottom": dirty_rect_bottom,
            "dirty_rect_cell_count": report.seed_rect_cell_count,
            "selected_scalar_dependency_row": selected_scalar_dependency_row,
            "expanded_range_edge_floor": expanded_range_edge_floor,
            "installed_range_scalar_edges": installed_range_scalar_edges,
            "total_scalar_edges": report.total_scalar_edges,
            "total_compressed_range_edges": report.total_compressed_range_edges,
            "total_edge_count": total_edge_count,
            "indexed_scalar_candidate_count": report.indexed_scalar_candidate_count,
            "matched_scalar_dependent_count": report.matched_scalar_dependent_count,
            "indexed_compressed_range_candidate_count": report.indexed_compressed_range_candidate_count,
            "matched_compressed_range_dependent_count": report.matched_compressed_range_dependent_count,
            "indexed_candidate_sum": indexed_candidate_sum,
            "indexed_candidate_ratio_micros": indexed_candidate_ratio_micros,
            "direct_dependent_count": report.direct_dependents.len(),
            "dirty_closure_size": report.dirty_closure.len(),
            "dirty_closure_contains_range_dependent": report.dirty_closure.contains(&selected_range_dependent),
            "dirty_closure_contains_scalar_dependent": report.dirty_closure.contains(&selected_scalar_dependent),
            "dirty_closure_contains_downstream": report.dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-12",
                "dirty-rect-1M queries scalar and compressed range consumers through block indexes without expanding the rectangle or scanning all edges",
                report.seed_rect_cell_count == 11
                    && installed_range_scalar_edges == 0
                    && report.indexed_scalar_candidate_count < report.total_scalar_edges
                    && report.indexed_compressed_range_candidate_count < report.total_compressed_range_edges
                    && report.matched_scalar_dependent_count == 1
                    && report.matched_compressed_range_dependent_count == 1
                    && indexed_candidate_ratio_micros <= 10_000
                    && report.dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-DIRTY-RECT-1M",
                "an 11-cell dirty rectangle dirties the selected range consumer, scalar consumer, and downstream dependent",
                report.direct_dependents.len() == 2
                    && report.dirty_closure.len() == 3
                    && report.dirty_closure.contains(&selected_range_dependent)
                    && report.dirty_closure.contains(&selected_scalar_dependent)
                    && report.dirty_closure.contains(&downstream)
            )
        ]
    }))
}

fn hide_storm_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    const VISIBILITY_DEPENDENCY_COUNT: u32 = 1_000;
    if options.rows < VISIBILITY_DEPENDENCY_COUNT {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "hide-storm-1m requires at least 1000 rows".to_string(),
        });
    }
    if options.cols < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "hide-storm-1m requires at least 3 columns".to_string(),
        });
    }

    let range_height = options.rows / VISIBILITY_DEPENDENCY_COUNT;
    let covered_rows = range_height * VISIBILITY_DEPENDENCY_COUNT;
    let selected_range_index = VISIBILITY_DEPENDENCY_COUNT / 2;
    let selected_start = selected_range_index * range_height + 1;
    let selected_seed_row = selected_start + (range_height / 2).min(range_height - 1);
    let selected_dependent = address(selected_range_index + 1, 2);
    let downstream = address(1, 3);
    let seed_dependency = GridAxisVisibilityDependency::rows(selected_seed_row, selected_seed_row);

    let mut invalidation = GridInvalidationRef::new(bounds);
    for range_index in 0..VISIBILITY_DEPENDENCY_COUNT {
        let start_row = range_index * range_height + 1;
        let end_row = start_row + range_height - 1;
        invalidation.set_cell_dependencies(
            address(range_index + 1, 2),
            [GridDependency::AxisVisibility(
                GridAxisVisibilityDependency::rows(start_row, end_row),
            )],
        )?;
    }
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(selected_dependent.clone())],
    )?;

    let query = invalidation.axis_visibility_query_report(seed_dependency.clone())?;
    let dirty_closure = invalidation.dirty_closure_for_axis_visibility(seed_dependency.clone())?;
    let total_axis_visibility_edges = invalidation.axis_visibility_edge_count();
    let naive_candidate_floor = total_axis_visibility_edges;
    let indexed_candidate_ratio_micros = micros_ratio(
        u64::try_from(query.indexed_candidate_count).unwrap_or(u64::MAX),
        u64::try_from(naive_candidate_floor).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "visibility_dependency_count": VISIBILITY_DEPENDENCY_COUNT,
            "visibility_range_height": range_height,
            "covered_rows": covered_rows,
            "total_axis_visibility_edges": total_axis_visibility_edges,
            "naive_candidate_floor": naive_candidate_floor,
            "indexed_candidate_count": query.indexed_candidate_count,
            "matched_dependent_count": query.matched_dependent_count,
            "indexed_candidate_ratio_micros": indexed_candidate_ratio_micros,
            "visibility_seed_row": selected_seed_row,
            "dirty_closure_size": dirty_closure.len(),
            "dirty_closure_contains_selected_dependent": dirty_closure.contains(&selected_dependent),
            "dirty_closure_contains_downstream": dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-24",
                "hidden-row visibility invalidation uses the block index instead of scanning every hidden-sensitive range",
                total_axis_visibility_edges
                    == usize::try_from(VISIBILITY_DEPENDENCY_COUNT).unwrap_or(usize::MAX)
                    && query.indexed_candidate_count < naive_candidate_floor
                    && query.indexed_candidate_count <= 4
                    && query.matched_dependent_count == 1
                    && indexed_candidate_ratio_micros <= 4_000
                    && dirty_closure.contains(&selected_dependent)
                    && dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-HIDE-STORM-1M",
                "a hidden-row change inside one of 1000 hidden-sensitive row bands dirties only the matching aggregate chain",
                dirty_closure.len() == 2
                    && dirty_closure.contains(&selected_dependent)
                    && dirty_closure.contains(&downstream)
            )
        ]
    }))
}

fn aggregate_context_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 3 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "aggregate-context-1m requires at least 3 rows".to_string(),
        });
    }
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "aggregate-context-1m requires at least 2 columns".to_string(),
        });
    }

    let manual_hidden_row = (options.rows / 3).max(2);
    let mut filtered_hidden_row = ((options.rows.saturating_mul(2)) / 3).max(3);
    if filtered_hidden_row == manual_hidden_row {
        filtered_hidden_row = filtered_hidden_row.saturating_add(1).min(options.rows);
    }

    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.axis_state_mut().set_row(
        manual_hidden_row,
        GridAxisProps {
            hidden_manual: true,
            ..GridAxisProps::visible()
        },
    );
    sheet.axis_state_mut().set_row(
        filtered_hidden_row,
        GridAxisProps {
            hidden_filter: true,
            ..GridAxisProps::visible()
        },
    );
    let provider = sheet.host_info_provider(1, 2);
    let reference = ReferenceLike::new(ReferenceKind::Area, format!("A1:A{}", options.rows));
    let report = provider
        .aggregate_context_query_report(&reference)
        .map_err(|source| GridScaleRunnerError::InvalidOptions {
            detail: format!("aggregate-context provider report failed: {source:?}"),
        })?;
    let axis_run_ratio_micros = micros_ratio(
        u64::try_from(report.axis_run_probe_count).unwrap_or(u64::MAX),
        u64::try_from(report.declared_cell_count).unwrap_or(u64::MAX),
    );

    Ok(json!({
        "counters": {
            "aggregate_reference_declared_cells": report.declared_cell_count,
            "aggregate_reference_rows": report.rows,
            "aggregate_reference_cols": report.cols,
            "manual_hidden_row": manual_hidden_row,
            "filtered_hidden_row": filtered_hidden_row,
            "explicit_axis_row_entries_visited": report.explicit_axis_row_entries_visited,
            "default_row_runs": report.default_row_runs,
            "row_context_runs": report.row_context_runs,
            "axis_run_probe_count": report.axis_run_probe_count,
            "axis_run_probe_ratio_micros": axis_run_ratio_micros,
            "per_cell_context_expansion_count": report.per_cell_context_expansion_count,
            "manually_hidden_rows": report.manually_hidden_rows,
            "filtered_hidden_rows": report.filtered_hidden_rows
        },
        "register_assertions": [
            register_assertion(
                "P-28",
                "aggregate host context provider probes AxisState row runs instead of every referenced cell",
                report.declared_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && report.explicit_axis_row_entries_visited == 2
                    && report.default_row_runs == 3
                    && report.axis_run_probe_count == 5
                    && report.axis_run_probe_count < report.declared_cell_count
                    && report.per_cell_context_expansion_count == report.declared_cell_count
                    && report.manually_hidden_rows == 1
                    && report.filtered_hidden_rows == 1
            ),
            register_assertion(
                "GRID-AGGREGATE-CONTEXT-1M",
                "a 1M-row SUBTOTAL-style context query reads five row-context runs and records the current per-cell seam expansion",
                report.rows == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && report.cols == 1
                    && report.axis_run_probe_count == 5
                    && axis_run_ratio_micros <= 5
            )
        ]
    }))
}

fn spill_anchor_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "spill-anchor-1m requires at least 2 columns".to_string(),
        });
    }

    let middle_row = (options.rows / 2).max(1);
    let provider =
        ExcelGridReferenceSystemProvider::new("book:grid-scale", "sheet:grid-scale", 1, 2)
            .with_bounds(bounds)
            .with_cell_value(
                "book:grid-scale",
                "sheet:grid-scale",
                1,
                1,
                CalcValue::number(1.0),
            )
            .with_cell_value(
                "book:grid-scale",
                "sheet:grid-scale",
                middle_row,
                1,
                CalcValue::number(f64::from(middle_row)),
            )
            .with_cell_value(
                "book:grid-scale",
                "sheet:grid-scale",
                options.rows,
                1,
                CalcValue::number(f64::from(options.rows)),
            )
            .with_spill_extent(
                "book:grid-scale",
                "sheet:grid-scale",
                1,
                1,
                ExcelGridResolvedRect {
                    workbook_id: "book:grid-scale".to_string(),
                    sheet_id: "sheet:grid-scale".to_string(),
                    top_row: 1,
                    left_col: 1,
                    bottom_row: options.rows,
                    right_col: 1,
                },
            );
    let report = provider
        .spill_anchor_dereference_report(&ReferenceLike::new(ReferenceKind::SpillAnchor, "A1#"))
        .map_err(|source| GridScaleRunnerError::InvalidOptions {
            detail: format!("spill-anchor provider report failed: {source:?}"),
        })?;

    Ok(json!({
        "counters": {
            "spill_extent_declared_cells": report.declared_cell_count,
            "spill_ledger_probe_count": report.ledger_probe_count,
            "spill_extent_cells_scanned_for_ledger": report.extent_cells_scanned_for_ledger,
            "provider_value_entries_scanned": report.value_entries_scanned,
            "defined_cells_returned": report.defined_cells_returned,
            "anchor_row": report.anchor.row,
            "anchor_col": report.anchor.col
        },
        "register_assertions": [
            register_assertion(
                "P-25",
                "A1# resolves its spill extent with one ledger probe and no extent scan",
                report.declared_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && report.ledger_probe_count == 1
                    && report.extent_cells_scanned_for_ledger == 0
                    && report.value_entries_scanned == 3
                    && report.defined_cells_returned == 3
            ),
            register_assertion(
                "GRID-SPILL-ANCHOR-1M",
                "1M-row A1# provider report remains occupancy-proportional for stored values",
                report.declared_cell_count == usize::try_from(options.rows).unwrap_or(usize::MAX)
                    && report.value_entries_scanned < report.declared_cell_count
                    && report.anchor == address(1, 1)
            )
        ]
    }))
}

fn spill_epoch_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "spill-epoch-1m requires at least 4 rows".to_string(),
        });
    }
    if options.cols < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "spill-epoch-1m requires at least 4 columns".to_string(),
        });
    }

    let a1_anchor = address(1, 1);
    let unrelated_anchor = address(2, 2);
    let a1_consumer = address(1, 3);
    let downstream = address(1, 4);
    let a1_extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    let a1_shrunk_extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows - 1,
        1,
        bounds,
    )?;
    let unrelated_extent =
        GridRect::new("book:grid-scale", "sheet:grid-scale", 2, 2, 3, 2, bounds)?;
    let spill_facts = BTreeMap::from([
        (
            a1_anchor.clone(),
            GridSpillFact {
                anchor: a1_anchor.clone(),
                extent: a1_extent.clone(),
                blocked: false,
            },
        ),
        (
            unrelated_anchor.clone(),
            GridSpillFact {
                anchor: unrelated_anchor.clone(),
                extent: unrelated_extent.clone(),
                blocked: false,
            },
        ),
    ]);
    let mut base_ledger = GridSpillEpochLedger::default();
    let base_ledger_update = base_ledger.update_from_spill_facts(&spill_facts, |fact| {
        if fact.anchor == a1_anchor {
            "a1:v1".to_string()
        } else {
            "unrelated:v1".to_string()
        }
    });
    let base_snapshots = base_ledger.snapshots();

    let mut unchanged_ledger = base_ledger.clone();
    let unchanged_ledger_update = unchanged_ledger.update_from_spill_facts(&spill_facts, |fact| {
        if fact.anchor == a1_anchor {
            "a1:v1".to_string()
        } else {
            "unrelated:v1".to_string()
        }
    });

    let mut unrelated_churn_ledger = base_ledger.clone();
    let unrelated_churn_ledger_update =
        unrelated_churn_ledger.update_from_spill_facts(&spill_facts, |fact| {
            if fact.anchor == a1_anchor {
                "a1:v1".to_string()
            } else {
                "unrelated:v2".to_string()
            }
        });

    let mut a1_only_facts = BTreeMap::new();
    a1_only_facts.insert(
        a1_anchor.clone(),
        spill_facts.get(&a1_anchor).cloned().unwrap(),
    );
    let mut a1_only_ledger = GridSpillEpochLedger::default();
    a1_only_ledger.update_from_spill_facts(&a1_only_facts, |_| "a1:v1".to_string());
    let a1_only_snapshots = a1_only_ledger.snapshots();

    let mut value_change_ledger = a1_only_ledger.clone();
    let value_change_ledger_update =
        value_change_ledger.update_from_spill_facts(&a1_only_facts, |_| "a1:v2".to_string());

    let mut shrunk_facts = BTreeMap::new();
    shrunk_facts.insert(
        a1_anchor.clone(),
        GridSpillFact {
            anchor: a1_anchor.clone(),
            extent: a1_shrunk_extent,
            blocked: false,
        },
    );
    let mut extent_change_ledger = a1_only_ledger.clone();
    let extent_change_ledger_update =
        extent_change_ledger.update_from_spill_facts(&shrunk_facts, |_| "a1:v1".to_string());

    let mut invalidation = GridInvalidationRef::new(bounds);
    invalidation.set_cell_dependencies(
        a1_consumer.clone(),
        [GridDependency::SpillFact(GridSpillDependency::anchor(
            a1_anchor.clone(),
        ))],
    )?;
    invalidation.set_cell_dependencies(
        downstream.clone(),
        [GridDependency::Cell(a1_consumer.clone())],
    )?;

    let unchanged = invalidation.dirty_closure_for_spill_epoch_changes(
        base_snapshots.values().cloned(),
        unchanged_ledger.snapshots().values().cloned(),
    )?;
    let unrelated_churn = invalidation.dirty_closure_for_spill_epoch_changes(
        base_snapshots.values().cloned(),
        unrelated_churn_ledger.snapshots().values().cloned(),
    )?;
    let value_change = invalidation.dirty_closure_for_spill_epoch_changes(
        a1_only_snapshots.values().cloned(),
        value_change_ledger.snapshots().values().cloned(),
    )?;
    let extent_change = invalidation.dirty_closure_for_spill_epoch_changes(
        a1_only_snapshots.values().cloned(),
        extent_change_ledger.snapshots().values().cloned(),
    )?;

    let mut optimized_commit_sheet =
        GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let mut optimized_commit_valuation =
        GridOptimizedValuation::new("book:grid-scale", "sheet:grid-scale", bounds);
    optimized_commit_valuation.set_spill_fact(spill_facts.get(&a1_anchor).cloned().ok_or_else(
        || GridScaleRunnerError::InvalidOptions {
            detail: "spill-epoch commit setup missing A1 fact".to_string(),
        },
    )?)?;
    let optimized_commit_first = optimized_commit_sheet
        .commit_spill_publication_from_valuation(&optimized_commit_valuation)?;
    let optimized_commit_second = optimized_commit_sheet
        .commit_spill_publication_from_valuation(&optimized_commit_valuation)?;

    let mut optimized_commit_shrunk =
        GridOptimizedValuation::new("book:grid-scale", "sheet:grid-scale", bounds);
    optimized_commit_shrunk.set_spill_fact(shrunk_facts.get(&a1_anchor).cloned().ok_or_else(
        || GridScaleRunnerError::InvalidOptions {
            detail: "spill-epoch commit setup missing shrunk A1 fact".to_string(),
        },
    )?)?;
    let optimized_commit_extent =
        optimized_commit_sheet.commit_spill_publication_from_valuation(&optimized_commit_shrunk)?;

    Ok(json!({
        "counters": {
            "spill_extent_declared_cells": a1_extent.cell_count(),
            "spill_epoch_base_added": base_ledger_update.anchors_added,
            "spill_epoch_unchanged_preserved": unchanged_ledger_update.epochs_preserved,
            "spill_epoch_unrelated_value_changed": unrelated_churn_ledger_update.value_changed_anchors,
            "spill_epoch_a1_value_changed": value_change_ledger_update.value_changed_anchors,
            "spill_epoch_a1_extent_changed": extent_change_ledger_update.extent_changed_anchors,
            "optimized_spill_commit_first_added": optimized_commit_first.ledger_update.anchors_added,
            "optimized_spill_commit_first_committed_anchors": optimized_commit_first.committed_epoch_anchors,
            "optimized_spill_commit_second_preserved": optimized_commit_second.ledger_update.epochs_preserved,
            "optimized_spill_commit_extent_changed": optimized_commit_extent.ledger_update.extent_changed_anchors,
            "optimized_spill_commit_current_epoch_anchors": optimized_commit_sheet.spill_epoch_ledger().entries().len(),
            "spill_dependency_edges": invalidation.spill_edge_count(),
            "unchanged_anchors_compared": unchanged.anchors_compared,
            "unchanged_changed_anchors": unchanged.changed_anchors.len(),
            "unchanged_dirty_closure_size": unchanged.dirty_closure.len(),
            "unrelated_changed_anchors": unrelated_churn.changed_anchors.len(),
            "unrelated_value_epoch_changed_anchors": unrelated_churn.value_epoch_changed_anchors,
            "unrelated_dirty_closure_size": unrelated_churn.dirty_closure.len(),
            "value_changed_anchors": value_change.changed_anchors.len(),
            "value_epoch_changed_anchors": value_change.value_epoch_changed_anchors,
            "value_dirty_closure_size": value_change.dirty_closure.len(),
            "value_dirty_closure_contains_consumer": value_change.dirty_closure.contains(&a1_consumer),
            "value_dirty_closure_contains_downstream": value_change.dirty_closure.contains(&downstream),
            "extent_changed_anchors": extent_change.changed_anchors.len(),
            "extent_epoch_changed_anchors": extent_change.extent_epoch_changed_anchors,
            "extent_dirty_closure_size": extent_change.dirty_closure.len(),
            "extent_dirty_closure_contains_consumer": extent_change.dirty_closure.contains(&a1_consumer),
            "extent_dirty_closure_contains_downstream": extent_change.dirty_closure.contains(&downstream)
        },
        "register_assertions": [
            register_assertion(
                "P-27",
                "A1# consumers dirty only when their spill anchor extent or value epoch changes",
                a1_extent.cell_count() == u64::from(options.rows)
                    && base_ledger_update.anchors_added == 2
                    && unchanged_ledger_update.epochs_preserved == 2
                    && unrelated_churn_ledger_update.value_changed_anchors == 1
                    && value_change_ledger_update.value_changed_anchors == 1
                    && extent_change_ledger_update.extent_changed_anchors == 1
                    && optimized_commit_first.ledger_update.anchors_added == 1
                    && optimized_commit_second.ledger_update.epochs_preserved == 1
                    && optimized_commit_extent.ledger_update.extent_changed_anchors == 1
                    && optimized_commit_sheet.spill_epoch_ledger().entries().len() == 1
                    && invalidation.spill_edge_count() == 1
                    && unchanged.dirty_closure.is_empty()
                    && unrelated_churn.changed_anchors.len() == 1
                    && unrelated_churn.dirty_closure.is_empty()
                    && value_change.value_epoch_changed_anchors == 1
                    && value_change.dirty_closure.len() == 2
                    && value_change.dirty_closure.contains(&a1_consumer)
                    && value_change.dirty_closure.contains(&downstream)
                    && extent_change.extent_epoch_changed_anchors == 1
                    && extent_change.dirty_closure.len() == 2
                    && extent_change.dirty_closure.contains(&a1_consumer)
                    && extent_change.dirty_closure.contains(&downstream)
            ),
            register_assertion(
                "GRID-SPILL-EPOCH-1M",
                "a 1M-row A1# extent has epoch-precise invalidation for unchanged, unrelated, value, and extent changes",
                unchanged.anchors_compared == 2
                    && unchanged.changed_anchors.is_empty()
                    && unrelated_churn.dirty_closure.is_empty()
                    && value_change.dirty_closure == extent_change.dirty_closure
                    && value_change.dirty_closure.len() == 2
            )
        ]
    }))
}

fn filter_spill_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.rows < 4 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "filter-spill-1m requires at least 4 rows".to_string(),
        });
    }
    if options.cols < 5 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "filter-spill-1m requires at least 5 columns".to_string(),
        });
    }

    let anchor = address(1, 1);
    let old_extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;
    let new_bottom_row = (options.rows / 2).max(1);
    let new_extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        new_bottom_row,
        1,
        bounds,
    )?;
    let mut old_spill_rows = vec![1, (options.rows / 2).max(1), options.rows];
    old_spill_rows.sort_unstable();
    old_spill_rows.dedup();
    let mut new_spill_rows = vec![1, (new_bottom_row / 2).max(1), new_bottom_row];
    new_spill_rows.sort_unstable();
    new_spill_rows.dedup();
    let unrelated_sparse_value_count = options.rows.min(1_000);

    let mut valuation = GridOptimizedValuation::new("book:grid-scale", "sheet:grid-scale", bounds);
    valuation.set_spill_fact(GridSpillFact {
        anchor: anchor.clone(),
        extent: old_extent.clone(),
        blocked: false,
    })?;
    for row in &old_spill_rows {
        valuation.insert_sparse_computed_value(
            address(*row, 1),
            u64::from(*row),
            CalcValue::number(f64::from(*row)),
            GridOptimizedCellSource::SparsePoint,
        )?;
    }
    for row in 1..=unrelated_sparse_value_count {
        valuation.insert_sparse_computed_value(
            address(row, 2),
            u64::from(row),
            CalcValue::number(f64::from(row * 10)),
            GridOptimizedCellSource::SparsePoint,
        )?;
    }

    let sparse_values_before_clear = valuation.sparse_computed_cells();
    let clear_report = valuation.clear_formula_output_for_anchor_report(&anchor)?;
    let sparse_values_after_clear = valuation.sparse_computed_cells();
    valuation.set_spill_fact(GridSpillFact {
        anchor: anchor.clone(),
        extent: new_extent.clone(),
        blocked: false,
    })?;
    for row in &new_spill_rows {
        valuation.insert_sparse_computed_value(
            address(*row, 1),
            u64::from(*row).saturating_add(10_000),
            CalcValue::number(f64::from(*row)),
            GridOptimizedCellSource::SparsePoint,
        )?;
    }
    let sparse_values_after_respill = valuation.sparse_computed_cells();

    let grid_cell_capacity = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let old_new_extent_cell_budget = old_extent.cell_count();
    let respill_sparse_cells_touched = clear_report
        .indexed_candidate_count
        .saturating_add(new_spill_rows.len());
    let indexed_clear_ratio_micros = micros_ratio(
        u64::try_from(clear_report.indexed_candidate_count).unwrap_or(u64::MAX),
        u64::try_from(clear_report.naive_sparse_value_scan_floor).unwrap_or(u64::MAX),
    );
    let respill_touch_ratio_micros = micros_ratio(
        u64::try_from(respill_sparse_cells_touched).unwrap_or(u64::MAX),
        grid_cell_capacity,
    );
    let old_tail_cleared =
        valuation.read_cell(&address(options.rows, 1)).computed == CalcValue::empty();
    let new_tail_written = valuation.read_cell(&address(new_bottom_row, 1)).computed
        == CalcValue::number(f64::from(new_bottom_row));
    let unrelated_tail_kept = valuation
        .read_cell(&address(unrelated_sparse_value_count, 2))
        .computed
        == CalcValue::number(f64::from(unrelated_sparse_value_count * 10));

    let mut filter_sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let source_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        2,
        bounds,
    )?;
    let include_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        3,
        options.rows,
        3,
        bounds,
    )?;
    filter_sheet.put_dense_number_region_with(source_rect, |address| {
        f64::from(address.row) * 100.0 + f64::from(address.col)
    })?;
    filter_sheet.put_dense_literal_region_with(include_rect, |address| {
        CalcValue::logical(address.row <= new_bottom_row)
    })?;
    filter_sheet.set_formula(
        address(1, 4),
        GridFormulaCell::new(
            format!("=FILTER(A1:B{},C1:C{})", options.rows, options.rows),
            format!(
                "excel.grid.v1:filter-spill:R1C1:R{}C2:R1C3:R{}C3",
                options.rows, options.rows
            ),
        ),
    )?;
    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (filter_valuation, filter_committed) = filter_sheet
        .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(
            materialization_limit,
        )?;
    let filter_recalc = &filter_committed.recalc;
    let filter_spill_commit = &filter_committed.spill_commit;
    let filter_anchor = address(1, 4);
    let filter_spill_fact = filter_valuation.spill_facts().get(&filter_anchor);
    let filter_spill_extent_declared_cells =
        filter_spill_fact.map_or(0, |fact| fact.extent.cell_count());
    let filter_spill_extent_rows = filter_spill_fact.map_or(0, |fact| fact.extent.row_count());
    let filter_spill_extent_cols = filter_spill_fact.map_or(0, |fact| fact.extent.col_count());
    let filter_sheet_committed_spill_fact_entries = filter_sheet.spill_facts().len();
    let filter_sheet_committed_epoch_anchors = filter_sheet.spill_epoch_ledger().entries().len();
    let middle_output_row = (new_bottom_row / 2).max(1);
    let filter_first_left_value = filter_valuation.read_cell(&address(1, 4)).computed;
    let filter_first_right_value = filter_valuation.read_cell(&address(1, 5)).computed;
    let filter_middle_left_value = filter_valuation
        .read_cell(&address(middle_output_row, 4))
        .computed;
    let filter_middle_right_value = filter_valuation
        .read_cell(&address(middle_output_row, 5))
        .computed;
    let filter_last_left_value = filter_valuation
        .read_cell(&address(new_bottom_row, 4))
        .computed;
    let filter_last_right_value = filter_valuation
        .read_cell(&address(new_bottom_row, 5))
        .computed;
    let filter_vacated_left_value = filter_valuation
        .read_cell(&address(
            new_bottom_row.saturating_add(1).min(options.rows),
            4,
        ))
        .computed;
    let filter_vacated_right_value = filter_valuation
        .read_cell(&address(
            new_bottom_row.saturating_add(1).min(options.rows),
            5,
        ))
        .computed;

    filter_sheet.set_literal(address(new_bottom_row, 3), CalcValue::logical(false))?;
    let (filter_lifecycle_valuation, filter_lifecycle_committed) = filter_sheet
        .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(
            materialization_limit,
        )?;
    let filter_lifecycle_recalc = &filter_lifecycle_committed.recalc;
    let filter_lifecycle_spill_commit = &filter_lifecycle_committed.spill_commit;
    let filter_lifecycle_rows = new_bottom_row.saturating_sub(1).max(1);
    let filter_lifecycle_spill_fact = filter_lifecycle_valuation.spill_facts().get(&filter_anchor);
    let filter_lifecycle_spill_extent_declared_cells =
        filter_lifecycle_spill_fact.map_or(0, |fact| fact.extent.cell_count());
    let filter_lifecycle_spill_extent_rows =
        filter_lifecycle_spill_fact.map_or(0, |fact| fact.extent.row_count());
    let filter_lifecycle_spill_extent_cols =
        filter_lifecycle_spill_fact.map_or(0, |fact| fact.extent.col_count());
    let filter_lifecycle_sheet_committed_spill_fact_entries = filter_sheet.spill_facts().len();
    let filter_lifecycle_sheet_committed_epoch_anchors =
        filter_sheet.spill_epoch_ledger().entries().len();
    let filter_lifecycle_epoch = filter_sheet
        .spill_epoch_ledger()
        .snapshot_for(&filter_anchor)
        .map_or(0, |snapshot| snapshot.value_epoch);
    let filter_lifecycle_first_left_value = filter_lifecycle_valuation
        .read_cell(&address(1, 4))
        .computed;
    let filter_lifecycle_first_right_value = filter_lifecycle_valuation
        .read_cell(&address(1, 5))
        .computed;
    let filter_lifecycle_last_left_value = filter_lifecycle_valuation
        .read_cell(&address(filter_lifecycle_rows, 4))
        .computed;
    let filter_lifecycle_last_right_value = filter_lifecycle_valuation
        .read_cell(&address(filter_lifecycle_rows, 5))
        .computed;
    let filter_lifecycle_vacated_left_value = filter_lifecycle_valuation
        .read_cell(&address(new_bottom_row, 4))
        .computed;
    let filter_lifecycle_vacated_right_value = filter_lifecycle_valuation
        .read_cell(&address(new_bottom_row, 5))
        .computed;

    let horizontal_source_rows = options.rows.saturating_sub(1);
    let mut column_filter_sheet =
        GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let horizontal_source_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        horizontal_source_rows,
        3,
        bounds,
    )?;
    let horizontal_include_rect = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        options.rows,
        1,
        options.rows,
        3,
        bounds,
    )?;
    column_filter_sheet.put_dense_number_region_with(horizontal_source_rect, |address| {
        f64::from(address.row) * 100.0 + f64::from(address.col)
    })?;
    column_filter_sheet.put_dense_literal_region_with(horizontal_include_rect, |address| {
        CalcValue::logical(address.col != 2)
    })?;
    column_filter_sheet.set_formula(
        address(1, 4),
        GridFormulaCell::new(
            format!(
                "=FILTER(A1:C{},A{}:C{})",
                horizontal_source_rows, options.rows, options.rows
            ),
            format!(
                "excel.grid.v1:filter-spill-columns:R1C1:R{}C3:R{}C1:R{}C3",
                horizontal_source_rows, options.rows, options.rows
            ),
        ),
    )?;
    let (column_filter_valuation, column_filter_committed) = column_filter_sheet
        .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(
            materialization_limit,
        )?;
    let column_filter_recalc = &column_filter_committed.recalc;
    let column_filter_spill_commit = &column_filter_committed.spill_commit;
    let column_filter_anchor = address(1, 4);
    let column_filter_spill_fact = column_filter_valuation
        .spill_facts()
        .get(&column_filter_anchor);
    let column_filter_spill_extent_declared_cells =
        column_filter_spill_fact.map_or(0, |fact| fact.extent.cell_count());
    let column_filter_spill_extent_rows =
        column_filter_spill_fact.map_or(0, |fact| fact.extent.row_count());
    let column_filter_spill_extent_cols =
        column_filter_spill_fact.map_or(0, |fact| fact.extent.col_count());
    let column_filter_sheet_committed_spill_fact_entries = column_filter_sheet.spill_facts().len();
    let column_filter_sheet_committed_epoch_anchors =
        column_filter_sheet.spill_epoch_ledger().entries().len();
    let horizontal_middle_output_row = (horizontal_source_rows / 2).max(1);
    let column_filter_first_left_value = column_filter_valuation.read_cell(&address(1, 4)).computed;
    let column_filter_first_right_value =
        column_filter_valuation.read_cell(&address(1, 5)).computed;
    let column_filter_middle_left_value = column_filter_valuation
        .read_cell(&address(horizontal_middle_output_row, 4))
        .computed;
    let column_filter_middle_right_value = column_filter_valuation
        .read_cell(&address(horizontal_middle_output_row, 5))
        .computed;
    let column_filter_last_left_value = column_filter_valuation
        .read_cell(&address(horizontal_source_rows, 4))
        .computed;
    let column_filter_last_right_value = column_filter_valuation
        .read_cell(&address(horizontal_source_rows, 5))
        .computed;

    let counters = json_object([
        ("grid_cell_capacity", json!(grid_cell_capacity)),
        (
            "old_spill_extent_declared_cells",
            json!(clear_report.old_extent_cell_count),
        ),
        (
            "new_spill_extent_declared_cells",
            json!(new_extent.cell_count()),
        ),
        (
            "old_new_extent_cell_budget",
            json!(old_new_extent_cell_budget),
        ),
        ("old_sparse_spill_values", json!(old_spill_rows.len())),
        (
            "new_sparse_spill_values_written",
            json!(new_spill_rows.len()),
        ),
        (
            "unrelated_sparse_values",
            json!(unrelated_sparse_value_count),
        ),
        (
            "sparse_values_before_clear",
            json!(sparse_values_before_clear),
        ),
        (
            "sparse_values_after_clear",
            json!(sparse_values_after_clear),
        ),
        (
            "sparse_values_after_respill",
            json!(sparse_values_after_respill),
        ),
        (
            "naive_sparse_value_scan_floor",
            json!(clear_report.naive_sparse_value_scan_floor),
        ),
        (
            "indexed_clear_candidate_count",
            json!(clear_report.indexed_candidate_count),
        ),
        (
            "sparse_values_removed",
            json!(clear_report.sparse_values_removed),
        ),
        (
            "indexed_clear_ratio_micros",
            json!(indexed_clear_ratio_micros),
        ),
        (
            "respill_sparse_cells_touched",
            json!(respill_sparse_cells_touched),
        ),
        (
            "respill_touch_ratio_micros",
            json!(respill_touch_ratio_micros),
        ),
        ("old_tail_cleared", json!(old_tail_cleared)),
        ("new_tail_written", json!(new_tail_written)),
        ("unrelated_tail_kept", json!(unrelated_tail_kept)),
        (
            "filter_formula_spill_extent_declared_cells",
            json!(filter_spill_extent_declared_cells),
        ),
        (
            "filter_formula_spill_facts_published",
            json!(filter_recalc.spill_facts_published),
        ),
        (
            "filter_formula_spill_facts_blocked",
            json!(filter_recalc.spill_facts_blocked),
        ),
        (
            "filter_formula_spill_ghost_cells_published",
            json!(filter_recalc.spill_ghost_cells_published),
        ),
        (
            "filter_formula_spill_commit_previous_fact_entries",
            json!(filter_spill_commit.previous_spill_fact_entries),
        ),
        (
            "filter_formula_spill_commit_committed_fact_entries",
            json!(filter_spill_commit.committed_spill_fact_entries),
        ),
        (
            "filter_formula_spill_commit_anchors_added",
            json!(filter_spill_commit.ledger_update.anchors_added),
        ),
        (
            "filter_formula_spill_commit_current_epoch_anchors",
            json!(filter_sheet_committed_epoch_anchors),
        ),
        (
            "filter_formula_sheet_committed_spill_fact_entries",
            json!(filter_sheet_committed_spill_fact_entries),
        ),
        (
            "filter_formula_spill_extent_declared_rows",
            json!(filter_spill_extent_rows),
        ),
        (
            "filter_formula_spill_extent_declared_cols",
            json!(filter_spill_extent_cols),
        ),
        (
            "filter_formula_computed_dense_value_regions",
            json!(filter_recalc.computed_dense_value_regions),
        ),
        (
            "filter_formula_computed_dense_cells",
            json!(filter_valuation.dense_computed_cells()),
        ),
        (
            "filter_formula_computed_dense_numeric_packed_cells",
            json!(filter_valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "filter_formula_computed_sparse_cells",
            json!(filter_valuation.sparse_computed_cells()),
        ),
        (
            "filter_formula_first_left_value",
            json!(calc_value_display_text(&filter_first_left_value)),
        ),
        (
            "filter_formula_first_right_value",
            json!(calc_value_display_text(&filter_first_right_value)),
        ),
        (
            "filter_formula_middle_left_value",
            json!(calc_value_display_text(&filter_middle_left_value)),
        ),
        (
            "filter_formula_middle_right_value",
            json!(calc_value_display_text(&filter_middle_right_value)),
        ),
        (
            "filter_formula_last_left_value",
            json!(calc_value_display_text(&filter_last_left_value)),
        ),
        (
            "filter_formula_last_right_value",
            json!(calc_value_display_text(&filter_last_right_value)),
        ),
        (
            "filter_formula_vacated_left_value",
            json!(calc_value_display_text(&filter_vacated_left_value)),
        ),
        (
            "filter_formula_vacated_right_value",
            json!(calc_value_display_text(&filter_vacated_right_value)),
        ),
        ("filter_lifecycle_sparse_mask_overrides", json!(1)),
        (
            "filter_lifecycle_spill_extent_declared_cells",
            json!(filter_lifecycle_spill_extent_declared_cells),
        ),
        (
            "filter_lifecycle_spill_extent_declared_rows",
            json!(filter_lifecycle_spill_extent_rows),
        ),
        (
            "filter_lifecycle_spill_extent_declared_cols",
            json!(filter_lifecycle_spill_extent_cols),
        ),
        (
            "filter_lifecycle_spill_facts_published",
            json!(filter_lifecycle_recalc.spill_facts_published),
        ),
        (
            "filter_lifecycle_spill_facts_blocked",
            json!(filter_lifecycle_recalc.spill_facts_blocked),
        ),
        (
            "filter_lifecycle_spill_ghost_cells_published",
            json!(filter_lifecycle_recalc.spill_ghost_cells_published),
        ),
        (
            "filter_lifecycle_spill_commit_previous_fact_entries",
            json!(filter_lifecycle_spill_commit.previous_spill_fact_entries),
        ),
        (
            "filter_lifecycle_spill_commit_committed_fact_entries",
            json!(filter_lifecycle_spill_commit.committed_spill_fact_entries),
        ),
        (
            "filter_lifecycle_spill_commit_anchors_added",
            json!(filter_lifecycle_spill_commit.ledger_update.anchors_added),
        ),
        (
            "filter_lifecycle_spill_commit_anchors_changed",
            json!(filter_lifecycle_spill_commit.ledger_update.anchors_changed),
        ),
        (
            "filter_lifecycle_spill_commit_extent_changed_anchors",
            json!(
                filter_lifecycle_spill_commit
                    .ledger_update
                    .extent_changed_anchors
            ),
        ),
        (
            "filter_lifecycle_spill_commit_value_changed_anchors",
            json!(
                filter_lifecycle_spill_commit
                    .ledger_update
                    .value_changed_anchors
            ),
        ),
        (
            "filter_lifecycle_spill_commit_blocked_changed_anchors",
            json!(
                filter_lifecycle_spill_commit
                    .ledger_update
                    .blocked_changed_anchors
            ),
        ),
        (
            "filter_lifecycle_spill_commit_current_epoch_anchors",
            json!(filter_lifecycle_sheet_committed_epoch_anchors),
        ),
        (
            "filter_lifecycle_sheet_committed_spill_fact_entries",
            json!(filter_lifecycle_sheet_committed_spill_fact_entries),
        ),
        (
            "filter_lifecycle_committed_value_epoch",
            json!(filter_lifecycle_epoch),
        ),
        (
            "filter_lifecycle_computed_dense_value_regions",
            json!(filter_lifecycle_recalc.computed_dense_value_regions),
        ),
        (
            "filter_lifecycle_computed_dense_cells",
            json!(filter_lifecycle_valuation.dense_computed_cells()),
        ),
        (
            "filter_lifecycle_computed_dense_numeric_packed_cells",
            json!(filter_lifecycle_valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "filter_lifecycle_computed_sparse_cells",
            json!(filter_lifecycle_valuation.sparse_computed_cells()),
        ),
        (
            "filter_lifecycle_first_left_value",
            json!(calc_value_display_text(&filter_lifecycle_first_left_value)),
        ),
        (
            "filter_lifecycle_first_right_value",
            json!(calc_value_display_text(&filter_lifecycle_first_right_value)),
        ),
        (
            "filter_lifecycle_last_left_value",
            json!(calc_value_display_text(&filter_lifecycle_last_left_value)),
        ),
        (
            "filter_lifecycle_last_right_value",
            json!(calc_value_display_text(&filter_lifecycle_last_right_value)),
        ),
        (
            "filter_lifecycle_vacated_left_value",
            json!(calc_value_display_text(
                &filter_lifecycle_vacated_left_value
            )),
        ),
        (
            "filter_lifecycle_vacated_right_value",
            json!(calc_value_display_text(
                &filter_lifecycle_vacated_right_value
            )),
        ),
        ("column_filter_source_rows", json!(horizontal_source_rows)),
        (
            "column_filter_spill_extent_declared_cells",
            json!(column_filter_spill_extent_declared_cells),
        ),
        (
            "column_filter_spill_facts_published",
            json!(column_filter_recalc.spill_facts_published),
        ),
        (
            "column_filter_spill_facts_blocked",
            json!(column_filter_recalc.spill_facts_blocked),
        ),
        (
            "column_filter_spill_ghost_cells_published",
            json!(column_filter_recalc.spill_ghost_cells_published),
        ),
        (
            "column_filter_spill_commit_previous_fact_entries",
            json!(column_filter_spill_commit.previous_spill_fact_entries),
        ),
        (
            "column_filter_spill_commit_committed_fact_entries",
            json!(column_filter_spill_commit.committed_spill_fact_entries),
        ),
        (
            "column_filter_spill_commit_anchors_added",
            json!(column_filter_spill_commit.ledger_update.anchors_added),
        ),
        (
            "column_filter_spill_commit_current_epoch_anchors",
            json!(column_filter_sheet_committed_epoch_anchors),
        ),
        (
            "column_filter_sheet_committed_spill_fact_entries",
            json!(column_filter_sheet_committed_spill_fact_entries),
        ),
        (
            "column_filter_spill_extent_declared_rows",
            json!(column_filter_spill_extent_rows),
        ),
        (
            "column_filter_spill_extent_declared_cols",
            json!(column_filter_spill_extent_cols),
        ),
        (
            "column_filter_computed_dense_value_regions",
            json!(column_filter_recalc.computed_dense_value_regions),
        ),
        (
            "column_filter_computed_dense_cells",
            json!(column_filter_valuation.dense_computed_cells()),
        ),
        (
            "column_filter_computed_dense_numeric_packed_cells",
            json!(column_filter_valuation.dense_computed_numeric_packed_cells()),
        ),
        (
            "column_filter_computed_sparse_cells",
            json!(column_filter_valuation.sparse_computed_cells()),
        ),
        (
            "column_filter_first_left_value",
            json!(calc_value_display_text(&column_filter_first_left_value)),
        ),
        (
            "column_filter_first_right_value",
            json!(calc_value_display_text(&column_filter_first_right_value)),
        ),
        (
            "column_filter_middle_left_value",
            json!(calc_value_display_text(&column_filter_middle_left_value)),
        ),
        (
            "column_filter_middle_right_value",
            json!(calc_value_display_text(&column_filter_middle_right_value)),
        ),
        (
            "column_filter_last_left_value",
            json!(calc_value_display_text(&column_filter_last_left_value)),
        ),
        (
            "column_filter_last_right_value",
            json!(calc_value_display_text(&column_filter_last_right_value)),
        ),
    ]);
    let register_assertions = json!([
        register_assertion(
            "P-23",
            "re-spill old-output cleanup uses the sparse index over the old spill extent instead of scanning the whole sparse value map or sheet",
            clear_report.had_spill_fact
                && clear_report.old_extent_cell_count == u64::from(options.rows)
                && new_extent.cell_count() < clear_report.old_extent_cell_count
                && old_new_extent_cell_budget == clear_report.old_extent_cell_count
                && clear_report.naive_sparse_value_scan_floor == sparse_values_before_clear
                && clear_report.indexed_candidate_count == old_spill_rows.len()
                && clear_report.sparse_values_removed == old_spill_rows.len()
                && clear_report.indexed_candidate_count
                    < clear_report.naive_sparse_value_scan_floor
                && u64::try_from(respill_sparse_cells_touched).unwrap_or(u64::MAX)
                    < grid_cell_capacity
                && sparse_values_after_clear
                    == usize::try_from(unrelated_sparse_value_count).unwrap_or(usize::MAX)
                && old_tail_cleared
                && new_tail_written
                && unrelated_tail_kept
        ),
        register_assertion(
            "GRID-FILTER-SPILL-1M",
            "a 1M-row re-spill resize clears only old spill outputs, preserves unrelated sparse values, and a real two-column FILTER formula publishes dense output",
            old_extent.cell_count() == u64::from(options.rows)
                && clear_report.indexed_candidate_count == old_spill_rows.len()
                && sparse_values_after_respill
                    == usize::try_from(unrelated_sparse_value_count).unwrap_or(usize::MAX)
                        + new_spill_rows.len()
                && clear_report.old_extent == old_extent
                && valuation
                    .spill_facts()
                    .get(&anchor)
                    .is_some_and(|fact| fact.extent == new_extent)
                && filter_spill_extent_declared_cells
                    == u64::from(new_bottom_row).saturating_mul(2)
                && filter_spill_extent_rows == new_bottom_row
                && filter_spill_extent_cols == 2
                && filter_recalc.spill_facts_published == 1
                && filter_recalc.spill_facts_blocked == 0
                && filter_recalc.spill_ghost_cells_published
                    == usize::try_from(
                        u64::from(new_bottom_row)
                            .saturating_mul(2)
                            .saturating_sub(1)
                    )
                    .unwrap_or(usize::MAX)
                && filter_spill_commit.previous_spill_fact_entries == 0
                && filter_spill_commit.committed_spill_fact_entries == 1
                && filter_spill_commit.ledger_update.anchors_added == 1
                && filter_sheet_committed_spill_fact_entries == 1
                && filter_sheet_committed_epoch_anchors == 1
                && filter_valuation.sparse_computed_cells() == 0
                && filter_first_left_value == CalcValue::number(101.0)
                && filter_first_right_value == CalcValue::number(102.0)
                && filter_middle_left_value
                    == CalcValue::number(f64::from(middle_output_row) * 100.0 + 1.0)
                && filter_middle_right_value
                    == CalcValue::number(f64::from(middle_output_row) * 100.0 + 2.0)
                && filter_last_left_value
                    == CalcValue::number(f64::from(new_bottom_row) * 100.0 + 1.0)
                && filter_last_right_value
                    == CalcValue::number(f64::from(new_bottom_row) * 100.0 + 2.0)
                && filter_vacated_left_value == CalcValue::empty()
                && filter_vacated_right_value == CalcValue::empty()
        ),
        register_assertion(
            "GRID-FILTER-LIFECYCLE-1M",
            "a later committed optimized FILTER recalc shrinks the dense output, clears vacated ghosts, and advances the spill epoch",
            filter_lifecycle_spill_extent_declared_cells
                == u64::from(filter_lifecycle_rows).saturating_mul(2)
                && filter_lifecycle_spill_extent_rows == filter_lifecycle_rows
                && filter_lifecycle_spill_extent_cols == 2
                && filter_lifecycle_recalc.spill_facts_published == 1
                && filter_lifecycle_recalc.spill_facts_blocked == 0
                && filter_lifecycle_recalc.spill_ghost_cells_published
                    == usize::try_from(
                        u64::from(filter_lifecycle_rows)
                            .saturating_mul(2)
                            .saturating_sub(1)
                    )
                    .unwrap_or(usize::MAX)
                && filter_lifecycle_spill_commit.previous_spill_fact_entries == 1
                && filter_lifecycle_spill_commit.committed_spill_fact_entries == 1
                && filter_lifecycle_spill_commit.ledger_update.anchors_added == 0
                && filter_lifecycle_spill_commit.ledger_update.anchors_changed == 1
                && filter_lifecycle_spill_commit
                    .ledger_update
                    .extent_changed_anchors
                    == 1
                && filter_lifecycle_spill_commit
                    .ledger_update
                    .value_changed_anchors
                    == 1
                && filter_lifecycle_spill_commit
                    .ledger_update
                    .blocked_changed_anchors
                    == 0
                && filter_lifecycle_sheet_committed_spill_fact_entries == 1
                && filter_lifecycle_sheet_committed_epoch_anchors == 1
                && filter_lifecycle_epoch == 2
                && filter_lifecycle_valuation.sparse_computed_cells() == 1
                && filter_lifecycle_first_left_value == CalcValue::number(101.0)
                && filter_lifecycle_first_right_value == CalcValue::number(102.0)
                && filter_lifecycle_last_left_value
                    == CalcValue::number(f64::from(filter_lifecycle_rows) * 100.0 + 1.0)
                && filter_lifecycle_last_right_value
                    == CalcValue::number(f64::from(filter_lifecycle_rows) * 100.0 + 2.0)
                && filter_lifecycle_vacated_left_value == CalcValue::empty()
                && filter_lifecycle_vacated_right_value == CalcValue::empty()
        ),
        register_assertion(
            "GRID-FILTER-COLUMN-SPILL-1M",
            "a 1M-row three-column source with a horizontal include row publishes and commits a dense two-column FILTER output",
            column_filter_spill_extent_declared_cells
                == u64::from(horizontal_source_rows).saturating_mul(2)
                && column_filter_spill_extent_rows == horizontal_source_rows
                && column_filter_spill_extent_cols == 2
                && column_filter_recalc.spill_facts_published == 1
                && column_filter_recalc.spill_facts_blocked == 0
                && column_filter_recalc.spill_ghost_cells_published
                    == usize::try_from(
                        u64::from(horizontal_source_rows)
                            .saturating_mul(2)
                            .saturating_sub(1)
                    )
                    .unwrap_or(usize::MAX)
                && column_filter_spill_commit.previous_spill_fact_entries == 0
                && column_filter_spill_commit.committed_spill_fact_entries == 1
                && column_filter_spill_commit.ledger_update.anchors_added == 1
                && column_filter_sheet_committed_spill_fact_entries == 1
                && column_filter_sheet_committed_epoch_anchors == 1
                && column_filter_valuation.sparse_computed_cells() == 0
                && column_filter_first_left_value == CalcValue::number(101.0)
                && column_filter_first_right_value == CalcValue::number(103.0)
                && column_filter_middle_left_value
                    == CalcValue::number(f64::from(horizontal_middle_output_row) * 100.0 + 1.0)
                && column_filter_middle_right_value
                    == CalcValue::number(f64::from(horizontal_middle_output_row) * 100.0 + 3.0)
                && column_filter_last_left_value
                    == CalcValue::number(f64::from(horizontal_source_rows) * 100.0 + 1.0)
                && column_filter_last_right_value
                    == CalcValue::number(f64::from(horizontal_source_rows) * 100.0 + 3.0)
        )
    ]);
    Ok(json!({
        "counters": counters,
        "register_assertions": register_assertions
    }))
}

fn sequence_spill_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 1 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "sequence-spill-1m requires at least 1 column".to_string(),
        });
    }

    let anchor = address(1, 1);
    let middle_row = (options.rows / 2).max(1);
    let mut sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    sheet.set_formula(
        anchor.clone(),
        GridFormulaCell::new(
            format!("=SEQUENCE({})", options.rows),
            format!("excel.grid.v1:sequence:R1C1:{}#", options.rows),
        ),
    )?;

    let materialization_limit = u64::from(options.rows).saturating_mul(u64::from(options.cols));
    let (valuation, committed) = sheet
        .recalculate_mark_all_dirty_compact_with_oxfml_and_commit_spill_publication(
            materialization_limit,
        )?;
    let recalc = &committed.recalc;
    let spill_commit = &committed.spill_commit;
    let first_value = valuation.read_cell(&address(1, 1)).computed;
    let middle_value = valuation.read_cell(&address(middle_row, 1)).computed;
    let last_value = valuation.read_cell(&address(options.rows, 1)).computed;
    let spill_fact = valuation.spill_facts().get(&anchor);
    let spill_extent_cell_count = spill_fact.map_or(0, |fact| fact.extent.cell_count());
    let committed_spill_fact_entries = sheet.spill_facts().len();
    let committed_epoch_anchors = sheet.spill_epoch_ledger().entries().len();

    Ok(json!({
        "counters": {
            "spill_extent_declared_cells": spill_extent_cell_count,
            "spill_facts_published": recalc.spill_facts_published,
            "spill_facts_blocked": recalc.spill_facts_blocked,
            "spill_ghost_cells_published": recalc.spill_ghost_cells_published,
            "spill_commit_previous_fact_entries": spill_commit.previous_spill_fact_entries,
            "spill_commit_committed_fact_entries": spill_commit.committed_spill_fact_entries,
            "spill_commit_anchors_added": spill_commit.ledger_update.anchors_added,
            "spill_commit_current_epoch_anchors": committed_epoch_anchors,
            "sheet_committed_spill_fact_entries": committed_spill_fact_entries,
            "computed_dense_value_regions": recalc.computed_dense_value_regions,
            "computed_dense_cells": valuation.dense_computed_cells(),
            "computed_dense_numeric_packed_cells": valuation.dense_computed_numeric_packed_cells(),
            "computed_sparse_cells": valuation.sparse_computed_cells(),
            "formula_cells": recalc.formula_cells,
            "formula_evaluations": recalc.formula_evaluations,
            "sample_first_value": calc_value_display_text(&first_value),
            "sample_middle_row": middle_row,
            "sample_middle_value": calc_value_display_text(&middle_value),
            "sample_last_value": calc_value_display_text(&last_value)
        },
        "register_assertions": [
            register_assertion(
                "P-23",
                "successful dynamic-array spill publication stores a 1M SEQUENCE payload as one dense computed region instead of sparse cells",
                recalc.spill_facts_published == 1
                    && recalc.spill_facts_blocked == 0
                    && recalc.spill_ghost_cells_published == usize::try_from(options.rows.saturating_sub(1)).unwrap_or(usize::MAX)
                    && spill_commit.previous_spill_fact_entries == 0
                    && spill_commit.committed_spill_fact_entries == 1
                    && spill_commit.ledger_update.anchors_added == 1
                    && committed_spill_fact_entries == 1
                    && committed_epoch_anchors == 1
                    && recalc.computed_dense_value_regions == 1
                    && valuation.dense_computed_cells() == u64::from(options.rows)
                    && valuation.dense_computed_numeric_packed_cells() == u64::from(options.rows)
                    && valuation.sparse_computed_cells() == 0
            ),
            register_assertion(
                "GRID-SEQUENCE-SPILL-1M",
                "a 1M-row SEQUENCE spill remains dense-region backed and preserves sampled values",
                spill_extent_cell_count == u64::from(options.rows)
                    && first_value == CalcValue::number(1.0)
                    && middle_value == CalcValue::number(f64::from(middle_row))
                    && last_value == CalcValue::number(f64::from(options.rows))
            )
        ]
    }))
}

fn spill_blockage_1m_scale(options: &GridScaleOptions) -> Result<Value, GridScaleRunnerError> {
    let bounds = bounded_grid_options(options)?;
    if options.cols < 2 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "spill-blockage-1m requires at least 2 columns".to_string(),
        });
    }

    let anchor = address(1, 1);
    let extent = GridRect::new(
        "book:grid-scale",
        "sheet:grid-scale",
        1,
        1,
        options.rows,
        1,
        bounds,
    )?;

    let empty_sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let empty_report = empty_sheet.optimized_spill_blockage_probe_report(&anchor, &extent)?;

    let mut blocked_sheet = GridOptimizedSheet::new("book:grid-scale", "sheet:grid-scale", bounds);
    let far_blocker = address(options.rows, 1);
    blocked_sheet.set_literal(far_blocker.clone(), CalcValue::number(99.0))?;
    let blocked_report = blocked_sheet.optimized_spill_blockage_probe_report(&anchor, &extent)?;

    let empty_compact_probe_count =
        u64::try_from(empty_report.compact_blocker_probe_count()).unwrap_or(u64::MAX);
    let blocked_compact_probe_count =
        u64::try_from(blocked_report.compact_blocker_probe_count()).unwrap_or(u64::MAX);
    let empty_probe_ratio_micros = micros_ratio(
        empty_compact_probe_count,
        empty_report.naive_extent_cell_probe_floor,
    );
    let blocked_probe_ratio_micros = micros_ratio(
        blocked_compact_probe_count,
        blocked_report.naive_extent_cell_probe_floor,
    );

    Ok(json!({
        "counters": {
            "spill_extent_declared_cells": empty_report.extent_cell_count,
            "empty_naive_extent_cell_probe_floor": empty_report.naive_extent_cell_probe_floor,
            "empty_compact_blocker_probe_count": empty_compact_probe_count,
            "empty_probe_ratio_micros": empty_probe_ratio_micros,
            "empty_blocked": empty_report.blocked,
            "far_blocker_row": far_blocker.row,
            "far_blocker_naive_extent_cell_probe_floor": blocked_report.naive_extent_cell_probe_floor,
            "far_blocker_compact_blocker_probe_count": blocked_compact_probe_count,
            "far_blocker_sparse_point_candidates": blocked_report.sparse_point_candidates,
            "far_blocker_probe_ratio_micros": blocked_probe_ratio_micros,
            "far_blocker_blocked": blocked_report.blocked
        },
        "register_assertions": [
            register_assertion(
                "P-26",
                "spill blockage probes compact occupied candidates and never empty cells across a 1M-row intended extent",
                empty_report.extent_cell_count == u64::from(options.rows)
                    && empty_report.naive_extent_cell_probe_floor == u64::from(options.rows)
                    && empty_compact_probe_count == 0
                    && !empty_report.blocked
                    && blocked_report.naive_extent_cell_probe_floor == u64::from(options.rows)
                    && blocked_compact_probe_count == 1
                    && blocked_report.sparse_point_candidates == 1
                    && blocked_report.blocked
                    && blocked_compact_probe_count < blocked_report.naive_extent_cell_probe_floor
            ),
            register_assertion(
                "GRID-SPILL-BLOCKAGE-1M",
                "a far sparse blocker in a 1M-row spill extent is found by one compact probe instead of a row scan",
                blocked_report.extent_cell_count == u64::from(options.rows)
                    && blocked_report.blocked
                    && blocked_compact_probe_count < blocked_report.naive_extent_cell_probe_floor
            )
        ]
    }))
}

#[derive(Debug, Clone, Default)]
struct InsertStormReport {
    dense_region_metadata_visits: u64,
    repeated_formula_region_metadata_visits: u64,
    compact_region_metadata_touches: u64,
    max_dense_regions_after: usize,
    max_repeated_formula_regions_after: usize,
    dense_value_regions_dropped: usize,
    repeated_formula_regions_dropped: usize,
    repeated_formula_segments_transformed: usize,
    repeated_formula_reference_transforms: usize,
}

impl InsertStormReport {
    fn from_reports(reports: &[GridOptimizedStructuralEditReport]) -> Self {
        let mut storm = Self::default();
        for report in reports {
            storm.dense_region_metadata_visits = storm.dense_region_metadata_visits.saturating_add(
                u64::try_from(report.dense_value_regions_before).unwrap_or(u64::MAX),
            );
            storm.repeated_formula_region_metadata_visits = storm
                .repeated_formula_region_metadata_visits
                .saturating_add(
                    u64::try_from(report.repeated_formula_regions_before).unwrap_or(u64::MAX),
                );
            storm.max_dense_regions_after = storm
                .max_dense_regions_after
                .max(report.dense_value_regions_after);
            storm.max_repeated_formula_regions_after = storm
                .max_repeated_formula_regions_after
                .max(report.repeated_formula_regions_after);
            storm.dense_value_regions_dropped += report.dense_value_regions_dropped;
            storm.repeated_formula_regions_dropped += report.repeated_formula_regions_dropped;
            storm.repeated_formula_segments_transformed +=
                report.repeated_formula_segments_transformed;
            storm.repeated_formula_reference_transforms +=
                report.repeated_formula_reference_transforms;
        }
        storm.compact_region_metadata_touches = storm
            .dense_region_metadata_visits
            .saturating_add(storm.repeated_formula_region_metadata_visits);
        storm
    }
}

fn register_assertion(id: &str, claim: &str, passed: bool) -> Value {
    json!({
        "id": id,
        "claim": claim,
        "passed": passed
    })
}

fn json_object(entries: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    let mut object = serde_json::Map::new();
    for (key, value) in entries {
        object.insert(key.to_string(), value);
    }
    Value::Object(object)
}

fn authored_cell_display_text(cell: &GridAuthoredCell) -> String {
    match cell {
        GridAuthoredCell::Literal(value) => calc_value_display_text(value),
        GridAuthoredCell::Formula(formula) => formula.source_text.clone(),
    }
}

fn integer_display(value: f64) -> String {
    format!("{value:.0}")
}

fn pascal_r1c1_expected_value(row: u32, col: u32) -> f64 {
    let width = usize::try_from(col).unwrap_or(usize::MAX);
    if width == 0 {
        return 0.0;
    }
    let mut values = vec![1.0; width];
    for _ in 2..=row {
        values[0] = 1.0;
        for index in 1..width {
            values[index] += values[index - 1];
        }
    }
    values[width - 1]
}

fn sum_pyramid_row_for_level1(level1_index: u32, level1_height: u32) -> u32 {
    level1_index.saturating_mul(level1_height).saturating_add(1)
}

fn bounded_grid_options(
    options: &GridScaleOptions,
) -> Result<ExcelGridBounds, GridScaleRunnerError> {
    if options.rows == 0 || options.cols == 0 {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: "rows and cols must be positive".to_string(),
        });
    }
    if options.rows > ExcelGridBounds::strict_excel().max_rows
        || options.cols > ExcelGridBounds::strict_excel().max_cols
    {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: format!(
                "rows/cols exceed strict Excel bounds: {}x{}",
                options.rows, options.cols
            ),
        });
    }
    Ok(ExcelGridBounds {
        max_rows: options.rows,
        max_cols: options.cols,
    })
}

fn validate_grid_scale_options(options: &GridScaleOptions) -> Result<(), GridScaleRunnerError> {
    validate_run_id(&options.run_id)?;
    if options.profile == GridScaleProfile::SparseWholeColumn {
        return Ok(());
    }
    bounded_grid_options(options).map(|_| ())
}

fn validate_run_id(run_id: &str) -> Result<(), GridScaleRunnerError> {
    if run_id.is_empty()
        || !run_id
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_'))
    {
        return Err(GridScaleRunnerError::InvalidOptions {
            detail: format!("invalid run id '{run_id}'"),
        });
    }
    Ok(())
}

fn address(row: u32, col: u32) -> ExcelGridCellAddress {
    ExcelGridCellAddress::new("book:grid-scale", "sheet:grid-scale", row, col)
}

fn zig_zag_col(row: u32, cols: u32) -> u32 {
    ((row - 1) % cols) + 1
}

fn insert_storm_rows(occupied_rows: u32) -> [u32; 3] {
    [
        (occupied_rows / 4).max(2),
        (occupied_rows / 2).max(3),
        ((occupied_rows.saturating_mul(3)) / 4).max(4),
    ]
}

fn micros_ratio(numerator: u64, denominator: u64) -> u64 {
    if denominator == 0 {
        return 0;
    }
    numerator.saturating_mul(1_000_000) / denominator
}

fn relative_artifact_path<const N: usize>(parts: [&str; N]) -> String {
    parts.join("/")
}

fn create_directory(path: &Path) -> Result<(), GridScaleRunnerError> {
    fs::create_dir_all(path).map_err(|source| GridScaleRunnerError::CreateDirectory {
        path: path.display().to_string(),
        source,
    })
}

fn write_json(path: &Path, value: &Value) -> Result<(), GridScaleRunnerError> {
    let text = serde_json::to_string_pretty(value).expect("grid scale JSON should serialize");
    fs::write(path, text).map_err(|source| GridScaleRunnerError::WriteFile {
        path: path.display().to_string(),
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sparse_whole_column_scale_counts_occupied_slots_only() {
        let options = GridScaleOptions::default_for(GridScaleProfile::SparseWholeColumn, "scale");
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["declared_cell_count"],
            1_048_576
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["slots_visited"],
            3
        );
    }

    #[test]
    fn full_column_1m_scale_counts_dense_occupied_slots_only() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::FullColumn1M, "fullcol")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["declared_cell_count"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["dense_value_cells_visited"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sum"],
            "5050"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["warm_cells_visited"],
            0
        );
    }

    #[test]
    fn dense_values_scale_keeps_values_region_backed() {
        let options = GridScaleOptions {
            rows: 10,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::DenseValues, "dense")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["dense_value_cells"],
            40
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
    }

    #[test]
    fn sparse_singletons_scale_keeps_sparse_points_compact() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 7,
            ..GridScaleOptions::default_for(GridScaleProfile::SparseSingletons, "singletons")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["sparse_point_cells"],
            100
        );
        assert!(
            execution.counter_summary_json["counters"]["sparse_point_bytes_per_cell_micros"]
                .as_u64()
                .unwrap()
                <= 85_000_000
        );
    }

    #[test]
    fn zig_zag_1m_scale_spans_full_width_sparse_points() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 16,
            ..GridScaleOptions::default_for(GridScaleProfile::ZigZag1M, "zigzag")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["sparse_point_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["columns_spanned"],
            16
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["sample_last_col"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["sample_last_value"],
            "100"
        );
    }

    #[test]
    fn repeated_r1c1_scale_prepares_template_once_and_warms_clean() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::RepeatedR1C1, "r1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["warm_cells_visited"],
            0
        );
    }

    #[test]
    fn fill_down_r1c1_scale_runs_template_once_and_warms_clean() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 1,
            ..GridScaleOptions::default_for(GridScaleProfile::FillDownR1C1, "fill")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "100"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["warm_cells_visited"],
            0
        );
    }

    #[test]
    fn pascal_r1c1_scale_publishes_2d_recurrence_as_dense_output() {
        let options = GridScaleOptions {
            rows: 2_000,
            cols: 8,
            ..GridScaleOptions::default_for(GridScaleProfile::PascalR1C1, "pascal")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["repeated_formula_cells"],
            13_993
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_plan_cache_misses"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            7
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["sample_first_formula_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["sample_last_formula_value"],
            execution.counter_summary_json["counters"]["expected_last_formula_value"]
        );
    }

    #[test]
    fn boring_1mx10_scale_combines_dense_values_and_repeated_formulas() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 10,
            ..GridScaleOptions::default_for(GridScaleProfile::Boring1Mx10, "boring")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["dense_value_cells"],
            800
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["repeated_formula_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "400032"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["warm_cells_visited"],
            0
        );
    }

    #[test]
    fn direct_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 3,
            ..GridScaleOptions::default_for(GridScaleProfile::DirectR1C1, "directr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_direct_value"],
            "10"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_parenthesized_value"],
            "10"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_direct_value"],
            "500"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_parenthesized_value"],
            "500"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_direct_value"],
            "1000"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_parenthesized_value"],
            "1000"
        );
    }

    #[test]
    fn unary_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::UnaryR1C1, "unaryr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            400
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            400
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_direct_value"],
            "-10"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_parenthesized_value"],
            "-15"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_arithmetic_value"],
            "-19"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_direct_value"],
            "-500"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_parenthesized_value"],
            "-505"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_arithmetic_value"],
            "-999"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_direct_value"],
            "-1000"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_parenthesized_value"],
            "-1005"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_arithmetic_value"],
            "-1999"
        );
    }

    #[test]
    fn argument_aggregate_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 8,
            ..GridScaleOptions::default_for(
                GridScaleProfile::ArgumentAggregateR1C1,
                "argaggregater1c1",
            )
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            600
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            6
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            6
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            7
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            800
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            800
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_sum_value"],
            "16"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_count_value"],
            "3"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_product_value"],
            "20"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_min_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_max_value"],
            "10"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_sum_value"],
            "555"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_product_value"],
            "50000"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_sum_value"],
            "1105"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_product_value"],
            "200000"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_min_value"],
            "5"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_max_value"],
            "1000"
        );
    }

    #[test]
    fn math_function_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 5,
            ..GridScaleOptions::default_for(GridScaleProfile::MathFunctionR1C1, "mathr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_abs_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_sqrt_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_power_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_abs_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_sqrt_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_power_value"],
            "2500"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_abs_value"],
            "100"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_sqrt_value"],
            "100"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_power_value"],
            "10000"
        );
    }

    #[test]
    fn mod_function_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 5,
            ..GridScaleOptions::default_for(GridScaleProfile::ModFunctionR1C1, "modr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_mod_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_if_value"],
            "3"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_power_mod_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_mod_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_if_value"],
            "25"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_power_mod_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_mod_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_if_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_power_mod_value"],
            "4"
        );
    }

    #[test]
    fn rounding_function_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 5,
            ..GridScaleOptions::default_for(GridScaleProfile::RoundingFunctionR1C1, "roundr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_round_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_roundup_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_rounddown_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_round_value"],
            "51"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_roundup_value"],
            "51"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_rounddown_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_round_value"],
            "101"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_roundup_value"],
            "101"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_rounddown_value"],
            "100"
        );
    }

    #[test]
    fn integer_function_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 5,
            ..GridScaleOptions::default_for(GridScaleProfile::IntegerFunctionR1C1, "intr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_int_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_trunc_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_trunc_tens_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_int_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_trunc_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_trunc_tens_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_int_value"],
            "100"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_trunc_value"],
            "100"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_trunc_tens_value"],
            "100"
        );
    }

    #[test]
    fn log_function_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 6,
            ..GridScaleOptions::default_for(GridScaleProfile::LogFunctionR1C1, "logr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            400
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            5
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            600
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            600
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_exp_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_ln_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_log10_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_log_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_exp_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_ln_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_log10_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_log_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_exp_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_ln_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_log10_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_log_value"],
            "2"
        );
    }

    #[test]
    fn trig_function_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::TrigFunctionR1C1, "trigr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            400
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            400
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_sin_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_cos_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_tan_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_sin_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_cos_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_tan_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_sin_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_cos_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_tan_value"],
            "0"
        );
    }

    #[test]
    fn angle_function_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 6,
            ..GridScaleOptions::default_for(GridScaleProfile::AngleFunctionR1C1, "angler1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            400
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            5
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            600
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            600
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_radians_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_degrees_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_sin_degrees_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_pi_value"],
            "3.141592653589793"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_radians_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_degrees_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_sin_degrees_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_pi_value"],
            "3.141592653589793"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_radians_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_degrees_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_sin_degrees_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_pi_value"],
            "3.141592653589793"
        );
    }

    #[test]
    fn reference_function_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 6,
            ..GridScaleOptions::default_for(
                GridScaleProfile::ReferenceFunctionR1C1,
                "referencer1c1",
            )
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["literal_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            600
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            6
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            6
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            6
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            600
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            600
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_row_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_column_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_ref_column_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_rows_value"],
            "3"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_columns_value"],
            "3"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_row_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_ref_row_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_row_value"],
            "100"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_ref_row_value"],
            "100"
        );
    }

    #[test]
    fn logical_function_r1c1_1m_scale_publishes_dense_logical_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 5,
            ..GridScaleOptions::default_for(GridScaleProfile::LogicalFunctionR1C1, "logicalr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_and_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_or_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_not_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["true_sample_row"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["true_sample_and_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["true_sample_or_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["true_sample_not_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_and_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_or_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_not_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_and_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_or_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_not_value"],
            "true"
        );
    }

    #[test]
    fn if_logical_r1c1_1m_scale_publishes_dense_numeric_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 5,
            ..GridScaleOptions::default_for(GridScaleProfile::IfLogicalR1C1, "iflogicalr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_and_if_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_or_if_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_not_if_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["true_sample_row"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["true_sample_and_if_value"],
            "6"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["true_sample_or_if_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["true_sample_not_if_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_and_if_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_or_if_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_not_if_value"],
            "50"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_and_if_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_or_if_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_not_if_value"],
            "100"
        );
    }

    #[test]
    fn two_left_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 10,
            ..GridScaleOptions::default_for(GridScaleProfile::TwoLeftR1C1, "twoleft")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "2015"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "300023"
        );
    }

    #[test]
    fn absolute_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 3,
            ..GridScaleOptions::default_for(GridScaleProfile::AbsoluteR1C1, "absolute")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "3"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "201"
        );
    }

    #[test]
    fn division_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::DivisionR1C1, "division")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "100"
        );
    }

    #[test]
    fn decimal_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::DecimalLiteralR1C1, "decimal")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "100"
        );
    }

    #[test]
    fn recursive_binary_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 5,
            ..GridScaleOptions::default_for(
                GridScaleProfile::RecursiveBinaryR1C1,
                "recursivebinary",
            )
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_precedence_value"],
            "21"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_parenthesized_value"],
            "22"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_precedence_value"],
            "1050"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_parenthesized_value"],
            "1100"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_precedence_value"],
            "2100"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_parenthesized_value"],
            "2200"
        );
    }

    #[test]
    fn if_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::IfR1C1, "ifr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_formula_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["positive_tail_formula_value"],
            "99"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "0"
        );
    }

    #[test]
    fn if_branch_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::IfBranchR1C1, "ifbranchr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_formula_value"],
            "-25"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["positive_tail_formula_value"],
            "198"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "-50"
        );
    }

    #[test]
    fn nested_if_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 3,
            ..GridScaleOptions::default_for(GridScaleProfile::NestedIfR1C1, "nestedifr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_templates_prepared"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["threshold_row"],
            50
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_formula_value"],
            "51"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["after_threshold_row"],
            51
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["after_threshold_formula_value"],
            "153"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "200"
        );
    }

    #[test]
    fn iferror_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 3,
            ..GridScaleOptions::default_for(GridScaleProfile::IfErrorR1C1, "iferrorr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_formula_value"],
            "0"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["positive_tail_formula_value"],
            "99"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "0"
        );
    }

    #[test]
    fn comparison_r1c1_1m_scale_publishes_dense_logical_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::ComparisonR1C1, "comparisonr1c1")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_formula_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["positive_tail_formula_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "false"
        );
    }

    #[test]
    fn comparison_expression_r1c1_1m_scale_publishes_dense_logical_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 3,
            ..GridScaleOptions::default_for(
                GridScaleProfile::ComparisonExpressionR1C1,
                "comparisonexprr1c1",
            )
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_formula_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["positive_tail_formula_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "false"
        );
    }

    #[test]
    fn comparison_iferror_r1c1_1m_scale_publishes_dense_logical_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 3,
            ..GridScaleOptions::default_for(
                GridScaleProfile::ComparisonIfErrorR1C1,
                "comparisoniferrorr1c1",
            )
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_logical_packed_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_formula_value"],
            "false"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["positive_tail_formula_value"],
            "true"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "false"
        );
    }

    #[test]
    fn sum_row_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::SumRowR1C1, "sumrow")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "6"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "600"
        );
    }

    #[test]
    fn sumsq_row_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::SumSqRowR1C1, "sumsqrow")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "14"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "140000"
        );
    }

    #[test]
    fn count_row_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::CountRowR1C1, "countrow")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "3"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "3"
        );
    }

    #[test]
    fn product_row_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::ProductRowR1C1, "productrow")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "6"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "6"
        );
    }

    #[test]
    fn average_row_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::AverageRowR1C1, "averagerow")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "200"
        );
    }

    #[test]
    fn min_max_row_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 5,
            ..GridScaleOptions::default_for(GridScaleProfile::MinMaxRowR1C1, "minmaxrow")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_min_formula_value"],
            "1"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_max_formula_value"],
            "3"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_min_formula_value"],
            "100"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_max_formula_value"],
            "300"
        );
    }

    #[test]
    fn sum_window_r1c1_1m_scale_publishes_dense_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::SumWindowR1C1, "sumwindow")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            98
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            "6"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            "297"
        );
    }

    #[test]
    fn division_error_r1c1_1m_scale_publishes_dense_error_formula_output() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::DivisionErrorR1C1, "division-error")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_formula_value"],
            execution.counter_summary_json["counters"]["expected_formula_error"]
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_formula_value"],
            execution.counter_summary_json["counters"]["expected_formula_error"]
        );
    }

    #[test]
    fn division_error_propagation_r1c1_1m_scale_keeps_error_chain_dense() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 3,
            ..GridScaleOptions::default_for(
                GridScaleProfile::DivisionErrorPropagationR1C1,
                "division-error-prop",
            )
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_direct_error_value"],
            execution.counter_summary_json["counters"]["expected_formula_error"]
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_propagated_error_value"],
            execution.counter_summary_json["counters"]["expected_formula_error"]
        );
    }

    #[test]
    fn aggregate_error_r1c1_1m_scale_keeps_error_and_recovery_dense() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::AggregateErrorR1C1, "aggregate-error")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_aggregate_error_value"],
            execution.counter_summary_json["counters"]["expected_formula_error"]
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_aggregate_error_value"],
            execution.counter_summary_json["counters"]["expected_formula_error"]
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_recovered_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_recovered_value"],
            execution.counter_summary_json["counters"]["expected_last_recovered_value"]
        );
    }

    #[test]
    fn text_function_r1c1_1m_scale_keeps_text_outputs_dense() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 5,
            ..GridScaleOptions::default_for(GridScaleProfile::TextFunctionR1C1, "text-function")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            400
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            5
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            100
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_len_value"],
            "7"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_left_value"],
            "Row"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_right_value"],
            "Grid"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_concat_value"],
            "RowGrid"
        );
    }

    #[test]
    fn index_function_r1c1_1m_scale_keeps_lookup_outputs_dense() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 6,
            ..GridScaleOptions::default_for(GridScaleProfile::IndexFunctionR1C1, "index-function")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            400
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            6
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_numeric_lookup_value"],
            "10"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_text_lookup_value"],
            "Index"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_dynamic_lookup_value"],
            execution.counter_summary_json["counters"]["expected_middle_dynamic_lookup_value"]
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_dynamic_lookup_value"],
            execution.counter_summary_json["counters"]["expected_last_dynamic_lookup_value"]
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_ref_error_value"],
            execution.counter_summary_json["counters"]["expected_ref_error"]
        );
    }

    #[test]
    fn match_function_r1c1_1m_scale_keeps_lookup_outputs_dense() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 6,
            ..GridScaleOptions::default_for(GridScaleProfile::MatchFunctionR1C1, "match-function")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            500
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_match_position_value"],
            "2"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_index_match_value"],
            execution.counter_summary_json["counters"]["expected_middle_index_match_value"]
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_no_match_value"],
            execution.counter_summary_json["counters"]["expected_no_match"]
        );
    }

    #[test]
    fn vlookup_function_r1c1_1m_scale_keeps_lookup_outputs_dense() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 7,
            ..GridScaleOptions::default_for(
                GridScaleProfile::VLookupFunctionR1C1,
                "vlookup-function",
            )
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells"],
            400
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compiled_formula_plans_cached"],
            4
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            7
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            300
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_text_lookup_value"],
            "Lookup"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["middle_numeric_lookup_value"],
            execution.counter_summary_json["counters"]["expected_middle_numeric_lookup_value"]
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["last_no_match_value"],
            execution.counter_summary_json["counters"]["expected_no_match"]
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_ref_error_value"],
            execution.counter_summary_json["counters"]["expected_ref_error"]
        );
    }

    #[test]
    fn plan_cache_rounds_1m_scale_reuses_template_after_first_round() {
        let options = GridScaleOptions {
            rows: 100,
            cols: 10,
            ..GridScaleOptions::default_for(GridScaleProfile::PlanCacheRounds1M, "plancache")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(execution.counter_summary_json["counters"]["rounds"], 3);
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_cells_per_round"],
            200
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_round_misses"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["later_round_misses"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["total_misses"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["total_hits"],
            599
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["cached_compiled_plan_count"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["first_round_compiled_plan_misses"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["later_round_compiled_plan_misses"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["total_compiled_plan_hits"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["total_compiled_plan_misses"],
            1
        );
    }

    #[test]
    fn insert_storm_1m_scale_keeps_structural_edits_region_compact() {
        let options = GridScaleOptions {
            rows: 64,
            cols: 10,
            ..GridScaleOptions::default_for(GridScaleProfile::InsertStorm1M, "insertstorm")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["edits_applied"],
            6
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["actual_deleted_cells"],
            30
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["sparse_point_cells_final"],
            0
        );
        assert!(
            execution.counter_summary_json["counters"]["compact_region_metadata_touches"]
                .as_u64()
                .unwrap()
                < execution.counter_summary_json["counters"]["naive_cell_rewrite_floor"]
                    .as_u64()
                    .unwrap()
        );
    }

    #[test]
    fn cow_retention_1m_scale_shares_dense_payloads_across_retained_roots() {
        let options = GridScaleOptions {
            rows: 64,
            cols: 10,
            ..GridScaleOptions::default_for(GridScaleProfile::CowRetention1M, "cowretention")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["retained_revision_count"],
            7
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["unique_dense_payloads"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["sparse_point_cells_final"],
            0
        );
        assert!(
            execution.counter_summary_json["counters"]["cow_retained_bytes"]
                .as_u64()
                .unwrap()
                < execution.counter_summary_json["counters"]
                    ["naive_full_snapshot_retention_bytes_floor"]
                    .as_u64()
                    .unwrap()
        );
    }

    #[test]
    fn publication_delta_1m_scale_reports_compact_delta_entries() {
        let options = GridScaleOptions {
            rows: 2_000,
            cols: 2,
            ..GridScaleOptions::default_for(
                GridScaleProfile::PublicationDelta1M,
                "publicationdelta",
            )
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["publication_entries_total"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["dense_region_entries_changed"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["naive_current_computed_cell_publication_floor"],
            4_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["previous_changed_input"],
            "1000"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["current_changed_input"],
            "11000"
        );
        assert!(
            execution.counter_summary_json["counters"]["publication_entries_total"]
                .as_u64()
                .unwrap()
                < execution.counter_summary_json["counters"]
                    ["naive_current_computed_cell_publication_floor"]
                    .as_u64()
                .unwrap()
        );
    }

    #[test]
    fn tile_stream_64k_scale_reports_tile_bounded_frame() {
        let options = GridScaleOptions::default_for(GridScaleProfile::TileStream64K, "tilestream");
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["tile_subscribed_cells"],
            64_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["dense_value_cells_visited"],
            64_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["sparse_value_cells_visited"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["unrelated_sparse_cells"],
            1_000
        );
        assert!(
            execution.counter_summary_json["counters"]["estimated_frame_bytes"]
                .as_u64()
                .unwrap()
                < execution.counter_summary_json["counters"]["full_grid_dense_numeric_bytes_floor"]
                    .as_u64()
                    .unwrap()
        );
    }

    #[test]
    fn viewport_64k_of_1m_scale_evaluates_visible_upstream_cone() {
        let options = GridScaleOptions::default_for(GridScaleProfile::Viewport64K, "viewport");
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["visible_cell_count"],
            64_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["visible_upstream_cell_count"],
            192_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["cells_evaluated_before_visible_complete"],
            192_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["formula_evaluations_before_visible_complete"],
            128_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["snapshot_sparse_value_cells_visited"],
            0
        );
        assert!(
            execution.counter_summary_json["counters"]["cells_evaluated_before_visible_complete"]
                .as_u64()
                .unwrap()
                < execution.counter_summary_json["counters"]["full_recalc_occupied_cell_floor"]
                    .as_u64()
                    .unwrap()
        );
    }

    #[test]
    fn range_invalidation_1m_scale_keeps_reverse_edges_compressed() {
        let options = GridScaleOptions {
            rows: 2_000,
            cols: 3,
            ..GridScaleOptions::default_for(GridScaleProfile::RangeInvalidation1M, "rangeinv")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["expanded_scalar_edge_floor"],
            2_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["installed_range_scalar_edges"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["compressed_range_edges"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["dirty_closure_size"],
            3
        );
    }

    #[test]
    fn range_query_1m_scale_uses_compressed_range_index() {
        let options = GridScaleOptions::default_for(GridScaleProfile::RangeQuery1M, "rangequery");
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["total_compressed_range_edges"],
            1_000
        );
        assert!(
            execution.counter_summary_json["counters"]["indexed_candidate_count"]
                .as_u64()
                .unwrap()
                < execution.counter_summary_json["counters"]["naive_candidate_floor"]
                    .as_u64()
                    .unwrap()
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["matched_dependent_count"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["dirty_closure_size"],
            3
        );
    }

    #[test]
    fn sum_pyramid_1m_scale_keeps_pyramid_ranges_compressed() {
        let options = GridScaleOptions::default_for(GridScaleProfile::SumPyramid1M, "sumpyramid");
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["total_compressed_range_edges"],
            1_111
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["installed_range_scalar_edges"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["scalar_edges_total"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["dirty_closure_size"],
            6
        );
        assert!(
            execution.counter_summary_json["counters"]["indexed_candidate_sum"]
                .as_u64()
                .unwrap()
                < execution.counter_summary_json["counters"]["total_compressed_range_edges"]
                    .as_u64()
                    .unwrap()
        );
    }

    #[test]
    fn dirty_rect_1m_scale_queries_dirty_rectangle_through_indexes() {
        let options = GridScaleOptions::default_for(GridScaleProfile::DirtyRect1M, "dirtyrect");
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["dirty_rect_cell_count"],
            11
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["matched_scalar_dependent_count"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["matched_compressed_range_dependent_count"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["dirty_closure_size"],
            3
        );
        assert!(
            execution.counter_summary_json["counters"]["indexed_candidate_sum"]
                .as_u64()
                .unwrap()
                < execution.counter_summary_json["counters"]["total_edge_count"]
                    .as_u64()
                    .unwrap()
        );
    }

    #[test]
    fn hide_storm_1m_scale_uses_axis_visibility_index() {
        let options = GridScaleOptions::default_for(GridScaleProfile::HideStorm1M, "hidestorm");
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["total_axis_visibility_edges"],
            1_000
        );
        assert!(
            execution.counter_summary_json["counters"]["indexed_candidate_count"]
                .as_u64()
                .unwrap()
                < execution.counter_summary_json["counters"]["naive_candidate_floor"]
                    .as_u64()
                    .unwrap()
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["matched_dependent_count"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["dirty_closure_size"],
            2
        );
    }

    #[test]
    fn spill_anchor_1m_scale_reports_one_ledger_probe() {
        let options = GridScaleOptions {
            rows: 2_000,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::SpillAnchor1M, "spillanchor")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["spill_extent_declared_cells"],
            2_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["spill_ledger_probe_count"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["spill_extent_cells_scanned_for_ledger"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["provider_value_entries_scanned"],
            3
        );
    }

    #[test]
    fn spill_epoch_1m_scale_reports_epoch_precise_dirty_closure() {
        let options = GridScaleOptions {
            rows: 2_000,
            cols: 4,
            ..GridScaleOptions::default_for(GridScaleProfile::SpillEpoch1M, "spillepoch")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["spill_extent_declared_cells"],
            2_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["unchanged_dirty_closure_size"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["unrelated_dirty_closure_size"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["value_dirty_closure_size"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["extent_dirty_closure_size"],
            2
        );
    }

    #[test]
    fn spill_blockage_1m_scale_checks_compact_blockers_only() {
        let options = GridScaleOptions {
            rows: 2_000,
            cols: 2,
            ..GridScaleOptions::default_for(GridScaleProfile::SpillBlockage1M, "spillblockage")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["spill_extent_declared_cells"],
            2_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["empty_compact_blocker_probe_count"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["far_blocker_compact_blocker_probe_count"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["far_blocker_sparse_point_candidates"],
            1
        );
    }

    #[test]
    fn filter_spill_1m_scale_reports_indexed_respill_clear() {
        let options = GridScaleOptions {
            rows: 2_000,
            cols: 5,
            ..GridScaleOptions::default_for(GridScaleProfile::FilterSpill1M, "filterspill")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["old_spill_extent_declared_cells"],
            2_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["indexed_clear_candidate_count"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["sparse_values_removed"],
            3
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["sparse_values_after_clear"],
            1_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_formula_spill_extent_declared_rows"],
            1_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_formula_spill_extent_declared_cols"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_formula_computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_formula_last_right_value"],
            "100002"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_lifecycle_spill_extent_declared_rows"],
            999
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_lifecycle_spill_extent_declared_cols"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_lifecycle_spill_commit_previous_fact_entries"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_lifecycle_spill_commit_extent_changed_anchors"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_lifecycle_spill_commit_value_changed_anchors"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_lifecycle_committed_value_epoch"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_lifecycle_computed_sparse_cells"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_lifecycle_last_right_value"],
            "99902"
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["filter_lifecycle_vacated_right_value"],
            ""
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["column_filter_spill_extent_declared_rows"],
            1_999
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["column_filter_spill_extent_declared_cols"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["column_filter_spill_commit_committed_fact_entries"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["column_filter_computed_sparse_cells"],
            0
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["column_filter_last_right_value"],
            "199903"
        );
    }

    #[test]
    fn sequence_spill_1m_scale_publishes_dense_computed_region() {
        let options = GridScaleOptions {
            rows: 2_000,
            cols: 1,
            ..GridScaleOptions::default_for(GridScaleProfile::SequenceSpill1M, "seqspill")
        };
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["spill_extent_declared_cells"],
            2_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_value_regions"],
            1
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_dense_numeric_packed_cells"],
            2_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["computed_sparse_cells"],
            0
        );
    }

    #[test]
    fn aggregate_context_1m_scale_reports_axis_run_probes() {
        let options =
            GridScaleOptions::default_for(GridScaleProfile::AggregateContext1M, "aggcontext");
        let execution = execute_grid_scale_model(&options, "artifact/root").unwrap();
        assert_eq!(execution.summary.failed_register_assertion_count, 0);
        assert_eq!(
            execution.counter_summary_json["counters"]["aggregate_reference_declared_cells"],
            1_000_000
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["explicit_axis_row_entries_visited"],
            2
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["axis_run_probe_count"],
            5
        );
        assert_eq!(
            execution.counter_summary_json["counters"]["per_cell_context_expansion_count"],
            1_000_000
        );
    }
}
