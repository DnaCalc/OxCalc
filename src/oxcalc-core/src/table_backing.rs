#![forbid(unsafe_code)]

//! Profile-neutral table backing seam (`TableBacking` / `TableSpec`).
//!
//! A table is one logical abstraction with two backings: a tree-standalone
//! table (body cells are child nodes) and a grid-overlay table (body cells are
//! a rect on a sheet). Both implement [`TableBacking`] and produce the same
//! OxFml [`TableDescriptor`] interop shape, differing only in coordinate refs
//! and the row-identity scheme. Invariant **T1**
//! ([`descriptors_agree_modulo_coordinates`]) is the conformance gate that keeps
//! the two backings one abstraction rather than two that silently drift.
//!
//! Today this is the descriptor seam; it will grow the shared section-resolution
//! arithmetic (the `selected_rows_for_sparse_reader` family, already written over
//! `&TableDescriptor`) behind the trait as the unification proceeds.

use oxfml_core::interface::TableDescriptor;

/// The resolved, backing-neutral table fact.
///
/// Wraps the OxFml [`TableDescriptor`] interop shape — the coordinate language
/// (A1 `*_ref` strings) both backings already speak.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableSpec {
    pub descriptor: TableDescriptor,
}

impl TableSpec {
    #[must_use]
    pub fn new(descriptor: TableDescriptor) -> Self {
        Self { descriptor }
    }
}

/// A backing that can produce a profile-neutral table descriptor.
///
/// Implemented by the grid table overlay (`GridTableOverlay`) and the tree table
/// projection (`TreeCalcTableNodeProjection`). Both already build a
/// `TableDescriptor`; this trait is the single seam they share, so the rest of
/// the engine consumes a table the same way regardless of backing.
pub trait TableBacking {
    fn table_spec(&self) -> TableSpec;
}

/// Invariant **T1**: two backings describe the *same logical table* when their
/// descriptors agree on identity — table id/name, column id/name/ordinal, and
/// header/totals presence — ignoring the coordinate refs (`*_ref`) and the
/// row-identity scheme, which are legitimately backing-specific (a grid table
/// lives at sheet coordinates; a tree table at a virtual anchor).
#[must_use]
pub fn descriptors_agree_modulo_coordinates(a: &TableDescriptor, b: &TableDescriptor) -> bool {
    a.table_id == b.table_id
        && a.table_name == b.table_name
        && a.header_row_present == b.header_row_present
        && a.totals_row_present == b.totals_row_present
        && a.columns.len() == b.columns.len()
        && a.columns.iter().zip(b.columns.iter()).all(|(x, y)| {
            x.column_id == y.column_id && x.column_name == y.column_name && x.ordinal == y.ordinal
        })
}
