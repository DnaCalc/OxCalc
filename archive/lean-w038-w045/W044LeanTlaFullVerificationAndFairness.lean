import Std

namespace OxCalc.CoreEngine.W044.LeanTlaFullVerificationAndFairness

inductive W044ProofModelEvidenceKind where
  | leanInventory
  | leanTlaPredecessorBridge
  | leanRustMixedDynamicBridge
  | leanRustPublicationFenceBridge
  | callableCarrierBridge
  | leanStage2SchedulerPackPredicate
  | tlaRoutineBoundedModel
  | tlaStage2PartitionModel
  | tlaStage2EquivalenceInput
  | tlaFairnessBoundary
  | leanFullVerificationBoundary
  | tlaFullVerificationBoundary
  | rustTotalityDependency
  | externalSemanticAuthority
  | specEvolutionGuard
  | w073TypedFormattingGuard
  deriving DecidableEq, Repr

inductive W044ProofModelDispositionKind where
  | checkedLeanEvidence
  | checkedLeanBridgeEvidence
  | checkedLeanRefinementBridge
  | checkedLeanPublicationFenceBridge
  | checkedLeanCallableCarrierBridge
  | checkedLeanPolicyPredicate
  | boundedModelBoundary
  | boundedStage2ModelEvidence
  | boundedStage2EquivalenceEvidence
  | exactModelAssumptionBoundary
  | exactLeanVerificationBlocker
  | exactTlaVerificationBlocker
  | exactRustDependencyBlocker
  | acceptedExternalSeamBoundary
  | acceptedSpecEvolutionGuard
  | acceptedFormattingBoundary
  deriving DecidableEq, Repr

