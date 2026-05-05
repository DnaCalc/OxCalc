import Std

namespace OxCalc.CoreEngine.W038.AssumptionDischargeAndTotality

inductive EvidenceKind where
  | localLeanProof
  | boundedTlaModel
  | externalSemanticAuthority
  | replayOrPackGate
  | specEvolutionGuard
  deriving DecidableEq, Repr

inductive DispositionKind where
  | explicitTotalityBoundary
  | boundedModelBoundary
  | acceptedExternalSeamBoundary
  | exactRemainingBlocker
  | acceptedSpecEvolutionGuard
  deriving DecidableEq, Repr

structure AssumptionDischargeRow where
  rowId : String
  obligationId : String
  evidenceKind : EvidenceKind
  dispositionKind : DispositionKind
  localCheckedProof : Bool
  boundedModel : Bool
  acceptedExternalSeam : Bool
  acceptedBoundary : Bool
  totalityBoundary : Bool
  exactRemainingBlocker : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def IsNonPromoting (row : AssumptionDischargeRow) : Bool :=
  !row.promotionClaim

def IsAcceptedBoundary (row : AssumptionDischargeRow) : Bool :=
  row.acceptedBoundary && !row.promotionClaim

def IsExactBlocker (row : AssumptionDischargeRow) : Bool :=
  row.exactRemainingBlocker && !row.promotionClaim

def IsTotalityBoundary (row : AssumptionDischargeRow) : Bool :=
  row.totalityBoundary && !row.promotionClaim

