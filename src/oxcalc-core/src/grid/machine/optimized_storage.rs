//! Compact computed-value storage for the optimized grid engine: packed
//! dense value payloads, dense value regions and views, repeated-formula
//! regions, versioned authored/computed cells, and the SparsePointMap that
//! keeps the sparse point map and its row/column occupancy indexes
//! consistent. Internal to the machine (the recalc, valuation, and R1C1
//! paths touch these representations directly, so members are pub(super));
//! shares the machine's types via `use super::*`.

use super::*;

#[derive(Debug, Clone, PartialEq)]
pub(super) enum GridDenseValuePayload {
    CalcValues(Vec<CalcValue>),
    Numbers(Vec<f64>),
    Logicals(Vec<bool>),
    RepeatedCalcValue { value: CalcValue, len: usize },
}

impl GridDenseValuePayload {
    pub(super) fn from_calc_array(array: &CalcArray) -> Self {
        let mut numbers = Vec::with_capacity(array.cell_count());
        let mut logicals = Vec::with_capacity(array.cell_count());
        for value in array.iter_row_major() {
            if value.rich.is_some() {
                return Self::CalcValues(array.iter_row_major().cloned().collect());
            }
            match &value.core {
                CoreValue::Number(number) if logicals.is_empty() => numbers.push(*number),
                CoreValue::Logical(logical) if numbers.is_empty() => logicals.push(*logical),
                _ => return Self::CalcValues(array.iter_row_major().cloned().collect()),
            }
        }
        if !logicals.is_empty() {
            return Self::Logicals(logicals);
        }
        Self::Numbers(numbers)
    }

    pub(super) fn from_calc_values(values: Vec<CalcValue>) -> Self {
        let mut numbers = Vec::with_capacity(values.len());
        let mut logicals = Vec::with_capacity(values.len());
        for value in &values {
            if value.rich.is_some() {
                return Self::from_non_packed_calc_values(values);
            }
            match &value.core {
                CoreValue::Number(number) if logicals.is_empty() => numbers.push(*number),
                CoreValue::Logical(logical) if numbers.is_empty() => logicals.push(*logical),
                _ => return Self::from_non_packed_calc_values(values),
            }
        }
        if !logicals.is_empty() {
            return Self::Logicals(logicals);
        }
        Self::Numbers(numbers)
    }

    pub(super) fn from_non_packed_calc_values(values: Vec<CalcValue>) -> Self {
        if let Some(first) = values.first() {
            if values.iter().all(|value| value == first) {
                return Self::RepeatedCalcValue {
                    value: first.clone(),
                    len: values.len(),
                };
            }
        }
        Self::CalcValues(values)
    }

    pub(super) fn from_numbers(values: Vec<f64>) -> Self {
        Self::Numbers(values)
    }

    pub(super) fn value_at_index(&self, index: usize) -> Option<CalcValue> {
        match self {
            Self::CalcValues(values) => values.get(index).cloned(),
            Self::Numbers(values) => values.get(index).copied().map(CalcValue::number),
            Self::Logicals(values) => values.get(index).copied().map(CalcValue::logical),
            Self::RepeatedCalcValue { value, len } => (index < *len).then(|| value.clone()),
        }
    }

