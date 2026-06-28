//! The unified grid overlay abstraction: a rect-claiming adornment on a sheet.
//!
//! Generalizes the four ad-hoc rect-claimers - structured tables
//! (`GridTableOverlay`), merged regions (`GridMergedRegion`), feature-rendered
//! regions (`FeatureRenderedRegion`), and spill extents (`GridSpillFact`) -
//! behind one [`GridOverlay`] value with a [`kind`](GridOverlay::kind)
//! discriminant, so the blocker and axis-edit passes can iterate one set
//! instead of four. This bead (OVL-2) is an **adapter only**: every method
//! forwards to the existing per-type predicate, and
//! `overlay_set_blockage_probe` is proven equal to the legacy probe before any
//! production path is rerouted (OVL-3+). Nothing here changes behaviour yet.
//!
//! The `transform_for_axis_edit` method returns `Self`, so the overlay value is
//! a closed `enum` rather than a `dyn` trait object: the machine derives
//! `PartialEq`/`Clone` and the warm-no-op token compares overlays structurally.

use super::*;

/// The four rect-claiming overlay stores that live identically on
/// `GridOptimizedSheet` and `GridCalcRefSheet`, collapsed into one value
/// (OVL-5a). The `pub(super)` fields preserve the prior direct-field-assignment
/// seams (`project_authored_to_reference`, the axis-edit `mem::take` rebuild);
/// external access still routes through the sheets' accessor methods
/// (`spill_facts`, `table_overlays`, `merged_regions`,
/// `feature_rendered_regions`). The field order matches the prior struct order,
/// so the sheets' derived `Debug`/`PartialEq` keep identical semantics.
#[derive(Debug, Clone, PartialEq, Default)]
pub(super) struct GridOverlaySet {
    pub(super) merged_regions: Vec<GridMergedRegion>,
    pub(super) feature_rendered_regions: Vec<FeatureRenderedRegion>,
    pub(super) spill_facts: BTreeMap<ExcelGridCellAddress, GridSpillFact>,
    pub(super) table_overlays: BTreeMap<String, GridTableOverlay>,
}

/// The family of a grid overlay. A stable discriminant for per-kind reporting
/// and routing. The seam variants (`Cse`/`ConditionalFormat`/`RichObject`/
/// `Extension`) are reserved for later beads and have no concrete overlay value
/// yet.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayKind {
    Table,
    Merged,
    FeatureRendered,
    Spill,
    Cse,
    ConditionalFormat,
    RichObject,
    Extension,
}

/// How an overlay blocks a spill that would overlap it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpillBlock {
    /// Does not block.
    None,
    /// Blocks implicitly: another live spill in the way yields `#SPILL!`.
    Implicit,
    /// Blocks hard: a structural occupant (merge, table feature) the spill
    /// cannot grow through.
    Hard,
}

/// Whether an overlay admits a structural axis edit (insert/delete row or
/// column) that intersects it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditAdmission {
    Allow,
    Refuse { detail: String },
}

/// A rect-claiming overlay on a grid sheet. The four concrete claimers plus the
/// inert `Extension` seam (OVL-6) reserving a seat for the spatial families with
/// no engine support yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GridOverlay {
    Table(GridTableOverlay),
    Merged(GridMergedRegion),
    FeatureRendered(FeatureRenderedRegion),
    Spill(GridSpillFact),
    /// An inert, forward-looking overlay (OVL-6); see `GridOverlayExtension`.
    Extension(GridOverlayExtension),
}

/// An inert, forward-looking overlay seam (OVL-6). It reserves a seat for the
/// spatial families that have no engine support yet (legacy CSE arrays,
/// conditional-format runs, RichObject placements, and open-ended extensions
/// such as charts, shapes, or data-validation) instead of giving each a bespoke
/// variant. `kind_tag` records which family it stands for; `claimed_rect` is the
/// single rect it occupies; `block_mode` and `refuses_axis_edit` carry its
/// spill-blockage and structural-edit policy (built today with the inert
/// `SpillBlock::None` / `false`, so an extension is a blockage and admission
/// no-op); `payload` is an opaque family-specific tag. Nothing in the live
/// engine constructs one yet; CSE-1 / CF-1 / RICH-1 will, adding storage and a
/// redistribution arm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridOverlayExtension {
    pub kind_tag: OverlayKind,
    pub claimed_rect: GridRect,
    pub block_mode: SpillBlock,
    pub refuses_axis_edit: bool,
    pub payload: String,
}

