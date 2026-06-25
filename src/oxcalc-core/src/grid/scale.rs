#![forbid(unsafe_code)]

//! Procedural scale/performance runner for the W061 strict Excel grid lane.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use oxfunc_core::value::{CalcValue, ExcelText, ReferenceKind, ReferenceLike, WorksheetErrorCode};
use serde_json::{Value, json};
use thiserror::Error;

use crate::coordinator::calc_value_display_text;
use crate::grid::authored::{GridAuthoredCell, GridFormulaCell};
use crate::grid::coords::{ExcelGridBounds, ExcelGridCellAddress};
use crate::grid::error::GridRefError;
use crate::grid::geometry::GridRect;
use crate::grid::machine::{
    GridAxisEdit, GridAxisProps, GridAxisVisibilityDependency, GridDependency, GridEngineMode,
    GridInvalidationRef, GridOptimizedCellSource, GridOptimizedSheet,
    GridOptimizedStructuralEditReport, GridOptimizedValuation, GridSpillDependency,
    GridSpillEpochLedger, GridSpillFact,
};
use crate::grid::reference_engine::ExcelGridReferenceSystemProvider;
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

mod scenarios_aggregate;
mod scenarios_layout;
mod scenarios_logical;
mod scenarios_lookup;
mod scenarios_r1c1_scalar;
mod scenarios_special;
use scenarios_aggregate::*;
use scenarios_layout::*;
use scenarios_logical::*;
use scenarios_lookup::*;
use scenarios_r1c1_scalar::*;
use scenarios_special::*;

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