def FullLeanTotalityBoundaryRow : AssumptionDischargeRow :=
  { rowId := "w038.proof.full-lean-totality-boundary",
    obligationId := "W038-OBL-008",
    evidenceKind := EvidenceKind.localLeanProof,
    dispositionKind := DispositionKind.explicitTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def FullTlaBoundedModelBoundaryRow : AssumptionDischargeRow :=
  { rowId := "w038.model.full-tla-bounded-model-boundary",
    obligationId := "W038-OBL-009",
    evidenceKind := EvidenceKind.boundedTlaModel,
    dispositionKind := DispositionKind.boundedModelBoundary,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

def GeneralOxFuncKernelExternalBoundaryRow : AssumptionDischargeRow :=
  { rowId := "w038.external.general-oxfunc-lambda-kernel",
    obligationId := "W038-OBL-010",
    evidenceKind := EvidenceKind.externalSemanticAuthority,
    dispositionKind := DispositionKind.acceptedExternalSeamBoundary,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := true,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def Stage2ReplayEquivalenceBlockerRow : AssumptionDischargeRow :=
  { rowId := "w038.model.stage2-replay-equivalence-blocker",
    obligationId := "W038-OBL-009",
    evidenceKind := EvidenceKind.replayOrPackGate,
    dispositionKind := DispositionKind.exactRemainingBlocker,
    localCheckedProof := false,
    boundedModel := true,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def PackGradeReplayBlockerRow : AssumptionDischargeRow :=
  { rowId := "w038.pack.grade-replay-blocker",
    obligationId := "W038-OBL-019",
    evidenceKind := EvidenceKind.replayOrPackGate,
    dispositionKind := DispositionKind.exactRemainingBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def C5BlockerRow : AssumptionDischargeRow :=
  { rowId := "w038.capability.c5-blocker",
    obligationId := "W038-OBL-020",
    evidenceKind := EvidenceKind.replayOrPackGate,
    dispositionKind := DispositionKind.exactRemainingBlocker,
    localCheckedProof := false,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := false,
    exactRemainingBlocker := true,
    promotionClaim := false }

def SpecEvolutionGuardRow : AssumptionDischargeRow :=
  { rowId := "w038.spec-evolution.guard",
    obligationId := "W038-OBL-008",
    evidenceKind := EvidenceKind.specEvolutionGuard,
    dispositionKind := DispositionKind.acceptedSpecEvolutionGuard,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := true,
    totalityBoundary := false,
    exactRemainingBlocker := false,
    promotionClaim := false }

def CallableMetadataProjectionBlockerRow : AssumptionDischargeRow :=
  { rowId := "w038.callable.metadata-projection-blocker",
    obligationId := "W038-OBL-007",
    evidenceKind := EvidenceKind.localLeanProof,
    dispositionKind := DispositionKind.explicitTotalityBoundary,
    localCheckedProof := true,
    boundedModel := false,
    acceptedExternalSeam := false,
    acceptedBoundary := false,
    totalityBoundary := true,
    exactRemainingBlocker := true,
    promotionClaim := false }

theorem fullLeanTotality_isExactNonPromotingBoundary :
    IsExactBlocker FullLeanTotalityBoundaryRow = true
      /\ IsTotalityBoundary FullLeanTotalityBoundaryRow = true := by
  constructor <;> rfl

theorem fullTlaBoundedModel_isBoundedAndNonPromoting :
    FullTlaBoundedModelBoundaryRow.boundedModel = true
      /\ IsExactBlocker FullTlaBoundedModelBoundaryRow = true := by
  constructor <;> rfl

theorem generalOxFuncKernel_isAcceptedExternalBoundary :
    GeneralOxFuncKernelExternalBoundaryRow.acceptedExternalSeam = true
      /\ IsAcceptedBoundary GeneralOxFuncKernelExternalBoundaryRow = true := by
  constructor <;> rfl

theorem stage2ReplayEquivalence_isExactBlocker :
    IsExactBlocker Stage2ReplayEquivalenceBlockerRow = true := by
  rfl

theorem packGradeReplay_isExactBlocker :
    IsExactBlocker PackGradeReplayBlockerRow = true := by
  rfl

theorem c5_isExactBlocker :
    IsExactBlocker C5BlockerRow = true := by
  rfl

theorem specEvolutionGuard_isAcceptedNonPromotingBoundary :
    IsAcceptedBoundary SpecEvolutionGuardRow = true := by
  rfl

theorem callableMetadataProjection_isExactTotalityBoundary :
    IsExactBlocker CallableMetadataProjectionBlockerRow = true
      /\ IsTotalityBoundary CallableMetadataProjectionBlockerRow = true := by
  constructor <;> rfl

theorem allW038AssumptionRows_nonPromoting :
    IsNonPromoting FullLeanTotalityBoundaryRow = true
      /\ IsNonPromoting FullTlaBoundedModelBoundaryRow = true
      /\ IsNonPromoting GeneralOxFuncKernelExternalBoundaryRow = true
      /\ IsNonPromoting Stage2ReplayEquivalenceBlockerRow = true
      /\ IsNonPromoting PackGradeReplayBlockerRow = true
      /\ IsNonPromoting C5BlockerRow = true
      /\ IsNonPromoting SpecEvolutionGuardRow = true
      /\ IsNonPromoting CallableMetadataProjectionBlockerRow = true := by
  simp [
    IsNonPromoting,
    FullLeanTotalityBoundaryRow,
    FullTlaBoundedModelBoundaryRow,
    GeneralOxFuncKernelExternalBoundaryRow,
    Stage2ReplayEquivalenceBlockerRow,
    PackGradeReplayBlockerRow,
    C5BlockerRow,
    SpecEvolutionGuardRow,
    CallableMetadataProjectionBlockerRow
  ]

structure W038AssumptionSummary where
  assumptionRows : Nat
  localProofRows : Nat
  boundedModelRows : Nat
  acceptedExternalSeamRows : Nat
  acceptedBoundaryRows : Nat
  totalityBoundaryRows : Nat
  exactRemainingBlockerRows : Nat
  fullLeanVerificationPromoted : Bool
  fullTlaVerificationPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  stage2PolicyPromoted : Bool
  packGradeReplayPromoted : Bool
  c5Promoted : Bool
  deriving DecidableEq, Repr

def W038AssumptionSummaryValue : W038AssumptionSummary :=
  { assumptionRows := 8,
    localProofRows := 3,
    boundedModelRows := 2,
    acceptedExternalSeamRows := 1,
    acceptedBoundaryRows := 2,
    totalityBoundaryRows := 3,
    exactRemainingBlockerRows := 6,
    fullLeanVerificationPromoted := false,
    fullTlaVerificationPromoted := false,
    generalOxFuncKernelPromoted := false,
    stage2PolicyPromoted := false,
    packGradeReplayPromoted := false,
    c5Promoted := false }

theorem w038AssumptionSummary_hasEightRows :
    W038AssumptionSummaryValue.assumptionRows = 8 := by
  rfl

theorem w038AssumptionSummary_hasSixExactBlockers :
    W038AssumptionSummaryValue.exactRemainingBlockerRows = 6 := by
  rfl

theorem w038AssumptionSummary_noFullLeanPromotion :
    W038AssumptionSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

theorem w038AssumptionSummary_noFullTlaPromotion :
    W038AssumptionSummaryValue.fullTlaVerificationPromoted = false := by
  rfl

theorem w038AssumptionSummary_noGeneralOxFuncKernelPromotion :
    W038AssumptionSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

theorem w038AssumptionSummary_noStage2PolicyPromotion :
    W038AssumptionSummaryValue.stage2PolicyPromoted = false := by
  rfl

theorem w038AssumptionSummary_noPackGradeReplayPromotion :
    W038AssumptionSummaryValue.packGradeReplayPromoted = false := by
  rfl

theorem w038AssumptionSummary_noC5Promotion :
    W038AssumptionSummaryValue.c5Promoted = false := by
  rfl

end OxCalc.CoreEngine.W038.AssumptionDischargeAndTotality
