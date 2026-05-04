import Std

namespace OxCalc.CoreEngine.W033

abbrev NodeId := Nat
abbrev SnapshotId := Nat
abbrev EpochId := Nat
abbrev CandidateId := String
abbrev CommitId := String
abbrev RejectId := String
abbrev PublicationId := String
abbrev TraceId := String
abbrev ReplayId := String
abbrev InvocationContractRef := String

inductive RejectKind where
  | snapshotMismatch
  | artifactTokenMismatch
  | publicationFenceMismatch
  | dynamicDependencyFailure
  | capabilityMismatch
  | syntheticCycleReject
  | hostInjectedFailure
  deriving DecidableEq, Repr

inductive RuntimeEffectFamily where
  | dynamicDependency
  | capabilitySensitive
  | shapeTopology
  | executionRestriction
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
  targetSet : List NodeId
  deriving DecidableEq, Repr

structure CommitBundleFact where
  commitId : CommitId
  candidateId : CandidateId
  snapshotId : SnapshotId
  epochId : EpochId
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
  nodeId : NodeId
  family : RuntimeEffectFamily
  traceId : Option TraceId
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

def PublishedViewFromCommit (commit : CommitBundleFact) : PublishedView :=
  { viewId := commit.publishedViewId,
    sourceCandidateId := commit.candidateId,
    epochId := commit.epochId }

def CandidateIdentityIsNotPublicationIdentity (state : CoordinatorState) : Prop :=
  forall candidate published,
    state.acceptedCandidate = some candidate ->
      state.publishedView = some published ->
        candidate.candidateId != published.viewId

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

def ProtectedOverlaySafe (overlays : List OverlayFact) : Prop :=
  forall overlay,
    overlay ∈ overlays ->
      overlay.isProtected = true ->
        overlay.evictionEligible = false

def ApplyReject (state : CoordinatorState) (reject : RejectFact) : CoordinatorState :=
  { state with
    inFlightCandidate := none,
    acceptedCandidate := none,
    rejectLog := reject :: state.rejectLog }

def ApplyPublish (state : CoordinatorState) (commit : CommitBundleFact) : CoordinatorState :=
  { state with
    inFlightCandidate := none,
    acceptedCandidate := none,
    publishedView := some (PublishedViewFromCommit commit),
    commitHistory := commit :: state.commitHistory }

theorem applyReject_noPublish
    (state : CoordinatorState)
    (reject : RejectFact) :
    RejectIsNoPublish state (ApplyReject state reject) := by
  simp [RejectIsNoPublish, ApplyReject]

theorem applyPublish_atomic
    (state : CoordinatorState)
    (commit : CommitBundleFact) :
    AtomicPublication
      state
      (ApplyPublish state commit)
      commit
      (PublishedViewFromCommit commit) := by
  simp [AtomicPublication, ApplyPublish, PublishedViewFromCommit]

theorem conservativeAffectedSet_refl
    (nodes : List NodeId) :
    ConservativeAffectedSet nodes nodes := by
  intro node membership
  exact membership

theorem emptyOverlays_safe : ProtectedOverlaySafe [] := by
  intro overlay membership
  cases membership

def RequiresRuntimeVisibility (carrier : LetLambdaCarrierFact) : Prop :=
  carrier.dependencyVisible = true \/ carrier.runtimeEffectVisible = true

def CarrierHasInvocationContract (carrier : LetLambdaCarrierFact) : Prop :=
  carrier.invocationContractRef != ""

end OxCalc.CoreEngine.W033
