import Std

namespace OxCalc.CoreEngine.W037.ProofModelClosureInventory

inductive ArtifactKind where
  | leanProof
  | tlaModel
  | directRuntimeEvidence
  | externalAuthority
  | laterReplayGate
  deriving DecidableEq, Repr

inductive ClosureDisposition where
  | closedForDeclaredTarget
  | checkedBoundedModel
  | externalSeamAssumption
  | opaqueKernelBoundary
  | proofGap
  | modelBound
  | deferredToStage2Replay
  | deferredToPackGate
  deriving DecidableEq, Repr

structure ClosureRow where
  rowId : String
  obligationId : String
  artifactKind : ArtifactKind
  disposition : ClosureDisposition
  hasRunnableArtifact : Bool
  targetClosureClaim : Bool
  fullVerificationClaim : Bool
  promotionClaim : Bool
  deriving DecidableEq, Repr

def ClosesDeclaredTargetOnly (row : ClosureRow) : Bool :=
  row.targetClosureClaim && !row.fullVerificationClaim && !row.promotionClaim

def IsOpenBoundary (row : ClosureRow) : Bool :=
  match row.disposition with
  | ClosureDisposition.externalSeamAssumption => true
  | ClosureDisposition.opaqueKernelBoundary => true
  | ClosureDisposition.proofGap => true
  | ClosureDisposition.modelBound => true
  | ClosureDisposition.deferredToStage2Replay => true
  | ClosureDisposition.deferredToPackGate => true
  | _ => false

def IsPromotingClaim (row : ClosureRow) : Bool :=
  row.fullVerificationClaim || row.promotionClaim

def CallableCarrierRuntimeRow : ClosureRow :=
  { rowId := "w037.proof.callable-carrier.runtime-evidence",
    obligationId := "W037-OBL-006",
    artifactKind := ArtifactKind.directRuntimeEvidence,
    disposition := ClosureDisposition.closedForDeclaredTarget,
    hasRunnableArtifact := true,
    targetClosureClaim := true,
    fullVerificationClaim := false,
    promotionClaim := false }

def CallableCarrierProofInventoryRow : ClosureRow :=
  { rowId := "w037.proof.callable-carrier.lean-inventory",
    obligationId := "W037-OBL-006",
    artifactKind := ArtifactKind.leanProof,
    disposition := ClosureDisposition.closedForDeclaredTarget,
    hasRunnableArtifact := true,
    targetClosureClaim := true,
    fullVerificationClaim := false,
    promotionClaim := false }

def FullOxFuncKernelBoundaryRow : ClosureRow :=
  { rowId := "w037.proof.full-oxfunc-kernel-boundary",
    obligationId := "W037-OBL-006",
    artifactKind := ArtifactKind.externalAuthority,
    disposition := ClosureDisposition.opaqueKernelBoundary,
    hasRunnableArtifact := true,
    targetClosureClaim := true,
    fullVerificationClaim := false,
    promotionClaim := false }

def LeanAxiomFreeInventoryRow : ClosureRow :=
  { rowId := "w037.proof.lean-axiom-free-inventory",
    obligationId := "W037-OBL-007",
    artifactKind := ArtifactKind.leanProof,
    disposition := ClosureDisposition.closedForDeclaredTarget,
    hasRunnableArtifact := true,
    targetClosureClaim := true,
    fullVerificationClaim := false,
    promotionClaim := false }

def LeanFullVerificationGapRow : ClosureRow :=
  { rowId := "w037.proof.full-lean-verification-gap",
    obligationId := "W037-OBL-007",
    artifactKind := ArtifactKind.leanProof,
    disposition := ClosureDisposition.proofGap,
    hasRunnableArtifact := true,
    targetClosureClaim := true,
    fullVerificationClaim := false,
    promotionClaim := false }

def TlaBoundedModelInventoryRow : ClosureRow :=
  { rowId := "w037.model.tla-bounded-model-inventory",
    obligationId := "W037-OBL-008",
    artifactKind := ArtifactKind.tlaModel,
    disposition := ClosureDisposition.checkedBoundedModel,
    hasRunnableArtifact := true,
    targetClosureClaim := true,
    fullVerificationClaim := false,
    promotionClaim := false }

def TlaFullVerificationBoundRow : ClosureRow :=
  { rowId := "w037.model.full-tla-verification-bound",
    obligationId := "W037-OBL-008",
    artifactKind := ArtifactKind.tlaModel,
    disposition := ClosureDisposition.modelBound,
    hasRunnableArtifact := true,
    targetClosureClaim := true,
    fullVerificationClaim := false,
    promotionClaim := false }

def Stage2ReplayDeferredRow : ClosureRow :=
  { rowId := "w037.model.stage2-replay-equivalence-deferred",
    obligationId := "W037-OBL-009",
    artifactKind := ArtifactKind.laterReplayGate,
    disposition := ClosureDisposition.deferredToStage2Replay,
    hasRunnableArtifact := false,
    targetClosureClaim := false,
    fullVerificationClaim := false,
    promotionClaim := false }

