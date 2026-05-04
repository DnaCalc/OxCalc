import Std

namespace OxCalc.CoreEngine.W033Post

abbrev NodeId := Nat
abbrev SnapshotId := Nat
abbrev EpochId := Nat
abbrev CandidateId := String
abbrev CommitId := String
abbrev PublicationId := String
abbrev RejectId := String
abbrev TraceId := String
abbrev ReplayId := String
abbrev InvocationContractRef := String

inductive RejectKind where
  | snapshotMismatch
  | artifactTokenMismatch
  | publicationFenceMismatch
  | dynamicDependencyFailure
  | callableInvocationMismatch
  | capabilityMismatch
  deriving DecidableEq, Repr

inductive RuntimeEffectFamily where
  | staticDependency
  | runtimeDependency
  | callableCarrier
  | capabilitySensitive
  | overlayRetention
  deriving DecidableEq, Repr

inductive CallableOriginKind where
  | inlineLambda
  | helperBoundLambda
  | adoptedDefinedName
  | returnedLambda
  | opaqueExternal
  deriving DecidableEq, Repr

inductive CaptureMode where
  | noCapture
  | lexicalExact
  | lexicalOpaque
  deriving DecidableEq, Repr

structure CandidateFact where
  candidateId : CandidateId
  snapshotId : SnapshotId
  compatibilityBasis : String
  capabilityToken : Option String
  targetSet : List NodeId
  deriving DecidableEq, Repr

structure CommitBundleFact where
  commitId : CommitId
  candidateId : CandidateId
  snapshotId : SnapshotId
  epochId : EpochId
  compatibilityBasis : String
  capabilityToken : Option String
  publishedViewId : PublicationId
  deriving DecidableEq, Repr

structure RejectFact where
  rejectId : RejectId
  candidateId : CandidateId
  rejectKind : RejectKind
  traceId : Option TraceId
  deriving DecidableEq, Repr

structure FenceFact where
  snapshotId : SnapshotId
  compatibilityBasis : String
  capabilityToken : Option String
  deriving DecidableEq, Repr

structure RuntimeEffectFact where
  sourceNodeId : NodeId
  targetNodeId : NodeId
  family : RuntimeEffectFamily
  traceId : Option TraceId
  deriving DecidableEq, Repr

structure DependencyFacts where
  staticDeps : List NodeId
  runtimeDeps : List NodeId
  deriving DecidableEq, Repr

structure LetLambdaCarrierFact where
  callableId : String
  originKind : CallableOriginKind
  captureMode : CaptureMode
  arityShape : Nat
  invocationContractRef : InvocationContractRef
  dependencyVisible : Bool
  runtimeEffectVisible : Bool
  replayId : Option ReplayId
  deriving DecidableEq, Repr

structure OverlayFact where
  ownerNodeId : NodeId
  snapshotId : SnapshotId
  compatibilityBasis : String
  isProtected : Bool
  evictionEligible : Bool
  deriving DecidableEq, Repr

structure PublishedView where
  viewId : PublicationId
  sourceCandidateId : CandidateId
  epochId : EpochId
  deriving DecidableEq, Repr

structure CoordinatorState where
  inFlightCandidate : Option CandidateFact
  acceptedCandidate : Option CandidateFact
  publishedView : Option PublishedView
  commitHistory : List CommitBundleFact
  rejectLog : List RejectFact
  runtimeEffects : List RuntimeEffectFact
  overlays : List OverlayFact
  deriving Repr

def FenceCompatible (commit : CommitBundleFact) (fence : FenceFact) : Prop :=
  commit.snapshotId = fence.snapshotId
    /\ commit.compatibilityBasis = fence.compatibilityBasis
    /\ commit.capabilityToken = fence.capabilityToken

def PublishedViewFromCommit (commit : CommitBundleFact) : PublishedView :=
  { viewId := commit.publishedViewId,
    sourceCandidateId := commit.candidateId,
    epochId := commit.epochId }

def ApplyPublishWithFence
    (state : CoordinatorState)
    (commit : CommitBundleFact)
    (fence : FenceFact)
    (_compatible : FenceCompatible commit fence) : CoordinatorState :=
  { state with
    inFlightCandidate := none,
    acceptedCandidate := none,
    publishedView := some (PublishedViewFromCommit commit),
    commitHistory := commit :: state.commitHistory }

def ApplyReject (state : CoordinatorState) (reject : RejectFact) : CoordinatorState :=
  { state with
    inFlightCandidate := none,
    acceptedCandidate := none,
    rejectLog := reject :: state.rejectLog }

def RejectIsNoPublish (before after : CoordinatorState) : Prop :=
  after.publishedView = before.publishedView
    /\ after.commitHistory = before.commitHistory

def AtomicPublication
    (before after : CoordinatorState)
    (commit : CommitBundleFact)
    (published : PublishedView) : Prop :=
  after.publishedView = some published
    /\ after.commitHistory = commit :: before.commitHistory
    /\ published.sourceCandidateId = commit.candidateId
    /\ published.epochId = commit.epochId

def ConservativeAffectedSet (required actual : List NodeId) : Prop :=
  forall node, node ∈ required -> node ∈ actual

def DependencyClosure (facts : DependencyFacts) (extra : List NodeId) : List NodeId :=
  facts.staticDeps ++ facts.runtimeDeps ++ extra

