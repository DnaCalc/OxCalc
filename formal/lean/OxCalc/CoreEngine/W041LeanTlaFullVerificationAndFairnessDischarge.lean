import Std

namespace OxCalc.CoreEngine.W041.LeanTlaFullVerificationAndFairnessDischarge

inductive W041ProofModelEvidenceKind where
  | leanInventory
  | leanTlaPredecessorBridge
  | leanRustDynamicRefinementBridge
  | leanStage2PolicyPredicate
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

inductive W041ProofModelDispositionKind where
  | checkedLeanEvidence
  | checkedLeanBridgeEvidence
  | checkedLeanRefinementBridge
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

structure W041ProofModelRow where
  rowId : String
  obligationId : String
  evidenceKind : W041ProofModelEvidenceKind
  dispositionKind : W041ProofModelDispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W041ProofModelRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W041ProofModelRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W041ProofModelRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsAcceptedBoundary (row : W041ProofModelRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def LeanInventoryNoPlaceholderEvidenceRow : W041ProofModelRow :=
  { rowId := "w041.lean-inventory-checked-no-placeholder-evidence",
    obligationId := "W041-OBL-010",
    evidenceKind := W041ProofModelEvidenceKind.leanInventory,
    dispositionKind := W041ProofModelDispositionKind.checkedLeanEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanTlaPredecessorBridgeRow : W041ProofModelRow :=
  { rowId := "w041.lean-tla-predecessor-bridge",
    obligationId := "W041-OBL-010",
    evidenceKind := W041ProofModelEvidenceKind.leanTlaPredecessorBridge,
    dispositionKind := W041ProofModelDispositionKind.checkedLeanBridgeEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanRustDynamicRefinementBridgeRow : W041ProofModelRow :=
  { rowId := "w041.lean-rust-dynamic-refinement-bridge",
    obligationId := "W041-OBL-009",
    evidenceKind := W041ProofModelEvidenceKind.leanRustDynamicRefinementBridge,
    dispositionKind := W041ProofModelDispositionKind.checkedLeanRefinementBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanStage2PolicyPredicateRow : W041ProofModelRow :=
  { rowId := "w041.lean-stage2-policy-predicate-carried",
    obligationId := "W041-OBL-011",
    evidenceKind := W041ProofModelEvidenceKind.leanStage2PolicyPredicate,
    dispositionKind := W041ProofModelDispositionKind.checkedLeanPolicyPredicate,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaRoutineConfigBoundedModelBoundaryRow : W041ProofModelRow :=
  { rowId := "w041.tla-routine-config-bounded-model-boundary",
    obligationId := "W041-OBL-011",
    evidenceKind := W041ProofModelEvidenceKind.tlaRoutineBoundedModel,
    dispositionKind := W041ProofModelDispositionKind.boundedModelBoundary,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def TlaStage2PartitionBoundedModelRow : W041ProofModelRow :=
  { rowId := "w041.tla-stage2-partition-bounded-model-evidence",
    obligationId := "W041-OBL-011",
    evidenceKind := W041ProofModelEvidenceKind.tlaStage2PartitionModel,
    dispositionKind := W041ProofModelDispositionKind.boundedStage2ModelEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def Stage2EquivalenceBoundedModelInputRow : W041ProofModelRow :=
  { rowId := "w041.stage2-equivalence-bounded-model-input",
    obligationId := "W041-OBL-011",
    evidenceKind := W041ProofModelEvidenceKind.tlaStage2EquivalenceInput,
    dispositionKind := W041ProofModelDispositionKind.boundedStage2EquivalenceEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaFairnessSchedulerAssumptionBoundaryRow : W041ProofModelRow :=
  { rowId := "w041.tla-fairness-scheduler-assumption-boundary",
    obligationId := "W041-OBL-011",
    evidenceKind := W041ProofModelEvidenceKind.tlaFairnessBoundary,
    dispositionKind := W041ProofModelDispositionKind.exactModelAssumptionBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullLeanVerificationBlockerRow : W041ProofModelRow :=
  { rowId := "w041.full-lean-verification-exact-blocker",
    obligationId := "W041-OBL-010",
    evidenceKind := W041ProofModelEvidenceKind.leanFullVerificationBoundary,
    dispositionKind := W041ProofModelDispositionKind.exactLeanVerificationBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullTlaVerificationBlockerRow : W041ProofModelRow :=
  { rowId := "w041.full-tla-verification-exact-blocker",
    obligationId := "W041-OBL-011",
    evidenceKind := W041ProofModelEvidenceKind.tlaFullVerificationBoundary,
    dispositionKind := W041ProofModelDispositionKind.exactTlaVerificationBlocker,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def RustTotalityDependencyBlockerRow : W041ProofModelRow :=
  { rowId := "w041.rust-totality-dependency-exact-blocker",
    obligationId := "W041-OBL-009",
    evidenceKind := W041ProofModelEvidenceKind.rustTotalityDependency,
    dispositionKind := W041ProofModelDispositionKind.exactRustDependencyBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaExternalOxFuncBoundaryRow : W041ProofModelRow :=
  { rowId := "w041.let-lambda-external-oxfunc-boundary",
    obligationId := "W041-OBL-028",
    evidenceKind := W041ProofModelEvidenceKind.externalSemanticAuthority,
    dispositionKind := W041ProofModelDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def FormalModelSpecEvolutionGuardRow : W041ProofModelRow :=
  { rowId := "w041.formal-model-spec-evolution-guard",
    obligationId := "W041-OBL-010",
    evidenceKind := W041ProofModelEvidenceKind.specEvolutionGuard,
    dispositionKind := W041ProofModelDispositionKind.acceptedSpecEvolutionGuard,
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

theorem allW041ProofModelRows_nonPromoting :
    IsNonPromoting LeanInventoryNoPlaceholderEvidenceRow = true
      /\ IsNonPromoting LeanTlaPredecessorBridgeRow = true
      /\ IsNonPromoting LeanRustDynamicRefinementBridgeRow = true
      /\ IsNonPromoting LeanStage2PolicyPredicateRow = true
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
    LeanStage2PolicyPredicateRow,
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

structure W041LeanTlaSummary where
  proofModelRows : Nat
  localProofRows : Nat
  boundedModelRows : Nat
  acceptedExternalSeamRows : Nat
  acceptedBoundaryRows : Nat
  totalityBoundaryRows : Nat
  exactRemainingBlockerRows : Nat
  dynamicRefinementBridgeRows : Nat
  fullLeanVerificationPromoted : Bool
  fullTlaVerificationPromoted : Bool
  rustEngineTotalityPromoted : Bool
  rustRefinementPromoted : Bool
  stage2PolicyPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  deriving DecidableEq, Repr

def W041LeanTlaSummaryValue : W041LeanTlaSummary :=
  { proofModelRows := 13,
    localProofRows := 7,
    boundedModelRows := 4,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 2,
    totalityBoundaryRows := 5,
    exactRemainingBlockerRows := 5,
    dynamicRefinementBridgeRows := 1,
    fullLeanVerificationPromoted := false,
    fullTlaVerificationPromoted := false,
    rustEngineTotalityPromoted := false,
    rustRefinementPromoted := false,
    stage2PolicyPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w041LeanTlaSummary_hasThirteenRows :
    W041LeanTlaSummaryValue.proofModelRows = 13 := by
  rfl

theorem w041LeanTlaSummary_hasFiveExactBlockers :
    W041LeanTlaSummaryValue.exactRemainingBlockerRows = 5 := by
  rfl

theorem w041LeanTlaSummary_hasOneDynamicRefinementBridge :
    W041LeanTlaSummaryValue.dynamicRefinementBridgeRows = 1 := by
  rfl

theorem w041LeanTlaSummary_noFullLeanPromotion :
    W041LeanTlaSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

theorem w041LeanTlaSummary_noFullTlaPromotion :
    W041LeanTlaSummaryValue.fullTlaVerificationPromoted = false := by
  rfl

theorem w041LeanTlaSummary_noRustTotalityPromotion :
    W041LeanTlaSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w041LeanTlaSummary_noRustRefinementPromotion :
    W041LeanTlaSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w041LeanTlaSummary_noStage2PolicyPromotion :
    W041LeanTlaSummaryValue.stage2PolicyPromoted = false := by
  rfl

theorem w041LeanTlaSummary_noGeneralOxFuncKernelPromotion :
    W041LeanTlaSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W041.LeanTlaFullVerificationAndFairnessDischarge
