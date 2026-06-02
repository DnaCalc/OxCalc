#![forbid(unsafe_code)]

//! TreeCalc implementation of OxFunc's calc-time reference-system provider.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};

use oxfunc_core::resolver::{
    ReferenceDereferenceRequest, ReferenceEnumerationRequest, ReferenceFacts,
    ReferenceFactsRequest, ReferenceResolutionError, ReferenceSystemError,
    ReferenceSystemOperation, ReferenceSystemProvider, ReferenceTextResolutionMode,
    ReferenceTextResolveRequest, ResolvedReferenceCell, ResolvedReferenceExtent,
    ResolvedReferenceValues, reference_facts,
};
use oxfunc_core::value::{
    ArrayCellValue, CalcValue, CoreValue, EvalArray, EvalValue, ExcelText, ReferenceDisplay,
    ReferenceHandle, ReferenceHandleId, ReferenceLike, ReferenceSystemId, WorksheetErrorCode,
};

use crate::dependency::TreeReferenceCollectionFamily;
use crate::formula::{
    TreeCalcChildrenReferenceCollection, TreeCalcOrderedSelectorFamily,
    TreeCalcOrderedSelectorReferenceCollection, TreeCalcReferenceLiteralArrayCollection,
    TreeCalcReferenceLiteralArrayElement,
};
use crate::sparse_reader::{
    SparseRangeReader, TreeCalcChildrenSparseReader, TreeCalcOrderedSelectorSparseReader,
    TreeCalcReferenceLiteralArraySparseReader,
};
use crate::structural::{StructuralSnapshot, TreeNodeId};
use crate::tree_reference_resolution::{
    ContextHostNameResolution, resolve_context_host_name_token,
};

pub const TREECALC_REFERENCE_SYSTEM_ID: &str = "dna.treecalc.v1";

#[must_use]
pub fn treecalc_reference_system_id() -> ReferenceSystemId {
    ReferenceSystemId(TREECALC_REFERENCE_SYSTEM_ID.to_string())
}

#[must_use]
pub fn treecalc_node_reference_target(node_id: TreeNodeId) -> String {
    format!("treecalc.node:{}", node_id.0)
}

#[must_use]
pub fn treecalc_node_reference_like(node_id: TreeNodeId) -> ReferenceLike {
    let target = treecalc_node_reference_target(node_id);
    treecalc_opaque_reference_like(target.clone(), target)
}

#[must_use]
pub fn treecalc_collection_reference_like(host_ref_handle: impl Into<String>) -> ReferenceLike {
    let host_ref_handle = host_ref_handle.into();
    treecalc_opaque_reference_like(host_ref_handle.clone(), host_ref_handle)
}

#[must_use]
pub fn treecalc_opaque_reference_like(
    handle_id: impl Into<String>,
    display: impl Into<String>,
) -> ReferenceLike {
    let handle_id = handle_id.into();
    let display = display.into();
    ReferenceLike::opaque(
        treecalc_reference_system_id(),
        ReferenceHandle {
            id: ReferenceHandleId::from_bytes(handle_id.as_bytes().to_vec()),
        },
        Some(ReferenceDisplay {
            text: ExcelText::from_interop_assignment(&display),
        }),
    )
}

pub struct TreeCalcReferenceSystemProvider<'a> {
    structural_snapshot: Option<&'a StructuralSnapshot>,
    meta_node_ids: Option<&'a BTreeSet<TreeNodeId>>,
    owner_node_id: Option<TreeNodeId>,
    published_calc_values: Option<&'a BTreeMap<TreeNodeId, CalcValue>>,
    sparse_reference_values: Vec<TreeCalcResolvedReferenceValues>,
    collection_descriptors:
        BTreeMap<TreeCalcReferenceDescriptorIdentity, TreeCalcCollectionReferenceDescriptor>,
    text_resolutions: RefCell<Vec<TreeCalcRuntimeReferenceTextResolution>>,
}