def ProtectedOverlayRetained (before after : List OverlayFact) : Prop :=
  forall overlay,
    overlay ∈ before ->
      overlay.isProtected = true ->
        overlay ∈ after

def ProtectedOverlaySafe (overlays : List OverlayFact) : Prop :=
  forall overlay,
    overlay ∈ overlays ->
      overlay.isProtected = true ->
        overlay.evictionEligible = false

def RequiresRuntimeVisibility (carrier : LetLambdaCarrierFact) : Prop :=
  carrier.dependencyVisible = true \/ carrier.runtimeEffectVisible = true

def CarrierHasInvocationContract (carrier : LetLambdaCarrierFact) : Prop :=
  carrier.invocationContractRef != ""

def CarrierBridgeHonest (carrier : LetLambdaCarrierFact) : Prop :=
  RequiresRuntimeVisibility carrier -> CarrierHasInvocationContract carrier

structure ObservableStep where
  candidateId : CandidateId
  publicationId : Option PublicationId
  rejectKind : Option RejectKind
  traceId : Option TraceId
  deriving DecidableEq, Repr

def ReplayEquivalent (left right : List ObservableStep) : Prop :=
  left = right

def RejectStepNoPublication (step : ObservableStep) : Prop :=
  step.rejectKind.isSome = true -> step.publicationId = none

structure PostW033FormalEnvelope where
  fecBridgeChecked : Bool
  dependencyClosureChecked : Bool
  carrierVisibilityChecked : Bool
  replayEquivalenceChecked : Bool
  stage2ContentionPromoted : Bool
  deriving DecidableEq, Repr

def PostW033Envelope : PostW033FormalEnvelope :=
  { fecBridgeChecked := true,
    dependencyClosureChecked := true,
    carrierVisibilityChecked := true,
    replayEquivalenceChecked := true,
    stage2ContentionPromoted := false }

theorem snapshotMismatch_cannotPublish
    (commit : CommitBundleFact)
    (fence : FenceFact)
    (mismatch : commit.snapshotId ≠ fence.snapshotId) :
    ¬ FenceCompatible commit fence := by
  intro compatible
  exact mismatch compatible.left

theorem compatibilityMismatch_cannotPublish
    (commit : CommitBundleFact)
    (fence : FenceFact)
    (mismatch : commit.compatibilityBasis ≠ fence.compatibilityBasis) :
    ¬ FenceCompatible commit fence := by
  intro compatible
  exact mismatch compatible.right.left

theorem capabilityMismatch_cannotPublish
    (commit : CommitBundleFact)
    (fence : FenceFact)
    (mismatch : commit.capabilityToken ≠ fence.capabilityToken) :
    ¬ FenceCompatible commit fence := by
  intro compatible
  exact mismatch compatible.right.right

theorem applyPublishWithFence_atomic
    (state : CoordinatorState)
    (commit : CommitBundleFact)
    (fence : FenceFact)
    (compatible : FenceCompatible commit fence) :
    AtomicPublication
      state
      (ApplyPublishWithFence state commit fence compatible)
      commit
      (PublishedViewFromCommit commit) := by
  simp [AtomicPublication, ApplyPublishWithFence, PublishedViewFromCommit]

theorem applyReject_noPublish
    (state : CoordinatorState)
    (reject : RejectFact) :
    RejectIsNoPublish state (ApplyReject state reject) := by
  simp [RejectIsNoPublish, ApplyReject]

theorem dependencyClosure_containsStatic
    (facts : DependencyFacts)
    (extra : List NodeId) :
    ConservativeAffectedSet facts.staticDeps (DependencyClosure facts extra) := by
  intro node membership
  unfold DependencyClosure
  simp [membership]

theorem dependencyClosure_containsRuntime
    (facts : DependencyFacts)
    (extra : List NodeId) :
    ConservativeAffectedSet facts.runtimeDeps (DependencyClosure facts extra) := by
  intro node membership
  unfold DependencyClosure
  simp [membership]

theorem protectedOverlayRetained_refl
    (overlays : List OverlayFact) :
    ProtectedOverlayRetained overlays overlays := by
  intro _ membership _
  exact membership

theorem protectedOverlaySafe_nil :
    ProtectedOverlaySafe [] := by
  intro overlay membership
  cases membership

theorem visibleCarrier_requiresInvocationContract
    (carrier : LetLambdaCarrierFact)
    (honest : CarrierBridgeHonest carrier)
    (visible : RequiresRuntimeVisibility carrier) :
    CarrierHasInvocationContract carrier := by
  exact honest visible

theorem replayEquivalent_refl
    (history : List ObservableStep) :
    ReplayEquivalent history history := by
  rfl

theorem replayEquivalent_symm
    (left right : List ObservableStep)
    (equiv : ReplayEquivalent left right) :
    ReplayEquivalent right left := by
  exact Eq.symm equiv

theorem rejectStep_noPublication
    (candidateId : CandidateId)
    (rejectKind : RejectKind)
    (traceId : Option TraceId) :
    RejectStepNoPublication
      { candidateId := candidateId,
        publicationId := none,
        rejectKind := some rejectKind,
        traceId := traceId } := by
  intro _
  rfl

theorem postW033_noStage2ContentionPromotion :
    PostW033Envelope.stage2ContentionPromoted = false := by
  rfl

end OxCalc.CoreEngine.W033Post
