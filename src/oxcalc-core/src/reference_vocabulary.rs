#![forbid(unsafe_code)]

//! The profile structural-vocabulary layer (W062 D2 §1/§2).
//!
//! This module is the OxCalc-side vocabulary layer that names, per reference
//! profile, how the shared structural model's container roles are addressed.
//! It is attached to profiles through the [`OxCalcReferenceProfile`] subtrait
//! of OxFml's `ReferenceBindProfile`, so every OxCalc call site that needs
//! vocabulary holds `&dyn OxCalcReferenceProfile` while every crossing of the
//! OxFml seam upcasts to `&dyn ReferenceBindProfile` (contract V1). Vocabulary
//! types never cross into OxFml.
//!
//! **Scope of this bead (R3.1 — `calc-5kqg.20`).** This is the *layer*, not the
//! consumers. It defines the vocabulary traits and both shipped profile impls,
//! and it carries the D2 §2 container-role admissibility table and the D2 §6
//! per-profile [`ContainerDeletionPolicy`]. It performs **no** resolution: the
//! live `!`-qualifier resolution against a container catalog is R3.2+ work
//! (`WorkbookReferenceCatalog` / the `ContainerCatalog` surface do not exist
//! yet). [`ContainerResolution`] is defined here as the typed outcome the R3.2+
//! resolver will produce, so the shape is fixed once, but nothing in this bead
//! constructs it in a resolution path. Zero behavior change is expected.
//!
//! **Seam guardrail.** `NodeRole`, deletion policy, and container roles are
//! OxCalc model concepts; per the W077 GridBounds ratification they must not
//! move upstream. The subtrait lives entirely in OxCalc and OxFml continues to
//! see only `dyn ReferenceBindProfile` — no OxFml change is required or made.

use oxfml_core::binding::ReferenceBindProfile;

use crate::structural::{NodeRole, TreeNodeId};

/// A stable, rename-immune sheet identity, minted from the sheet node's
/// [`TreeNodeId`] at registration (W062 D2 §10). This is the sheet component of
/// every normal-form/dependency key: **the display name never enters keys**, so
/// a sheet rename is a pure render/catalog event with zero graph impact.
///
/// The token's string form is `sheet-node:{id}` (D2 §10). Existing fixture
/// constants such as `sheet:default` remain valid opaque tokens — this bead
/// mints the token *type* and the catalog lookups over it; it does **not**
/// rewrite any existing key string. Adoption of the token into key strings is
/// R3.3's resolution wiring, sequenced there by D2 §10, so nothing here flips a
/// format-stable key.
///
/// The `u64 <-> TreeNodeId` conversion is centralized here and in
/// [`ContainerResolution::sheet`] (R3.1 reviewer note): callers mint tokens and
/// build [`ContainerResolution::Sheet`] through these constructors rather than
/// casting inline.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SheetIdentityToken(String);

impl SheetIdentityToken {
    /// The D2 §10 token prefix. The full token is `{PREFIX}{id}`.
    const PREFIX: &'static str = "sheet-node:";

    /// Mints the rename-immune token for a sheet node. The token depends only on
    /// the node's stable [`TreeNodeId`], so it survives every rename of the
    /// sheet's display name.
    #[must_use]
    pub fn from_node_id(node_id: TreeNodeId) -> Self {
        Self(format!("{}{}", Self::PREFIX, node_id.0))
    }

    /// The token's canonical string form (`sheet-node:{id}`), as it would appear
    /// as the `{sheet}` component of a normal-form key.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The node id this token was minted from, recovered by parsing the
    /// canonical form. `None` for a foreign/opaque token string that does not
    /// carry the `sheet-node:` prefix (e.g. the legacy `sheet:default`
    /// fixture constant), which is a valid token but not node-minted.
    #[must_use]
    pub fn node_id(&self) -> Option<TreeNodeId> {
        self.0
            .strip_prefix(Self::PREFIX)
            .and_then(|rest| rest.parse::<u64>().ok())
            .map(TreeNodeId)
    }
}

