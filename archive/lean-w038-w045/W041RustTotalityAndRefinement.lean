import Std

namespace OxCalc.CoreEngine.W041.RustTotalityAndRefinement

inductive W041RustEvidenceKind where
  | rustResultCarrier
  | fixtureErrorCarrier
  | explicitDependencySeedEvidence
  | automaticDynamicTransitionEvidence
  | rustPanicAudit
  | optimizedCoreDisposition
  | externalSemanticAuthority
  | specEvolutionGuard
  deriving DecidableEq, Repr

inductive W041RustDispositionKind where
  | directTotalityEvidence
  | directRefinementEvidence
  | exactTotalityBoundary
  | exactRefinementBlocker
  | acceptedExternalSeamBoundary
  | acceptedSpecEvolutionGuard
  deriving DecidableEq, Repr

structure W041RustRow where
  rowId : String
  obligationId : String
  evidenceKind : W041RustEvidenceKind
  dispositionKind : W041RustDispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  refinementRow : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W041RustRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W041RustRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W041RustRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsRefinementRow (row : W041RustRow) : Bool :=
  row.refinementRow && !row.promotionClaim

def IsAcceptedBoundary (row : W041RustRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def ResultErrorCarrierRow : W041RustRow :=
  { rowId := "w041.result-error-carrier-totality-evidence",
    obligationId := "W041-OBL-007",
    evidenceKind := W041RustEvidenceKind.rustResultCarrier,
    dispositionKind := W041RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def FixtureSuccessorCatalogErrorRow : W041RustRow :=
  { rowId := "w041.fixture-successor-catalog-error-carrier",
    obligationId := "W041-OBL-008",
    evidenceKind := W041RustEvidenceKind.fixtureErrorCarrier,
    dispositionKind := W041RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def ExplicitDependencySeedRebindRow : W041RustRow :=
  { rowId := "w041.explicit-dependency-seed-rebind-regression",
    obligationId := "W041-OBL-009",
    evidenceKind := W041RustEvidenceKind.explicitDependencySeedEvidence,
    dispositionKind := W041RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    exactRemainingBlocker := false,
    promotionClaim := false }

def AutomaticDynamicTransitionRefinementRow : W041RustRow :=
  { rowId := "w041.automatic-dynamic-transition-refinement-evidence",
    obligationId := "W041-OBL-009",
    evidenceKind := W041RustEvidenceKind.automaticDynamicTransitionEvidence,
    dispositionKind := W041RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    exactRemainingBlocker := false,
    promotionClaim := false }

def RuntimePanicSurfaceBoundaryRow : W041RustRow :=
  { rowId := "w041.runtime-panic-surface-totality-boundary",
    obligationId := "W041-OBL-007",
    evidenceKind := W041RustEvidenceKind.rustPanicAudit,
    dispositionKind := W041RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def SnapshotFenceBoundaryRow : W041RustRow :=
  { rowId := "w041.snapshot-fence-refinement-boundary",
    obligationId := "W041-OBL-008",
    evidenceKind := W041RustEvidenceKind.optimizedCoreDisposition,
    dispositionKind := W041RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CapabilityViewFenceBoundaryRow : W041RustRow :=
  { rowId := "w041.capability-view-fence-refinement-boundary",
    obligationId := "W041-OBL-008",
    evidenceKind := W041RustEvidenceKind.optimizedCoreDisposition,
    dispositionKind := W041RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CallableMetadataProjectionBoundaryRow : W041RustRow :=
  { rowId := "w041.callable-metadata-projection-totality-boundary",
    obligationId := "W041-OBL-008",
    evidenceKind := W041RustEvidenceKind.optimizedCoreDisposition,
    dispositionKind := W041RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaCarrierExternalBoundaryRow : W041RustRow :=
  { rowId := "w041.let-lambda-carrier-external-boundary",
    obligationId := "W041-OBL-028",
    evidenceKind := W041RustEvidenceKind.externalSemanticAuthority,
    dispositionKind := W041RustDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def SpecEvolutionRefinementGuardRow : W041RustRow :=
  { rowId := "w041.spec-evolution-refinement-guard",
    obligationId := "W041-OBL-009",
    evidenceKind := W041RustEvidenceKind.specEvolutionGuard,
    dispositionKind := W041RustDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem automaticDynamicTransition_isDirectRefinementEvidence :
    IsRefinementRow AutomaticDynamicTransitionRefinementRow = true
      /\ AutomaticDynamicTransitionRefinementRow.exactRemainingBlocker = false
      /\ AutomaticDynamicTransitionRefinementRow.promotionClaim = false := by
  constructor
  · rfl
  constructor <;> rfl

theorem runtimePanicSurface_isRetainedTotalityBoundary :
    IsExactBlocker RuntimePanicSurfaceBoundaryRow = true
      /\ IsTotalityBoundary RuntimePanicSurfaceBoundaryRow = true := by
  constructor <;> rfl

theorem snapshotFence_isRetainedRefinementBoundary :
    IsExactBlocker SnapshotFenceBoundaryRow = true
      /\ IsTotalityBoundary SnapshotFenceBoundaryRow = true
      /\ IsRefinementRow SnapshotFenceBoundaryRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem capabilityViewFence_isRetainedRefinementBoundary :
    IsExactBlocker CapabilityViewFenceBoundaryRow = true
      /\ IsTotalityBoundary CapabilityViewFenceBoundaryRow = true
      /\ IsRefinementRow CapabilityViewFenceBoundaryRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem callableMetadataProjection_isRetainedTotalityBoundary :
    IsExactBlocker CallableMetadataProjectionBoundaryRow = true
      /\ IsTotalityBoundary CallableMetadataProjectionBoundaryRow = true
      /\ IsRefinementRow CallableMetadataProjectionBoundaryRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem letLambdaCarrier_isAcceptedExternalBoundary :
    LetLambdaCarrierExternalBoundaryRow.acceptedExternalSeam = true
      /\ IsAcceptedBoundary LetLambdaCarrierExternalBoundaryRow = true := by
  constructor <;> rfl

theorem allW041RustRows_nonPromoting :
    IsNonPromoting ResultErrorCarrierRow = true
      /\ IsNonPromoting FixtureSuccessorCatalogErrorRow = true
      /\ IsNonPromoting ExplicitDependencySeedRebindRow = true
      /\ IsNonPromoting AutomaticDynamicTransitionRefinementRow = true
      /\ IsNonPromoting RuntimePanicSurfaceBoundaryRow = true
      /\ IsNonPromoting SnapshotFenceBoundaryRow = true
      /\ IsNonPromoting CapabilityViewFenceBoundaryRow = true
      /\ IsNonPromoting CallableMetadataProjectionBoundaryRow = true
      /\ IsNonPromoting LetLambdaCarrierExternalBoundaryRow = true
      /\ IsNonPromoting SpecEvolutionRefinementGuardRow = true := by
  simp [
    IsNonPromoting,
    ResultErrorCarrierRow,
    FixtureSuccessorCatalogErrorRow,
    ExplicitDependencySeedRebindRow,
    AutomaticDynamicTransitionRefinementRow,
    RuntimePanicSurfaceBoundaryRow,
    SnapshotFenceBoundaryRow,
    CapabilityViewFenceBoundaryRow,
    CallableMetadataProjectionBoundaryRow,
    LetLambdaCarrierExternalBoundaryRow,
    SpecEvolutionRefinementGuardRow
  ]

structure W041RustSummary where
  rustRows : Nat
  localProofRows : Nat
  boundedModelRows : Nat
  acceptedExternalSeamRows : Nat
  acceptedBoundaryRows : Nat
  totalityBoundaryRows : Nat
  refinementRows : Nat
  exactRemainingBlockerRows : Nat
  automaticDynamicTransitionRefinementRows : Nat
  rustEngineTotalityPromoted : Bool
  rustRefinementPromoted : Bool
  fullOptimizedCoreVerificationPromoted : Bool
  callableMetadataProjectionPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  deriving DecidableEq, Repr

def W041RustSummaryValue : W041RustSummary :=
  { rustRows := 10,
    localProofRows := 7,
    boundedModelRows := 0,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 2,
    totalityBoundaryRows := 4,
    refinementRows := 5,
    exactRemainingBlockerRows := 4,
    automaticDynamicTransitionRefinementRows := 1,
    rustEngineTotalityPromoted := false,
    rustRefinementPromoted := false,
    fullOptimizedCoreVerificationPromoted := false,
    callableMetadataProjectionPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w041RustSummary_hasTenRows :
    W041RustSummaryValue.rustRows = 10 := by
  rfl

theorem w041RustSummary_hasOneAutomaticDynamicTransitionRefinementRow :
    W041RustSummaryValue.automaticDynamicTransitionRefinementRows = 1 := by
  rfl

theorem w041RustSummary_hasFourExactBlockers :
    W041RustSummaryValue.exactRemainingBlockerRows = 4 := by
  rfl

theorem w041RustSummary_noRustTotalityPromotion :
    W041RustSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w041RustSummary_noRustRefinementPromotion :
    W041RustSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w041RustSummary_noOptimizedCorePromotion :
    W041RustSummaryValue.fullOptimizedCoreVerificationPromoted = false := by
  rfl

theorem w041RustSummary_noCallableMetadataProjectionPromotion :
    W041RustSummaryValue.callableMetadataProjectionPromoted = false := by
  rfl

theorem w041RustSummary_noGeneralOxFuncKernelPromotion :
    W041RustSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W041.RustTotalityAndRefinement