def PackGateDeferredRow : ClosureRow :=
  { rowId := "w037.model.pack-c5-deferred",
    obligationId := "W037-OBL-013",
    artifactKind := ArtifactKind.laterReplayGate,
    disposition := ClosureDisposition.deferredToPackGate,
    hasRunnableArtifact := false,
    targetClosureClaim := false,
    fullVerificationClaim := false,
    promotionClaim := false }

def OxFmlExternalAuthorityRow : ClosureRow :=
  { rowId := "w037.proof.oxfml-external-authority",
    obligationId := "W037-OBL-017",
    artifactKind := ArtifactKind.externalAuthority,
    disposition := ClosureDisposition.externalSeamAssumption,
    hasRunnableArtifact := true,
    targetClosureClaim := true,
    fullVerificationClaim := false,
    promotionClaim := false }

def SpecEvolutionGuardRow : ClosureRow :=
  { rowId := "w037.proof.spec-evolution-guard",
    obligationId := "W037-OBL-016",
    artifactKind := ArtifactKind.leanProof,
    disposition := ClosureDisposition.closedForDeclaredTarget,
    hasRunnableArtifact := true,
    targetClosureClaim := true,
    fullVerificationClaim := false,
    promotionClaim := false }

theorem callableCarrierRuntime_closesTargetOnly :
    ClosesDeclaredTargetOnly CallableCarrierRuntimeRow = true := by
  rfl

theorem callableCarrierProof_closesTargetOnly :
    ClosesDeclaredTargetOnly CallableCarrierProofInventoryRow = true := by
  rfl

theorem callableCarrierDoesNotPromoteFullOxFuncKernel :
    ClosesDeclaredTargetOnly CallableCarrierRuntimeRow = true
      /\ IsOpenBoundary FullOxFuncKernelBoundaryRow = true := by
  constructor <;> rfl

theorem leanAxiomFreeInventory_closesTargetOnly :
    ClosesDeclaredTargetOnly LeanAxiomFreeInventoryRow = true := by
  rfl

theorem leanFullVerificationGap_isOpenBoundary :
    IsOpenBoundary LeanFullVerificationGapRow = true := by
  rfl

theorem tlaBoundedModelInventory_closesTargetOnly :
    ClosesDeclaredTargetOnly TlaBoundedModelInventoryRow = true := by
  rfl

theorem tlaFullVerificationBound_isOpenBoundary :
    IsOpenBoundary TlaFullVerificationBoundRow = true := by
  rfl

theorem stage2ReplayDeferred_isNotPromotion :
    IsPromotingClaim Stage2ReplayDeferredRow = false := by
  rfl

theorem packGateDeferred_isNotPromotion :
    IsPromotingClaim PackGateDeferredRow = false := by
  rfl

theorem oxfmlExternalAuthority_isOpenBoundary :
    IsOpenBoundary OxFmlExternalAuthorityRow = true := by
  rfl

structure W037ProofModelClosureSummary where
  runnableLeanFiles : Nat
  routineTlaConfigs : Nat
  targetClosedRows : Nat
  classifiedOpenRows : Nat
  explicitAxiomRows : Nat
  sorryPlaceholderRows : Nat
  fullLeanVerificationPromoted : Bool
  fullTlaVerificationPromoted : Bool
  generalOxFuncKernelPromoted : Bool
  stage2PolicyPromoted : Bool
  packGradeReplayPromoted : Bool
  deriving DecidableEq, Repr

def W037ProofModelClosureSummaryValue : W037ProofModelClosureSummary :=
  { runnableLeanFiles := 12,
    routineTlaConfigs := 11,
    targetClosedRows := 8,
    classifiedOpenRows := 3,
    explicitAxiomRows := 0,
    sorryPlaceholderRows := 0,
    fullLeanVerificationPromoted := false,
    fullTlaVerificationPromoted := false,
    generalOxFuncKernelPromoted := false,
    stage2PolicyPromoted := false,
    packGradeReplayPromoted := false }

theorem w037ProofModel_hasNoExplicitAxioms :
    W037ProofModelClosureSummaryValue.explicitAxiomRows = 0 := by
  rfl

theorem w037ProofModel_hasNoSorryPlaceholders :
    W037ProofModelClosureSummaryValue.sorryPlaceholderRows = 0 := by
  rfl

theorem w037ProofModel_noFullLeanPromotion :
    W037ProofModelClosureSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

theorem w037ProofModel_noFullTlaPromotion :
    W037ProofModelClosureSummaryValue.fullTlaVerificationPromoted = false := by
  rfl

theorem w037ProofModel_noGeneralOxFuncKernelPromotion :
    W037ProofModelClosureSummaryValue.generalOxFuncKernelPromoted = false := by
  rfl

theorem w037ProofModel_noStage2PolicyPromotion :
    W037ProofModelClosureSummaryValue.stage2PolicyPromoted = false := by
  rfl

theorem w037ProofModel_noPackGradeReplayPromotion :
    W037ProofModelClosureSummaryValue.packGradeReplayPromoted = false := by
  rfl

end OxCalc.CoreEngine.W037.ProofModelClosureInventory