/// A container role that the `!` qualifier's *left side* may name, in profile
/// vocabulary terms (W062 D2 §2). This is distinct from [`NodeRole`], the
/// authored structural-model role: `ContainerRole` is the *addressing*
/// vocabulary a profile admits for container qualification, while `NodeRole`
/// is the model fact a resolved qualifier ultimately lands on.
///
/// Profiles differ only in which of these they admit (D2 §2 table):
///
/// | Profile | admitted container roles |
/// | --- | --- |
/// | `excel.grid.v1` | [`Sheet`](ContainerRole::Sheet), [`Workbook`](ContainerRole::Workbook) (the `[Book]` external form) |
/// | `dna.treecalc.v1` | [`Workspace`](ContainerRole::Workspace) |
/// | `formula-only` | none |
///
/// Marked `#[non_exhaustive]`: future container roles extend this enum without
/// disturbing existing match arms.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContainerRole {
    /// A worksheet within a workbook — the left side of `Sheet1!A1`.
    Sheet,
    /// An external workbook — the bracketed `[Book2]` form of
    /// `[Book2]Sheet1!A1`. Admitted by the strict profile; resolution against a
    /// loaded sibling workspace is R3.7.
    Workbook,
    /// A workspace in the same `OxCalcTreeContext` — the left side of
    /// `Workspace!Path.To.Node`. Admitted by the tree profile; resolution is
    /// R3.6.
    Workspace,
}

/// Per-profile semantics for a container-qualified reference whose container is
/// deleted (W062 D2 §6). Carried by the vocabulary; the deletion transform
/// driver (R3.4, contract V7) consults it. Verbatim from D2 §6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContainerDeletionPolicy {
    /// Excel-faithful: a reference to a deleted container becomes a hard
    /// `#REF!` (a destructive transform of the bound record) and does **not**
    /// heal if a container with the same name is recreated. Undo heals via
    /// revision restore, not recreate. Used by the strict-excel profile.
    HardRefError,
    /// Lenient: a reference to a deleted/unavailable container becomes a
    /// dormant identity edge that heals when a container with the same
    /// normalized name reappears. Used by the tree profile (and the default for
    /// lenient profiles).
    DormantIdentityHeal,
}

/// The typed outcome of resolving a `!` container qualifier against the model
/// (W062 D2 §1). Defined here so the shape is fixed once; **constructed only by
/// the R3.2+ resolver**, never in this bead. The catalog-consuming
/// `resolve_container_qualifier` method that yields this lands with the
/// `WorkbookReferenceCatalog` / workspace-alias catalog in R3.2 (§4.1/§9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainerResolution {
    /// The qualifier named a sheet in this workbook; carries the resolved
    /// sheet node identity. `u64` mirrors `TreeNodeId`'s inner id without
    /// forcing this layer to depend on resolution-time node handles.
    Sheet { node_id: u64 },
    /// The qualifier named a workspace in the same context; carries the
    /// resolved workspace alias in normalized form.
    Workspace { alias: NormalizedContainerName },
    /// The qualifier named an external workbook (`[Book]Sheet`).
    External {
        book: NormalizedContainerName,
        sheet: NormalizedContainerName,
    },
    /// The qualifier named a container that does not currently exist; carries
    /// its normalized name for the dormant identity edge (heal admissibility is
    /// the [`ContainerDeletionPolicy`]).
    Dormant { name: NormalizedContainerName },
    /// The qualifier is not admissible in this profile, or is otherwise
    /// rejected; carries a stable diagnostic code and message.
    Rejected { code: String, message: String },
}

impl ContainerResolution {
    /// Constructs a [`ContainerResolution::Sheet`] from a [`TreeNodeId`],
    /// centralizing the `TreeNodeId -> u64` mapping (R3.1 reviewer note) so no
    /// call site casts inline. The variant stores the raw `u64` to avoid
    /// forcing this layer's public shape to depend on resolution-time handles;
    /// [`Self::sheet_node_id`] reverses the mapping.
    #[must_use]
    pub fn sheet(node_id: TreeNodeId) -> Self {
        Self::Sheet {
            node_id: node_id.0,
        }
    }

