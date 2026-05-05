import Std

namespace OxCalc.CoreEngine.W036.LeanCoverageExpansion

inductive ProofDisposition where
  | provedLocal
  | explicitAxiom
  | externalSeamAssumption
  | opaqueKernelBoundary
  | deferredToTlaLane
  | deferredToConformanceLane
  | harnessEvidenceBound
  deriving DecidableEq, Repr

structure CoverageRow where
  rowId : String
  obligationId : String
  summary : String
  disposition : ProofDisposition
  checkedLocalTheorem : Bool
  deriving DecidableEq, Repr

def CanPromoteAsLocalLeanTheorem (row : CoverageRow) : Bool :=
  match row.disposition with
  | ProofDisposition.provedLocal => row.checkedLocalTheorem
  | _ => false

def IsExplicitAxiomRow (row : CoverageRow) : Bool :=
  match row.disposition with
  | ProofDisposition.explicitAxiom => true
  | _ => false

def IsExternalOrOpaqueBoundary (row : CoverageRow) : Bool :=
  match row.disposition with
  | ProofDisposition.externalSeamAssumption => true
  | ProofDisposition.opaqueKernelBoundary => true
  | _ => false

def IsDeferredFormalLane (row : CoverageRow) : Bool :=
  match row.disposition with
  | ProofDisposition.deferredToTlaLane => true
  | ProofDisposition.deferredToConformanceLane => true
  | _ => false

def SnapshotFenceRejectRow : CoverageRow :=
  { rowId := "w036.lean.snapshot-fence.reject-no-publish",
    obligationId := "W036-OBL-006",
    summary := "snapshot-fence reject rows remain no-publish proof obligations",
    disposition := ProofDisposition.deferredToTlaLane,
    checkedLocalTheorem := false }

def CapabilityViewFenceRejectRow : CoverageRow :=
  { rowId := "w036.lean.capability-view-fence.reject-no-publish",
    obligationId := "W036-OBL-007",
    summary := "capability-view fence rows require W036 TLA/coordinator ownership",
    disposition := ProofDisposition.deferredToTlaLane,
    checkedLocalTheorem := false }

def MultiReaderOverlayReleaseRow : CoverageRow :=
  { rowId := "w036.lean.overlay.multi-reader-release-order",
    obligationId := "W036-OBL-002",
    summary := "multi-reader overlay release ordering belongs to the TLA lane",
    disposition := ProofDisposition.deferredToTlaLane,
    checkedLocalTheorem := false }

def DynamicBindHarnessRow : CoverageRow :=
  { rowId := "w036.lean.dynamic-bind.harness-first-fix",
    obligationId := "W036-OBL-003",
    summary := "dynamic bind projection has W036 harness evidence but is not a conformance match",
    disposition := ProofDisposition.harnessEvidenceBound,
    checkedLocalTheorem := false }

def DynamicNegativeHarnessRow : CoverageRow :=
  { rowId := "w036.lean.dynamic-negative.shape-update-harness",
    obligationId := "W036-OBL-005",
    summary := "dynamic negative and release/reclassification rows have W036 harness evidence",
    disposition := ProofDisposition.harnessEvidenceBound,
    checkedLocalTheorem := false }

def DeclaredGapNotMatchGuardRow : CoverageRow :=
  { rowId := "w036.lean.declared-gap.not-match-guard",
    obligationId := "W036-OBL-003..008",
    summary := "declared implementation gaps are not promoted as optimized/core-engine matches",
    disposition := ProofDisposition.provedLocal,
    checkedLocalTheorem := true }

def MatchPromotionGuardRow : CoverageRow :=
  { rowId := "w036.lean.match-promotion.zero-promoted",
    obligationId := "W036-OBL-003..008",
    summary := "W036 implementation-conformance closure has zero match-promoted rows",
    disposition := ProofDisposition.provedLocal,
    checkedLocalTheorem := true }

def OxFmlW073TypedFormattingRow : CoverageRow :=
  { rowId := "w036.lean.oxfml.w073-typed-formatting",
    obligationId := "W036-OBL-015",
    summary := "W073 typed conditional-formatting metadata remains OxFml-owned",
    disposition := ProofDisposition.externalSeamAssumption,
    checkedLocalTheorem := false }

