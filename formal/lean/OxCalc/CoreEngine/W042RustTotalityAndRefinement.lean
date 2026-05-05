import Std

namespace OxCalc.CoreEngine.W042.RustTotalityAndRefinement

inductive W042RustEvidenceKind where
  | rustResultCarrier
  | treecalcCounterpartPacket
  | explicitDependencySeedEvidence
  | automaticDynamicTransitionEvidence
  | snapshotFenceDeclaredProfile
  | capabilityViewDeclaredProfile
  | callableValueCarrier
  | rustPanicAudit
  | broaderDynamicCoverage
  | callableMetadataProjection
  | releaseGradeBoundary
  | externalSemanticAuthority
  | specEvolutionGuard
  deriving DecidableEq, Repr

inductive W042RustDispositionKind where
  | directTotalityEvidence
  | directRefinementEvidence
  | directDeclaredProfileRefinementEvidence
  | directCallableValueCarrierTotalityEvidence
  | exactTotalityBoundary
  | exactRefinementBlocker
  | exactReleaseGradeBoundary
  | acceptedExternalSeamBoundary
  | acceptedSpecEvolutionGuard
  deriving DecidableEq, Repr

structure W042RustRow where
  rowId : String
  obligationId : String
  evidenceKind : W042RustEvidenceKind
  dispositionKind : W042RustDispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  refinementRow : Bool
  automaticDynamicTransitionRow : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W042RustRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W042RustRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W042RustRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsRefinementRow (row : W042RustRow) : Bool :=
  row.refinementRow && !row.promotionClaim

