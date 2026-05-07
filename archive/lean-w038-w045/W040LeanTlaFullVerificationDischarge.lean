import Std

namespace OxCalc.CoreEngine.W040.LeanTlaFullVerificationDischarge

inductive W040ProofModelEvidenceKind where
  | leanInventory
  | leanRustBridge
  | leanStage2PolicyPredicate
  | tlaRoutineBoundedModel
  | tlaStage2PartitionModel
  | tlaFairnessBoundary
  | leanFullVerificationBoundary
  | tlaFullVerificationBoundary
  | rustTotalityDependency
  | externalSemanticAuthority
  | specEvolutionGuard
  deriving DecidableEq, Repr

inductive W040ProofModelDispositionKind where
  | checkedLeanEvidence
  | checkedLeanBridgeEvidence
  | checkedLeanPolicyPredicate
  | boundedModelBoundary
  | boundedStage2ModelEvidence
  | exactModelAssumptionBoundary
  | exactLeanVerificationBlocker
  | exactTlaVerificationBlocker
  | exactRustDependencyBlocker
  | acceptedExternalSeamBoundary
  | acceptedSpecEvolutionGuard
  deriving DecidableEq, Repr

structure W040ProofModelRow where
  rowId : String
  obligationId : String
  evidenceKind : W040ProofModelEvidenceKind
  dispositionKind : W040ProofModelDispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W040ProofModelRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W040ProofModelRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W040ProofModelRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsAcceptedBoundary (row : W040ProofModelRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def LeanInventoryNoPlaceholderEvidenceRow : W040ProofModelRow :=
  { rowId := "w040.lean-inventory-checked-no-placeholder-evidence",
    obligationId := "W040-OBL-008",
    evidenceKind := W040ProofModelEvidenceKind.leanInventory,
    dispositionKind := W040ProofModelDispositionKind.checkedLeanEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanRustTotalityBridgeRow : W040ProofModelRow :=
  { rowId := "w040.lean-rust-totality-classification-bridge",
    obligationId := "W040-OBL-008",
    evidenceKind := W040ProofModelEvidenceKind.leanRustBridge,
    dispositionKind := W040ProofModelDispositionKind.checkedLeanBridgeEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanStage2PolicyPredicateRow : W040ProofModelRow :=
  { rowId := "w040.lean-stage2-policy-predicate-carried",
    obligationId := "W040-OBL-008",
    evidenceKind := W040ProofModelEvidenceKind.leanStage2PolicyPredicate,
    dispositionKind := W040ProofModelDispositionKind.checkedLeanPolicyPredicate,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaRoutineConfigBoundedModelBoundaryRow : W040ProofModelRow :=
  { rowId := "w040.tla-routine-config-bounded-model-boundary",
    obligationId := "W040-OBL-009",
    evidenceKind := W040ProofModelEvidenceKind.tlaRoutineBoundedModel,
    dispositionKind := W040ProofModelDispositionKind.boundedModelBoundary,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def TlaStage2PartitionBoundedModelRow : W040ProofModelRow :=
  { rowId := "w040.tla-stage2-partition-bounded-model-evidence",
    obligationId := "W040-OBL-009",
    evidenceKind := W040ProofModelEvidenceKind.tlaStage2PartitionModel,
    dispositionKind := W040ProofModelDispositionKind.boundedStage2ModelEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaFairnessSchedulerAssumptionBoundaryRow : W040ProofModelRow :=
  { rowId := "w040.tla-fairness-scheduler-assumption-boundary",
    obligationId := "W040-OBL-009",
    evidenceKind := W040ProofModelEvidenceKind.tlaFairnessBoundary,
    dispositionKind := W040ProofModelDispositionKind.exactModelAssumptionBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullLeanVerificationBlockerRow : W040ProofModelRow :=
  { rowId := "w040.full-lean-verification-exact-blocker",
    obligationId := "W040-OBL-008",
    evidenceKind := W040ProofModelEvidenceKind.leanFullVerificationBoundary,
    dispositionKind := W040ProofModelDispositionKind.exactLeanVerificationBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullTlaVerificationBlockerRow : W040ProofModelRow :=
  { rowId := "w040.full-tla-verification-exact-blocker",
    obligationId := "W040-OBL-009",
    evidenceKind := W040ProofModelEvidenceKind.tlaFullVerificationBoundary,
    dispositionKind := W040ProofModelDispositionKind.exactTlaVerificationBlocker,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def RustTotalityDependencyBlockerRow : W040ProofModelRow :=
  { rowId := "w040.rust-totality-dependency-exact-blocker",
    obligationId := "W040-OBL-007",
    evidenceKind := W040ProofModelEvidenceKind.rustTotalityDependency,
    dispositionKind := W040ProofModelDispositionKind.exactRustDependencyBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaExternalOxFuncBoundaryRow : W040ProofModelRow :=
  { rowId := "w040.let-lambda-external-oxfunc-boundary",
    obligationId := "W040-OBL-020",
    evidenceKind := W040ProofModelEvidenceKind.externalSemanticAuthority,
    dispositionKind := W040ProofModelDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def FormalModelSpecEvolutionGuardRow : W040ProofModelRow :=
  { rowId := "w040.formal-model-spec-evolution-guard",
    obligationId := "W040-OBL-008",
    evidenceKind := W040ProofModelEvidenceKind.specEvolutionGuard,
    dispositionKind := W040ProofModelDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem leanInventoryRow_isCheckedNonPromoting :
    IsNonPromoting LeanInventoryNoPlaceholderEvidenceRow = true
      /\ LeanInventoryNoPlaceholderEvidenceRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem leanRustBridgeRow_isCheckedNonPromoting :
    IsNonPromoting LeanRustTotalityBridgeRow = true
      /\ LeanRustTotalityBridgeRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem stage2PredicateRow_isCheckedNonPromoting :
    IsNonPromoting LeanStage2PolicyPredicateRow = true
      /\ LeanStage2PolicyPredicateRow.exactRemainingBlocker = false := by
  constructor <;> rfl

theorem tlaRoutineBoundary_isBoundedExact :
    TlaRoutineConfigBoundedModelBoundaryRow.boundedModel = true
      /\ IsExactBlocker TlaRoutineConfigBoundedModelBoundaryRow = true
      /\ IsTotalityBoundary TlaRoutineConfigBoundedModelBoundaryRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem tlaStage2PartitionRow_isBoundedNonPromoting :
    TlaStage2PartitionBoundedModelRow.boundedModel = true
      /\ TlaStage2PartitionBoundedModelRow.exactRemainingBlocker = false := by
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

theorem specEvolutionGuard_isAccepted :
    IsAcceptedBoundary FormalModelSpecEvolutionGuardRow = true := by
  rfl

theorem allW040ProofModelRows_nonPromoting :
    IsNonPromoting LeanInventoryNoPlaceholderEvidenceRow = true
      /\ IsNonPromoting LeanRustTotalityBridgeRow = true
      /\ IsNonPromoting LeanStage2PolicyPredicateRow = true
      /\ IsNonPromoting TlaRoutineConfigBoundedModelBoundaryRow = true
      /\ IsNonPromoting TlaStage2PartitionBoundedModelRow = true
      /\ IsNonPromoting TlaFairnessSchedulerAssumptionBoundaryRow = true
      /\ IsNonPromoting FullLeanVerificationBlockerRow = true
      /\ IsNonPromoting FullTlaVerificationBlockerRow = true
      /\ IsNonPromoting RustTotalityDependencyBlockerRow = true
      /\ IsNonPromoting LetLambdaExternalOxFuncBoundaryRow = true
      /\ IsNonPromoting FormalModelSpecEvolutionGuardRow = true := by
  simp [
    IsNonPromoting,
    LeanInventoryNoPlaceholderEvidenceRow,
    LeanRustTotalityBridgeRow,
    LeanStage2PolicyPredicateRow,
    TlaRoutineConfigBoundedModelBoundaryRow,
    TlaStage2PartitionBoundedModelRow,
    TlaFairnessSchedulerAssumptionBoundaryRow,
    FullLeanVerificationBlockerRow,
    FullTlaVerificationBlockerRow,
    RustTotalityDependencyBlockerRow,
    LetLambdaExternalOxFuncBoundaryRow,
    FormalModelSpecEvolutionGuardRow
  ]

structure W040LeanTlaSummary where
  proofModelRows : Nat
  localProofRows : Nat
  boundedModelRows : Nat
  acceptedExternalSeamRows : Nat
  acceptedBoundaryRows : Nat
  totalityBoundaryRows : Nat
  exactRemainingBlockerRows : Nat
  fullLeanVerificationPromoted : Bool
  fullTlaVerificationPromoted : Bool
  rustEngineTotalityPromoted : Bool
  stage2PolicyPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  deriving DecidableEq, Repr

def W040LeanTlaSummaryValue : W040LeanTlaSummary :=
  { proofModelRows := 11,
    localProofRows := 6,
    boundedModelRows := 3,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 2,
    totalityBoundaryRows := 5,
    exactRemainingBlockerRows := 5,
    fullLeanVerificationPromoted := false,
    fullTlaVerificationPromoted := false,
    rustEngineTotalityPromoted := false,
    stage2PolicyPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w040LeanTlaSummary_hasElevenRows :
    W040LeanTlaSummaryValue.proofModelRows = 11 := by
  rfl

theorem w040LeanTlaSummary_hasFiveExactBlockers :
    W040LeanTlaSummaryValue.exactRemainingBlockerRows = 5 := by
  rfl

theorem w040LeanTlaSummary_noFullLeanPromotion :
    W040LeanTlaSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

theorem w040LeanTlaSummary_noFullTlaPromotion :
    W040LeanTlaSummaryValue.fullTlaVerificationPromoted = false := by
  rfl

theorem w040LeanTlaSummary_noRustTotalityPromotion :
    W040LeanTlaSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w040LeanTlaSummary_noStage2PolicyPromotion :
    W040LeanTlaSummaryValue.stage2PolicyPromoted = false := by
  rfl

theorem w040LeanTlaSummary_noGeneralOxFuncKernelPromotion :
    W040LeanTlaSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W040.LeanTlaFullVerificationDischarge
