//! Row/column axis state for the strict-excel-grid engines: per-axis
//! properties (size, hidden-manual, hidden-filter, outline level), the
//! sparse axis property maps, hidden-row visibility ranges, and axis
//! structural-edit application. Internal to the machine; shares the
//! machine's types via `use super::*`.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridAxisProps {
    pub size_twips: Option<u32>,
    pub hidden_manual: bool,
    pub hidden_filter: bool,
    pub outline_level: u8,
    pub collapsed: bool,
}

impl GridAxisProps {
    #[must_use]
    pub const fn visible() -> Self {
        Self {
            size_twips: None,
            hidden_manual: false,
            hidden_filter: false,
            outline_level: 0,
            collapsed: false,
        }
    }
}

impl Default for GridAxisProps {
    fn default() -> Self {
        Self::visible()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GridAxisState {
    rows: BTreeMap<u32, GridAxisProps>,
    cols: BTreeMap<u32, GridAxisProps>,
}

impl GridAxisState {
    #[must_use]
    pub fn row(&self, row: u32) -> GridAxisProps {
        self.rows.get(&row).cloned().unwrap_or_default()
    }

    #[must_use]
    pub fn col(&self, col: u32) -> GridAxisProps {
        self.cols.get(&col).cloned().unwrap_or_default()
    }

    pub fn set_row(&mut self, row: u32, props: GridAxisProps) {
        self.rows.insert(row, props);
    }

    pub fn set_col(&mut self, col: u32, props: GridAxisProps) {
        self.cols.insert(col, props);
    }

    #[must_use]
    pub fn hidden_sensitive_row_context(
        &self,
        rows: impl IntoIterator<Item = u32>,
    ) -> GridVisibilityRange {
        let mut total_rows = 0;
        let mut manually_hidden_rows = 0;
        let mut filtered_hidden_rows = 0;
        for row in rows {
            total_rows += 1;
            let props = self.row(row);
            if props.hidden_manual {
                manually_hidden_rows += 1;
            }
            if props.hidden_filter {
                filtered_hidden_rows += 1;
            }
        }
        GridVisibilityRange {
            total_rows,
            manually_hidden_rows,
            filtered_hidden_rows,
        }
    }

    pub(super) fn aggregate_row_context_runs(
        &self,
        first_row: u32,
        last_row: u32,
    ) -> (Vec<GridAggregateRowContextRun>, usize, usize) {
        let mut runs = Vec::new();
        let mut default_row_runs = 0_usize;
        let mut explicit_axis_row_entries_visited = 0_usize;
        let mut cursor = first_row;
        for (&row, props) in self.rows.range(first_row..=last_row) {
            explicit_axis_row_entries_visited += 1;
            if cursor < row {
                push_aggregate_row_context_run(
                    &mut runs,
                    cursor,
                    row - 1,
                    GridAxisProps::visible(),
                );
                default_row_runs += 1;
            }
            push_aggregate_row_context_run(&mut runs, row, row, props.clone());
            cursor = row.saturating_add(1);
        }
        if cursor <= last_row {
            push_aggregate_row_context_run(&mut runs, cursor, last_row, GridAxisProps::visible());
            default_row_runs += 1;
        }
        (runs, explicit_axis_row_entries_visited, default_row_runs)
    }

    pub(super) fn apply_axis_edit(
        &mut self,
        edit: GridAxisEdit,
        bounds: ExcelGridBounds,
    ) -> Result<(usize, usize), GridRefError> {
        validate_axis_edit(edit, bounds)?;
        let max = axis_max(edit.axis, bounds);
        let map = match edit.axis {
            GridAxis::Row => &mut self.rows,
            GridAxis::Column => &mut self.cols,
        };
        let old = std::mem::take(map);
        let mut kept = 0;
        let mut dropped = 0;
        for (index, props) in old {
            match transform_axis_index(index, edit.kind, max)? {
                Some(new_index) => {
                    map.insert(new_index, props);
                    kept += 1;
                }
                None => dropped += 1,
            }
        }
        Ok((kept, dropped))
    }
}

pub(super) fn push_aggregate_row_context_run(
    runs: &mut Vec<GridAggregateRowContextRun>,
    first_row: u32,
    last_row: u32,
    props: GridAxisProps,
) {
    if first_row > last_row {
        return;
    }
    if let Some(last) = runs.last_mut() {
        if last.last_row.saturating_add(1) == first_row && last.props == props {
            last.last_row = last_row;
            return;
        }
    }
    runs.push(GridAggregateRowContextRun {
        first_row,
        last_row,
        props,
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GridVisibilityRange {
    pub total_rows: u32,
    pub manually_hidden_rows: u32,
    pub filtered_hidden_rows: u32,
}
