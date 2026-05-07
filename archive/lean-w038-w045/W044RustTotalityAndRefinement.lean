import Std

namespace OxCalc.CoreEngine.W044.RustTotalityAndRefinement

inductive W044RustEvidenceKind where
  | rustResultCarrier
  | treecalcPacket
  | predecessorRustRegression
  | mixedDynamicTransitionEvidence
  | publicationFenceNoPublishEvidence
  | predecessorDynamicRegression
  | snapshotFenceBreadth
  | capabilityViewBreadth
  | callableValueCarrier
  | rustPanicAudit
  | broaderDynamicCoverage
  | callableMetadataProjection
  | releaseGradeBoundary
  | externalSemanticAuthority
  | specEvolutionGuard
  | w073TypedFormattingGuard
  deriving DecidableEq, Repr

inductive W044RustDispositionKind where
  | directTotalityEvidence
  | directRefinementEvidence
  | carriedRefinementEvidence
  | exactRefinementBlocker
  | directCallableValueCarrierTotalityEvidence
  | exactTotalityBoundary
  | exactReleaseGradeBoundary
  | acceptedExternalSeamBoundary
  | acceptedSpecEvolutionGuard
  | acceptedFormattingBoundary
  deriving DecidableEq, Repr

structure W044RustRow where
  rowId : String
  obligationId : String
  evidenceKind : W044RustEvidenceKind
  dispositionKind : W044RustDispositionKind
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

