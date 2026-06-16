#![forbid(unsafe_code)]

//! Reference-profile selection helpers for OxCalc hosts.
//!
//! This is a small naming adapter over host-owned profiles. OxFml still owns
//! formula structure and the profile trait; OxCalc chooses which host profile,
//! if any, should be supplied for a calculation mode.

use oxfml_core::binding::ReferenceBindProfile;

use crate::excel_grid_reference::{
    EXCEL_GRID_PROFILE_ID, STRICT_EXCEL_GRID_PROFILE_ALIAS, StrictExcelGridReferenceProfile,
};
use crate::tree_reference_system::{TREECALC_REFERENCE_SYSTEM_ID, treecalc_reference_bind_profile};

pub const FORMULA_ONLY_PROFILE_ID: &str = "formula-only";
pub const HOST_CAPABILITIES_ALIAS_PREFIX: &str = "host-capabilities:";

static STRICT_EXCEL_GRID_REFERENCE_BIND_PROFILE: StrictExcelGridReferenceProfile =
    StrictExcelGridReferenceProfile::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OxCalcReferenceProfileSelection {
    FormulaOnly,
    DnaTreeCalcV1,
    ExcelGridV1,
}

impl OxCalcReferenceProfileSelection {
    #[must_use]
    pub const fn canonical_profile_id(self) -> &'static str {
        match self {
            OxCalcReferenceProfileSelection::FormulaOnly => FORMULA_ONLY_PROFILE_ID,
            OxCalcReferenceProfileSelection::DnaTreeCalcV1 => TREECALC_REFERENCE_SYSTEM_ID,
            OxCalcReferenceProfileSelection::ExcelGridV1 => EXCEL_GRID_PROFILE_ID,
        }
    }

    #[must_use]
    pub fn reference_bind_profile(self) -> Option<&'static dyn ReferenceBindProfile> {
        match self {
            OxCalcReferenceProfileSelection::FormulaOnly => None,
            OxCalcReferenceProfileSelection::DnaTreeCalcV1 => {
                Some(treecalc_reference_bind_profile())
            }
            OxCalcReferenceProfileSelection::ExcelGridV1 => {
                Some(&STRICT_EXCEL_GRID_REFERENCE_BIND_PROFILE)
            }
        }
    }
}

#[must_use]
pub fn resolve_reference_profile_selection(input: &str) -> Option<OxCalcReferenceProfileSelection> {
    let trimmed = input.trim();
    let compatibility_alias = strip_host_capabilities_alias(trimmed);
    match compatibility_alias.to_ascii_lowercase().as_str() {
        FORMULA_ONLY_PROFILE_ID => Some(OxCalcReferenceProfileSelection::FormulaOnly),
        TREECALC_REFERENCE_SYSTEM_ID | "treecalc" | "oxcalc-tree" => {
            Some(OxCalcReferenceProfileSelection::DnaTreeCalcV1)
        }
        EXCEL_GRID_PROFILE_ID | STRICT_EXCEL_GRID_PROFILE_ALIAS => {
            Some(OxCalcReferenceProfileSelection::ExcelGridV1)
        }
        _ => None,
    }
}

#[must_use]
pub fn reference_bind_profile_for_selection(
    selection: OxCalcReferenceProfileSelection,
) -> Option<&'static dyn ReferenceBindProfile> {
    selection.reference_bind_profile()
}

fn strip_host_capabilities_alias(input: &str) -> &str {
    if input.len() < HOST_CAPABILITIES_ALIAS_PREFIX.len()
        || !input.is_char_boundary(HOST_CAPABILITIES_ALIAS_PREFIX.len())
    {
        return input;
    }
    let (prefix, rest) = input.split_at(HOST_CAPABILITIES_ALIAS_PREFIX.len());
    if prefix.eq_ignore_ascii_case(HOST_CAPABILITIES_ALIAS_PREFIX) {
        rest
    } else {
        input
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selector_resolves_canonical_profile_names() {
        assert_eq!(
            resolve_reference_profile_selection(FORMULA_ONLY_PROFILE_ID),
            Some(OxCalcReferenceProfileSelection::FormulaOnly)
        );
        assert_eq!(
            resolve_reference_profile_selection(TREECALC_REFERENCE_SYSTEM_ID),
            Some(OxCalcReferenceProfileSelection::DnaTreeCalcV1)
        );
        assert_eq!(
            resolve_reference_profile_selection(EXCEL_GRID_PROFILE_ID),
            Some(OxCalcReferenceProfileSelection::ExcelGridV1)
        );
    }

    #[test]
    fn selector_keeps_strict_excel_grid_as_alias() {
        let selection = resolve_reference_profile_selection(STRICT_EXCEL_GRID_PROFILE_ALIAS)
            .expect("strict Excel alias should resolve");

        assert_eq!(
            selection.canonical_profile_id(),
            EXCEL_GRID_PROFILE_ID,
            "strict-excel-grid is a mode alias, not the canonical OxFml profile id"
        );
        assert_eq!(
            selection
                .reference_bind_profile()
                .expect("grid selection should supply profile")
                .profile_id(),
            EXCEL_GRID_PROFILE_ID
        );
    }

    #[test]
    fn selector_accepts_host_capabilities_aliases_as_compatibility_inputs() {
        assert_eq!(
            resolve_reference_profile_selection("host-capabilities:strict-excel-grid"),
            Some(OxCalcReferenceProfileSelection::ExcelGridV1)
        );
        assert_eq!(
            resolve_reference_profile_selection("host-capabilities:dna.treecalc.v1"),
            Some(OxCalcReferenceProfileSelection::DnaTreeCalcV1)
        );
        assert_eq!(
            resolve_reference_profile_selection("host-capabilities:formula-only"),
            Some(OxCalcReferenceProfileSelection::FormulaOnly)
        );
        assert_eq!(
            resolve_reference_profile_selection("Host-Capabilities:Strict-Excel-Grid"),
            Some(OxCalcReferenceProfileSelection::ExcelGridV1)
        );
    }

    #[test]
    fn formula_only_selection_supplies_no_reference_bind_profile() {
        assert_eq!(
            OxCalcReferenceProfileSelection::FormulaOnly.canonical_profile_id(),
            FORMULA_ONLY_PROFILE_ID
        );
        assert!(
            reference_bind_profile_for_selection(OxCalcReferenceProfileSelection::FormulaOnly)
                .is_none()
        );
    }

    #[test]
    fn selector_rejects_unknown_profiles() {
        assert_eq!(resolve_reference_profile_selection("not-a-profile"), None);
    }
}
