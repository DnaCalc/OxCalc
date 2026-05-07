import Std

namespace OxCalc.CoreEngine.W043.LeanTlaFullVerificationAndFairness

inductive W043ProofModelEvidenceKind where
  | leanInventory
  | leanTlaPredecessorBridge
  | leanRustDynamicAdditionBridge
  | leanRustDynamicReleaseBridge
  | callableCarrierBridge
  | leanStage2AnalyzerPackPredicate
  | tlaRoutineBoundedModel
  | tlaStage2PartitionModel
  | tlaStage2EquivalenceInput
  | tlaFairnessBoundary
  | leanFullVerificationBoundary
  | tlaFullVerificationBoundary
  | rustTotalityDependency
  | externalSemanticAuthority
  | specEvolutionGuard
  deriving DecidableEq, Repr

inductive W043ProofModelDispositionKind where
  | checkedLeanEvidence
  | checkedLeanBridgeEvidence
  | checkedLeanRefinementBridge
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
  deriving DecidableEq, Repr

structure W043ProofModelRow where
  rowId : String
  obligationId : String
  evidenceKind : W043ProofModelEvidenceKind
  dispositionKind : W043ProofModelDispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W043ProofModelRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W043ProofModelRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W043ProofModelRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsAcceptedBoundary (row : W043ProofModelRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def LeanInventoryNoPlaceholderEvidenceRow : W043ProofModelRow :=
  { rowId := "w043.lean-inventory-checked-no-placeholder-evidence",
    obligationId := "W043-OBL-012",
    evidenceKind := W043ProofModelEvidenceKind.leanInventory,
    dispositionKind := W043ProofModelDispositionKind.checkedLeanEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanTlaPredecessorBridgeRow : W043ProofModelRow :=
  { rowId := "w043.lean-tla-predecessor-bridge",
    obligationId := "W043-OBL-012",
    evidenceKind := W043ProofModelEvidenceKind.leanTlaPredecessorBridge,
    dispositionKind := W043ProofModelDispositionKind.checkedLeanBridgeEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanRustDynamicAdditionBridgeRow : W043ProofModelRow :=
  { rowId := "w043.lean-rust-dynamic-addition-refinement-bridge",
    obligationId := "W043-OBL-014",
    evidenceKind := W043ProofModelEvidenceKind.leanRustDynamicAdditionBridge,
    dispositionKind := W043ProofModelDispositionKind.checkedLeanRefinementBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanRustDynamicReleaseBridgeRow : W043ProofModelRow :=
  { rowId := "w043.lean-rust-dynamic-release-refinement-bridge",
    obligationId := "W043-OBL-014",
    evidenceKind := W043ProofModelEvidenceKind.leanRustDynamicReleaseBridge,
    dispositionKind := W043ProofModelDispositionKind.checkedLeanRefinementBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanCallableCarrierBridgeRow : W043ProofModelRow :=
  { rowId := "w043.lean-callable-carrier-boundary-bridge",
    obligationId := "W043-OBL-015",
    evidenceKind := W043ProofModelEvidenceKind.callableCarrierBridge,
    dispositionKind := W043ProofModelDispositionKind.checkedLeanCallableCarrierBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanStage2AnalyzerPackPredicateRow : W043ProofModelRow :=
  { rowId := "w043.lean-stage2-analyzer-pack-predicate-carried",
    obligationId := "W043-OBL-014",
    evidenceKind := W043ProofModelEvidenceKind.leanStage2AnalyzerPackPredicate,
    dispositionKind := W043ProofModelDispositionKind.checkedLeanPolicyPredicate,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaRoutineConfigBoundedModelBoundaryRow : W043ProofModelRow :=
  { rowId := "w043.tla-routine-config-bounded-model-boundary",
    obligationId := "W043-OBL-013",
    evidenceKind := W043ProofModelEvidenceKind.tlaRoutineBoundedModel,
    dispositionKind := W043ProofModelDispositionKind.boundedModelBoundary,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def TlaStage2PartitionBoundedModelRow : W043ProofModelRow :=
  { rowId := "w043.tla-stage2-partition-bounded-model-evidence",
    obligationId := "W043-OBL-013",
    evidenceKind := W043ProofModelEvidenceKind.tlaStage2PartitionModel,
    dispositionKind := W043ProofModelDispositionKind.boundedStage2ModelEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def Stage2EquivalenceBoundedModelInputRow : W043ProofModelRow :=
  { rowId := "w043.stage2-equivalence-bounded-model-input",
    obligationId := "W043-OBL-014",
    evidenceKind := W043ProofModelEvidenceKind.tlaStage2EquivalenceInput,
    dispositionKind := W043ProofModelDispositionKind.boundedStage2EquivalenceEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaFairnessSchedulerAssumptionBoundaryRow : W043ProofModelRow :=
  { rowId := "w043.tla-fairness-scheduler-unbounded-boundary",
    obligationId := "W043-OBL-013",
    evidenceKind := W043ProofModelEvidenceKind.tlaFairnessBoundary,
    dispositionKind := W043ProofModelDispositionKind.exactModelAssumptionBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullLeanVerificationBlockerRow : W043ProofModelRow :=
  { rowId := "w043.full-lean-verification-exact-blocker",
    obligationId := "W043-OBL-012",
    evidenceKind := W043ProofModelEvidenceKind.leanFullVerificationBoundary,
    dispositionKind := W043ProofModelDispositionKind.exactLeanVerificationBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullTlaVerificationBlockerRow : W043ProofModelRow :=
  { rowId := "w043.full-tla-verification-exact-blocker",
    obligationId := "W043-OBL-013",
    evidenceKind := W043ProofModelEvidenceKind.tlaFullVerificationBoundary,
    dispositionKind := W043ProofModelDispositionKind.exactTlaVerificationBlocker,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def RustTotalityDependencyBlockerRow : W043ProofModelRow :=
  { rowId := "w043.rust-totality-dependency-exact-blocker",
    obligationId := "W043-OBL-014",
    evidenceKind := W043ProofModelEvidenceKind.rustTotalityDependency,
    dispositionKind := W043ProofModelDispositionKind.exactRustDependencyBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaExternalOxFuncBoundaryRow : W043ProofModelRow :=
  { rowId := "w043.let-lambda-external-oxfunc-boundary",
    obligationId := "W043-OBL-036",
    evidenceKind := W043ProofModelEvidenceKind.externalSemanticAuthority,
    dispositionKind := W043ProofModelDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def FormalModelSpecEvolutionGuardRow : W043ProofModelRow :=
  { rowId := "w043.formal-model-spec-evolution-guard",
    obligationId := "W043-OBL-014",
    evidenceKind := W043ProofModelEvidenceKind.specEvolutionGuard,
    dispositionKind := W043ProofModelDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem dynamicRefinementBridges_areCheckedNonPromoting :
    IsNonPromoting LeanRustDynamicAdditionBridgeRow = true
      /\ IsNonPromoting LeanRustDynamicReleaseBridgeRow = true
      /\ LeanRustDynamicAdditionBridgeRow.exactRemainingBlocker = false
      /\ LeanRustDynamicReleaseBridgeRow.exactRemainingBlocker = false := by
  constructor
  · rfl
  constructor
  · rfl
  constructor <;> rfl

theorem callableCarrierBridge_isNotSufficiencyPromotion :
    IsNonPromoting LeanCallableCarrierBridgeRow = true
      /\ LeanCallableCarrierBridgeRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem stage2AnalyzerPackInput_isCheckedNonPromoting :
    IsNonPromoting LeanStage2AnalyzerPackPredicateRow = true
      /\ LeanStage2AnalyzerPackPredicateRow.exactRemainingBlocker = false := by
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

theorem allW043ProofModelRows_nonPromoting :
    IsNonPromoting LeanInventoryNoPlaceholderEvidenceRow = true
      /\ IsNonPromoting LeanTlaPredecessorBridgeRow = true
      /\ IsNonPromoting LeanRustDynamicAdditionBridgeRow = true
      /\ IsNonPromoting LeanRustDynamicReleaseBridgeRow = true
      /\ IsNonPromoting LeanCallableCarrierBridgeRow = true
      /\ IsNonPromoting LeanStage2AnalyzerPackPredicateRow = true
      /\ IsNonPromoting TlaRoutineConfigBoundedModelBoundaryRow = true
      /\ IsNonPromoting TlaStage2PartitionBoundedModelRow = true
      /\ IsNonPromoting Stage2EquivalenceBoundedModelInputRow = true
      /\ IsNonPromoting TlaFairnessSchedulerAssumptionBoundaryRow = true
      /\ IsNonPromoting FullLeanVerificationBlockerRow = true
      /\ IsNonPromoting FullTlaVerificationBlockerRow = true
      /\ IsNonPromoting RustTotalityDependencyBlockerRow = true
      /\ IsNonPromoting LetLambdaExternalOxFuncBoundaryRow = true
      /\ IsNonPromoting FormalModelSpecEvolutionGuardRow = true := by
  simp [
    IsNonPromoting,
    LeanInventoryNoPlaceholderEvidenceRow,
    LeanTlaPredecessorBridgeRow,
    LeanRustDynamicAdditionBridgeRow,
    LeanRustDynamicReleaseBridgeRow,
    LeanCallableCarrierBridgeRow,
    LeanStage2AnalyzerPackPredicateRow,
    TlaRoutineConfigBoundedModelBoundaryRow,
    TlaStage2PartitionBoundedModelRow,
    Stage2EquivalenceBoundedModelInputRow,
    TlaFairnessSchedulerAssumptionBoundaryRow,
    FullLeanVerificationBlockerRow,
    FullTlaVerificationBlockerRow,
    RustTotalityDependencyBlockerRow,
    LetLambdaExternalOxFuncBoundaryRow,
    FormalModelSpecEvolutionGuardRow
  ]

structure W043LeanTlaSummary where
  proofModelRows : Nat
  localProofRows : Nat
  boundedModelRows : Nat
  acceptedExternalSeamRows : Nat
  acceptedBoundaryRows : Nat
  totalityBoundaryRows : Nat
  exactRemainingBlockerRows : Nat
  dynamicRefinementBridgeRows : Nat
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

def W043LeanTlaSummaryValue : W043LeanTlaSummary :=
  { proofModelRows := 15,
    localProofRows := 9,
    boundedModelRows := 4,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 2,
    totalityBoundaryRows := 5,
    exactRemainingBlockerRows := 5,
    dynamicRefinementBridgeRows := 2,
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

theorem w043LeanTlaSummary_hasFifteenRows :
    W043LeanTlaSummaryValue.proofModelRows = 15 := by
  rfl

theorem w043LeanTlaSummary_hasFiveExactBlockers :
    W043LeanTlaSummaryValue.exactRemainingBlockerRows = 5 := by
  rfl

theorem w043LeanTlaSummary_hasTwoDynamicRefinementBridges :
    W043LeanTlaSummaryValue.dynamicRefinementBridgeRows = 2 := by
  rfl

theorem w043LeanTlaSummary_hasOneCallableCarrierBridge :
    W043LeanTlaSummaryValue.callableCarrierBridgeRows = 1 := by
  rfl

theorem w043LeanTlaSummary_noFullLeanPromotion :
    W043LeanTlaSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

theorem w043LeanTlaSummary_noFullTlaPromotion :
    W043LeanTlaSummaryValue.fullTlaVerificationPromoted = false := by
  rfl

theorem w043LeanTlaSummary_noSchedulerFairnessPromotion :
    W043LeanTlaSummaryValue.schedulerFairnessPromoted = false := by
  rfl

theorem w043LeanTlaSummary_noUnboundedModelCoveragePromotion :
    W043LeanTlaSummaryValue.unboundedModelCoveragePromoted = false := by
  rfl

theorem w043LeanTlaSummary_noRustTotalityPromotion :
    W043LeanTlaSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w043LeanTlaSummary_noRustRefinementPromotion :
    W043LeanTlaSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w043LeanTlaSummary_noStage2PolicyPromotion :
    W043LeanTlaSummaryValue.stage2PolicyPromoted = false := by
  rfl

theorem w043LeanTlaSummary_noCallableCarrierSufficiencyPromotion :
    W043LeanTlaSummaryValue.callableCarrierSufficiencyPromoted = false := by
  rfl

theorem w043LeanTlaSummary_noGeneralOxFuncKernelPromotion :
    W043LeanTlaSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W043.LeanTlaFullVerificationAndFairness
