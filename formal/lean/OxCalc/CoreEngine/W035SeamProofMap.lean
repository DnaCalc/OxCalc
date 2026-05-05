import Std

namespace OxCalc.CoreEngine.W035.SeamProofMap

inductive ProofOwner where
  | oxcalc
  | oxfml
  | oxfunc
  | tlaLane
  deriving DecidableEq, Repr

inductive SeamProofStatus where
  | provedLocal
  | explicitAxiom
  | externalSeamAssumption
  | opaqueKernelBoundary
  | deferredToLaterLane
  deriving DecidableEq, Repr

structure SeamFact where
  factId : String
  obligationId : String
  owner : ProofOwner
  status : SeamProofStatus
  hasCheckedLocalProof : Bool
  deriving DecidableEq, Repr

def CanPromoteAsOxCalcProof (fact : SeamFact) : Bool :=
  match fact.owner, fact.status with
  | ProofOwner.oxcalc, SeamProofStatus.provedLocal => fact.hasCheckedLocalProof
  | _, _ => false

def RequiresExternalAuthority (fact : SeamFact) : Bool :=
  match fact.owner, fact.status with
  | ProofOwner.oxfml, SeamProofStatus.externalSeamAssumption => true
  | ProofOwner.oxfunc, SeamProofStatus.opaqueKernelBoundary => true
  | _, SeamProofStatus.explicitAxiom => true
  | _, _ => false

def RequiresFutureFormalLane (fact : SeamFact) : Bool :=
  match fact.status with
  | SeamProofStatus.deferredToLaterLane => true
  | _ => false

def SnapshotFenceRejectNoPublishFact : SeamFact :=
  { factId := "w035.snapshot-fence.reject-no-publish",
    obligationId := "W035-OBL-007",
    owner := ProofOwner.oxcalc,
    status := SeamProofStatus.provedLocal,
    hasCheckedLocalProof := true }

def DependencyNoUnderInvalidationFact : SeamFact :=
  { factId := "w035.dependency.no-under-invalidation",
    obligationId := "W035-OBL-007",
    owner := ProofOwner.oxcalc,
    status := SeamProofStatus.provedLocal,
    hasCheckedLocalProof := true }

def OverlayProtectedRetentionFact : SeamFact :=
  { factId := "w035.overlay.protected-retention",
    obligationId := "W035-OBL-007",
    owner := ProofOwner.oxcalc,
    status := SeamProofStatus.provedLocal,
    hasCheckedLocalProof := true }

def CallableCarrierIdentityFact : SeamFact :=
  { factId := "w035.callable.carrier-identity",
    obligationId := "W035-OBL-004",
    owner := ProofOwner.oxcalc,
    status := SeamProofStatus.provedLocal,
    hasCheckedLocalProof := true }

def OxFmlFenceArtifactMeaningFact : SeamFact :=
  { factId := "w035.oxfml.fence-artifact-meaning",
    obligationId := "W035-OBL-008",
    owner := ProofOwner.oxfml,
    status := SeamProofStatus.externalSeamAssumption,
    hasCheckedLocalProof := false }

def OxFmlW073TypedConditionalFormattingFact : SeamFact :=
  { factId := "w035.oxfml.w073-typed-conditional-formatting",
    obligationId := "W035-OBL-013",
    owner := ProofOwner.oxfml,
    status := SeamProofStatus.externalSeamAssumption,
    hasCheckedLocalProof := false }

def OxFuncFullCallableKernelFact : SeamFact :=
  { factId := "w035.oxfunc.full-callable-kernel",
    obligationId := "W035-OBL-004",
    owner := ProofOwner.oxfunc,
    status := SeamProofStatus.opaqueKernelBoundary,
    hasCheckedLocalProof := false }

def TlaMultiReaderOverlayInterleavingFact : SeamFact :=
  { factId := "w035.tla.overlay.multi-reader-release-order",
    obligationId := "W035-OBL-003",
    owner := ProofOwner.tlaLane,
    status := SeamProofStatus.deferredToLaterLane,
    hasCheckedLocalProof := false }

theorem snapshotFenceRejectNoPublish_promotableAsLocalProof :
    CanPromoteAsOxCalcProof SnapshotFenceRejectNoPublishFact = true := by
  rfl

theorem callableCarrierIdentity_promotableAsLocalCarrierProof :
    CanPromoteAsOxCalcProof CallableCarrierIdentityFact = true := by
  rfl

theorem oxfmlFenceArtifactMeaning_requiresExternalAuthority :
    RequiresExternalAuthority OxFmlFenceArtifactMeaningFact = true := by
  rfl

theorem w073TypedConditionalFormatting_requiresExternalAuthority :
    RequiresExternalAuthority OxFmlW073TypedConditionalFormattingFact = true := by
  rfl

theorem w073TypedConditionalFormatting_notPromotableAsOxCalcProof :
    CanPromoteAsOxCalcProof OxFmlW073TypedConditionalFormattingFact = false := by
  rfl

theorem oxfuncFullCallableKernel_requiresExternalAuthority :
    RequiresExternalAuthority OxFuncFullCallableKernelFact = true := by
  rfl

theorem oxfuncFullCallableKernel_notPromotableAsOxCalcProof :
    CanPromoteAsOxCalcProof OxFuncFullCallableKernelFact = false := by
  rfl

theorem tlaMultiReaderOverlayInterleaving_requiresFutureLane :
    RequiresFutureFormalLane TlaMultiReaderOverlayInterleavingFact = true := by
  rfl

structure W035SeamProofMapEnvelope where
  localProvedRows : Nat
  explicitAxiomRows : Nat
  externalSeamAssumptionRows : Nat
  opaqueKernelBoundaryRows : Nat
  deferredRows : Nat
  w073TypedRuleOnlyRecorded : Bool
  thresholdStringCompatibilityPromoted : Bool
  fullOxFuncKernelPromoted : Bool
  fullLeanVerificationPromoted : Bool
  deriving DecidableEq, Repr

def W035SeamProofMapEnvelopeValue : W035SeamProofMapEnvelope :=
  { localProvedRows := 4,
    explicitAxiomRows := 0,
    externalSeamAssumptionRows := 2,
    opaqueKernelBoundaryRows := 1,
    deferredRows := 1,
    w073TypedRuleOnlyRecorded := true,
    thresholdStringCompatibilityPromoted := false,
    fullOxFuncKernelPromoted := false,
    fullLeanVerificationPromoted := false }

theorem w035SeamProofMap_recordsW073TypedRuleOnly :
    W035SeamProofMapEnvelopeValue.w073TypedRuleOnlyRecorded = true := by
  rfl

theorem w035SeamProofMap_doesNotPromoteThresholdStringCompatibility :
    W035SeamProofMapEnvelopeValue.thresholdStringCompatibilityPromoted = false := by
  rfl

theorem w035SeamProofMap_doesNotPromoteFullOxFuncKernel :
    W035SeamProofMapEnvelopeValue.fullOxFuncKernelPromoted = false := by
  rfl

theorem w035SeamProofMap_noFullLeanVerificationPromotion :
    W035SeamProofMapEnvelopeValue.fullLeanVerificationPromoted = false := by
  rfl

end OxCalc.CoreEngine.W035.SeamProofMap
