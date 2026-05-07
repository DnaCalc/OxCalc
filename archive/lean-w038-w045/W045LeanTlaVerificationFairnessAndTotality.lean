import Std

namespace OxCalc.CoreEngine.W045.LeanTlaVerificationFairnessAndTotality

inductive W045ProofModelEvidenceKind where
  | leanInventory
  | predecessorLeanTlaBridge
  | rustDynamicBridge
  | rustPublicationFenceBridge
  | callableCarrierBridge
  | stage2SchedulerPackPredicate
  | tlaRoutineBoundedModel
  | stage2PartitionModel
  | stage2EquivalenceInput
  | tlaFairnessBoundary
  | leanFullVerificationBoundary
  | tlaFullVerificationBoundary
  | rustTotalityDependency
  | stage2ProductionPolicyDependency
  | externalSemanticAuthority
  | specEvolutionGuard
  | w073TypedFormattingGuard
  | noProxyPromotionGuard
  deriving DecidableEq, Repr

inductive W045ProofModelDispositionKind where
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
  | exactStage2DependencyBlocker
  | acceptedExternalSeamBoundary
  | acceptedSpecEvolutionGuard
  | acceptedFormattingBoundary
  | acceptedNoProxyPromotionGuard
  deriving DecidableEq, Repr

structure W045ProofModelRow where
  rowId : String
  obligationIds : List String
  evidenceKind : W045ProofModelEvidenceKind
  dispositionKind : W045ProofModelDispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W045ProofModelRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W045ProofModelRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W045ProofModelRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsAcceptedBoundary (row : W045ProofModelRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def LeanInventoryNoPlaceholderEvidenceRow : W045ProofModelRow :=
  { rowId := "w045.lean-inventory-checked-no-placeholder-evidence",
    obligationIds := ["W045-OBL-016"],
    evidenceKind := W045ProofModelEvidenceKind.leanInventory,
    dispositionKind := W045ProofModelDispositionKind.checkedLeanEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def W044LeanTlaPredecessorBridgeRow : W045ProofModelRow :=
  { rowId := "w045.w044-lean-tla-predecessor-bridge",
    obligationIds := ["W045-OBL-016", "W045-OBL-017", "W045-OBL-018"],
    evidenceKind := W045ProofModelEvidenceKind.predecessorLeanTlaBridge,
    dispositionKind := W045ProofModelDispositionKind.checkedLeanBridgeEvidence,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanRustDynamicBridgeRow : W045ProofModelRow :=
  { rowId := "w045.lean-rust-dynamic-refinement-bridge",
    obligationIds := ["W045-OBL-014", "W045-OBL-018"],
    evidenceKind := W045ProofModelEvidenceKind.rustDynamicBridge,
    dispositionKind := W045ProofModelDispositionKind.checkedLeanRefinementBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanRustPublicationFenceBridgeRow : W045ProofModelRow :=
  { rowId := "w045.lean-rust-publication-fence-no-publish-bridge",
    obligationIds := ["W045-OBL-013", "W045-OBL-018"],
    evidenceKind := W045ProofModelEvidenceKind.rustPublicationFenceBridge,
    dispositionKind := W045ProofModelDispositionKind.checkedLeanPublicationFenceBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanCallableCarrierBridgeRow : W045ProofModelRow :=
  { rowId := "w045.lean-callable-carrier-boundary-bridge",
    obligationIds := ["W045-OBL-015", "W045-OBL-033"],
    evidenceKind := W045ProofModelEvidenceKind.callableCarrierBridge,
    dispositionKind := W045ProofModelDispositionKind.checkedLeanCallableCarrierBridge,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def LeanStage2SchedulerPackPredicateRow : W045ProofModelRow :=
  { rowId := "w045.lean-stage2-scheduler-pack-predicate-carried",
    obligationIds := ["W045-OBL-017", "W045-OBL-018"],
    evidenceKind := W045ProofModelEvidenceKind.stage2SchedulerPackPredicate,
    dispositionKind := W045ProofModelDispositionKind.checkedLeanPolicyPredicate,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaRoutineConfigBoundedModelBoundaryRow : W045ProofModelRow :=
  { rowId := "w045.tla-routine-config-bounded-model-boundary",
    obligationIds := ["W045-OBL-017"],
    evidenceKind := W045ProofModelEvidenceKind.tlaRoutineBoundedModel,
    dispositionKind := W045ProofModelDispositionKind.boundedModelBoundary,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def TlaStage2PartitionBoundedModelRow : W045ProofModelRow :=
  { rowId := "w045.tla-stage2-partition-bounded-model-evidence",
    obligationIds := ["W045-OBL-017", "W045-OBL-018"],
    evidenceKind := W045ProofModelEvidenceKind.stage2PartitionModel,
    dispositionKind := W045ProofModelDispositionKind.boundedStage2ModelEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def Stage2EquivalenceBoundedModelInputRow : W045ProofModelRow :=
  { rowId := "w045.stage2-equivalence-bounded-model-input",
    obligationIds := ["W045-OBL-017", "W045-OBL-018"],
    evidenceKind := W045ProofModelEvidenceKind.stage2EquivalenceInput,
    dispositionKind := W045ProofModelDispositionKind.boundedStage2EquivalenceEvidence,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def TlaFairnessSchedulerAssumptionBoundaryRow : W045ProofModelRow :=
  { rowId := "w045.tla-fairness-scheduler-unbounded-boundary",
    obligationIds := ["W045-OBL-017"],
    evidenceKind := W045ProofModelEvidenceKind.tlaFairnessBoundary,
    dispositionKind := W045ProofModelDispositionKind.exactModelAssumptionBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullLeanVerificationBlockerRow : W045ProofModelRow :=
  { rowId := "w045.full-lean-verification-exact-blocker",
    obligationIds := ["W045-OBL-016"],
    evidenceKind := W045ProofModelEvidenceKind.leanFullVerificationBoundary,
    dispositionKind := W045ProofModelDispositionKind.exactLeanVerificationBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullTlaVerificationBlockerRow : W045ProofModelRow :=
  { rowId := "w045.full-tla-verification-exact-blocker",
    obligationIds := ["W045-OBL-017"],
    evidenceKind := W045ProofModelEvidenceKind.tlaFullVerificationBoundary,
    dispositionKind := W045ProofModelDispositionKind.exactTlaVerificationBlocker,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def RustTotalityDependencyBlockerRow : W045ProofModelRow :=
  { rowId := "w045.rust-totality-dependency-exact-blocker",
    obligationIds := ["W045-OBL-014", "W045-OBL-018"],
    evidenceKind := W045ProofModelEvidenceKind.rustTotalityDependency,
    dispositionKind := W045ProofModelDispositionKind.exactRustDependencyBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def Stage2ProductionPolicyDependencyBlockerRow : W045ProofModelRow :=
  { rowId := "w045.stage2-production-policy-dependency-exact-blocker",
    obligationIds := ["W045-OBL-017", "W045-OBL-018", "W045-OBL-020"],
    evidenceKind := W045ProofModelEvidenceKind.stage2ProductionPolicyDependency,
    dispositionKind := W045ProofModelDispositionKind.exactStage2DependencyBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaExternalOxFuncBoundaryRow : W045ProofModelRow :=
  { rowId := "w045.let-lambda-external-oxfunc-boundary",
    obligationIds := ["W045-OBL-015", "W045-OBL-033"],
    evidenceKind := W045ProofModelEvidenceKind.externalSemanticAuthority,
    dispositionKind := W045ProofModelDispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def FormalModelSpecEvolutionGuardRow : W045ProofModelRow :=
  { rowId := "w045.formal-model-spec-evolution-guard",
    obligationIds := ["W045-OBL-002"],
    evidenceKind := W045ProofModelEvidenceKind.specEvolutionGuard,
    dispositionKind := W045ProofModelDispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def W073TypedFormattingGuardRow : W045ProofModelRow :=
  { rowId := "w045.w073-typed-formatting-lean-tla-boundary-guard",
    obligationIds := ["W045-OBL-003", "W045-OBL-031"],
    evidenceKind := W045ProofModelEvidenceKind.w073TypedFormattingGuard,
    dispositionKind := W045ProofModelDispositionKind.acceptedFormattingBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def NoProxyProofModelPromotionGuardRow : W045ProofModelRow :=
  { rowId := "w045.no-proxy-proof-model-promotion-guard",
    obligationIds := ["W045-OBL-001", "W045-OBL-002", "W045-OBL-016", "W045-OBL-017", "W045-OBL-018"],
    evidenceKind := W045ProofModelEvidenceKind.noProxyPromotionGuard,
    dispositionKind := W045ProofModelDispositionKind.acceptedNoProxyPromotionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

theorem w045RustBridgeRows_areCheckedNonPromoting :
    IsNonPromoting LeanRustDynamicBridgeRow = true
      /\ IsNonPromoting LeanRustPublicationFenceBridgeRow = true
      /\ LeanRustDynamicBridgeRow.exactRemainingBlocker = false
      /\ LeanRustPublicationFenceBridgeRow.exactRemainingBlocker = false := by
  constructor
  · rfl
  constructor
  · rfl
  constructor <;> rfl

theorem tlaRoutineBoundary_isBoundedExact :
    TlaRoutineConfigBoundedModelBoundaryRow.boundedModel = true
      /\ IsExactBlocker TlaRoutineConfigBoundedModelBoundaryRow = true
      /\ IsTotalityBoundary TlaRoutineConfigBoundedModelBoundaryRow = true := by
  constructor
  · rfl
  constructor <;> rfl

theorem proofModelExactBlockers_areTotalityBoundaries :
    IsTotalityBoundary TlaFairnessSchedulerAssumptionBoundaryRow = true
      /\ IsTotalityBoundary FullLeanVerificationBlockerRow = true
      /\ IsTotalityBoundary FullTlaVerificationBlockerRow = true
      /\ IsTotalityBoundary RustTotalityDependencyBlockerRow = true
      /\ IsTotalityBoundary Stage2ProductionPolicyDependencyBlockerRow = true := by
  constructor
  · rfl
  constructor
  · rfl
  constructor
  · rfl
  constructor <;> rfl

theorem dependencyBlockers_areExact :
    IsExactBlocker RustTotalityDependencyBlockerRow = true
      /\ IsExactBlocker Stage2ProductionPolicyDependencyBlockerRow = true := by
  constructor <;> rfl

theorem acceptedBoundaries_areNonPromoting :
    IsAcceptedBoundary LetLambdaExternalOxFuncBoundaryRow = true
      /\ IsAcceptedBoundary FormalModelSpecEvolutionGuardRow = true
      /\ IsAcceptedBoundary W073TypedFormattingGuardRow = true
      /\ IsAcceptedBoundary NoProxyProofModelPromotionGuardRow = true := by
  constructor
  · rfl
  constructor
  · rfl
  constructor <;> rfl

structure W045LeanTlaSummary where
  proofModelRows : Nat
  localProofRows : Nat
  boundedModelRows : Nat
  acceptedExternalSeamRows : Nat
  acceptedBoundaryRows : Nat
  totalityBoundaryRows : Nat
  exactRemainingBlockerRows : Nat
  dynamicRefinementBridgeRows : Nat
  publicationFenceBridgeRows : Nat
  rustDependencyBlockerRows : Nat
  stage2DependencyBlockerRows : Nat
  fullLeanVerificationPromoted : Bool
  fullTlaVerificationPromoted : Bool
  schedulerFairnessPromoted : Bool
  unboundedModelCoveragePromoted : Bool
  rustEngineTotalityPromoted : Bool
  rustRefinementPromoted : Bool
  stage2PolicyPromoted : Bool
  callableCarrierSufficiencyPromoted : Bool
  callableMetadataProjectionPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  deriving DecidableEq, Repr

def W045LeanTlaSummaryValue : W045LeanTlaSummary :=
  { proofModelRows := 18,
    localProofRows := 11,
    boundedModelRows := 4,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 4,
    totalityBoundaryRows := 6,
    exactRemainingBlockerRows := 6,
    dynamicRefinementBridgeRows := 1,
    publicationFenceBridgeRows := 1,
    rustDependencyBlockerRows := 1,
    stage2DependencyBlockerRows := 1,
    fullLeanVerificationPromoted := false,
    fullTlaVerificationPromoted := false,
    schedulerFairnessPromoted := false,
    unboundedModelCoveragePromoted := false,
    rustEngineTotalityPromoted := false,
    rustRefinementPromoted := false,
    stage2PolicyPromoted := false,
    callableCarrierSufficiencyPromoted := false,
    callableMetadataProjectionPromoted := false,
    generalOxFuncKernelPromoted := false }

theorem w045LeanTlaSummary_hasEighteenRows :
    W045LeanTlaSummaryValue.proofModelRows = 18 := by
  rfl

theorem w045LeanTlaSummary_hasSixExactBlockers :
    W045LeanTlaSummaryValue.exactRemainingBlockerRows = 6 := by
  rfl

theorem w045LeanTlaSummary_hasSixTotalityBoundaries :
    W045LeanTlaSummaryValue.totalityBoundaryRows = 6 := by
  rfl

theorem w045LeanTlaSummary_hasRustAndStage2DependencyBlockers :
    W045LeanTlaSummaryValue.rustDependencyBlockerRows = 1
      /\ W045LeanTlaSummaryValue.stage2DependencyBlockerRows = 1 := by
  constructor <;> rfl

theorem w045LeanTlaSummary_noFullLeanPromotion :
    W045LeanTlaSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

theorem w045LeanTlaSummary_noFullTlaPromotion :
    W045LeanTlaSummaryValue.fullTlaVerificationPromoted = false := by
  rfl

theorem w045LeanTlaSummary_noSchedulerFairnessPromotion :
    W045LeanTlaSummaryValue.schedulerFairnessPromoted = false := by
  rfl

theorem w045LeanTlaSummary_noUnboundedModelCoveragePromotion :
    W045LeanTlaSummaryValue.unboundedModelCoveragePromoted = false := by
  rfl

theorem w045LeanTlaSummary_noRustTotalityPromotion :
    W045LeanTlaSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w045LeanTlaSummary_noRustRefinementPromotion :
    W045LeanTlaSummaryValue.rustRefinementPromoted = false := by
  rfl

theorem w045LeanTlaSummary_noStage2PolicyPromotion :
    W045LeanTlaSummaryValue.stage2PolicyPromoted = false := by
  rfl

theorem w045LeanTlaSummary_noCallableCarrierSufficiencyPromotion :
    W045LeanTlaSummaryValue.callableCarrierSufficiencyPromoted = false := by
  rfl

theorem w045LeanTlaSummary_noCallableMetadataProjectionPromotion :
    W045LeanTlaSummaryValue.callableMetadataProjectionPromoted = false := by
  rfl

theorem w045LeanTlaSummary_noGeneralOxFuncKernelPromotion :
    W045LeanTlaSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

end OxCalc.CoreEngine.W045.LeanTlaVerificationFairnessAndTotality
