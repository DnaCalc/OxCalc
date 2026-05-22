#![forbid(unsafe_code)]

//! Sparse worksheet/reference reader substrate for W051.

use std::cell::Cell;
use std::collections::BTreeMap;

use oxfunc_core::value::{EvalValue, ExcelText};
use thiserror::Error;

use crate::formula::{
    TreeCalcChildrenReferenceCollection, TreeCalcOrderedSelectorReferenceCollection,
};
use crate::structural::{StructuralSnapshot, TreeNodeId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SparseCellCoord {
    pub row: u32,
    pub column: u32,
}

impl SparseCellCoord {
    #[must_use]
    pub const fn new(row: u32, column: u32) -> Self {
        Self { row, column }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SparseRangeExtent {
    pub start: SparseCellCoord,
    pub row_count: u32,
    pub column_count: u32,
}

impl SparseRangeExtent {
    #[must_use]
    pub const fn new(start: SparseCellCoord, row_count: u32, column_count: u32) -> Self {
        Self {
            start,
            row_count,
            column_count,
        }
    }

    #[must_use]
    pub fn contains(self, coord: SparseCellCoord) -> bool {
        let Some(row_end) = self.start.row.checked_add(self.row_count) else {
            return false;
        };
        let Some(column_end) = self.start.column.checked_add(self.column_count) else {
            return false;
        };

        coord.row >= self.start.row
            && coord.row < row_end
            && coord.column >= self.start.column
            && coord.column < column_end
    }

    #[must_use]
    pub fn declared_cell_count(self) -> u64 {
        u64::from(self.row_count) * u64::from(self.column_count)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SparseReaderIdentity {
    pub reader_id: String,
    pub source_identity: String,
    pub snapshot_identity: String,
}

impl SparseReaderIdentity {
    #[must_use]
    pub fn new(
        reader_id: impl Into<String>,
        source_identity: impl Into<String>,
        snapshot_identity: impl Into<String>,
    ) -> Self {
        Self {
            reader_id: reader_id.into(),
            source_identity: source_identity.into(),
            snapshot_identity: snapshot_identity.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SparseDefinedCell {
    pub coord: SparseCellCoord,
    pub value: EvalValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SparseCellRead {
    Defined(EvalValue),
    Blank,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SparseReaderAccessSummary {
    pub contains_calls: usize,
    pub read_at_calls: usize,
    pub defined_iter_calls: usize,
    pub defined_iter_yield_count: usize,
}

#[derive(Debug, Default)]
struct SparseReaderAccessTelemetry {
    contains_calls: Cell<usize>,
    read_at_calls: Cell<usize>,
    defined_iter_calls: Cell<usize>,
    defined_iter_yield_count: Cell<usize>,
}

impl SparseReaderAccessTelemetry {
    fn record_contains(&self) {
        self.contains_calls.set(self.contains_calls.get() + 1);
    }

    fn record_read_at(&self) {
        self.read_at_calls.set(self.read_at_calls.get() + 1);
    }

    fn record_defined_iter(&self) {
        self.defined_iter_calls
            .set(self.defined_iter_calls.get() + 1);
    }

    fn record_defined_yield(&self) {
        self.defined_iter_yield_count
            .set(self.defined_iter_yield_count.get() + 1);
    }

    fn summary(&self) -> SparseReaderAccessSummary {
        SparseReaderAccessSummary {
            contains_calls: self.contains_calls.get(),
            read_at_calls: self.read_at_calls.get(),
            defined_iter_calls: self.defined_iter_calls.get(),
            defined_iter_yield_count: self.defined_iter_yield_count.get(),
        }
    }
}

pub trait SparseRangeReader {
    fn reader_identity(&self) -> &SparseReaderIdentity;
    fn declared_extent(&self) -> SparseRangeExtent;
    fn defined_cardinality(&self) -> usize;
    fn defined_iter(&self) -> Box<dyn Iterator<Item = SparseDefinedCell> + '_>;
    fn read_at(&self, coord: SparseCellCoord) -> SparseCellRead;
    fn contains(&self, coord: SparseCellCoord) -> bool;
}

#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum SparseReaderError {
    #[error(
        "defined cell ({row},{column}) is outside declared sparse extent starting at ({start_row},{start_column}) with {row_count} rows and {column_count} columns"
    )]
    DefinedCellOutsideExtent {
        row: u32,
        column: u32,
        start_row: u32,
        start_column: u32,
        row_count: u32,
        column_count: u32,
    },
    #[error("TreeCalc children collection base node {base_node_id} is not present")]
    UnknownTreeCalcBase { base_node_id: TreeNodeId },
    #[error("TreeCalc children collection for base node {base_node_id} has too many members")]
    TreeCalcMemberOrdinalOverflow { base_node_id: TreeNodeId },
}

#[derive(Debug)]
pub struct WorksheetSparseRangeReader {
    identity: SparseReaderIdentity,
    extent: SparseRangeExtent,
    defined_cells: BTreeMap<SparseCellCoord, EvalValue>,
    telemetry: SparseReaderAccessTelemetry,
}

impl WorksheetSparseRangeReader {
    pub fn new(
        identity: SparseReaderIdentity,
        extent: SparseRangeExtent,
        defined_cells: impl IntoIterator<Item = (SparseCellCoord, EvalValue)>,
    ) -> Result<Self, SparseReaderError> {
        let mut indexed = BTreeMap::new();
        for (coord, value) in defined_cells {
            if !extent.contains(coord) {
                return Err(SparseReaderError::DefinedCellOutsideExtent {
                    row: coord.row,
                    column: coord.column,
                    start_row: extent.start.row,
                    start_column: extent.start.column,
                    row_count: extent.row_count,
                    column_count: extent.column_count,
                });
            }
            indexed.insert(coord, value);
        }

        Ok(Self {
            identity,
            extent,
            defined_cells: indexed,
            telemetry: SparseReaderAccessTelemetry::default(),
        })
    }

    #[must_use]
    pub fn access_summary(&self) -> SparseReaderAccessSummary {
        self.telemetry.summary()
    }
}

impl SparseRangeReader for WorksheetSparseRangeReader {
    fn reader_identity(&self) -> &SparseReaderIdentity {
        &self.identity
    }

    fn declared_extent(&self) -> SparseRangeExtent {
        self.extent
    }

    fn defined_cardinality(&self) -> usize {
        self.defined_cells.len()
    }

    fn defined_iter(&self) -> Box<dyn Iterator<Item = SparseDefinedCell> + '_> {
        self.telemetry.record_defined_iter();
        Box::new(self.defined_cells.iter().map(|(coord, value)| {
            self.telemetry.record_defined_yield();
            SparseDefinedCell {
                coord: *coord,
                value: value.clone(),
            }
        }))
    }

    fn read_at(&self, coord: SparseCellCoord) -> SparseCellRead {
        self.telemetry.record_read_at();
        if !self.extent.contains(coord) {
            return SparseCellRead::Blank;
        }
        self.defined_cells
            .get(&coord)
            .cloned()
            .map_or(SparseCellRead::Blank, SparseCellRead::Defined)
    }

    fn contains(&self, coord: SparseCellCoord) -> bool {
        self.telemetry.record_contains();
        self.extent.contains(coord)
    }
}

pub struct TreeCalcChildrenSparseReader {
    identity: SparseReaderIdentity,
    extent: SparseRangeExtent,
    collection: TreeCalcChildrenReferenceCollection,
    member_node_ids: Vec<TreeNodeId>,
    member_values: BTreeMap<TreeNodeId, EvalValue>,
    telemetry: SparseReaderAccessTelemetry,
}

pub struct TreeCalcOrderedSelectorSparseReader {
    identity: SparseReaderIdentity,
    extent: SparseRangeExtent,
    collection: TreeCalcOrderedSelectorReferenceCollection,
    member_values: BTreeMap<TreeNodeId, EvalValue>,
    telemetry: SparseReaderAccessTelemetry,
}

impl std::fmt::Debug for TreeCalcOrderedSelectorSparseReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeCalcOrderedSelectorSparseReader")
            .field("identity", &self.identity)
            .field("extent", &self.extent)
            .field("collection", &self.collection)
            .field("member_values", &self.member_values)
            .finish_non_exhaustive()
    }
}

impl TreeCalcOrderedSelectorSparseReader {
    pub fn new(
        structural_snapshot: &StructuralSnapshot,
        collection: TreeCalcOrderedSelectorReferenceCollection,
        member_values: impl IntoIterator<Item = (TreeNodeId, EvalValue)>,
    ) -> Result<Self, SparseReaderError> {
        structural_snapshot
            .try_get_node(collection.base_node_id)
            .ok_or(SparseReaderError::UnknownTreeCalcBase {
                base_node_id: collection.base_node_id,
            })?;
        let row_count = u32::try_from(collection.member_node_ids.len()).map_err(|_| {
            SparseReaderError::TreeCalcMemberOrdinalOverflow {
                base_node_id: collection.base_node_id,
            }
        })?;
        let admitted_members = collection
            .member_node_ids
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let member_values = member_values
            .into_iter()
            .filter(|(node_id, _)| admitted_members.contains(node_id))
            .collect::<BTreeMap<_, _>>();
        let identity = SparseReaderIdentity::new(
            collection.host_ref_handle.clone(),
            collection.opaque_selector.clone(),
            format!(
                "snapshot={};membership={};order={}",
                structural_snapshot.snapshot_id(),
                collection.membership_version,
                collection.order_version
            ),
        );

        Ok(Self {
            identity,
            extent: SparseRangeExtent::new(SparseCellCoord::new(1, 1), row_count, 1),
            collection,
            member_values,
            telemetry: SparseReaderAccessTelemetry::default(),
        })
    }

    pub fn from_published_values(
        structural_snapshot: &StructuralSnapshot,
        collection: TreeCalcOrderedSelectorReferenceCollection,
        published_values: &BTreeMap<TreeNodeId, String>,
    ) -> Result<Self, SparseReaderError> {
        Self::new(
            structural_snapshot,
            collection,
            published_values
                .iter()
                .map(|(node_id, value)| (*node_id, treecalc_published_value_to_eval_value(value))),
        )
    }

    #[must_use]
    pub fn collection(&self) -> &TreeCalcOrderedSelectorReferenceCollection {
        &self.collection
    }

    #[must_use]
    pub fn member_node_ids(&self) -> &[TreeNodeId] {
        &self.collection.member_node_ids
    }

    #[must_use]
    pub fn member_node_id_at(&self, coord: SparseCellCoord) -> Option<TreeNodeId> {
        if !self.extent.contains(coord) || coord.column != 1 {
            return None;
        }
        let ordinal = usize::try_from(coord.row.checked_sub(1)?).ok()?;
        self.collection.member_node_ids.get(ordinal).copied()
    }

    #[must_use]
    pub fn access_summary(&self) -> SparseReaderAccessSummary {
        self.telemetry.summary()
    }
}

impl SparseRangeReader for TreeCalcOrderedSelectorSparseReader {
    fn reader_identity(&self) -> &SparseReaderIdentity {
        &self.identity
    }

    fn declared_extent(&self) -> SparseRangeExtent {
        self.extent
    }

    fn defined_cardinality(&self) -> usize {
        self.collection
            .member_node_ids
            .iter()
            .filter(|node_id| self.member_values.contains_key(node_id))
            .count()
    }

    fn defined_iter(&self) -> Box<dyn Iterator<Item = SparseDefinedCell> + '_> {
        self.telemetry.record_defined_iter();
        Box::new(
            self.collection
                .member_node_ids
                .iter()
                .enumerate()
                .filter_map(|(index, node_id)| {
                    let value = self.member_values.get(node_id)?.clone();
                    self.telemetry.record_defined_yield();
                    Some(SparseDefinedCell {
                        coord: SparseCellCoord::new(
                            u32::try_from(index + 1).expect("member count was u32-checked"),
                            1,
                        ),
                        value,
                    })
                }),
        )
    }

    fn read_at(&self, coord: SparseCellCoord) -> SparseCellRead {
        self.telemetry.record_read_at();
        self.member_node_id_at(coord)
            .and_then(|node_id| self.member_values.get(&node_id))
            .cloned()
            .map_or(SparseCellRead::Blank, SparseCellRead::Defined)
    }

    fn contains(&self, coord: SparseCellCoord) -> bool {
        self.telemetry.record_contains();
        self.extent.contains(coord)
    }
}

impl std::fmt::Debug for TreeCalcChildrenSparseReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TreeCalcChildrenSparseReader")
            .field("identity", &self.identity)
            .field("extent", &self.extent)
            .field("collection", &self.collection)
            .field("member_node_ids", &self.member_node_ids)
            .field("member_values", &self.member_values)
            .finish_non_exhaustive()
    }
}

impl TreeCalcChildrenSparseReader {
    pub fn new(
        structural_snapshot: &StructuralSnapshot,
        collection: TreeCalcChildrenReferenceCollection,
        member_values: impl IntoIterator<Item = (TreeNodeId, EvalValue)>,
    ) -> Result<Self, SparseReaderError> {
        let base_node = structural_snapshot
            .try_get_node(collection.base_node_id)
            .ok_or(SparseReaderError::UnknownTreeCalcBase {
                base_node_id: collection.base_node_id,
            })?;
        let row_count = u32::try_from(base_node.child_ids.len()).map_err(|_| {
            SparseReaderError::TreeCalcMemberOrdinalOverflow {
                base_node_id: collection.base_node_id,
            }
        })?;
        let member_node_ids = base_node.child_ids.clone();
        let admitted_members = member_node_ids
            .iter()
            .copied()
            .collect::<std::collections::BTreeSet<_>>();
        let member_values = member_values
            .into_iter()
            .filter(|(node_id, _)| admitted_members.contains(node_id))
            .collect::<BTreeMap<_, _>>();
        let identity = SparseReaderIdentity::new(
            collection.host_ref_handle.clone(),
            collection.opaque_selector.clone(),
            format!(
                "snapshot={};membership={};order={}",
                structural_snapshot.snapshot_id(),
                collection.membership_version,
                collection.order_version
            ),
        );

        Ok(Self {
            identity,
            extent: SparseRangeExtent::new(SparseCellCoord::new(1, 1), row_count, 1),
            collection,
            member_node_ids,
            member_values,
            telemetry: SparseReaderAccessTelemetry::default(),
        })
    }

    pub fn from_published_values(
        structural_snapshot: &StructuralSnapshot,
        collection: TreeCalcChildrenReferenceCollection,
        published_values: &BTreeMap<TreeNodeId, String>,
    ) -> Result<Self, SparseReaderError> {
        Self::new(
            structural_snapshot,
            collection,
            published_values
                .iter()
                .map(|(node_id, value)| (*node_id, treecalc_published_value_to_eval_value(value))),
        )
    }

    #[must_use]
    pub fn collection(&self) -> &TreeCalcChildrenReferenceCollection {
        &self.collection
    }

    #[must_use]
    pub fn member_node_ids(&self) -> &[TreeNodeId] {
        &self.member_node_ids
    }

    #[must_use]
    pub fn member_node_id_at(&self, coord: SparseCellCoord) -> Option<TreeNodeId> {
        if !self.extent.contains(coord) || coord.column != 1 {
            return None;
        }
        let ordinal = usize::try_from(coord.row.checked_sub(1)?).ok()?;
        self.member_node_ids.get(ordinal).copied()
    }

    #[must_use]
    pub fn access_summary(&self) -> SparseReaderAccessSummary {
        self.telemetry.summary()
    }
}

impl SparseRangeReader for TreeCalcChildrenSparseReader {
    fn reader_identity(&self) -> &SparseReaderIdentity {
        &self.identity
    }

    fn declared_extent(&self) -> SparseRangeExtent {
        self.extent
    }

    fn defined_cardinality(&self) -> usize {
        self.member_node_ids
            .iter()
            .filter(|node_id| self.member_values.contains_key(node_id))
            .count()
    }

    fn defined_iter(&self) -> Box<dyn Iterator<Item = SparseDefinedCell> + '_> {
        self.telemetry.record_defined_iter();
        Box::new(
            self.member_node_ids
                .iter()
                .enumerate()
                .filter_map(|(index, node_id)| {
                    let value = self.member_values.get(node_id)?.clone();
                    self.telemetry.record_defined_yield();
                    Some(SparseDefinedCell {
                        coord: SparseCellCoord::new(
                            u32::try_from(index + 1).expect("member count was u32-checked"),
                            1,
                        ),
                        value,
                    })
                }),
        )
    }

    fn read_at(&self, coord: SparseCellCoord) -> SparseCellRead {
        self.telemetry.record_read_at();
        self.member_node_id_at(coord)
            .and_then(|node_id| self.member_values.get(&node_id))
            .cloned()
            .map_or(SparseCellRead::Blank, SparseCellRead::Defined)
    }

    fn contains(&self, coord: SparseCellCoord) -> bool {
        self.telemetry.record_contains();
        self.extent.contains(coord)
    }
}