impl GridOverlay {
    /// The overlay's family discriminant.
    #[must_use]
    pub fn kind(&self) -> OverlayKind {
        match self {
            Self::Table(_) => OverlayKind::Table,
            Self::Merged(_) => OverlayKind::Merged,
            Self::FeatureRendered(_) => OverlayKind::FeatureRendered,
            Self::Spill(_) => OverlayKind::Spill,
            Self::Extension(extension) => extension.kind_tag,
        }
    }

    /// The overlay's primary claimed rectangle (the table range, the merged
    /// rect, the feature rect, or the spill extent).
    #[must_use]
    pub fn claimed_rect(&self) -> &GridRect {
        match self {
            Self::Table(table) => &table.table_range,
            Self::Merged(region) => &region.rect,
            Self::FeatureRendered(region) => &region.rect,
            Self::Spill(fact) => &fact.extent,
            Self::Extension(extension) => &extension.claimed_rect,
        }
    }

    /// Every rectangle the overlay claims. A table claims its range plus its
    /// header, totals, and per-column bands; the others claim a single rect.
    #[must_use]
    pub fn claimed_rects(&self) -> Vec<&GridRect> {
        match self {
            Self::Table(table) => {
                let mut rects = vec![&table.table_range];
                if let Some(rect) = &table.header_rect {
                    rects.push(rect);
                }
                if let Some(rect) = &table.totals_rect {
                    rects.push(rect);
                }
                rects.extend(table.columns.iter().map(|column| &column.data_rect));
                rects
            }
            Self::Merged(region) => vec![&region.rect],
            Self::FeatureRendered(region) => vec![&region.rect],
            Self::Spill(fact) => vec![&fact.extent],
            Self::Extension(extension) => vec![&extension.claimed_rect],
        }
    }

    /// How this overlay blocks a spill. Mirrors the legacy probe exactly: a
    /// merge and a spill-blocking feature block hard; a published (unblocked)
    /// spill blocks implicitly (`#SPILL!`); a table does *not* block directly
    /// (its companion feature-rendered region does); a blocked spill does not
    /// block here (the blocked-formula anchor-containment pre-pass handles it).
    #[must_use]
    pub fn blocks_spill(&self) -> SpillBlock {
        match self {
            Self::Merged(_) => SpillBlock::Hard,
            Self::FeatureRendered(region) => {
                if feature_rendered_region_blocks_spill(&region.feature_kind) {
                    SpillBlock::Hard
                } else {
                    SpillBlock::None
                }
            }
            Self::Spill(fact) => {
                if fact.blocked {
                    SpillBlock::None
                } else {
                    SpillBlock::Implicit
                }
            }
            Self::Table(_) => SpillBlock::None,
            Self::Extension(extension) => extension.block_mode,
        }
    }

    /// Whether this overlay admits a structural axis edit intersecting it. Only
    /// the pivot family of feature-rendered regions refuses; everything else is
    /// transformed. The refusal detail matches the legacy
    /// `FeatureRenderedRegionEditRefused` string so the message is unchanged
    /// when the axis-edit pass routes through here (OVL-4).
    pub fn admit_axis_edit(&self, edit: GridAxisEdit) -> Result<EditAdmission, GridRefError> {
        match self {
            Self::FeatureRendered(region)
                if feature_rendered_region_axis_edit_refused(region, edit)? =>
            {
                Ok(EditAdmission::Refuse {
                    detail: format!(
                        "{:?} edit intersects claimed region R{}C{}:R{}C{}",
                        edit.axis,
                        region.rect.top_row,
                        region.rect.left_col,
                        region.rect.bottom_row,
                        region.rect.right_col
                    ),
                })
            }
            Self::Extension(extension)
                if extension.refuses_axis_edit
                    && axis_edit_intersects_rect(&extension.claimed_rect, edit)? =>
            {
                Ok(EditAdmission::Refuse {
                    detail: format!(
                        "{:?} edit intersects claimed region R{}C{}:R{}C{}",
                        edit.axis,
                        extension.claimed_rect.top_row,
                        extension.claimed_rect.left_col,
                        extension.claimed_rect.bottom_row,
                        extension.claimed_rect.right_col
                    ),
                })
            }
            _ => Ok(EditAdmission::Allow),
        }
    }