def IsNonPromoting (row : W044RustRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W044RustRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W044RustRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsRefinementRow (row : W044RustRow) : Bool :=
  row.refinementRow && !row.promotionClaim

def IsAcceptedBoundary (row : W044RustRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def ResultErrorCarrierRow : W044RustRow :=
  { rowId := "w044.result-error-carrier-totality-evidence",
    obligationId := "W044-OBL-013",
    evidenceKind := W044RustEvidenceKind.rustResultCarrier,
    dispositionKind := W044RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TreeCalcPacketRow : W044RustRow :=
  { rowId := "w044.treecalc-packet-totality-evidence",
    obligationId := "W044-OBL-013",
    evidenceKind := W044RustEvidenceKind.treecalcPacket,
    dispositionKind := W044RustDispositionKind.directTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def PredecessorRustRegressionRow : W044RustRow :=
  { rowId := "w044.w043-rust-refinement-regression",
    obligationId := "W044-OBL-014",
    evidenceKind := W044RustEvidenceKind.predecessorRustRegression,
    dispositionKind := W044RustDispositionKind.carriedRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def MixedDynamicTransitionRefinementRow : W044RustRow :=
  { rowId := "w044.mixed-dynamic-add-release-refinement-evidence",
    obligationId := "W044-OBL-014",
    evidenceKind := W044RustEvidenceKind.mixedDynamicTransitionEvidence,
    dispositionKind := W044RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := true,
    exactRemainingBlocker := false,
    promotionClaim := false }

def PublicationFenceNoPublishRow : W044RustRow :=
  { rowId := "w044.publication-fence-no-publish-refinement-evidence",
    obligationId := "W044-OBL-014",
    evidenceKind := W044RustEvidenceKind.publicationFenceNoPublishEvidence,
    dispositionKind := W044RustDispositionKind.directRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def PredecessorDynamicRegressionRow : W044RustRow :=
  { rowId := "w044.w043-dynamic-transition-regression-evidence",
    obligationId := "W044-OBL-014",
    evidenceKind := W044RustEvidenceKind.predecessorDynamicRegression,
    dispositionKind := W044RustDispositionKind.carriedRefinementEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def SnapshotFenceBreadthBoundaryRow : W044RustRow :=
  { rowId := "w044.snapshot-fence-breadth-refinement-boundary",
    obligationId := "W044-OBL-014",
    evidenceKind := W044RustEvidenceKind.snapshotFenceBreadth,
    dispositionKind := W044RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CapabilityViewBreadthBoundaryRow : W044RustRow :=
  { rowId := "w044.capability-view-breadth-refinement-boundary",
    obligationId := "W044-OBL-014",
    evidenceKind := W044RustEvidenceKind.capabilityViewBreadth,
    dispositionKind := W044RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CallableValueCarrierRow : W044RustRow :=
  { rowId := "w044.callable-value-carrier-totality-evidence",
    obligationId := "W044-OBL-015",
    evidenceKind := W044RustEvidenceKind.callableValueCarrier,
    dispositionKind := W044RustDispositionKind.directCallableValueCarrierTotalityEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def RuntimePanicSurfaceBoundaryRow : W044RustRow :=
  { rowId := "w044.runtime-panic-surface-totality-boundary",
    obligationId := "W044-OBL-012",
    evidenceKind := W044RustEvidenceKind.rustPanicAudit,
    dispositionKind := W044RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def BroaderDynamicTransitionBoundaryRow : W044RustRow :=
  { rowId := "w044.broader-dynamic-transition-coverage-refinement-boundary",
    obligationId := "W044-OBL-014",
    evidenceKind := W044RustEvidenceKind.broaderDynamicCoverage,
    dispositionKind := W044RustDispositionKind.exactRefinementBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CallableMetadataProjectionBoundaryRow : W044RustRow :=
  { rowId := "w044.callable-metadata-projection-totality-boundary",
    obligationId := "W044-OBL-015",
    evidenceKind := W044RustEvidenceKind.callableMetadataProjection,
    dispositionKind := W044RustDispositionKind.exactTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullOptimizedCoreReleaseGradeBoundaryRow : W044RustRow :=
  { rowId := "w044.full-optimized-core-release-grade-conformance-boundary",
    obligationId := "W044-OBL-014",
    evidenceKind := W044RustEvidenceKind.releaseGradeBoundary,
    dispositionKind := W044RustDispositionKind.exactReleaseGradeBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    refinementRow := true,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaCarrierExternalBoundaryRow : W044RustRow :=
  { rowId := "w044.let-lambda-carrier-external-boundary",
    obligationId := "W044-OBL-015",
    evidenceKind := W044RustEvidenceKind.externalSemanticAuthority,
    dispositionKind := W044RustDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def SpecEvolutionRefinementGuardRow : W044RustRow :=
  { rowId := "w044.spec-evolution-refinement-guard",
    obligationId := "W044-OBL-003",
    evidenceKind := W044RustEvidenceKind.specEvolutionGuard,
    dispositionKind := W044RustDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def W073TypedFormattingGuardRow : W044RustRow :=
  { rowId := "w044.w073-typed-formatting-rust-boundary-guard",
    obligationId := "W044-OBL-033",
    evidenceKind := W044RustEvidenceKind.w073TypedFormattingGuard,
    dispositionKind := W044RustDispositionKind.acceptedFormattingBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    refinementRow := false,
    automaticDynamicTransitionRow := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem mixedDynamicTransition_isDirectRefinementEvidence :
    IsRefinementRow MixedDynamicTransitionRefinementRow = true
      /\ MixedDynamicTransitionRefinementRow.automaticDynamicTransitionRow = true
      /\ MixedDynamicTransitionRefinementRow.exactRemainingBlocker = false
      /\ MixedDynamicTransitionRefinementRow.promotionClaim = false := by
  constructor
  · rfl
  constructor
  · rfl
  constructor <;> rfl

theorem publicationFenceNoPublish_isRefinementEvidence :
    IsRefinementRow PublicationFenceNoPublishRow = true
      /\ PublicationFenceNoPublishRow.exactRemainingBlocker = false
      /\ PublicationFenceNoPublishRow.promotionClaim = false := by
  constructor
  · rfl
  constructor <;> rfl

theorem counterpartBreadthRows_areExactRefinementBlockers :
    IsExactBlocker SnapshotFenceBreadthBoundaryRow = true
      /\ IsRefinementRow SnapshotFenceBreadthBoundaryRow = true
      /\ IsExactBlocker CapabilityViewBreadthBoundaryRow = true
      /\ IsRefinementRow CapabilityViewBreadthBoundaryRow = true := by
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

theorem retainedW044RustBoundaries_areExact :
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

theorem w073TypedFormattingGuard_isAcceptedBoundary :
    IsAcceptedBoundary W073TypedFormattingGuardRow = true
      /\ W073TypedFormattingGuardRow.promotionClaim = false := by
  constructor <;> rfl

theorem allW044RustRows_nonPromoting :
    IsNonPromoting ResultErrorCarrierRow = true
      /\ IsNonPromoting TreeCalcPacketRow = true
      /\ IsNonPromoting PredecessorRustRegressionRow = true
      /\ IsNonPromoting MixedDynamicTransitionRefinementRow = true
      /\ IsNonPromoting PublicationFenceNoPublishRow = true
      /\ IsNonPromoting PredecessorDynamicRegressionRow = true
      /\ IsNonPromoting SnapshotFenceBreadthBoundaryRow = true
      /\ IsNonPromoting CapabilityViewBreadthBoundaryRow = true
      /\ IsNonPromoting CallableValueCarrierRow = true
      /\ IsNonPromoting RuntimePanicSurfaceBoundaryRow = true
      /\ IsNonPromoting BroaderDynamicTransitionBoundaryRow = true
      /\ IsNonPromoting CallableMetadataProjectionBoundaryRow = true
      /\ IsNonPromoting FullOptimizedCoreReleaseGradeBoundaryRow = true
      /\ IsNonPromoting LetLambdaCarrierExternalBoundaryRow = true
      /\ IsNonPromoting SpecEvolutionRefinementGuardRow = true
      /\ IsNonPromoting W073TypedFormattingGuardRow = true := by
  simp [
    IsNonPromoting,
    ResultErrorCarrierRow,
    TreeCalcPacketRow,
    PredecessorRustRegressionRow,
    MixedDynamicTransitionRefinementRow,
    PublicationFenceNoPublishRow,
    PredecessorDynamicRegressionRow,
    SnapshotFenceBreadthBoundaryRow,
    CapabilityViewBreadthBoundaryRow,
    CallableValueCarrierRow,
    RuntimePanicSurfaceBoundaryRow,
    BroaderDynamicTransitionBoundaryRow,
    CallableMetadataProjectionBoundaryRow,
    FullOptimizedCoreReleaseGradeBoundaryRow,
    LetLambdaCarrierExternalBoundaryRow,
    SpecEvolutionRefinementGuardRow,
    W073TypedFormattingGuardRow
  ]

structure W044RustSummary where
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

def W044RustSummaryValue : W044RustSummary :=
  { rustRows := 16,
    localProofRows := 11,
    boundedModelRows := 0,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 3,
    totalityBoundaryRows := 4,
    refinementRows := 9,
    exactRemainingBlockerRows := 6,
    automaticDynamicTransitionRefinementRows := 1,
    rustEngineTotalityPromoted := false,
    rustRefinementPromoted := false,
    panicFreeCoreDomainPromoted := false,
    fullOptimizedCoreVerificationPromoted := false,
    callableMetadataProjectionPromoted := false,
    callableCarrierSufficiencyPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w044RustSummary_hasSixteenRows :
    W044RustSummaryValue.rustRows = 16 := by
  rfl

theorem w044RustSummary_hasOneMixedDynamicTransitionRefinementRow :
    W044RustSummaryValue.automaticDynamicTransitionRefinementRows = 1 := by
  rfl

theorem w044RustSummary_hasSixExactBlockers :
    W044RustSummaryValue.exactRemainingBlockerRows = 6 := by
  rfl

theorem w044RustSummary_noRustTotalityPromotion :
    W044RustSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w044RustSummary_noRustRefinementPromotion :
    W044RustSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w044RustSummary_noPanicFreeCorePromotion :
    W044RustSummaryValue.panicFreeCoreDomainPromoted = false := by
  rfl

theorem w044RustSummary_noOptimizedCorePromotion :
    W044RustSummaryValue.fullOptimizedCoreVerificationPromoted = false := by
  rfl

theorem w044RustSummary_noCallableMetadataProjectionPromotion :
    W044RustSummaryValue.callableMetadataProjectionPromoted = false := by
  rfl

theorem w044RustSummary_noCallableCarrierSufficiencyPromotion :
    W044RustSummaryValue.callableCarrierSufficiencyPromoted = false := by
  rfl

theorem w044RustSummary_noGeneralOxFuncKernelPromotion :
    W044RustSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W044.RustTotalityAndRefinement
