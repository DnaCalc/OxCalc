import Std

namespace OxCalc.CoreEngine.W042.LeanTlaFairnessFullVerificationExpansion

inductive W042ProofModelEvidenceKind where
  | leanInventory
  | leanTlaPredecessorBridge
  | leanRustDynamicRefinementBridge
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

inductive W042ProofModelDispositionKind where
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

structure W042ProofModelRow where
  rowId : String
  obligationId : String
  evidenceKind : W042ProofModelEvidenceKind
  dispositionKind : W042ProofModelDispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W042ProofModelRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W042ProofModelRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W042ProofModelRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsAcceptedBoundary (row : W042ProofModelRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def LeanInventoryNoPlaceholderEvidenceRow : W042ProofModelRow :=
  { rowId := "w042.lean-inventory-checked-no-placeholder-evidence",
    obligationId := "W042-OBL-010",
    evidenceKind := W042ProofModelEvidenceKind.leanInventory,
    dispositionKind := W042ProofModelDispositionKind.checkedLeanEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanTlaPredecessorBridgeRow : W042ProofModelRow :=
  { rowId := "w042.lean-tla-predecessor-bridge",
    obligationId := "W042-OBL-010",
    evidenceKind := W042ProofModelEvidenceKind.leanTlaPredecessorBridge,
    dispositionKind := W042ProofModelDispositionKind.checkedLeanBridgeEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanRustDynamicRefinementBridgeRow : W042ProofModelRow :=
  { rowId := "w042.lean-rust-dynamic-refinement-bridge",
    obligationId := "W042-OBL-009",
    evidenceKind := W042ProofModelEvidenceKind.leanRustDynamicRefinementBridge,
    dispositionKind := W042ProofModelDispositionKind.checkedLeanRefinementBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanCallableCarrierBridgeRow : W042ProofModelRow :=
  { rowId := "w042.lean-callable-carrier-boundary-bridge",
    obligationId := "W042-OBL-012",
    evidenceKind := W042ProofModelEvidenceKind.callableCarrierBridge,
    dispositionKind := W042ProofModelDispositionKind.checkedLeanCallableCarrierBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanStage2AnalyzerPackPredicateRow : W042ProofModelRow :=
  { rowId := "w042.lean-stage2-analyzer-pack-predicate-carried",
    obligationId := "W042-OBL-011",
    evidenceKind := W042ProofModelEvidenceKind.leanStage2AnalyzerPackPredicate,
    dispositionKind := W042ProofModelDispositionKind.checkedLeanPolicyPredicate,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaRoutineConfigBoundedModelBoundaryRow : W042ProofModelRow :=
  { rowId := "w042.tla-routine-config-bounded-model-boundary",
    obligationId := "W042-OBL-011",
    evidenceKind := W042ProofModelEvidenceKind.tlaRoutineBoundedModel,
    dispositionKind := W042ProofModelDispositionKind.boundedModelBoundary,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def TlaStage2PartitionBoundedModelRow : W042ProofModelRow :=
  { rowId := "w042.tla-stage2-partition-bounded-model-evidence",
    obligationId := "W042-OBL-011",
    evidenceKind := W042ProofModelEvidenceKind.tlaStage2PartitionModel,
    dispositionKind := W042ProofModelDispositionKind.boundedStage2ModelEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def Stage2EquivalenceBoundedModelInputRow : W042ProofModelRow :=
  { rowId := "w042.stage2-equivalence-bounded-model-input",
    obligationId := "W042-OBL-011",
    evidenceKind := W042ProofModelEvidenceKind.tlaStage2EquivalenceInput,
    dispositionKind := W042ProofModelDispositionKind.boundedStage2EquivalenceEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaFairnessSchedulerAssumptionBoundaryRow : W042ProofModelRow :=
  { rowId := "w042.tla-fairness-scheduler-unbounded-boundary",
    obligationId := "W042-OBL-011",
    evidenceKind := W042ProofModelEvidenceKind.tlaFairnessBoundary,
    dispositionKind := W042ProofModelDispositionKind.exactModelAssumptionBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullLeanVerificationBlockerRow : W042ProofModelRow :=
  { rowId := "w042.full-lean-verification-exact-blocker",
    obligationId := "W042-OBL-010",
    evidenceKind := W042ProofModelEvidenceKind.leanFullVerificationBoundary,
    dispositionKind := W042ProofModelDispositionKind.exactLeanVerificationBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullTlaVerificationBlockerRow : W042ProofModelRow :=
  { rowId := "w042.full-tla-verification-exact-blocker",
    obligationId := "W042-OBL-011",
    evidenceKind := W042ProofModelEvidenceKind.tlaFullVerificationBoundary,
    dispositionKind := W042ProofModelDispositionKind.exactTlaVerificationBlocker,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def RustTotalityDependencyBlockerRow : W042ProofModelRow :=
  { rowId := "w042.rust-totality-dependency-exact-blocker",
    obligationId := "W042-OBL-009",
    evidenceKind := W042ProofModelEvidenceKind.rustTotalityDependency,
    dispositionKind := W042ProofModelDispositionKind.exactRustDependencyBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaExternalOxFuncBoundaryRow : W042ProofModelRow :=
  { rowId := "w042.let-lambda-external-oxfunc-boundary",
    obligationId := "W042-OBL-033",
    evidenceKind := W042ProofModelEvidenceKind.externalSemanticAuthority,
    dispositionKind := W042ProofModelDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def FormalModelSpecEvolutionGuardRow : W042ProofModelRow :=
  { rowId := "w042.formal-model-spec-evolution-guard",
    obligationId := "W042-OBL-010",
    evidenceKind := W042ProofModelEvidenceKind.specEvolutionGuard,
    dispositionKind := W042ProofModelDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem dynamicRefinementBridge_isCheckedNonPromoting :
    IsNonPromoting LeanRustDynamicRefinementBridgeRow = true
      /\ LeanRustDynamicRefinementBridgeRow.exactRemainingBlocker = false := by
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

theorem allW042ProofModelRows_nonPromoting :
    IsNonPromoting LeanInventoryNoPlaceholderEvidenceRow = true
      /\ IsNonPromoting LeanTlaPredecessorBridgeRow = true
      /\ IsNonPromoting LeanRustDynamicRefinementBridgeRow = true
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
    LeanRustDynamicRefinementBridgeRow,
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

structure W042LeanTlaSummary where
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
  rustEngineTotalityPromoted : Bool
  rustRefinementPromoted : Bool
  stage2PolicyPromoted : Bool
  callableCarrierSufficiencyPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  deriving DecidableEq, Repr

def W042LeanTlaSummaryValue : W042LeanTlaSummary :=
  { proofModelRows := 14,
    localProofRows := 8,
    boundedModelRows := 4,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 2,
    totalityBoundaryRows := 5,
    exactRemainingBlockerRows := 5,
    dynamicRefinementBridgeRows := 1,
    callableCarrierBridgeRows := 1,
    fullLeanVerificationPromoted := false,
    fullTlaVerificationPromoted := false,
    rustEngineTotalityPromoted := false,
    rustRefinementPromoted := false,
    stage2PolicyPromoted := false,
    callableCarrierSufficiencyPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w042LeanTlaSummary_hasFourteenRows :
    W042LeanTlaSummaryValue.proofModelRows = 14 := by
  rfl

theorem w042LeanTlaSummary_hasFiveExactBlockers :
    W042LeanTlaSummaryValue.exactRemainingBlockerRows = 5 := by
  rfl

theorem w042LeanTlaSummary_hasOneDynamicRefinementBridge :
    W042LeanTlaSummaryValue.dynamicRefinementBridgeRows = 1 := by
  rfl

theorem w042LeanTlaSummary_hasOneCallableCarrierBridge :
    W042LeanTlaSummaryValue.callableCarrierBridgeRows = 1 := by
  rfl

theorem w042LeanTlaSummary_noFullLeanPromotion :
    W042LeanTlaSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

theorem w042LeanTlaSummary_noFullTlaPromotion :
    W042LeanTlaSummaryValue.fullTlaVerificationPromoted = false := by
  rfl

theorem w042LeanTlaSummary_noRustTotalityPromotion :
    W042LeanTlaSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w042LeanTlaSummary_noRustRefinementPromotion :
    W042LeanTlaSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w042LeanTlaSummary_noStage2PolicyPromotion :
    W042LeanTlaSummaryValue.stage2PolicyPromoted = false := by
  rfl

theorem w042LeanTlaSummary_noCallableCarrierSufficiencyPromotion :
    W042LeanTlaSummaryValue.callableCarrierSufficiencyPromoted = false := by
  rfl

theorem w042LeanTlaSummary_noGeneralOxFuncKernelPromotion :
    W042LeanTlaSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W042.LeanTlaFairnessFullVerificationExpansion