    /// Recovers the [`TreeNodeId`] from a [`ContainerResolution::Sheet`],
    /// centralizing the `u64 -> TreeNodeId` mapping. `None` for other variants.
    #[must_use]
    pub fn sheet_node_id(&self) -> Option<TreeNodeId> {
        match self {
            Self::Sheet { node_id } => Some(TreeNodeId(*node_id)),
            _ => None,
        }
    }
}

/// A container name folded to its case-insensitive canonical form, using the
/// one shared fold (contract V3, [`crate::structural::fold_name_case_insensitive`]).
/// This is the workspace/workbook analogue of [`NormalizedSheetName`]; both are
/// produced from the single fold so the two lanes cannot drift.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NormalizedContainerName(String);

impl NormalizedContainerName {
    /// Folds a display name to its case-insensitive canonical form via the
    /// shared V3 fold.
    #[must_use]
    pub fn from_symbol(symbol: &str) -> Self {
        Self(crate::structural::fold_name_case_insensitive(symbol))
    }

    /// The folded canonical form as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The OxCalc-side structural vocabulary for a reference profile (W062 D2 §1).
///
/// Carries the container-role vocabulary — the profile's display terms for
/// structural roles, the container roles the `!` qualifier admits (D2 §2), and
/// the [`ContainerDeletionPolicy`] (D2 §6). Object-safe by construction so it is
/// reachable as `&dyn StructuralVocabulary` through
/// [`OxCalcReferenceProfile::vocabulary`].
///
/// R3.1 is the layer only. The live catalog-consuming
/// `resolve_container_qualifier(&self, qualifier, catalog) -> ContainerResolution`
/// method from D2 §1 lands in R3.2 once the `ContainerCatalog` surface exists;
/// this trait admits it as an additive method then.
pub trait StructuralVocabulary {
    /// Profile term for a structural container role — how this profile names
    /// the model's roles for display/diagnostics (e.g. `"Sheet"` for
    /// strict-excel, `"Workspace"` for the tree profile). `None` means this
    /// profile has no display term for that role.
    fn role_term(&self, role: NodeRole) -> Option<&str>;

    /// Profile term for the document root role (D2 §1 "root role name") — e.g.
    /// `"Workbook"` for strict-excel, `"Workspace"` for the tree profile.
    fn root_role_term(&self) -> &str;

    /// Whether the `!` qualifier's left side may name a container of `role` in
    /// this profile (D2 §2). This is the admissibility query the acceptance
    /// criteria pin to the D2 §2 table.
    fn admits_container_role(&self, role: ContainerRole) -> bool;

    /// The container roles this profile admits for `!` qualification, in a
    /// stable order (D2 §2). Empty for the formula-only vocabulary.
    fn admitted_container_roles(&self) -> &[ContainerRole];

    /// The deletion semantics this profile applies to dangling
    /// container-qualified references (D2 §6).
    fn container_deletion_policy(&self) -> ContainerDeletionPolicy;
}

/// The OxCalc-internal profile handle (contract V1): a `ReferenceBindProfile`
/// that additionally exposes its [`StructuralVocabulary`]. The single
/// object-safe method keeps `dyn` compatibility; OxFml never sees this subtrait.
pub trait OxCalcReferenceProfile: ReferenceBindProfile {
    /// The one vocabulary object for this bound profile. Bind time and
    /// resolution time see the same object by construction (D2 §1), so no
    /// second registry can drift.
    fn vocabulary(&self) -> &dyn StructuralVocabulary;
}

/// The strict-excel-grid vocabulary (`excel.grid.v1`). Admits `Sheet` and the
/// external `Workbook` (`[Book]`) container roles (D2 §2); hard `#REF!` on
/// container deletion (D2 §6).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StrictExcelGridVocabulary;

const STRICT_EXCEL_CONTAINER_ROLES: &[ContainerRole] =
    &[ContainerRole::Sheet, ContainerRole::Workbook];

impl StructuralVocabulary for StrictExcelGridVocabulary {
    fn role_term(&self, role: NodeRole) -> Option<&str> {
        // Exhaustive within-crate match. `NodeRole` is `#[non_exhaustive]`, so a
        // future role variant will (correctly) force this arm to be revisited
        // rather than silently defaulting to `None`.
        match role {
            NodeRole::Workbook => Some("Workbook"),
            NodeRole::Sheet => Some("Sheet"),
        }
    }

