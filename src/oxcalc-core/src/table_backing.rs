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
//! It carries the descriptor seam plus descriptor-pure structured-section
//! resolution ([`TableBacking::resolve_section`], a selection over the
//! descriptor's A1 refs). Precise body-range arithmetic that excludes
//! header/totals rows (the `selected_rows_for_sparse_reader` family, already over
//! `&TableDescriptor`) and the scattered A1 utilities it needs are a later
//! refinement.

use oxfml_core::StructuredSectionKind;
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

/// The A1 region ref(s) a structured-reference section resolves to on a backing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SectionResolution {
    /// The section's A1 region ref(s) in the backing's coordinate space.
    Refs(Vec<String>),
    /// The section is absent on this table (e.g. `Totals` with no totals row).
    Absent,
}

/// A backing that can produce a profile-neutral table descriptor and resolve
/// structured-reference sections over it.
///
/// Implemented by the grid table overlay (`GridTableOverlay`) and the tree table
/// projection (`TreeCalcTableNodeProjection`). Both already build a
/// `TableDescriptor`; this trait is the single seam they share, so the rest of
/// the engine consumes a table the same way regardless of backing.
pub trait TableBacking {
    fn table_spec(&self) -> TableSpec;

    /// Resolve a structured-reference section (optionally filtered to specific
    /// columns by id) to the A1 region ref(s) on this backing's descriptor.
    /// Descriptor-pure: a default selection over the descriptor's existing A1
    /// refs, identical across backings. (Precise body-range arithmetic that
    /// excludes header/totals rows needs A1 parsing and is a later refinement;
    /// this returns whole-region refs.)
    fn resolve_section(
        &self,
        section: StructuredSectionKind,
        column_ids: &[String],
    ) -> SectionResolution {
        resolve_section_over_descriptor(&self.table_spec().descriptor, section, column_ids)
    }
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

/// Resolve a structured-reference section over a `TableDescriptor` to its A1
/// region ref(s). The descriptor already carries A1 refs (the coordinate
/// language both backings speak), so this is a pure selection: headers/totals
/// pick the header/totals region ref; a column filter picks those columns' data
/// range refs; `Data`/`All`/`ThisRow` without a column filter pick the whole
/// table range ref. Backing-neutral; no coordinate arithmetic.
#[must_use]
pub fn resolve_section_over_descriptor(
    descriptor: &TableDescriptor,
    section: StructuredSectionKind,
    column_ids: &[String],
) -> SectionResolution {
    let column_ref = |id: &str| {
        descriptor
            .columns
            .iter()
            .find(|column| column.column_id == id)
            .map(|column| column.column_range_ref.clone())
    };
    let refs: Vec<String> = match section {
        StructuredSectionKind::Headers => {
            descriptor.header_region_ref.clone().into_iter().collect()
        }
        StructuredSectionKind::Totals => descriptor.totals_region_ref.clone().into_iter().collect(),
        StructuredSectionKind::Data
        | StructuredSectionKind::All
        | StructuredSectionKind::ThisRow => {
            if column_ids.is_empty() {
                vec![descriptor.table_range_ref.clone()]
            } else {
                column_ids.iter().filter_map(|id| column_ref(id)).collect()
            }
        }
    };
    if refs.is_empty() {
        SectionResolution::Absent
    } else {
        SectionResolution::Refs(refs)
    }
}