structure W044ProofModelRow where
  rowId : String
  obligationId : String
  evidenceKind : W044ProofModelEvidenceKind
  dispositionKind : W044ProofModelDispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W044ProofModelRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W044ProofModelRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W044ProofModelRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsAcceptedBoundary (row : W044ProofModelRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def LeanInventoryNoPlaceholderEvidenceRow : W044ProofModelRow :=
  { rowId := "w044.lean-inventory-checked-no-placeholder-evidence",
    obligationId := "W044-OBL-016",
    evidenceKind := W044ProofModelEvidenceKind.leanInventory,
    dispositionKind := W044ProofModelDispositionKind.checkedLeanEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanTlaPredecessorBridgeRow : W044ProofModelRow :=
  { rowId := "w044.lean-tla-predecessor-bridge",
    obligationId := "W044-OBL-016",
    evidenceKind := W044ProofModelEvidenceKind.leanTlaPredecessorBridge,
    dispositionKind := W044ProofModelDispositionKind.checkedLeanBridgeEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanRustMixedDynamicBridgeRow : W044ProofModelRow :=
  { rowId := "w044.lean-rust-mixed-dynamic-refinement-bridge",
    obligationId := "W044-OBL-018",
    evidenceKind := W044ProofModelEvidenceKind.leanRustMixedDynamicBridge,
    dispositionKind := W044ProofModelDispositionKind.checkedLeanRefinementBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanRustPublicationFenceBridgeRow : W044ProofModelRow :=
  { rowId := "w044.lean-rust-publication-fence-no-publish-bridge",
    obligationId := "W044-OBL-018",
    evidenceKind := W044ProofModelEvidenceKind.leanRustPublicationFenceBridge,
    dispositionKind := W044ProofModelDispositionKind.checkedLeanPublicationFenceBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanCallableCarrierBridgeRow : W044ProofModelRow :=
  { rowId := "w044.lean-callable-carrier-boundary-bridge",
    obligationId := "W044-OBL-015",
    evidenceKind := W044ProofModelEvidenceKind.callableCarrierBridge,
    dispositionKind := W044ProofModelDispositionKind.checkedLeanCallableCarrierBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanStage2SchedulerPackPredicateRow : W044ProofModelRow :=
  { rowId := "w044.lean-stage2-scheduler-pack-predicate-carried",
    obligationId := "W044-OBL-018",
    evidenceKind := W044ProofModelEvidenceKind.leanStage2SchedulerPackPredicate,
    dispositionKind := W044ProofModelDispositionKind.checkedLeanPolicyPredicate,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaRoutineConfigBoundedModelBoundaryRow : W044ProofModelRow :=
  { rowId := "w044.tla-routine-config-bounded-model-boundary",
    obligationId := "W044-OBL-017",
    evidenceKind := W044ProofModelEvidenceKind.tlaRoutineBoundedModel,
    dispositionKind := W044ProofModelDispositionKind.boundedModelBoundary,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def TlaStage2PartitionBoundedModelRow : W044ProofModelRow :=
  { rowId := "w044.tla-stage2-partition-bounded-model-evidence",
    obligationId := "W044-OBL-017",
    evidenceKind := W044ProofModelEvidenceKind.tlaStage2PartitionModel,
    dispositionKind := W044ProofModelDispositionKind.boundedStage2ModelEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def Stage2EquivalenceBoundedModelInputRow : W044ProofModelRow :=
  { rowId := "w044.stage2-equivalence-bounded-model-input",
    obligationId := "W044-OBL-018",
    evidenceKind := W044ProofModelEvidenceKind.tlaStage2EquivalenceInput,
    dispositionKind := W044ProofModelDispositionKind.boundedStage2EquivalenceEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaFairnessSchedulerAssumptionBoundaryRow : W044ProofModelRow :=
  { rowId := "w044.tla-fairness-scheduler-unbounded-boundary",
    obligationId := "W044-OBL-017",
    evidenceKind := W044ProofModelEvidenceKind.tlaFairnessBoundary,
    dispositionKind := W044ProofModelDispositionKind.exactModelAssumptionBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullLeanVerificationBlockerRow : W044ProofModelRow :=
  { rowId := "w044.full-lean-verification-exact-blocker",
    obligationId := "W044-OBL-016",
    evidenceKind := W044ProofModelEvidenceKind.leanFullVerificationBoundary,
    dispositionKind := W044ProofModelDispositionKind.exactLeanVerificationBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullTlaVerificationBlockerRow : W044ProofModelRow :=
  { rowId := "w044.full-tla-verification-exact-blocker",
    obligationId := "W044-OBL-017",
    evidenceKind := W044ProofModelEvidenceKind.tlaFullVerificationBoundary,
    dispositionKind := W044ProofModelDispositionKind.exactTlaVerificationBlocker,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def RustTotalityDependencyBlockerRow : W044ProofModelRow :=
  { rowId := "w044.rust-totality-dependency-exact-blocker",
    obligationId := "W044-OBL-018",
    evidenceKind := W044ProofModelEvidenceKind.rustTotalityDependency,
    dispositionKind := W044ProofModelDispositionKind.exactRustDependencyBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaExternalOxFuncBoundaryRow : W044ProofModelRow :=
  { rowId := "w044.let-lambda-external-oxfunc-boundary",
    obligationId := "W044-OBL-015",
    evidenceKind := W044ProofModelEvidenceKind.externalSemanticAuthority,
    dispositionKind := W044ProofModelDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def FormalModelSpecEvolutionGuardRow : W044ProofModelRow :=
  { rowId := "w044.formal-model-spec-evolution-guard",
    obligationId := "W044-OBL-003",
    evidenceKind := W044ProofModelEvidenceKind.specEvolutionGuard,
    dispositionKind := W044ProofModelDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def W073TypedFormattingGuardRow : W044ProofModelRow :=
  { rowId := "w044.w073-typed-formatting-lean-tla-boundary-guard",
    obligationId := "W044-OBL-033",
    evidenceKind := W044ProofModelEvidenceKind.w073TypedFormattingGuard,
    dispositionKind := W044ProofModelDispositionKind.acceptedFormattingBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem w044DynamicAndPublicationBridges_areCheckedNonPromoting :
    IsNonPromoting LeanRustMixedDynamicBridgeRow = true
      /\ IsNonPromoting LeanRustPublicationFenceBridgeRow = true
      /\ LeanRustMixedDynamicBridgeRow.exactRemainingBlocker = false
      /\ LeanRustPublicationFenceBridgeRow.exactRemainingBlocker = false := by
  constructor
  · rfl
  constructor
  · rfl
  constructor <;> rfl

theorem callableCarrierBridge_isNotSufficiencyPromotion :
    IsNonPromoting LeanCallableCarrierBridgeRow = true
      /\ LeanCallableCarrierBridgeRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem stage2SchedulerPackInput_isCheckedNonPromoting :
    IsNonPromoting LeanStage2SchedulerPackPredicateRow = true
      /\ LeanStage2SchedulerPackPredicateRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem stage2EquivalenceInput_isBoundedNonPromoting :
    Stage2EquivalenceBoundedModelInputRow.boundedModel = true
      /\ Stage2EquivalenceBoundedModelInputRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem tlaRoutineBoundary_isBoundedExact :
    TlaRoutineConfigBoundedModelBoundaryRow.boundedModel = true
      /\ IsExactBlocker TlaRoutineConfigBoundedModelBoundaryRow = true
      /\ IsTotalityBoundary TlaRoutineConfigBoundedModelBoundaryRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem tlaFairnessBoundary_isExact :
    IsExactBlocker TlaFairnessSchedulerAssumptionBoundaryRow = true
      /\ IsTotalityBoundary TlaFairnessSchedulerAssumptionBoundaryRow = true := by
  constructor <;> rfl

theorem fullLeanBlocker_isExactTotalityBoundary :
    IsExactBlocker FullLeanVerificationBlockerRow = true
      /\ IsTotalityBoundary FullLeanVerificationBlockerRow = true := by
  constructor <;> rfl

theorem fullTlaBlocker_isExactTotalityBoundary :
    IsExactBlocker FullTlaVerificationBlockerRow = true
      /\ IsTotalityBoundary FullTlaVerificationBlockerRow = true := by
  constructor <;> rfl

theorem rustDependencyBlocker_isExactTotalityBoundary :
    IsExactBlocker RustTotalityDependencyBlockerRow = true
      /\ IsTotalityBoundary RustTotalityDependencyBlockerRow = true := by
  constructor <;> rfl

theorem letLambdaBoundary_isAcceptedExternal :
    LetLambdaExternalOxFuncBoundaryRow.acceptedExternalSeam = true
      /\ IsAcceptedBoundary LetLambdaExternalOxFuncBoundaryRow = true := by
  constructor <;> rfl

theorem w073TypedFormattingGuard_isAcceptedBoundary :
    IsAcceptedBoundary W073TypedFormattingGuardRow = true
      /\ W073TypedFormattingGuardRow.promotionClaim = false := by
  constructor <;> rfl

theorem allW044ProofModelRows_nonPromoting :
    IsNonPromoting LeanInventoryNoPlaceholderEvidenceRow = true
      /\ IsNonPromoting LeanTlaPredecessorBridgeRow = true
      /\ IsNonPromoting LeanRustMixedDynamicBridgeRow = true
      /\ IsNonPromoting LeanRustPublicationFenceBridgeRow = true
      /\ IsNonPromoting LeanCallableCarrierBridgeRow = true
      /\ IsNonPromoting LeanStage2SchedulerPackPredicateRow = true
      /\ IsNonPromoting TlaRoutineConfigBoundedModelBoundaryRow = true
      /\ IsNonPromoting TlaStage2PartitionBoundedModelRow = true
      /\ IsNonPromoting Stage2EquivalenceBoundedModelInputRow = true
      /\ IsNonPromoting TlaFairnessSchedulerAssumptionBoundaryRow = true
      /\ IsNonPromoting FullLeanVerificationBlockerRow = true
      /\ IsNonPromoting FullTlaVerificationBlockerRow = true
      /\ IsNonPromoting RustTotalityDependencyBlockerRow = true
      /\ IsNonPromoting LetLambdaExternalOxFuncBoundaryRow = true
      /\ IsNonPromoting FormalModelSpecEvolutionGuardRow = true
      /\ IsNonPromoting W073TypedFormattingGuardRow = true := by
  simp [
    IsNonPromoting,
    LeanInventoryNoPlaceholderEvidenceRow,
    LeanTlaPredecessorBridgeRow,
    LeanRustMixedDynamicBridgeRow,
    LeanRustPublicationFenceBridgeRow,
    LeanCallableCarrierBridgeRow,
    LeanStage2SchedulerPackPredicateRow,
    TlaRoutineConfigBoundedModelBoundaryRow,
    TlaStage2PartitionBoundedModelRow,
    Stage2EquivalenceBoundedModelInputRow,
    TlaFairnessSchedulerAssumptionBoundaryRow,
    FullLeanVerificationBlockerRow,
    FullTlaVerificationBlockerRow,
    RustTotalityDependencyBlockerRow,
    LetLambdaExternalOxFuncBoundaryRow,
    FormalModelSpecEvolutionGuardRow,
    W073TypedFormattingGuardRow
  ]

structure W044LeanTlaSummary where
  proofModelRows : Nat
  localProofRows : Nat
  boundedModelRows : Nat
  acceptedExternalSeamRows : Nat
  acceptedBoundaryRows : Nat
  totalityBoundaryRows : Nat
  exactRemainingBlockerRows : Nat
  dynamicRefinementBridgeRows : Nat
  publicationFenceBridgeRows : Nat
  callableCarrierBridgeRows : Nat
  fullLeanVerificationPromoted : Bool
  fullTlaVerificationPromoted : Bool
  schedulerFairnessPromoted : Bool
  unboundedModelCoveragePromoted : Bool
  rustEngineTotalityPromoted : Bool
  rustRefinementPromoted : Bool
  stage2PolicyPromoted : Bool
  callableCarrierSufficiencyPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  deriving DecidableEq, Repr

def W044LeanTlaSummaryValue : W044LeanTlaSummary :=
  { proofModelRows := 16,
    localProofRows := 10,
    boundedModelRows := 4,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 3,
    totalityBoundaryRows := 5,
    exactRemainingBlockerRows := 5,
    dynamicRefinementBridgeRows := 1,
    publicationFenceBridgeRows := 1,
    callableCarrierBridgeRows := 1,
    fullLeanVerificationPromoted := false,
    fullTlaVerificationPromoted := false,
    schedulerFairnessPromoted := false,
    unboundedModelCoveragePromoted := false,
    rustEngineTotalityPromoted := false,
    rustRefinementPromoted := false,
    stage2PolicyPromoted := false,
    callableCarrierSufficiencyPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w044LeanTlaSummary_hasSixteenRows :
    W044LeanTlaSummaryValue.proofModelRows = 16 := by
  rfl

theorem w044LeanTlaSummary_hasFiveExactBlockers :
    W044LeanTlaSummaryValue.exactRemainingBlockerRows = 5 := by
  rfl

theorem w044LeanTlaSummary_hasOneMixedDynamicBridge :
    W044LeanTlaSummaryValue.dynamicRefinementBridgeRows = 1 := by
  rfl

theorem w044LeanTlaSummary_hasOnePublicationFenceBridge :
    W044LeanTlaSummaryValue.publicationFenceBridgeRows = 1 := by
  rfl

theorem w044LeanTlaSummary_hasOneCallableCarrierBridge :
    W044LeanTlaSummaryValue.callableCarrierBridgeRows = 1 := by
  rfl

theorem w044LeanTlaSummary_noFullLeanPromotion :
    W044LeanTlaSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

theorem w044LeanTlaSummary_noFullTlaPromotion :
    W044LeanTlaSummaryValue.fullTlaVerificationPromoted = false := by
  rfl

theorem w044LeanTlaSummary_noSchedulerFairnessPromotion :
    W044LeanTlaSummaryValue.schedulerFairnessPromoted = false := by
  rfl

theorem w044LeanTlaSummary_noUnboundedModelCoveragePromotion :
    W044LeanTlaSummaryValue.unboundedModelCoveragePromoted = false := by
  rfl

theorem w044LeanTlaSummary_noRustTotalityPromotion :
    W044LeanTlaSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w044LeanTlaSummary_noRustRefinementPromotion :
    W044LeanTlaSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w044LeanTlaSummary_noStage2PolicyPromotion :
    W044LeanTlaSummaryValue.stage2PolicyPromoted = false := by
  rfl

theorem w044LeanTlaSummary_noCallableCarrierSufficiencyPromotion :
    W044LeanTlaSummaryValue.callableCarrierSufficiencyPromoted = false := by
  rfl

theorem w044LeanTlaSummary_noGeneralOxFuncKernelPromotion :
    W044LeanTlaSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W044.LeanTlaFullVerificationAndFairness