    fn root_role_term(&self) -> &str {
        "Workbook"
    }

    fn admits_container_role(&self, role: ContainerRole) -> bool {
        STRICT_EXCEL_CONTAINER_ROLES.contains(&role)
    }

    fn admitted_container_roles(&self) -> &[ContainerRole] {
        STRICT_EXCEL_CONTAINER_ROLES
    }

    fn container_deletion_policy(&self) -> ContainerDeletionPolicy {
        ContainerDeletionPolicy::HardRefError
    }
}

/// The tree-profile vocabulary (`dna.treecalc.v1`). Admits the `Workspace`
/// container role (D2 §2); dormant-identity heal on container deletion (D2 §6).
///
/// One vocabulary per profile: both shipped tree profile objects
/// (`TreeCalcReferenceBindProfile` and `TreeCalcContextReferenceBindProfile`)
/// are the same profile family and share this vocabulary (D2 §1).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct TreeCalcVocabulary;

const TREECALC_CONTAINER_ROLES: &[ContainerRole] = &[ContainerRole::Workspace];

impl StructuralVocabulary for TreeCalcVocabulary {
    fn role_term(&self, role: NodeRole) -> Option<&str> {
        // The tree profile does not name Workbook/Sheet roles; its container
        // vocabulary is the Workspace, expressed via the root role term.
        let _ = role;
        None
    }

    fn root_role_term(&self) -> &str {
        "Workspace"
    }

    fn admits_container_role(&self, role: ContainerRole) -> bool {
        TREECALC_CONTAINER_ROLES.contains(&role)
    }

    fn admitted_container_roles(&self) -> &[ContainerRole] {
        TREECALC_CONTAINER_ROLES
    }

    fn container_deletion_policy(&self) -> ContainerDeletionPolicy {
        ContainerDeletionPolicy::DormantIdentityHeal
    }
}

/// The single shared strict-excel vocabulary instance (vocabulary is stateless
/// data, so one static serves every strict profile object).
pub static STRICT_EXCEL_GRID_VOCABULARY: StrictExcelGridVocabulary = StrictExcelGridVocabulary;

/// The single shared tree-profile vocabulary instance.
pub static TREECALC_VOCABULARY: TreeCalcVocabulary = TreeCalcVocabulary;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::structural::NormalizedSheetName;

    // --- Role admissibility: the D2 §2 table, verbatim -------------------

    #[test]
    fn strict_vocabulary_admits_sheet_and_workbook_container_roles() {
        let vocab = StrictExcelGridVocabulary;
        assert!(vocab.admits_container_role(ContainerRole::Sheet));
        assert!(vocab.admits_container_role(ContainerRole::Workbook));
        assert!(!vocab.admits_container_role(ContainerRole::Workspace));
        assert_eq!(
            vocab.admitted_container_roles(),
            &[ContainerRole::Sheet, ContainerRole::Workbook]
        );
    }

    #[test]
    fn tree_vocabulary_admits_only_workspace_container_role() {
        let vocab = TreeCalcVocabulary;
        assert!(vocab.admits_container_role(ContainerRole::Workspace));
        assert!(!vocab.admits_container_role(ContainerRole::Sheet));
        assert!(!vocab.admits_container_role(ContainerRole::Workbook));
        assert_eq!(
            vocab.admitted_container_roles(),
            &[ContainerRole::Workspace]
        );
    }

    // --- Role terms (D2 §1 display vocabulary) ---------------------------