def OxFmlFenceArtifactMeaningRow : CoverageRow :=
  { rowId := "w036.lean.oxfml.fence-artifact-meaning",
    obligationId := "W036-OBL-017",
    summary := "OxFml fence artifact meaning remains external seam authority",
    disposition := ProofDisposition.externalSeamAssumption,
    checkedLocalTheorem := false }

def FullOxFuncCallableKernelRow : CoverageRow :=
  { rowId := "w036.lean.oxfunc.full-callable-kernel",
    obligationId := "W036-OBL-004",
    summary := "full OxFunc LAMBDA kernel remains opaque to OxCalc Lean proof inventory",
    disposition := ProofDisposition.opaqueKernelBoundary,
    checkedLocalTheorem := false }

theorem declaredGapNotMatchGuard_promotableAsLocalTheorem :
    CanPromoteAsLocalLeanTheorem DeclaredGapNotMatchGuardRow = true := by
  rfl

theorem matchPromotionGuard_promotableAsLocalTheorem :
    CanPromoteAsLocalLeanTheorem MatchPromotionGuardRow = true := by
  rfl

theorem dynamicBindHarness_notPromotableAsLocalTheorem :
    CanPromoteAsLocalLeanTheorem DynamicBindHarnessRow = false := by
  rfl

theorem dynamicNegativeHarness_notPromotableAsLocalTheorem :
    CanPromoteAsLocalLeanTheorem DynamicNegativeHarnessRow = false := by
  rfl

theorem snapshotFence_deferredToFormalLane :
    IsDeferredFormalLane SnapshotFenceRejectRow = true := by
  rfl

theorem capabilityViewFence_deferredToFormalLane :
    IsDeferredFormalLane CapabilityViewFenceRejectRow = true := by
  rfl

theorem multiReaderOverlay_deferredToFormalLane :
    IsDeferredFormalLane MultiReaderOverlayReleaseRow = true := by
  rfl

theorem w073TypedFormatting_externalBoundary :
    IsExternalOrOpaqueBoundary OxFmlW073TypedFormattingRow = true := by
  rfl

theorem fullOxFuncCallableKernel_opaqueBoundary :
    IsExternalOrOpaqueBoundary FullOxFuncCallableKernelRow = true := by
  rfl

structure W036LeanCoverageSummary where
  localTheoremRows : Nat
  explicitAxiomRows : Nat
  externalSeamRows : Nat
  opaqueKernelRows : Nat
  tlaDeferredRows : Nat
  conformanceDeferredRows : Nat
  harnessFirstFixRows : Nat
  matchPromotedRows : Nat
  fullLeanVerificationPromoted : Bool
  deriving DecidableEq, Repr

def W036LeanCoverageSummaryValue : W036LeanCoverageSummary :=
  { localTheoremRows := 2,
    explicitAxiomRows := 0,
    externalSeamRows := 2,
    opaqueKernelRows := 1,
    tlaDeferredRows := 3,
    conformanceDeferredRows := 0,
    harnessFirstFixRows := 2,
    matchPromotedRows := 0,
    fullLeanVerificationPromoted := false }

def W036LeanCoverageHasNoExplicitAxioms
    (summary : W036LeanCoverageSummary) : Prop :=
  summary.explicitAxiomRows = 0

def W036LeanCoverageDoesNotPromoteFullVerification
    (summary : W036LeanCoverageSummary) : Prop :=
  summary.fullLeanVerificationPromoted = false

def W036LeanCoverageDoesNotPromoteDeclaredGapMatches
    (summary : W036LeanCoverageSummary) : Prop :=
  summary.matchPromotedRows = 0

theorem w036LeanCoverage_hasNoExplicitAxioms :
    W036LeanCoverageHasNoExplicitAxioms W036LeanCoverageSummaryValue := by
  rfl

theorem w036LeanCoverage_noFullLeanPromotion :
    W036LeanCoverageDoesNotPromoteFullVerification W036LeanCoverageSummaryValue := by
  rfl

theorem w036LeanCoverage_noDeclaredGapMatchPromotion :
    W036LeanCoverageDoesNotPromoteDeclaredGapMatches W036LeanCoverageSummaryValue := by
  rfl

theorem w036LeanCoverage_recordsTwoHarnessFirstFixRows :
    W036LeanCoverageSummaryValue.harnessFirstFixRows = 2 := by
  rfl

end OxCalc.CoreEngine.W036.LeanCoverageExpansion