    /// Transform the overlay for a structural axis edit, forwarding to the
    /// existing per-type transform. Returns `None` when the overlay is dropped
    /// (its rect fell entirely inside a deletion). Errors when a feature refuses
    /// the edit - identical to the legacy paths.
    pub fn transform_for_axis_edit(
        &self,
        edit: GridAxisEdit,
        bounds: ExcelGridBounds,
    ) -> Result<Option<Self>, GridRefError> {
        match self {
            Self::Table(table) => Ok(table
                .transform_for_axis_edit(edit, bounds)?
                .map(Self::Table)),
            Self::Merged(region) => {
                let (Some(rect), _) = transform_rect_for_edit(&region.rect, edit, bounds)? else {
                    return Ok(None);
                };
                Ok(Some(Self::Merged(GridMergedRegion { rect })))
            }
            Self::FeatureRendered(region) => {
                if feature_rendered_region_axis_edit_refused(region, edit)? {
                    return Err(GridRefError::FeatureRenderedRegionEditRefused {
                        feature_kind: region.feature_kind.clone(),
                        detail: format!(
                            "{:?} edit intersects claimed region R{}C{}:R{}C{}",
                            edit.axis,
                            region.rect.top_row,
                            region.rect.left_col,
                            region.rect.bottom_row,
                            region.rect.right_col
                        ),
                    });
                }
                let (Some(rect), outcome) = transform_rect_for_edit(&region.rect, edit, bounds)?
                else {
                    return Ok(None);
                };
                let mut needs_refresh = region.needs_refresh;
                if feature_rendered_region_marks_refresh_on_transform(&region.feature_kind)
                    && outcome != GridStructuralTransformOutcome::Unchanged
                {
                    needs_refresh = true;
                }
                Ok(Some(Self::FeatureRendered(FeatureRenderedRegion {
                    rect,
                    feature_kind: region.feature_kind.clone(),
                    needs_refresh,
                })))
            }
            Self::Spill(fact) => {
                let Some(anchor) = transform_address_for_edit(&fact.anchor, edit, bounds)? else {
                    return Ok(None);
                };
                let (Some(extent), _) = transform_rect_for_edit(&fact.extent, edit, bounds)? else {
                    return Ok(None);
                };
                Ok(Some(Self::Spill(GridSpillFact {
                    anchor,
                    extent,
                    blocked: fact.blocked,
                })))
            }
            Self::Extension(extension) => {
                // Geometric only: a refusing extension is rejected by the
                // upfront admit pass before transform runs (a future bead that
                // stores extensions must extend that pass to cover them), so the
                // transform never re-checks refuses_axis_edit here.
                let (Some(claimed_rect), _) =
                    transform_rect_for_edit(&extension.claimed_rect, edit, bounds)?
                else {
                    return Ok(None);
                };
                Ok(Some(Self::Extension(GridOverlayExtension {
                    kind_tag: extension.kind_tag,
                    claimed_rect,
                    block_mode: extension.block_mode,
                    refuses_axis_edit: extension.refuses_axis_edit,
                    payload: extension.payload.clone(),
                })))
            }
        }
    }
}

/// Whether a structural axis edit cuts through `rect` along its axis - the same
/// intersection test `feature_rendered_region_axis_edit_refused` applies, lifted
/// to a bare rect so the `GridOverlay::Extension` seam can share it.
fn axis_edit_intersects_rect(rect: &GridRect, edit: GridAxisEdit) -> Result<bool, GridRefError> {
    let (start, end) = rect_axis_range(rect, edit.axis);
    match edit.kind {
        GridAxisEditKind::Insert { before, .. } => Ok(start < before && before <= end),
        GridAxisEditKind::Delete { first, count } => {
            let last = delete_last(first, count)?;
            Ok(first <= end && start <= last)
        }
    }
}