fn treecalc_published_value_to_eval_value(value: &str) -> EvalValue {
    if let Ok(number) = value.parse::<f64>() {
        EvalValue::Number(number)
    } else if let Ok(logical) = value.parse::<bool>() {
        EvalValue::Logical(logical)
    } else {
        EvalValue::Text(ExcelText::from_interop_assignment(value))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use oxfunc_core::value::{EvalValue, ExcelText};

    use crate::formula::{
        TreeCalcChildrenReferenceCollection, TreeCalcOrderedSelectorFamily,
        TreeCalcOrderedSelectorReferenceCollection,
    };
    use crate::structural::{
        StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
    };

    use super::{
        SparseCellCoord, SparseCellRead, SparseDefinedCell, SparseRangeExtent, SparseRangeReader,
        SparseReaderError, SparseReaderIdentity, TreeCalcChildrenSparseReader,
        TreeCalcOrderedSelectorSparseReader, WorksheetSparseRangeReader,
    };

    fn identity() -> SparseReaderIdentity {
        SparseReaderIdentity::new(
            "reader:sheet1:a:c",
            "worksheet:Sheet1!A:C",
            "snapshot:worksheet:v1",
        )
    }

    #[test]
    fn worksheet_sparse_reader_preserves_defined_and_blank_cells() {
        let reader = WorksheetSparseRangeReader::new(
            identity(),
            SparseRangeExtent::new(SparseCellCoord::new(1, 1), 10, 3),
            [
                (SparseCellCoord::new(1, 1), EvalValue::Number(10.0)),
                (
                    SparseCellCoord::new(5, 2),
                    EvalValue::Text(ExcelText::from_interop_assignment("")),
                ),
                (SparseCellCoord::new(10, 3), EvalValue::Logical(false)),
            ],
        )
        .unwrap();

        assert_eq!(reader.declared_extent().declared_cell_count(), 30);
        assert_eq!(reader.defined_cardinality(), 3);
        assert!(reader.contains(SparseCellCoord::new(10, 3)));
        assert!(!reader.contains(SparseCellCoord::new(11, 3)));
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(EvalValue::Number(10.0))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(5, 2)),
            SparseCellRead::Defined(EvalValue::Text(ExcelText::from_interop_assignment("")))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(3, 2)),
            SparseCellRead::Blank
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(11, 3)),
            SparseCellRead::Blank
        );
    }

    #[test]
    fn worksheet_sparse_reader_iterates_defined_cells_deterministically() {
        let reader = WorksheetSparseRangeReader::new(
            identity(),
            SparseRangeExtent::new(SparseCellCoord::new(1, 1), 20, 20),
            [
                (SparseCellCoord::new(10, 4), EvalValue::Number(10.0)),
                (SparseCellCoord::new(2, 4), EvalValue::Number(2.0)),
                (SparseCellCoord::new(2, 2), EvalValue::Number(1.0)),
            ],
        )
        .unwrap();

        let cells = reader.defined_iter().collect::<Vec<_>>();
        assert_eq!(
            cells,
            vec![
                SparseDefinedCell {
                    coord: SparseCellCoord::new(2, 2),
                    value: EvalValue::Number(1.0),
                },
                SparseDefinedCell {
                    coord: SparseCellCoord::new(2, 4),
                    value: EvalValue::Number(2.0),
                },
                SparseDefinedCell {
                    coord: SparseCellCoord::new(10, 4),
                    value: EvalValue::Number(10.0),
                },
            ]
        );
    }

    #[test]
    fn worksheet_sparse_reader_rejects_defined_cells_outside_extent() {
        let err = WorksheetSparseRangeReader::new(
            identity(),
            SparseRangeExtent::new(SparseCellCoord::new(1, 1), 1, 1),
            [(SparseCellCoord::new(2, 1), EvalValue::Number(1.0))],
        )
        .unwrap_err();

        assert_eq!(
            err,
            SparseReaderError::DefinedCellOutsideExtent {
                row: 2,
                column: 1,
                start_row: 1,
                start_column: 1,
                row_count: 1,
                column_count: 1,
            }
        );
    }

    #[test]
    fn worksheet_sparse_reader_exposes_non_dense_large_range_evidence() {
        let reader = WorksheetSparseRangeReader::new(
            SparseReaderIdentity::new(
                "reader:sheet1:a:a",
                "worksheet:Sheet1!A:A",
                "snapshot:large:v1",
            ),
            SparseRangeExtent::new(SparseCellCoord::new(1, 1), 1_048_576, 1),
            [
                (SparseCellCoord::new(1, 1), EvalValue::Number(1.0)),
                (SparseCellCoord::new(1_048_576, 1), EvalValue::Number(2.0)),
            ],
        )
        .unwrap();

        assert_eq!(reader.declared_extent().declared_cell_count(), 1_048_576);
        assert_eq!(reader.defined_cardinality(), 2);

        let sum = reader
            .defined_iter()
            .map(|cell| match cell.value {
                EvalValue::Number(value) => value,
                _ => 0.0,
            })
            .sum::<f64>();

        assert_eq!(sum, 3.0);
        assert_eq!(reader.access_summary().defined_iter_calls, 1);
        assert_eq!(reader.access_summary().defined_iter_yield_count, 2);
        assert_eq!(reader.access_summary().read_at_calls, 0);
        assert_eq!(reader.access_summary().contains_calls, 0);
    }

    fn node(
        node_id: u64,
        kind: StructuralNodeKind,
        symbol: &str,
        parent_id: Option<u64>,
        child_ids: &[u64],
    ) -> StructuralNode {
        StructuralNode {
            node_id: TreeNodeId(node_id),
            kind,
            symbol: symbol.to_string(),
            parent_id: parent_id.map(TreeNodeId),
            child_ids: child_ids.iter().copied().map(TreeNodeId).collect(),
            formula_artifact_id: None,
            bind_artifact_id: None,
            constant_value: None,
        }
    }

    fn tree_snapshot(child_ids: &[u64]) -> StructuralSnapshot {
        let mut nodes = vec![
            node(1, StructuralNodeKind::Root, "Root", None, &[2]),
            node(2, StructuralNodeKind::Container, "Base", Some(1), child_ids),
        ];
        nodes.extend(child_ids.iter().map(|child_id| {
            node(
                *child_id,
                StructuralNodeKind::Calculation,
                &format!("Child{child_id}"),
                Some(2),
                &[],
            )
        }));
        StructuralSnapshot::create(StructuralSnapshotId(1), TreeNodeId(1), nodes).unwrap()
    }

    fn children_collection() -> TreeCalcChildrenReferenceCollection {
        TreeCalcChildrenReferenceCollection::new(TreeNodeId(2), "@CHILDREN")
    }

    #[test]
    fn treecalc_children_reader_projects_ordered_members_as_sparse_range() {
        let snapshot = tree_snapshot(&[3, 4, 5]);
        let reader = TreeCalcChildrenSparseReader::new(
            &snapshot,
            children_collection(),
            [
                (TreeNodeId(3), EvalValue::Number(10.0)),
                (
                    TreeNodeId(4),
                    EvalValue::Text(ExcelText::from_interop_assignment("")),
                ),
            ],
        )
        .unwrap();

        assert_eq!(
            reader.member_node_ids(),
            &[TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)]
        );
        assert_eq!(reader.declared_extent().row_count, 3);
        assert_eq!(reader.declared_extent().column_count, 1);
        assert_eq!(reader.defined_cardinality(), 2);
        assert_eq!(
            reader.reader_identity().reader_id,
            "treecalc-hostref:v1:children:node:2"
        );
        assert_eq!(
            reader.member_node_id_at(SparseCellCoord::new(2, 1)),
            Some(TreeNodeId(4))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(EvalValue::Number(10.0))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(2, 1)),
            SparseCellRead::Defined(EvalValue::Text(ExcelText::from_interop_assignment("")))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(3, 1)),
            SparseCellRead::Blank
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 2)),
            SparseCellRead::Blank
        );
    }

    #[test]
    fn treecalc_children_reader_iterates_defined_values_in_member_order() {
        let snapshot = tree_snapshot(&[5, 3, 4]);
        let reader = TreeCalcChildrenSparseReader::new(
            &snapshot,
            children_collection(),
            [
                (TreeNodeId(3), EvalValue::Number(30.0)),
                (TreeNodeId(4), EvalValue::Number(40.0)),
                (TreeNodeId(5), EvalValue::Number(50.0)),
                (TreeNodeId(99), EvalValue::Number(990.0)),
            ],
        )
        .unwrap();

        let cells = reader.defined_iter().collect::<Vec<_>>();
        assert_eq!(
            cells,
            vec![
                SparseDefinedCell {
                    coord: SparseCellCoord::new(1, 1),
                    value: EvalValue::Number(50.0),
                },
                SparseDefinedCell {
                    coord: SparseCellCoord::new(2, 1),
                    value: EvalValue::Number(30.0),
                },
                SparseDefinedCell {
                    coord: SparseCellCoord::new(3, 1),
                    value: EvalValue::Number(40.0),
                },
            ]
        );
        assert_eq!(reader.access_summary().defined_iter_calls, 1);
        assert_eq!(reader.access_summary().defined_iter_yield_count, 3);
    }

    #[test]
    fn treecalc_children_reader_reflects_membership_add_remove_and_reorder() {
        let initial = TreeCalcChildrenSparseReader::new(
            &tree_snapshot(&[3, 4]),
            children_collection(),
            [
                (TreeNodeId(3), EvalValue::Number(3.0)),
                (TreeNodeId(4), EvalValue::Number(4.0)),
                (TreeNodeId(5), EvalValue::Number(5.0)),
            ],
        )
        .unwrap();
        let added = TreeCalcChildrenSparseReader::new(
            &tree_snapshot(&[3, 4, 5]),
            children_collection(),
            [
                (TreeNodeId(3), EvalValue::Number(3.0)),
                (TreeNodeId(4), EvalValue::Number(4.0)),
                (TreeNodeId(5), EvalValue::Number(5.0)),
            ],
        )
        .unwrap();
        let reordered = TreeCalcChildrenSparseReader::new(
            &tree_snapshot(&[4, 3]),
            children_collection(),
            [
                (TreeNodeId(3), EvalValue::Number(3.0)),
                (TreeNodeId(4), EvalValue::Number(4.0)),
            ],
        )
        .unwrap();

        assert_eq!(initial.member_node_ids(), &[TreeNodeId(3), TreeNodeId(4)]);
        assert_eq!(initial.defined_cardinality(), 2);
        assert_eq!(
            added.member_node_ids(),
            &[TreeNodeId(3), TreeNodeId(4), TreeNodeId(5)]
        );
        assert_eq!(added.defined_cardinality(), 3);
        assert_eq!(reordered.member_node_ids(), &[TreeNodeId(4), TreeNodeId(3)]);
        assert_eq!(
            reordered.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(EvalValue::Number(4.0))
        );
    }

    #[test]
    fn treecalc_children_reader_rejects_missing_base_node() {
        let snapshot = tree_snapshot(&[3, 4]);
        let mut collection = children_collection();
        collection.base_node_id = TreeNodeId(99);
        let err = TreeCalcChildrenSparseReader::new(
            &snapshot,
            collection,
            [(TreeNodeId(3), EvalValue::Number(3.0))],
        )
        .unwrap_err();

        assert_eq!(
            err,
            SparseReaderError::UnknownTreeCalcBase {
                base_node_id: TreeNodeId(99),
            }
        );
    }

    #[test]
    fn treecalc_children_reader_can_adapt_published_tree_values() {
        let snapshot = tree_snapshot(&[3, 4, 5]);
        let reader = TreeCalcChildrenSparseReader::from_published_values(
            &snapshot,
            children_collection(),
            &BTreeMap::from([
                (TreeNodeId(3), "12.5".to_string()),
                (TreeNodeId(4), "true".to_string()),
                (TreeNodeId(5), "leaf".to_string()),
                (TreeNodeId(99), "ignored".to_string()),
            ]),
        )
        .unwrap();

        assert_eq!(reader.defined_cardinality(), 3);
        assert_eq!(
            reader.read_at(SparseCellCoord::new(1, 1)),
            SparseCellRead::Defined(EvalValue::Number(12.5))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(2, 1)),
            SparseCellRead::Defined(EvalValue::Logical(true))
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(3, 1)),
            SparseCellRead::Defined(EvalValue::Text(ExcelText::from_interop_assignment("leaf")))
        );
    }

    #[test]
    fn treecalc_ordered_selector_reader_uses_resolver_supplied_member_order() {
        let snapshot = tree_snapshot(&[3, 4, 5, 6]);
        let collection = TreeCalcOrderedSelectorReferenceCollection::new(
            TreeCalcOrderedSelectorFamily::PrecedingV1,
            TreeNodeId(5),
            "@PRECEDING",
            [TreeNodeId(3), TreeNodeId(4)],
        );
        let reader = TreeCalcOrderedSelectorSparseReader::new(
            &snapshot,
            collection,
            [
                (TreeNodeId(4), EvalValue::Number(40.0)),
                (TreeNodeId(3), EvalValue::Number(30.0)),
                (TreeNodeId(6), EvalValue::Number(60.0)),
            ],
        )
        .unwrap();

        assert_eq!(reader.member_node_ids(), &[TreeNodeId(3), TreeNodeId(4)]);
        assert_eq!(reader.declared_extent().row_count, 2);
        assert_eq!(reader.defined_cardinality(), 2);
        assert_eq!(
            reader.reader_identity().reader_id,
            "treecalc-hostref:v1:preceding:node:5"
        );
        assert!(
            reader
                .reader_identity()
                .source_identity
                .contains("selector=Preceding")
        );
        assert_eq!(
            reader.defined_iter().collect::<Vec<_>>(),
            vec![
                SparseDefinedCell {
                    coord: SparseCellCoord::new(1, 1),
                    value: EvalValue::Number(30.0),
                },
                SparseDefinedCell {
                    coord: SparseCellCoord::new(2, 1),
                    value: EvalValue::Number(40.0),
                },
            ]
        );
        assert_eq!(
            reader.read_at(SparseCellCoord::new(3, 1)),
            SparseCellRead::Blank
        );
    }
}
