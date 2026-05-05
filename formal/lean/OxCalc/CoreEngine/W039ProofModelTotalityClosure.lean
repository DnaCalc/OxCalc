import Std

namespace OxCalc.CoreEngine.W039.ProofModelTotalityClosure

inductive W039EvidenceKind where
  | localLeanProof
  | boundedTlaModel
  | optimizedCoreDisposition
  | externalSemanticAuthority
  | replayOrPackGate
  deriving DecidableEq, Repr

inductive W039DispositionKind where
  | explicitTotalityBoundary
  | boundedModelBoundary
  | exactOptimizedCoreBlocker
  | acceptedExternalSeamBoundary
  | exactPromotionGateBlocker
  deriving DecidableEq, Repr

structure W039ProofModelRow where
  rowId : String
  obligationId : String
  evidenceKind : W039EvidenceKind
  dispositionKind : W039DispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : W039ProofModelRow) : Bool :=
  !row.promotionClaim

def IsExactBlocker (row : W039ProofModelRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : W039ProofModelRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def IsAcceptedBoundary (row : W039ProofModelRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def LeanTotalityBoundaryRow : W039ProofModelRow :=
  { rowId := "w039.proof.lean-totality-boundary",
    obligationId := "W039-OBL-006",
    evidenceKind := W039EvidenceKind.localLeanProof,
    dispositionKind := W039DispositionKind.explicitTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def TlaBoundedModelBoundaryRow : W039ProofModelRow :=
  { rowId := "w039.model.tla-bounded-model-boundary",
    obligationId := "W039-OBL-007",
    evidenceKind := W039EvidenceKind.boundedTlaModel,
    dispositionKind := W039DispositionKind.boundedModelBoundary,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def RustRefinementBoundaryRow : W039ProofModelRow :=
  { rowId := "w039.rust-engine.refinement-boundary",
    obligationId := "W039-OBL-008",
    evidenceKind := W039EvidenceKind.optimizedCoreDisposition,
    dispositionKind := W039DispositionKind.exactOptimizedCoreBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def CallableMetadataProjectionBoundaryRow : W039ProofModelRow :=
  { rowId := "w039.callable.metadata-projection-boundary",
    obligationId := "W039-OBL-004",
    evidenceKind := W039EvidenceKind.optimizedCoreDisposition,
    dispositionKind := W039DispositionKind.exactOptimizedCoreBlocker,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def LetLambdaExternalOxFuncBoundaryRow : W039ProofModelRow :=
  { rowId := "w039.let-lambda.external-oxfunc-boundary",
    obligationId := "W039-OBL-019",
    evidenceKind := W039EvidenceKind.externalSemanticAuthority,
    dispositionKind := W039DispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def Stage2PartitionPolicyBlockerRow : W039ProofModelRow :=
  { rowId := "w039.stage2.partition-policy-blocker",
    obligationId := "W039-OBL-009",
    evidenceKind := W039EvidenceKind.boundedTlaModel,
    dispositionKind := W039DispositionKind.exactPromotionGateBlocker,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def PackC5ReleaseGateBlockerRow : W039ProofModelRow :=
  { rowId := "w039.pack-c5.release-gate-blocker",
    obligationId := "W039-OBL-020",
    evidenceKind := W039EvidenceKind.replayOrPackGate,
    dispositionKind := W039DispositionKind.exactPromotionGateBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

theorem leanTotalityBoundary_isExact :
    IsExactBlocker LeanTotalityBoundaryRow = true
      /\ IsTotalityBoundary LeanTotalityBoundaryRow = true := by
  constructor <;> rfl

theorem tlaBoundary_isBoundedExact :
    TlaBoundedModelBoundaryRow.boundedModel = true
      /\ IsExactBlocker TlaBoundedModelBoundaryRow = true := by
  constructor <;> rfl

theorem rustRefinementBoundary_isExactTotality :
    IsExactBlocker RustRefinementBoundaryRow = true
      /\ IsTotalityBoundary RustRefinementBoundaryRow = true := by
  constructor <;> rfl

theorem callableMetadataBoundary_isExactTotality :
    IsExactBlocker CallableMetadataProjectionBoundaryRow = true
      /\ IsTotalityBoundary CallableMetadataProjectionBoundaryRow = true := by
  constructor <;> rfl

theorem letLambdaExternalBoundary_isAccepted :
    LetLambdaExternalOxFuncBoundaryRow.acceptedExternalSeam = true
      /\ IsAcceptedBoundary LetLambdaExternalOxFuncBoundaryRow = true := by
  constructor <;> rfl

theorem stage2PartitionPolicy_isExactBlocker :
    IsExactBlocker Stage2PartitionPolicyBlockerRow = true := by
  rfl

theorem packC5ReleaseGate_isExactBlocker :
    IsExactBlocker PackC5ReleaseGateBlockerRow = true := by
  rfl

theorem allW039ProofModelRows_nonPromoting :
    IsNonPromoting LeanTotalityBoundaryRow = true
      /\ IsNonPromoting TlaBoundedModelBoundaryRow = true
      /\ IsNonPromoting RustRefinementBoundaryRow = true
      /\ IsNonPromoting CallableMetadataProjectionBoundaryRow = true
      /\ IsNonPromoting LetLambdaExternalOxFuncBoundaryRow = true
      /\ IsNonPromoting Stage2PartitionPolicyBlockerRow = true
      /\ IsNonPromoting PackC5ReleaseGateBlockerRow = true := by
  simp [
    IsNonPromoting,
    LeanTotalityBoundaryRow,
    TlaBoundedModelBoundaryRow,
    RustRefinementBoundaryRow,
    CallableMetadataProjectionBoundaryRow,
    LetLambdaExternalOxFuncBoundaryRow,
    Stage2PartitionPolicyBlockerRow,
    PackC5ReleaseGateBlockerRow
  ]

structure W039ProofModelSummary where
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
  packGradeReplayPromoted : Bool
  c5Promoted : Bool
  deriving DecidableEq, Repr

def W039ProofModelSummaryValue : W039ProofModelSummary :=
  { proofModelRows := 7,
    localProofRows := 3,
    boundedModelRows := 2,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 1,
    totalityBoundaryRows := 4,
    exactRemainingBlockerRows := 6,
    fullLeanVerificationPromoted := false,
    fullTlaVerificationPromoted := false,
    rustEngineTotalityPromoted := false,
    stage2PolicyPromoted := false,
    packGradeReplayPromoted := false,
    c5Promoted := false }

theorem w039ProofModelSummary_hasSevenRows :
    W039ProofModelSummaryValue.proofModelRows = 7 := by
  rfl

theorem w039ProofModelSummary_hasSixExactBlockers :
    W039ProofModelSummaryValue.exactRemainingBlockerRows = 6 := by
  rfl

theorem w039ProofModelSummary_noFullLeanPromotion :
    W039ProofModelSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

theorem w039ProofModelSummary_noFullTlaPromotion :
    W039ProofModelSummaryValue.fullTlaVerificationPromoted = false := by
  rfl

theorem w039ProofModelSummary_noRustTotalityPromotion :
    W039ProofModelSummaryValue.rustEngineTotalityPromoted = false := by
  rfl

theorem w039ProofModelSummary_noStage2PolicyPromotion :
    W039ProofModelSummaryValue.stage2PolicyPromoted = false := by
  rfl

theorem w039ProofModelSummary_noPackGradeReplayPromotion :
    W039ProofModelSummaryValue.packGradeReplayPromoted = false := by
  rfl

theorem w039ProofModelSummary_noC5Promotion :
    W039ProofModelSummaryValue.c5Promoted = false := by
  rfl

end OxCalc.CoreEngine.W039.ProofModelTotalityClosure