def IsAcceptedBoundary (row : W042RustRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def ResultErrorCarrierRow : W042RustRow :=
  { rowId := "w042.result-error-carrier-totality-evidence",
    obligationId := "W042-OBL-007",
    evidenceKind := W042RustEvidenceKind.rustResultCarrier,
    dispositionKind := W042RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TreeCalcCounterpartPacketRow : W042RustRow :=
  { rowId := "w042.treecalc-counterpart-packet-totality-evidence",
    obligationId := "W042-OBL-008",
    evidenceKind := W042RustEvidenceKind.treecalcCounterpartPacket,
    dispositionKind := W042RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def ExplicitDependencySeedRebindRow : W042RustRow :=
  { rowId := "w042.explicit-dependency-seed-rebind-regression",
    obligationId := "W042-OBL-009",
    evidenceKind := W042RustEvidenceKind.explicitDependencySeedEvidence,
    dispositionKind := W042RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def AutomaticDynamicTransitionRefinementRow : W042RustRow :=
  { rowId := "w042.automatic-dynamic-transition-refinement-evidence",
    obligationId := "W042-OBL-009",
    evidenceKind := W042RustEvidenceKind.automaticDynamicTransitionEvidence,
    dispositionKind := W042RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := true,
    exactRemainingBlocker := false,
    promotionClaim := false }

def SnapshotFenceDeclaredProfileRow : W042RustRow :=
  { rowId := "w042.snapshot-fence-declared-profile-refinement-evidence",
    obligationId := "W042-OBL-003",
    evidenceKind := W042RustEvidenceKind.snapshotFenceDeclaredProfile,
    dispositionKind := W042RustDispositionKind.directDeclaredProfileRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def CapabilityViewDeclaredProfileRow : W042RustRow :=
  { rowId := "w042.capability-view-declared-profile-refinement-evidence",
    obligationId := "W042-OBL-004",
    evidenceKind := W042RustEvidenceKind.capabilityViewDeclaredProfile,
    dispositionKind := W042RustDispositionKind.directDeclaredProfileRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def CallableValueCarrierRow : W042RustRow :=
  { rowId := "w042.callable-value-carrier-totality-evidence",
    obligationId := "W042-OBL-008",
    evidenceKind := W042RustEvidenceKind.callableValueCarrier,
    dispositionKind := W042RustDispositionKind.directCallableValueCarrierTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def RuntimePanicSurfaceBoundaryRow : W042RustRow :=
  { rowId := "w042.runtime-panic-surface-totality-boundary",
    obligationId := "W042-OBL-007",
    evidenceKind := W042RustEvidenceKind.rustPanicAudit,
    dispositionKind := W042RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def BroaderDynamicTransitionBoundaryRow : W042RustRow :=
  { rowId := "w042.broader-dynamic-transition-coverage-refinement-boundary",
    obligationId := "W042-OBL-002",
    evidenceKind := W042RustEvidenceKind.broaderDynamicCoverage,
    dispositionKind := W042RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CallableMetadataProjectionBoundaryRow : W042RustRow :=
  { rowId := "w042.callable-metadata-projection-totality-boundary",
    obligationId := "W042-OBL-005",
    evidenceKind := W042RustEvidenceKind.callableMetadataProjection,
    dispositionKind := W042RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullOptimizedCoreReleaseGradeBoundaryRow : W042RustRow :=
  { rowId := "w042.full-optimized-core-release-grade-conformance-boundary",
    obligationId := "W042-OBL-032",
    evidenceKind := W042RustEvidenceKind.releaseGradeBoundary,
    dispositionKind := W042RustDispositionKind.exactReleaseGradeBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaCarrierExternalBoundaryRow : W042RustRow :=
  { rowId := "w042.let-lambda-carrier-external-boundary",
    obligationId := "W042-OBL-012",
    evidenceKind := W042RustEvidenceKind.externalSemanticAuthority,
    dispositionKind := W042RustDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def SpecEvolutionRefinementGuardRow : W042RustRow :=
  { rowId := "w042.spec-evolution-refinement-guard",
    obligationId := "W042-OBL-009",
    evidenceKind := W042RustEvidenceKind.specEvolutionGuard,
    dispositionKind := W042RustDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem automaticDynamicTransition_isDirectRefinementEvidence :
    IsRefinementRow AutomaticDynamicTransitionRefinementRow = true
      /\ AutomaticDynamicTransitionRefinementRow.automaticDynamicTransitionRow = true
      /\ AutomaticDynamicTransitionRefinementRow.exactRemainingBlocker = false
      /\ AutomaticDynamicTransitionRefinementRow.promotionClaim = false := by
  constructor
  · rfl
  constructor
  · rfl
  constructor <;> rfl

theorem declaredProfileCounterparts_areNonPromotingRefinementEvidence :
    IsRefinementRow SnapshotFenceDeclaredProfileRow = true
      /\ SnapshotFenceDeclaredProfileRow.exactRemainingBlocker = false
      /\ IsRefinementRow CapabilityViewDeclaredProfileRow = true
      /\ CapabilityViewDeclaredProfileRow.exactRemainingBlocker = false := by
  constructor
  · rfl
  constructor
  · rfl
  constructor <;> rfl

theorem callableValueCarrier_isNotMetadataProjection :
    CallableValueCarrierRow.exactRemainingBlocker = false
      /\ CallableMetadataProjectionBoundaryRow.exactRemainingBlocker = true
      /\ IsExactBlocker CallableMetadataProjectionBoundaryRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem retainedW042RustBoundaries_areExact :
    IsExactBlocker RuntimePanicSurfaceBoundaryRow = true
      /\ IsExactBlocker BroaderDynamicTransitionBoundaryRow = true
      /\ IsExactBlocker CallableMetadataProjectionBoundaryRow = true
      /\ IsExactBlocker FullOptimizedCoreReleaseGradeBoundaryRow = true := by
  constructor
  · rfl
  constructor
  · rfl
  constructor <;> rfl

theorem letLambdaCarrier_isAcceptedExternalBoundary :
    LetLambdaCarrierExternalBoundaryRow.acceptedExternalSeam = true
      /\ IsAcceptedBoundary LetLambdaCarrierExternalBoundaryRow = true := by
  constructor <;> rfl

theorem allW042RustRows_nonPromoting :
    IsNonPromoting ResultErrorCarrierRow = true
      /\ IsNonPromoting TreeCalcCounterpartPacketRow = true
      /\ IsNonPromoting ExplicitDependencySeedRebindRow = true
      /\ IsNonPromoting AutomaticDynamicTransitionRefinementRow = true
      /\ IsNonPromoting SnapshotFenceDeclaredProfileRow = true
      /\ IsNonPromoting CapabilityViewDeclaredProfileRow = true
      /\ IsNonPromoting CallableValueCarrierRow = true
      /\ IsNonPromoting RuntimePanicSurfaceBoundaryRow = true
      /\ IsNonPromoting BroaderDynamicTransitionBoundaryRow = true
      /\ IsNonPromoting CallableMetadataProjectionBoundaryRow = true
      /\ IsNonPromoting FullOptimizedCoreReleaseGradeBoundaryRow = true
      /\ IsNonPromoting LetLambdaCarrierExternalBoundaryRow = true
      /\ IsNonPromoting SpecEvolutionRefinementGuardRow = true := by
  simp [
    IsNonPromoting,
    ResultErrorCarrierRow,
    TreeCalcCounterpartPacketRow,
    ExplicitDependencySeedRebindRow,
    AutomaticDynamicTransitionRefinementRow,
    SnapshotFenceDeclaredProfileRow,
    CapabilityViewDeclaredProfileRow,
    CallableValueCarrierRow,
    RuntimePanicSurfaceBoundaryRow,
    BroaderDynamicTransitionBoundaryRow,
    CallableMetadataProjectionBoundaryRow,
    FullOptimizedCoreReleaseGradeBoundaryRow,
    LetLambdaCarrierExternalBoundaryRow,
    SpecEvolutionRefinementGuardRow
  ]

structure W042RustSummary where
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

def W042RustSummaryValue : W042RustSummary :=
  { rustRows := 13,
    localProofRows := 10,
    boundedModelRows := 0,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 2,
    totalityBoundaryRows := 4,
    refinementRows := 7,
    exactRemainingBlockerRows := 4,
    automaticDynamicTransitionRefinementRows := 1,
    rustEngineTotalityPromoted := false,
    rustRefinementPromoted := false,
    fullOptimizedCoreVerificationPromoted := false,
    callableMetadataProjectionPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w042RustSummary_hasThirteenRows :
    W042RustSummaryValue.rustRows = 13 := by
  rfl

theorem w042RustSummary_hasOneAutomaticDynamicTransitionRefinementRow :
    W042RustSummaryValue.automaticDynamicTransitionRefinementRows = 1 := by
  rfl

theorem w042RustSummary_hasFourExactBlockers :
    W042RustSummaryValue.exactRemainingBlockerRows = 4 := by
  rfl

theorem w042RustSummary_noRustTotalityPromotion :
    W042RustSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w042RustSummary_noRustRefinementPromotion :
    W042RustSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w042RustSummary_noOptimizedCorePromotion :
    W042RustSummaryValue.fullOptimizedCoreVerificationPromoted = false := by
  rfl

theorem w042RustSummary_noCallableMetadataProjectionPromotion :
    W042RustSummaryValue.callableMetadataProjectionPromoted = false := by
  rfl

theorem w042RustSummary_noGeneralOxFuncKernelPromotion :
    W042RustSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W042.RustTotalityAndRefinement