impl<'a> TreeCalcReferenceSystemProvider<'a> {
    #[must_use]
    pub fn new(
        structural_snapshot: &'a StructuralSnapshot,
        meta_node_ids: &'a BTreeSet<TreeNodeId>,
        owner_node_id: TreeNodeId,
        published_calc_values: &'a BTreeMap<TreeNodeId, CalcValue>,
    ) -> Self {
        Self {
            structural_snapshot: Some(structural_snapshot),
            meta_node_ids: Some(meta_node_ids),
            owner_node_id: Some(owner_node_id),
            published_calc_values: Some(published_calc_values),
            sparse_reference_values: Vec::new(),
            collection_descriptors: BTreeMap::new(),
            text_resolutions: RefCell::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn sparse_only() -> Self {
        Self {
            structural_snapshot: None,
            meta_node_ids: None,
            owner_node_id: None,
            published_calc_values: None,
            sparse_reference_values: Vec::new(),
            collection_descriptors: BTreeMap::new(),
            text_resolutions: RefCell::new(Vec::new()),
        }
    }

    #[must_use]
    pub fn with_sparse_reference_values(
        mut self,
        reference: ReferenceLike,
        values: ResolvedReferenceValues,
    ) -> Self {
        self.sparse_reference_values
            .push(TreeCalcResolvedReferenceValues { reference, values });
        self
    }

    #[must_use]
    pub fn with_sparse_reader(
        self,
        reference: ReferenceLike,
        reader: &impl SparseRangeReader,
    ) -> Self {
        self.with_sparse_reference_values(reference, resolved_values_from_sparse_reader(reader))
    }

    #[must_use]
    pub fn with_collection_descriptor(
        mut self,
        descriptor: TreeCalcCollectionReferenceDescriptor,
    ) -> Self {
        self.collection_descriptors
            .insert(descriptor.descriptor_identity(), descriptor);
        self
    }

    #[must_use]
    pub fn collection_descriptor_count(&self) -> usize {
        self.collection_descriptors.len()
    }

    #[must_use]
    pub fn runtime_text_resolutions(&self) -> Vec<TreeCalcRuntimeReferenceTextResolution> {
        self.text_resolutions.borrow().clone()
    }

    fn treecalc_reference_error(&self, reference: &ReferenceLike) -> ReferenceResolutionError {
        ReferenceResolutionError::UnresolvedReference {
            target: reference.target.clone(),
        }
    }
}

impl ReferenceSystemProvider for TreeCalcReferenceSystemProvider<'_> {
    fn dereference(
        &self,
        request: &ReferenceDereferenceRequest,
    ) -> Result<EvalValue, ReferenceResolutionError> {
        let Some(node_id) = treecalc_node_id_from_reference(&request.reference) else {
            return Err(self.treecalc_reference_error(&request.reference));
        };
        let Some(published_calc_values) = self.published_calc_values else {
            return Err(ReferenceResolutionError::ProviderFailure {
                detail: "treecalc provider has no published CalcValue scope".to_string(),
            });
        };
        let Some(value) = published_calc_values.get(&node_id) else {
            return Err(ReferenceResolutionError::ProviderFailure {
                detail: format!("treecalc reference {node_id} has no published CalcValue"),
            });
        };
        Ok(calc_value_to_eval_value(value))
    }

    fn enumerate_values(
        &self,
        request: &ReferenceEnumerationRequest,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        if let Some(values) = self.values_from_collection_descriptor(&request.reference)? {
            return Ok(Some(values));
        }

        Ok(self
            .sparse_reference_values
            .iter()
            .find(|entry| references_match(&entry.reference, &request.reference))
            .map(|entry| entry.values.clone()))
    }

    fn resolve_text(
        &self,
        request: &ReferenceTextResolveRequest,
    ) -> Result<ReferenceLike, ReferenceSystemError> {
        if request.mode != ReferenceTextResolutionMode::Indirect {
            return Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::ResolveText,
            });
        }
        let (Some(structural_snapshot), Some(meta_node_ids), Some(owner_node_id)) = (
            self.structural_snapshot,
            self.meta_node_ids,
            self.owner_node_id,
        ) else {
            return Err(ReferenceSystemError::Unsupported {
                operation: ReferenceSystemOperation::ResolveText,
            });
        };
        match resolve_context_host_name_token(
            &request.text,
            owner_node_id,
            structural_snapshot,
            meta_node_ids,
        ) {
            ContextHostNameResolution::Resolved(target_node_id) => {
                let reference = treecalc_node_reference_like(target_node_id);
                self.text_resolutions
                    .borrow_mut()
                    .push(TreeCalcRuntimeReferenceTextResolution {
                        owner_node_id,
                        target_node_id,
                        reference_text: request.text.clone(),
                        mode: request.mode,
                        a1_style: request.a1_style,
                        reference_like: reference.clone(),
                    });
                Ok(reference)
            }
            ContextHostNameResolution::Ambiguous => Err(ReferenceSystemError::ProviderFailure {
                detail: format!("ambiguous TreeCalc reference text '{}'", request.text),
            }),
            ContextHostNameResolution::Unsupported(reason) => {
                Err(ReferenceSystemError::ProviderFailure {
                    detail: format!(
                        "unsupported TreeCalc reference text '{}': {reason}",
                        request.text
                    ),
                })
            }
            ContextHostNameResolution::Unresolved => {
                Err(ReferenceSystemError::InvalidReferenceText {
                    text: request.text.clone(),
                })
            }
        }
    }

    fn facts(
        &self,
        request: &ReferenceFactsRequest,
    ) -> Result<ReferenceFacts, ReferenceSystemError> {
        Ok(reference_facts(&request.reference))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcRuntimeReferenceTextResolution {
    pub owner_node_id: TreeNodeId,
    pub target_node_id: TreeNodeId,
    pub reference_text: String,
    pub mode: ReferenceTextResolutionMode,
    pub a1_style: Option<bool>,
    pub reference_like: ReferenceLike,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeCalcCollectionReferenceDescriptor {
    pub host_ref_handle: String,
    pub family: TreeReferenceCollectionFamily,
    pub base_node_id: TreeNodeId,
    pub source_span_utf8: Option<(usize, usize)>,
    pub source_token_text: String,
    pub opaque_selector: String,
    pub member_node_ids: Vec<TreeNodeId>,
    pub membership_version: String,
    pub order_version: String,
}

impl TreeCalcCollectionReferenceDescriptor {
    #[must_use]
    pub fn descriptor_identity(&self) -> TreeCalcReferenceDescriptorIdentity {
        TreeCalcReferenceDescriptorIdentity::from_host_ref_handle(&self.host_ref_handle)
    }

    #[must_use]
    pub fn reference_like(&self) -> ReferenceLike {
        treecalc_collection_reference_like(&self.host_ref_handle)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TreeCalcReferenceDescriptorIdentity(String);

impl TreeCalcReferenceDescriptorIdentity {
    #[must_use]
    pub fn from_host_ref_handle(host_ref_handle: impl Into<String>) -> Self {
        Self(host_ref_handle.into())
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

struct TreeCalcResolvedReferenceValues {
    reference: ReferenceLike,
    values: ResolvedReferenceValues,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcSparseReferenceCell {
    pub row: usize,
    pub col: usize,
    pub value: ArrayCellValue,
}

impl TreeCalcSparseReferenceCell {
    #[must_use]
    pub fn new(row: usize, col: usize, value: ArrayCellValue) -> Self {
        Self { row, col, value }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TreeCalcSparseReferenceValuesBinding {
    pub reference: ReferenceLike,
    pub declared_rows: usize,
    pub declared_cols: usize,
    pub defined_cells: Vec<TreeCalcSparseReferenceCell>,
    pub reader_identity: Option<String>,
}

impl TreeCalcSparseReferenceValuesBinding {
    #[must_use]
    pub fn resolved_values(&self) -> ResolvedReferenceValues {
        ResolvedReferenceValues::new(
            ResolvedReferenceExtent::new(self.declared_rows, self.declared_cols),
            self.defined_cells
                .iter()
                .map(|cell| ResolvedReferenceCell::new(cell.row, cell.col, cell.value.clone()))
                .collect(),
            self.reader_identity.clone(),
        )
    }
}

impl TreeCalcReferenceSystemProvider<'_> {
    fn values_from_collection_descriptor(
        &self,
        reference: &ReferenceLike,
    ) -> Result<Option<ResolvedReferenceValues>, ReferenceResolutionError> {
        let Some(handle) = treecalc_handle_text(reference) else {
            return Ok(None);
        };
        let descriptor_identity = TreeCalcReferenceDescriptorIdentity::from_host_ref_handle(handle);
        let Some(descriptor) = self.collection_descriptors.get(&descriptor_identity) else {
            return Ok(None);
        };
        let Some(structural_snapshot) = self.structural_snapshot else {
            return Ok(None);
        };
        let Some(published_calc_values) = self.published_calc_values else {
            return Ok(None);
        };

        let values = match descriptor.family {
            TreeReferenceCollectionFamily::ChildrenV1 => {
                let reader = TreeCalcChildrenSparseReader::from_published_calc_values(
                    structural_snapshot,
                    descriptor.children_collection(),
                    published_calc_values,
                )
                .map_err(|error| ReferenceResolutionError::ProviderFailure {
                    detail: format!(
                        "failed to reconstruct TreeCalc children reference '{}': {error}",
                        descriptor.host_ref_handle
                    ),
                })?;
                resolved_values_from_sparse_reader(&reader)
            }
            TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => {
                let reader = TreeCalcReferenceLiteralArraySparseReader::from_published_calc_values(
                    structural_snapshot,
                    descriptor.reference_literal_array_collection()?,
                    published_calc_values,
                )
                .map_err(|error| ReferenceResolutionError::ProviderFailure {
                    detail: format!(
                        "failed to reconstruct TreeCalc reference-literal array '{}': {error}",
                        descriptor.host_ref_handle
                    ),
                })?;
                resolved_values_from_sparse_reader(&reader)
            }
            TreeReferenceCollectionFamily::SiblingSetV1
            | TreeReferenceCollectionFamily::PrecedingV1
            | TreeReferenceCollectionFamily::FollowingV1
            | TreeReferenceCollectionFamily::AncestorsV1
            | TreeReferenceCollectionFamily::RecursiveDescendantsV1 => {
                let reader = TreeCalcOrderedSelectorSparseReader::from_published_calc_values(
                    structural_snapshot,
                    descriptor.ordered_selector_collection()?,
                    published_calc_values,
                )
                .map_err(|error| ReferenceResolutionError::ProviderFailure {
                    detail: format!(
                        "failed to reconstruct TreeCalc ordered selector reference '{}': {error}",
                        descriptor.host_ref_handle
                    ),
                })?;
                resolved_values_from_sparse_reader(&reader)
            }
        };

        Ok(Some(values))
    }
}

impl TreeCalcCollectionReferenceDescriptor {
    fn children_collection(&self) -> TreeCalcChildrenReferenceCollection {
        TreeCalcChildrenReferenceCollection {
            host_ref_handle: self.host_ref_handle.clone(),
            base_node_id: self.base_node_id,
            source_span_utf8: self.source_span_utf8,
            source_token_text: self.source_token_text.clone(),
            opaque_selector: self.opaque_selector.clone(),
            membership_version: self.membership_version.clone(),
            order_version: self.order_version.clone(),
        }
    }

    fn reference_literal_array_collection(
        &self,
    ) -> Result<TreeCalcReferenceLiteralArrayCollection, ReferenceResolutionError> {
        let carrier_id = self
            .host_ref_handle
            .strip_prefix("treecalc-hostref:v1:reference_literal_array:")
            .unwrap_or(&self.host_ref_handle);
        let elements = self
            .member_node_ids
            .iter()
            .copied()
            .map(TreeCalcReferenceLiteralArrayElement::ReferenceNode);
        let mut collection = TreeCalcReferenceLiteralArrayCollection::reference_only_with_handle(
            carrier_id,
            self.host_ref_handle.clone(),
            self.base_node_id,
            self.source_token_text.clone(),
            elements,
        )
        .map_err(|error| ReferenceResolutionError::ProviderFailure {
            detail: format!(
                "failed to reconstruct TreeCalc reference-literal descriptor '{}': {error}",
                self.host_ref_handle
            ),
        })?;
        if let Some((start, end)) = self.source_span_utf8 {
            collection = collection.with_source_span_utf8(start, end);
        }
        Ok(collection)
    }

    fn ordered_selector_collection(
        &self,
    ) -> Result<TreeCalcOrderedSelectorReferenceCollection, ReferenceResolutionError> {
        let Some(family) = ordered_selector_family_from_dependency(self.family) else {
            return Err(ReferenceResolutionError::ProviderFailure {
                detail: format!(
                    "TreeCalc collection '{}' is not an ordered selector",
                    self.host_ref_handle
                ),
            });
        };
        Ok(TreeCalcOrderedSelectorReferenceCollection {
            family,
            host_ref_handle: self.host_ref_handle.clone(),
            base_node_id: self.base_node_id,
            member_node_ids: self.member_node_ids.clone(),
            source_span_utf8: self.source_span_utf8,
            source_token_text: self.source_token_text.clone(),
            opaque_selector: self.opaque_selector.clone(),
            membership_version: self.membership_version.clone(),
            order_version: self.order_version.clone(),
        })
    }
}

fn ordered_selector_family_from_dependency(
    family: TreeReferenceCollectionFamily,
) -> Option<TreeCalcOrderedSelectorFamily> {
    match family {
        TreeReferenceCollectionFamily::ChildrenV1
        | TreeReferenceCollectionFamily::ReferenceLiteralArrayV1 => None,
        TreeReferenceCollectionFamily::SiblingSetV1 => {
            Some(TreeCalcOrderedSelectorFamily::SiblingSetV1)
        }
        TreeReferenceCollectionFamily::PrecedingV1 => {
            Some(TreeCalcOrderedSelectorFamily::PrecedingV1)
        }
        TreeReferenceCollectionFamily::FollowingV1 => {
            Some(TreeCalcOrderedSelectorFamily::FollowingV1)
        }
        TreeReferenceCollectionFamily::AncestorsV1 => {
            Some(TreeCalcOrderedSelectorFamily::AncestorsV1)
        }
        TreeReferenceCollectionFamily::RecursiveDescendantsV1 => {
            Some(TreeCalcOrderedSelectorFamily::RecursiveDescendantsV1)
        }
    }
}

fn references_match(left: &ReferenceLike, right: &ReferenceLike) -> bool {
    left.system == right.system && left.identity == right.identity
}

fn treecalc_node_id_from_reference(reference: &ReferenceLike) -> Option<TreeNodeId> {
    treecalc_handle_text(reference)
        .or_else(|| {
            reference
                .target
                .strip_prefix("treecalc.node:")
                .map(str::to_string)
        })
        .and_then(|handle| {
            handle
                .strip_prefix("treecalc.node:")
                .and_then(|id| id.parse::<u64>().ok())
                .map(TreeNodeId)
        })
}

fn treecalc_handle_text(reference: &ReferenceLike) -> Option<String> {
    match &reference.identity {
        oxfunc_core::value::ReferenceIdentity::Opaque(handle) => {
            String::from_utf8(handle.id.bytes.clone()).ok()
        }
        oxfunc_core::value::ReferenceIdentity::Textual(textual) => {
            Some(textual.text.to_string_lossy())
        }
        oxfunc_core::value::ReferenceIdentity::Composite(_) => None,
    }
}

fn resolved_values_from_sparse_reader(reader: &impl SparseRangeReader) -> ResolvedReferenceValues {
    let extent = reader.declared_extent();
    let identity = reader.reader_identity();
    ResolvedReferenceValues::new(
        ResolvedReferenceExtent::new(
            usize::try_from(extent.row_count).unwrap_or(usize::MAX),
            usize::try_from(extent.column_count).unwrap_or(usize::MAX),
        ),
        reader
            .defined_iter()
            .map(|cell| {
                ResolvedReferenceCell::new(
                    usize::try_from(cell.coord.row).unwrap_or(usize::MAX),
                    usize::try_from(cell.coord.column).unwrap_or(usize::MAX),
                    eval_value_to_array_cell(cell.value),
                )
            })
            .collect(),
        Some(format!(
            "reader_id={};source={};snapshot={}",
            identity.reader_id, identity.source_identity, identity.snapshot_identity
        )),
    )
}

fn calc_value_to_eval_value(value: &CalcValue) -> EvalValue {
    match &value.core {
        CoreValue::Number(number) => EvalValue::Number(*number),
        CoreValue::Text(text) => EvalValue::Text(text.clone()),
        CoreValue::Logical(logical) => EvalValue::Logical(*logical),
        CoreValue::Error(code) => EvalValue::Error(*code),
        CoreValue::Empty => EvalValue::Text(ExcelText::from_interop_assignment("")),
        CoreValue::Missing => EvalValue::Error(WorksheetErrorCode::Value),
        CoreValue::Array(array) => {
            let cells = array
                .iter_row_major()
                .map(calc_value_to_array_cell_value)
                .collect::<Vec<_>>();
            EvalValue::Array(
                EvalArray::new(array.shape(), cells)
                    .expect("CalcArray invariants convert into EvalArray"),
            )
        }
        CoreValue::Reference(reference) => EvalValue::Reference(reference.clone()),
    }
}

fn calc_value_to_array_cell_value(value: &CalcValue) -> ArrayCellValue {
    match &value.core {
        CoreValue::Number(number) => ArrayCellValue::Number(*number),
        CoreValue::Text(text) => ArrayCellValue::Text(text.clone()),
        CoreValue::Logical(logical) => ArrayCellValue::Logical(*logical),
        CoreValue::Error(code) => ArrayCellValue::Error(*code),
        CoreValue::Empty => ArrayCellValue::EmptyCell,
        CoreValue::Missing | CoreValue::Array(_) | CoreValue::Reference(_) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
    }
}

fn eval_value_to_array_cell(value: EvalValue) -> ArrayCellValue {
    match value {
        EvalValue::Number(value) => ArrayCellValue::Number(value),
        EvalValue::Text(value) => ArrayCellValue::Text(value),
        EvalValue::Logical(value) => ArrayCellValue::Logical(value),
        EvalValue::Error(value) => ArrayCellValue::Error(value),
        EvalValue::Array(_) | EvalValue::Reference(_) | EvalValue::Lambda(_) => {
            ArrayCellValue::Error(WorksheetErrorCode::Value)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sparse_reader::{
        SparseCellCoord, SparseRangeExtent, SparseReaderIdentity, WorksheetSparseRangeReader,
    };
    use crate::structural::{
        StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId,
    };

    fn snapshot() -> StructuralSnapshot {
        StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            vec![
                StructuralNode {
                    node_id: TreeNodeId(1),
                    parent_id: None,
                    symbol: "Root".to_string(),
                    kind: StructuralNodeKind::Root,
                    child_ids: vec![TreeNodeId(2)],
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    parent_id: Some(TreeNodeId(1)),
                    symbol: "A".to_string(),
                    kind: StructuralNodeKind::Calculation,
                    child_ids: Vec::new(),
                },
            ],
        )
        .expect("test snapshot should be valid")
    }

    #[test]
    fn treecalc_provider_dereferences_node_reference() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::from([(TreeNodeId(2), CalcValue::number(42.0))]);
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values);

        let result = provider
            .dereference(&ReferenceDereferenceRequest {
                reference: treecalc_node_reference_like(TreeNodeId(2)),
            })
            .expect("node reference should dereference");

        assert_eq!(result, EvalValue::Number(42.0));
    }

    #[test]
    fn treecalc_provider_enumerates_sparse_reference_values_by_identity() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::new();
        let reference = treecalc_collection_reference_like("treecalc-hostref:v1:test");
        let reader = WorksheetSparseRangeReader::new(
            SparseReaderIdentity::new("reader:test", "source:test", "snapshot:test"),
            SparseRangeExtent::new(SparseCellCoord::new(0, 0), 1, 2),
            [
                (SparseCellCoord::new(0, 0), EvalValue::Number(1.0)),
                (SparseCellCoord::new(0, 1), EvalValue::Number(2.0)),
            ],
        )
        .expect("reader should be valid");
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values)
                .with_sparse_reader(reference.clone(), &reader);

        let result = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("enumeration should succeed")
            .expect("reference values should be present");

        assert_eq!(result.declared_extent, ResolvedReferenceExtent::new(1, 2));
        assert_eq!(result.defined_cardinality, 2);
    }

    #[test]
    fn treecalc_provider_enumerates_collection_from_shared_descriptor() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::from([(TreeNodeId(2), CalcValue::number(42.0))]);
        let handle = "treecalc-hostref:v1:children:1".to_string();
        let reference = treecalc_collection_reference_like(&handle);
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values)
                .with_collection_descriptor(TreeCalcCollectionReferenceDescriptor {
                    host_ref_handle: handle,
                    family: TreeReferenceCollectionFamily::ChildrenV1,
                    base_node_id: TreeNodeId(1),
                    source_span_utf8: None,
                    source_token_text: "A.@CHILDREN".to_string(),
                    opaque_selector: "oxcalc.treecalc.host_selector.v1:selector=Children;base=1"
                        .to_string(),
                    member_node_ids: vec![TreeNodeId(2)],
                    membership_version: "treecalc-membership:v1:base=1;members=2".to_string(),
                    order_version: "treecalc-order:v1:base=1;members=2".to_string(),
                });

        let result = provider
            .enumerate_values(&ReferenceEnumerationRequest { reference })
            .expect("descriptor-backed enumeration should succeed")
            .expect("descriptor should reconstruct sparse values");

        assert_eq!(result.declared_extent, ResolvedReferenceExtent::new(1, 1));
        assert_eq!(result.defined_cardinality, 1);
        assert_eq!(
            result.defined_cells,
            vec![ResolvedReferenceCell::new(
                1,
                1,
                ArrayCellValue::Number(42.0)
            )]
        );
    }

    #[test]
    fn treecalc_provider_interns_collection_descriptors_by_handle() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::from([(TreeNodeId(2), CalcValue::number(42.0))]);
        let handle = "treecalc-hostref:v1:children:1".to_string();
        let descriptor = TreeCalcCollectionReferenceDescriptor {
            host_ref_handle: handle.clone(),
            family: TreeReferenceCollectionFamily::ChildrenV1,
            base_node_id: TreeNodeId(1),
            source_span_utf8: None,
            source_token_text: "A.@CHILDREN".to_string(),
            opaque_selector: "oxcalc.treecalc.host_selector.v1:selector=Children;base=1"
                .to_string(),
            member_node_ids: vec![TreeNodeId(2)],
            membership_version: "treecalc-membership:v1:base=1;members=2".to_string(),
            order_version: "treecalc-order:v1:base=1;members=2".to_string(),
        };
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values)
                .with_collection_descriptor(descriptor.clone())
                .with_collection_descriptor(descriptor);

        assert_eq!(provider.collection_descriptor_count(), 1);

        let result = provider
            .enumerate_values(&ReferenceEnumerationRequest {
                reference: treecalc_collection_reference_like(handle),
            })
            .expect("descriptor-backed enumeration should succeed")
            .expect("interned descriptor should reconstruct sparse values");

        assert_eq!(result.defined_cardinality, 1);
    }

    #[test]
    fn treecalc_provider_resolves_runtime_text_through_host_names() {
        let snapshot = snapshot();
        let meta = BTreeSet::new();
        let values = BTreeMap::new();
        let provider =
            TreeCalcReferenceSystemProvider::new(&snapshot, &meta, TreeNodeId(1), &values);

        let reference = provider
            .resolve_text(&ReferenceTextResolveRequest {
                text: "A".to_string(),
                mode: ReferenceTextResolutionMode::Indirect,
                a1_style: Some(true),
                caller_context: None,
            })
            .expect("A should resolve as a TreeCalc host reference");

        assert!(references_match(
            &reference,
            &treecalc_node_reference_like(TreeNodeId(2))
        ));
        assert_eq!(provider.runtime_text_resolutions().len(), 1);
    }
}