    pub(super) fn estimated_payload_bytes(&self) -> u64 {
        match self {
            Self::CalcValues(values) => values
                .iter()
                .map(estimated_calc_value_bytes)
                .fold(0_u64, u64::saturating_add),
            Self::Numbers(values) => u64::try_from(values.len())
                .unwrap_or(u64::MAX)
                .saturating_mul(u64::try_from(std::mem::size_of::<f64>()).unwrap_or(u64::MAX)),
            Self::Logicals(values) => {
                u64::try_from(values.len())
                    .unwrap_or(u64::MAX)
                    .saturating_add(7)
                    / 8
            }
            Self::RepeatedCalcValue { value, .. } => estimated_calc_value_bytes(value)
                .saturating_add(u64::try_from(std::mem::size_of::<usize>()).unwrap_or(u64::MAX)),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct GridDenseValueStorage {
    payload: Arc<GridDenseValuePayload>,
    row_offset: u32,
    col_offset: u32,
    payload_col_count: u32,
}

impl GridDenseValueStorage {
    pub(super) fn new_for_rect(rect: &GridRect, payload: GridDenseValuePayload) -> Self {
        Self {
            payload: Arc::new(payload),
            row_offset: 0,
            col_offset: 0,
            payload_col_count: rect.col_count(),
        }
    }

    pub(super) fn slice_for_subrect(&self, parent_rect: &GridRect, subrect: &GridRect) -> Self {
        Self {
            payload: Arc::clone(&self.payload),
            row_offset: self
                .row_offset
                .saturating_add(subrect.top_row.saturating_sub(parent_rect.top_row)),
            col_offset: self
                .col_offset
                .saturating_add(subrect.left_col.saturating_sub(parent_rect.left_col)),
            payload_col_count: self.payload_col_count,
        }
    }

    pub(super) fn value_at_rect(&self, rect: &GridRect, row: u32, col: u32) -> Option<CalcValue> {
        if row < rect.top_row
            || row > rect.bottom_row
            || col < rect.left_col
            || col > rect.right_col
        {
            return None;
        }
        let row_offset = u64::from(
            self.row_offset
                .saturating_add(row.saturating_sub(rect.top_row)),
        );
        let col_offset = u64::from(
            self.col_offset
                .saturating_add(col.saturating_sub(rect.left_col)),
        );
        let index = row_offset
            .saturating_mul(u64::from(self.payload_col_count))
            .saturating_add(col_offset);
        let index = usize::try_from(index).ok()?;
        self.payload.value_at_index(index)
    }

    pub(super) fn row_major_values(&self, rect: &GridRect) -> Vec<CalcValue> {
        let mut values =
            Vec::with_capacity(usize::try_from(rect.cell_count()).unwrap_or(usize::MAX));
        for row in rect.top_row..=rect.bottom_row {
            for col in rect.left_col..=rect.right_col {
                if let Some(value) = self.value_at_rect(rect, row, col) {
                    values.push(value);
                }
            }
        }
        values
    }

    #[must_use]
    pub(super) fn packed_numeric_cells(&self, rect: &GridRect) -> u64 {
        match self.payload.as_ref() {
            GridDenseValuePayload::Numbers(_) => rect.cell_count(),
            GridDenseValuePayload::CalcValues(_)
            | GridDenseValuePayload::Logicals(_)
            | GridDenseValuePayload::RepeatedCalcValue { .. } => 0,
        }
    }

    #[must_use]
    pub(super) fn packed_logical_cells(&self, rect: &GridRect) -> u64 {
        match self.payload.as_ref() {
            GridDenseValuePayload::Logicals(_) => rect.cell_count(),
            GridDenseValuePayload::CalcValues(_)
            | GridDenseValuePayload::Numbers(_)
            | GridDenseValuePayload::RepeatedCalcValue { .. } => 0,
        }
    }

    #[must_use]
    pub(super) fn shared_payload_id(&self) -> usize {
        Arc::as_ptr(&self.payload) as usize
    }

    #[must_use]
    pub(super) fn shared_payload_bytes(&self) -> u64 {
        self.payload.estimated_payload_bytes()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridDenseValueRegion {
    pub rect: GridRect,
    pub(super) storage: GridDenseValueStorage,
    pub(super) revision: u64,
}

impl GridDenseValueRegion {
    #[must_use]
    pub fn row_major_values(&self) -> Vec<CalcValue> {
        self.storage.row_major_values(&self.rect)
    }

    pub(super) fn value_at(&self, address: &ExcelGridCellAddress) -> Option<CalcValue> {
        if !self.rect.contains(address) {
            return None;
        }
        self.value_at_row_col(address.row, address.col)
    }

    pub(super) fn value_at_row_col(&self, row: u32, col: u32) -> Option<CalcValue> {
        if row < self.rect.top_row
            || row > self.rect.bottom_row
            || col < self.rect.left_col
            || col > self.rect.right_col
        {
            return None;
        }
        self.storage.value_at_rect(&self.rect, row, col)
    }

    pub(super) fn estimated_authored_bytes(&self) -> u64 {
        u64::try_from(std::mem::size_of::<Self>())
            .unwrap_or(u64::MAX)
            .saturating_add(estimated_grid_rect_heap_bytes(&self.rect))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct GridComputedDenseValueRegion {
    pub rect: GridRect,
    pub(super) storage: GridDenseValueStorage,
    pub(super) revision: u64,
    pub(super) source: GridOptimizedCellSource,
}

impl GridComputedDenseValueRegion {
    #[must_use]
    pub fn cell_count(&self) -> u64 {
        self.rect.cell_count()
    }

    #[must_use]
    pub fn packed_numeric_cells(&self) -> u64 {
        self.storage.packed_numeric_cells(&self.rect)
    }

    #[must_use]
    pub fn packed_logical_cells(&self) -> u64 {
        self.storage.packed_logical_cells(&self.rect)
    }

    #[must_use]
    pub fn row_major_values(&self) -> Vec<CalcValue> {
        self.storage.row_major_values(&self.rect)
    }

    pub(super) fn value_at(&self, address: &ExcelGridCellAddress) -> Option<CalcValue> {
        if !self.rect.contains(address) {
            return None;
        }
        self.value_at_row_col(address.row, address.col)
    }

    pub(super) fn value_at_row_col(&self, row: u32, col: u32) -> Option<CalcValue> {
        if row < self.rect.top_row
            || row > self.rect.bottom_row
            || col < self.rect.left_col
            || col > self.rect.right_col
        {
            return None;
        }
        self.storage.value_at_rect(&self.rect, row, col)
    }
}

pub(super) fn dense_region_publication_key_matches(
    lhs: &GridComputedDenseValueRegion,
    rhs: &GridComputedDenseValueRegion,
) -> bool {
    lhs.rect == rhs.rect && lhs.source == rhs.source
}

pub(super) fn dense_region_publication_payload_matches(
    lhs: &GridComputedDenseValueRegion,
    rhs: &GridComputedDenseValueRegion,
) -> bool {
    lhs.row_major_values() == rhs.row_major_values()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridRepeatedFormulaRegion {
    pub rect: GridRect,
    pub formula: GridFormulaCell,
    pub(super) revision: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum GridOptimizedAuthoredCell {
    Number(f64),
    Literal(Box<CalcValue>),
    Formula(Box<GridFormulaCell>),
}

impl GridOptimizedAuthoredCell {
    pub(super) fn from_authored(cell: GridAuthoredCell) -> Self {
        match cell {
            GridAuthoredCell::Literal(value) => Self::literal(value),
            GridAuthoredCell::Formula(formula) => Self::Formula(Box::new(formula)),
        }
    }

    pub(super) fn literal(value: CalcValue) -> Self {
        if let CoreValue::Number(number) = value.core {
            if value.rich.is_none() {
                return Self::Number(number);
            }
            return Self::Literal(Box::new(CalcValue {
                core: CoreValue::Number(number),
                rich: value.rich,
            }));
        }
        Self::Literal(Box::new(value))
    }

    pub(super) fn formula(formula: GridFormulaCell) -> Self {
        Self::Formula(Box::new(formula))
    }

    pub(super) fn to_authored(&self) -> GridAuthoredCell {
        match self {
            Self::Number(number) => GridAuthoredCell::Literal(CalcValue::number(*number)),
            Self::Literal(value) => GridAuthoredCell::Literal((**value).clone()),
            Self::Formula(formula) => GridAuthoredCell::Formula((**formula).clone()),
        }
    }

    pub(super) fn literal_value(&self) -> Option<CalcValue> {
        match self {
            Self::Number(number) => Some(CalcValue::number(*number)),
            Self::Literal(value) => Some((**value).clone()),
            Self::Formula(_) => None,
        }
    }

    pub(super) fn formula_ref(&self) -> Option<&GridFormulaCell> {
        match self {
            Self::Formula(formula) => Some(formula.as_ref()),
            _ => None,
        }
    }

    pub(super) fn formula_mut(&mut self) -> Option<&mut GridFormulaCell> {
        match self {
            Self::Formula(formula) => Some(formula.as_mut()),
            _ => None,
        }
    }

    pub(super) fn estimated_authored_bytes(&self) -> u64 {
        match self {
            Self::Number(_) => u64::try_from(std::mem::size_of::<f64>()).unwrap_or(u64::MAX),
            Self::Literal(value) => estimated_calc_value_bytes(value),
            Self::Formula(formula) => estimated_formula_cell_bytes(formula),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct GridVersionedAuthoredCell {
    pub(super) revision: u64,
    pub(super) cell: GridOptimizedAuthoredCell,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct GridVersionedComputedCell {
    pub(super) revision: u64,
    pub(super) value: CalcValue,
    pub(super) source: GridOptimizedCellSource,
}

/// Sparse computed point-cell storage with row- and column-occupancy indexes
/// kept consistent behind a single mutating API, so the point map and its two
/// indexes cannot silently drift out of sync.
#[derive(Debug, Clone, Default, PartialEq)]
pub(super) struct SparsePointMap {
    points: BTreeMap<ExcelGridCellAddress, GridVersionedComputedCell>,
    by_row: BTreeMap<u32, BTreeSet<ExcelGridCellAddress>>,
    by_col: BTreeMap<u32, BTreeSet<ExcelGridCellAddress>>,
}

impl SparsePointMap {
    pub(super) fn len(&self) -> usize {
        self.points.len()
    }

    pub(super) fn get(&self, address: &ExcelGridCellAddress) -> Option<&GridVersionedComputedCell> {
        self.points.get(address)
    }

    pub(super) fn contains_key(&self, address: &ExcelGridCellAddress) -> bool {
        self.points.contains_key(address)
    }

    pub(super) fn keys(&self) -> impl Iterator<Item = &ExcelGridCellAddress> {
        self.points.keys()
    }

    pub(super) fn iter(
        &self,
    ) -> impl Iterator<Item = (&ExcelGridCellAddress, &GridVersionedComputedCell)> {
        self.points.iter()
    }

    /// Insert or replace the cell at `address`, keeping both occupancy indexes
    /// in step in the same call.
    pub(super) fn upsert(
        &mut self,
        address: ExcelGridCellAddress,
        cell: GridVersionedComputedCell,
    ) {
        self.by_row
            .entry(address.row)
            .or_default()
            .insert(address.clone());
        self.by_col
            .entry(address.col)
            .or_default()
            .insert(address.clone());
        self.points.insert(address, cell);
    }

    /// Remove the cell at `address`, unindexing it from both occupancy indexes
    /// in the same call.
    pub(super) fn remove(
        &mut self,
        address: &ExcelGridCellAddress,
    ) -> Option<GridVersionedComputedCell> {
        let removed = self.points.remove(address);
        if removed.is_some() {
            if let Some(indexed) = self.by_row.get_mut(&address.row) {
                indexed.remove(address);
                if indexed.is_empty() {
                    self.by_row.remove(&address.row);
                }
            }
            if let Some(indexed) = self.by_col.get_mut(&address.col) {
                indexed.remove(address);
                if indexed.is_empty() {
                    self.by_col.remove(&address.col);
                }
            }
        }
        removed
    }

    /// Occupancy-proportional enumeration of occupied addresses inside `rect`:
    /// scan whichever axis index is smaller, never the full rect area (P-20).
    pub(super) fn addresses_in_rect(&self, rect: &GridRect) -> Vec<ExcelGridCellAddress> {
        let mut addresses = BTreeSet::new();
        if rect.col_count() <= rect.row_count() {
            for col in rect.left_col..=rect.right_col {
                let Some(indexed) = self.by_col.get(&col) else {
                    continue;
                };
                addresses.extend(
                    indexed
                        .iter()
                        .filter(|address| {
                            rect.top_row <= address.row && address.row <= rect.bottom_row
                        })
                        .cloned(),
                );
            }
        } else {
            for row in rect.top_row..=rect.bottom_row {
                let Some(indexed) = self.by_row.get(&row) else {
                    continue;
                };
                addresses.extend(
                    indexed
                        .iter()
                        .filter(|address| {
                            rect.left_col <= address.col && address.col <= rect.right_col
                        })
                        .cloned(),
                );
            }
        }
        addresses.into_iter().collect()
    }
}
