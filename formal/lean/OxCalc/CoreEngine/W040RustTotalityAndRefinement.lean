import Std

namespace OxCalc.CoreEngine.W040.RustTotalityAndRefinement

inductive W040RustEvidenceKind where
  | rustResultCarrier
  | fixtureErrorCarrier
  | treeCalcExecutableEvidence
  | optimizedCoreDisposition
  | rustPanicAudit
  | externalSemanticAuthority
  | specEvolutionGuard
  deriving DecidableEq, Repr

inductive W040RustDispositionKind where
  | directTotalityEvidence
  | directRefinementEvidence
  | exactTotalityBoundary
  | exactRefinementBlocker
  | acceptedExternalSeamBoundary
  | acceptedSpecEvolutionGuard
  deriving DecidableEq, Repr

structure W040RustRow where
  rowId : String
  obligationId : String
  evidenceKind : W040RustEvidenceKind
  dispositionKind : W040RustDispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  refinementRow : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W040RustRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W040RustRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W040RustRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsRefinementRow (row : W040RustRow) : Bool :=
  row.refinementRow && !row.promotionClaim

def IsAcceptedBoundary (row : W040RustRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def ResultErrorCarrierRow : W040RustRow :=
  { rowId := "w040.result-error-carrier-totality-evidence",
    obligationId := "W040-OBL-006",
    evidenceKind := W040RustEvidenceKind.rustResultCarrier,
    dispositionKind := W040RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def FixtureInvalidationSeedErrorRow : W040RustRow :=
  { rowId := "w040.fixture-invalidation-seed-error-totality-evidence",
    obligationId := "W040-OBL-006",
    evidenceKind := W040RustEvidenceKind.fixtureErrorCarrier,
    dispositionKind := W040RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def DependencySeedRebindRefinementRow : W040RustRow :=
  { rowId := "w040.dependency-seed-rebind-refinement-evidence",
    obligationId := "W040-OBL-007",
    evidenceKind := W040RustEvidenceKind.treeCalcExecutableEvidence,
    dispositionKind := W040RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    exactRemainingBlocker := false,
    promotionClaim := false }

def DynamicTransitionRefinementBlockerRow : W040RustRow :=
  { rowId := "w040.dynamic-transition-refinement-exact-blocker",
    obligationId := "W040-OBL-007",
    evidenceKind := W040RustEvidenceKind.optimizedCoreDisposition,
    dispositionKind := W040RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def RuntimePanicSurfaceBoundaryRow : W040RustRow :=
  { rowId := "w040.runtime-panic-surface-totality-boundary",
    obligationId := "W040-OBL-006",
    evidenceKind := W040RustEvidenceKind.rustPanicAudit,
    dispositionKind := W040RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def SnapshotFenceBoundaryRow : W040RustRow :=
  { rowId := "w040.snapshot-fence-refinement-boundary",
    obligationId := "W040-OBL-003",
    evidenceKind := W040RustEvidenceKind.optimizedCoreDisposition,
    dispositionKind := W040RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CapabilityViewFenceBoundaryRow : W040RustRow :=
  { rowId := "w040.capability-view-fence-refinement-boundary",
    obligationId := "W040-OBL-003",
    evidenceKind := W040RustEvidenceKind.optimizedCoreDisposition,
    dispositionKind := W040RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CallableMetadataProjectionBoundaryRow : W040RustRow :=
  { rowId := "w040.callable-metadata-projection-totality-boundary",
    obligationId := "W040-OBL-004",
    evidenceKind := W040RustEvidenceKind.optimizedCoreDisposition,
    dispositionKind := W040RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaCarrierExternalBoundaryRow : W040RustRow :=
  { rowId := "w040.let-lambda-carrier-external-boundary",
    obligationId := "W040-OBL-020",
    evidenceKind := W040RustEvidenceKind.externalSemanticAuthority,
    dispositionKind := W040RustDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def SpecEvolutionRefinementGuardRow : W040RustRow :=
  { rowId := "w040.spec-evolution-refinement-guard",
    obligationId := "W040-OBL-007",
    evidenceKind := W040RustEvidenceKind.specEvolutionGuard,
    dispositionKind := W040RustDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem resultCarrier_isDirectNonPromoting :
    IsNonPromoting ResultErrorCarrierRow = true
      /\ ResultErrorCarrierRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem fixtureInvalidationSeedError_isDirectNonPromoting :
    IsNonPromoting FixtureInvalidationSeedErrorRow = true
      /\ FixtureInvalidationSeedErrorRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem dependencySeedRebind_isRefinementEvidence :
    IsRefinementRow DependencySeedRebindRefinementRow = true
      /\ DependencySeedRebindRefinementRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem dynamicTransition_isExactRefinementBoundary :
    IsExactBlocker DynamicTransitionRefinementBlockerRow = true
      /\ IsTotalityBoundary DynamicTransitionRefinementBlockerRow = true
      /\ IsRefinementRow DynamicTransitionRefinementBlockerRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem runtimePanicSurface_isExactTotalityBoundary :
    IsExactBlocker RuntimePanicSurfaceBoundaryRow = true
      /\ IsTotalityBoundary RuntimePanicSurfaceBoundaryRow = true := by
  constructor <;> rfl

theorem snapshotFence_isExactRefinementBoundary :
    IsExactBlocker SnapshotFenceBoundaryRow = true
      /\ IsTotalityBoundary SnapshotFenceBoundaryRow = true
      /\ IsRefinementRow SnapshotFenceBoundaryRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem capabilityViewFence_isExactRefinementBoundary :
    IsExactBlocker CapabilityViewFenceBoundaryRow = true
      /\ IsTotalityBoundary CapabilityViewFenceBoundaryRow = true
      /\ IsRefinementRow CapabilityViewFenceBoundaryRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem callableMetadataProjection_isExactTotalityBoundary :
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

theorem specEvolutionGuard_isAcceptedBoundary :
    IsAcceptedBoundary SpecEvolutionRefinementGuardRow = true := by
  rfl

theorem allW040RustRows_nonPromoting :
    IsNonPromoting ResultErrorCarrierRow = true
      /\ IsNonPromoting FixtureInvalidationSeedErrorRow = true
      /\ IsNonPromoting DependencySeedRebindRefinementRow = true
      /\ IsNonPromoting DynamicTransitionRefinementBlockerRow = true
      /\ IsNonPromoting RuntimePanicSurfaceBoundaryRow = true
      /\ IsNonPromoting SnapshotFenceBoundaryRow = true
      /\ IsNonPromoting CapabilityViewFenceBoundaryRow = true
      /\ IsNonPromoting CallableMetadataProjectionBoundaryRow = true
      /\ IsNonPromoting LetLambdaCarrierExternalBoundaryRow = true
      /\ IsNonPromoting SpecEvolutionRefinementGuardRow = true := by
  simp [
    IsNonPromoting,
    ResultErrorCarrierRow,
    FixtureInvalidationSeedErrorRow,
    DependencySeedRebindRefinementRow,
    DynamicTransitionRefinementBlockerRow,
    RuntimePanicSurfaceBoundaryRow,
    SnapshotFenceBoundaryRow,
    CapabilityViewFenceBoundaryRow,
    CallableMetadataProjectionBoundaryRow,
    LetLambdaCarrierExternalBoundaryRow,
    SpecEvolutionRefinementGuardRow
  ]

structure W040RustSummary where
  rustRows : Nat
  localProofRows : Nat
  boundedModelRows : Nat
  acceptedExternalSeamRows : Nat
  acceptedBoundaryRows : Nat
  totalityBoundaryRows : Nat
  refinementRows : Nat
  exactRemainingBlockerRows : Nat
  rustEngineTotalityPromoted : Bool
  rustRefinementPromoted : Bool
  fullOptimizedCoreVerificationPromoted : Bool
  callableMetadataProjectionPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  deriving DecidableEq, Repr

def W040RustSummaryValue : W040RustSummary :=
  { rustRows := 10,
    localProofRows := 7,
    boundedModelRows := 0,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 2,
    totalityBoundaryRows := 5,
    refinementRows := 5,
    exactRemainingBlockerRows := 5,
    rustEngineTotalityPromoted := false,
    rustRefinementPromoted := false,
    fullOptimizedCoreVerificationPromoted := false,
    callableMetadataProjectionPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w040RustSummary_hasTenRows :
    W040RustSummaryValue.rustRows = 10 := by
  rfl

theorem w040RustSummary_hasFiveExactBlockers :
    W040RustSummaryValue.exactRemainingBlockerRows = 5 := by
  rfl

theorem w040RustSummary_noRustTotalityPromotion :
    W040RustSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w040RustSummary_noRustRefinementPromotion :
    W040RustSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w040RustSummary_noOptimizedCorePromotion :
    W040RustSummaryValue.fullOptimizedCoreVerificationPromoted = false := by
  rfl

theorem w040RustSummary_noCallableMetadataProjectionPromotion :
    W040RustSummaryValue.callableMetadataProjectionPromoted = false := by
  rfl

theorem w040RustSummary_noGeneralOxFuncKernelPromotion :
    W040RustSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W040.RustTotalityAndRefinement