    #[test]
    fn strict_vocabulary_role_terms_and_root() {
        let vocab = StrictExcelGridVocabulary;
        assert_eq!(vocab.role_term(NodeRole::Workbook), Some("Workbook"));
        assert_eq!(vocab.role_term(NodeRole::Sheet), Some("Sheet"));
        assert_eq!(vocab.root_role_term(), "Workbook");
    }

    #[test]
    fn tree_vocabulary_root_is_workspace() {
        let vocab = TreeCalcVocabulary;
        assert_eq!(vocab.root_role_term(), "Workspace");
        assert_eq!(vocab.role_term(NodeRole::Sheet), None);
    }

    // --- Deletion policy typed per profile (D2 §6) -----------------------

    #[test]
    fn deletion_policies_are_typed_per_profile() {
        assert_eq!(
            StrictExcelGridVocabulary.container_deletion_policy(),
            ContainerDeletionPolicy::HardRefError
        );
        assert_eq!(
            TreeCalcVocabulary.container_deletion_policy(),
            ContainerDeletionPolicy::DormantIdentityHeal
        );
    }

    // --- Object safety / dyn compatibility -------------------------------

    #[test]
    fn structural_vocabulary_is_object_safe() {
        let vocab: &dyn StructuralVocabulary = &STRICT_EXCEL_GRID_VOCABULARY;
        assert!(vocab.admits_container_role(ContainerRole::Sheet));
        assert_eq!(vocab.root_role_term(), "Workbook");

        let tree: &dyn StructuralVocabulary = &TREECALC_VOCABULARY;
        assert_eq!(
            tree.container_deletion_policy(),
            ContainerDeletionPolicy::DormantIdentityHeal
        );
    }

    // --- Sheet identity token (D2 §10): rename-immune, node-minted -------

    #[test]
    fn sheet_identity_token_is_minted_from_node_id_in_canonical_form() {
        let token = SheetIdentityToken::from_node_id(TreeNodeId(7));
        assert_eq!(token.as_str(), "sheet-node:7");
        assert_eq!(token.node_id(), Some(TreeNodeId(7)));
    }

    #[test]
    fn sheet_identity_token_round_trips_through_node_id() {
        for id in [0u64, 1, 42, u64::MAX] {
            let token = SheetIdentityToken::from_node_id(TreeNodeId(id));
            assert_eq!(token.node_id(), Some(TreeNodeId(id)));
        }
    }

    #[test]
    fn legacy_opaque_token_string_has_no_minted_node_id() {
        // Existing fixture constants like `sheet:default` remain valid opaque
        // tokens but are not node-minted, so no node id is recoverable.
        let legacy = SheetIdentityToken("sheet:default".to_string());
        assert_eq!(legacy.node_id(), None);
        assert_eq!(legacy.as_str(), "sheet:default");
    }

    // --- Centralized TreeNodeId <-> u64 mapping (R3.1 reviewer note) -----

    #[test]
    fn container_resolution_sheet_constructor_centralizes_node_id_mapping() {
        let resolution = ContainerResolution::sheet(TreeNodeId(9));
        assert_eq!(resolution, ContainerResolution::Sheet { node_id: 9 });
        assert_eq!(resolution.sheet_node_id(), Some(TreeNodeId(9)));
    }

    #[test]
    fn container_resolution_sheet_node_id_is_none_for_other_variants() {
        let dormant = ContainerResolution::Dormant {
            name: NormalizedContainerName::from_symbol("Ghost"),
        };
        assert_eq!(dormant.sheet_node_id(), None);
    }

    // --- Shared V3 fold reaches the container-name newtype ---------------

    #[test]
    fn normalized_container_name_uses_shared_fold() {
        assert_eq!(
            NormalizedContainerName::from_symbol("Workspace").as_str(),
            NormalizedContainerName::from_symbol("WORKSPACE").as_str()
        );
        // Consistent with the sheet-name fold (same V3 function).
        assert_eq!(
            NormalizedContainerName::from_symbol("Sheet1").as_str(),
            NormalizedSheetName::from_symbol("Sheet1").as_str()
        );
    }
}
