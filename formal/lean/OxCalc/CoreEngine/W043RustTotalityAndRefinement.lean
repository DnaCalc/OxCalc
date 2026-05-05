import Std

namespace OxCalc.CoreEngine.W043.RustTotalityAndRefinement

inductive W043RustEvidenceKind where
  | rustResultCarrier
  | treecalcPacket
  | predecessorRustRegression
  | automaticDynamicAdditionEvidence
  | automaticDynamicReleaseEvidence
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

inductive W043RustDispositionKind where
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

structure W043RustRow where
  rowId : String
  obligationId : String
  evidenceKind : W043RustEvidenceKind
  dispositionKind : W043RustDispositionKind
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

def IsNonPromoting (row : W043RustRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W043RustRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W043RustRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsRefinementRow (row : W043RustRow) : Bool :=
  row.refinementRow && !row.promotionClaim

def IsAcceptedBoundary (row : W043RustRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def ResultErrorCarrierRow : W043RustRow :=
  { rowId := "w043.result-error-carrier-totality-evidence",
    obligationId := "W043-OBL-010",
    evidenceKind := W043RustEvidenceKind.rustResultCarrier,
    dispositionKind := W043RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TreeCalcPacketRow : W043RustRow :=
  { rowId := "w043.treecalc-packet-totality-evidence",
    obligationId := "W043-OBL-010",
    evidenceKind := W043RustEvidenceKind.treecalcPacket,
    dispositionKind := W043RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def PredecessorRustRegressionRow : W043RustRow :=
  { rowId := "w043.w042-rust-refinement-regression",
    obligationId := "W043-OBL-011",
    evidenceKind := W043RustEvidenceKind.predecessorRustRegression,
    dispositionKind := W043RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def AutomaticDynamicAdditionRefinementRow : W043RustRow :=
  { rowId := "w043.automatic-dynamic-addition-refinement-evidence",
    obligationId := "W043-OBL-011",
    evidenceKind := W043RustEvidenceKind.automaticDynamicAdditionEvidence,
    dispositionKind := W043RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := true,
    exactRemainingBlocker := false,
    promotionClaim := false }

def AutomaticDynamicReleaseRefinementRow : W043RustRow :=
  { rowId := "w043.automatic-dynamic-release-refinement-evidence",
    obligationId := "W043-OBL-011",
    evidenceKind := W043RustEvidenceKind.automaticDynamicReleaseEvidence,
    dispositionKind := W043RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := true,
    exactRemainingBlocker := false,
    promotionClaim := false }

def SnapshotFenceDeclaredProfileRow : W043RustRow :=
  { rowId := "w043.snapshot-fence-declared-profile-refinement-evidence",
    obligationId := "W043-OBL-011",
    evidenceKind := W043RustEvidenceKind.snapshotFenceDeclaredProfile,
    dispositionKind := W043RustDispositionKind.directDeclaredProfileRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def CapabilityViewDeclaredProfileRow : W043RustRow :=
  { rowId := "w043.capability-view-declared-profile-refinement-evidence",
    obligationId := "W043-OBL-011",
    evidenceKind := W043RustEvidenceKind.capabilityViewDeclaredProfile,
    dispositionKind := W043RustDispositionKind.directDeclaredProfileRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def CallableValueCarrierRow : W043RustRow :=
  { rowId := "w043.callable-value-carrier-totality-evidence",
    obligationId := "W043-OBL-010",
    evidenceKind := W043RustEvidenceKind.callableValueCarrier,
    dispositionKind := W043RustDispositionKind.directCallableValueCarrierTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def RuntimePanicSurfaceBoundaryRow : W043RustRow :=
  { rowId := "w043.runtime-panic-surface-totality-boundary",
    obligationId := "W043-OBL-009",
    evidenceKind := W043RustEvidenceKind.rustPanicAudit,
    dispositionKind := W043RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def BroaderDynamicTransitionBoundaryRow : W043RustRow :=
  { rowId := "w043.broader-dynamic-transition-coverage-refinement-boundary",
    obligationId := "W043-OBL-011",
    evidenceKind := W043RustEvidenceKind.broaderDynamicCoverage,
    dispositionKind := W043RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CallableMetadataProjectionBoundaryRow : W043RustRow :=
  { rowId := "w043.callable-metadata-projection-totality-boundary",
    obligationId := "W043-OBL-010",
    evidenceKind := W043RustEvidenceKind.callableMetadataProjection,
    dispositionKind := W043RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullOptimizedCoreReleaseGradeBoundaryRow : W043RustRow :=
  { rowId := "w043.full-optimized-core-release-grade-conformance-boundary",
    obligationId := "W043-OBL-011",
    evidenceKind := W043RustEvidenceKind.releaseGradeBoundary,
    dispositionKind := W043RustDispositionKind.exactReleaseGradeBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaCarrierExternalBoundaryRow : W043RustRow :=
  { rowId := "w043.let-lambda-carrier-external-boundary",
    obligationId := "W043-OBL-015",
    evidenceKind := W043RustEvidenceKind.externalSemanticAuthority,
    dispositionKind := W043RustDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def SpecEvolutionRefinementGuardRow : W043RustRow :=
  { rowId := "w043.spec-evolution-refinement-guard",
    obligationId := "W043-OBL-011",
    evidenceKind := W043RustEvidenceKind.specEvolutionGuard,
    dispositionKind := W043RustDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem automaticDynamicTransitions_areDirectRefinementEvidence :
    IsRefinementRow AutomaticDynamicAdditionRefinementRow = true
      /\ IsRefinementRow AutomaticDynamicReleaseRefinementRow = true
      /\ AutomaticDynamicAdditionRefinementRow.automaticDynamicTransitionRow = true
      /\ AutomaticDynamicReleaseRefinementRow.automaticDynamicTransitionRow = true
      /\ AutomaticDynamicAdditionRefinementRow.exactRemainingBlocker = false
      /\ AutomaticDynamicReleaseRefinementRow.exactRemainingBlocker = false
      /\ AutomaticDynamicAdditionRefinementRow.promotionClaim = false
      /\ AutomaticDynamicReleaseRefinementRow.promotionClaim = false := by
  constructor
  · rfl
  constructor
  · rfl
  constructor
  · rfl
  constructor
  · rfl
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

theorem retainedW043RustBoundaries_areExact :
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

theorem allW043RustRows_nonPromoting :
    IsNonPromoting ResultErrorCarrierRow = true
      /\ IsNonPromoting TreeCalcPacketRow = true
      /\ IsNonPromoting PredecessorRustRegressionRow = true
      /\ IsNonPromoting AutomaticDynamicAdditionRefinementRow = true
      /\ IsNonPromoting AutomaticDynamicReleaseRefinementRow = true
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
    TreeCalcPacketRow,
    PredecessorRustRegressionRow,
    AutomaticDynamicAdditionRefinementRow,
    AutomaticDynamicReleaseRefinementRow,
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

structure W043RustSummary where
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
  panicFreeCoreDomainPromoted : Bool
  fullOptimizedCoreVerificationPromoted : Bool
  callableMetadataProjectionPromoted : Bool
  callableCarrierSufficiencyPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  deriving DecidableEq, Repr

def W043RustSummaryValue : W043RustSummary :=
  { rustRows := 14,
    localProofRows := 11,
    boundedModelRows := 0,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 2,
    totalityBoundaryRows := 4,
    refinementRows := 8,
    exactRemainingBlockerRows := 4,
    automaticDynamicTransitionRefinementRows := 2,
    rustEngineTotalityPromoted := false,
    rustRefinementPromoted := false,
    panicFreeCoreDomainPromoted := false,
    fullOptimizedCoreVerificationPromoted := false,
    callableMetadataProjectionPromoted := false,
    callableCarrierSufficiencyPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w043RustSummary_hasFourteenRows :
    W043RustSummaryValue.rustRows = 14 := by
  rfl

theorem w043RustSummary_hasTwoAutomaticDynamicTransitionRefinementRows :
    W043RustSummaryValue.automaticDynamicTransitionRefinementRows = 2 := by
  rfl

theorem w043RustSummary_hasFourExactBlockers :
    W043RustSummaryValue.exactRemainingBlockerRows = 4 := by
  rfl

theorem w043RustSummary_noRustTotalityPromotion :
    W043RustSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w043RustSummary_noRustRefinementPromotion :
    W043RustSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w043RustSummary_noPanicFreeCorePromotion :
    W043RustSummaryValue.panicFreeCoreDomainPromoted = false := by
  rfl

theorem w043RustSummary_noOptimizedCorePromotion :
    W043RustSummaryValue.fullOptimizedCoreVerificationPromoted = false := by
  rfl

theorem w043RustSummary_noCallableMetadataProjectionPromotion :
    W043RustSummaryValue.callableMetadataProjectionPromoted = false := by
  rfl

theorem w043RustSummary_noCallableCarrierSufficiencyPromotion :
    W043RustSummaryValue.callableCarrierSufficiencyPromoted = false := by
  rfl

theorem w043RustSummary_noGeneralOxFuncKernelPromotion :
    W043RustSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W043.RustTotalityAndRefinement
