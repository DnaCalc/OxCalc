import Std

namespace OxCalc.CoreEngine.W034.PublicationFences

abbrev SnapshotEpoch := Nat
abbrev EpochId := Nat
abbrev CandidateId := String
abbrev CommitId := String
abbrev PublicationId := String
abbrev RejectId := String
abbrev CapabilityViewKey := String
abbrev CompatibilityBasis := String

inductive RejectKind where
  | snapshotMismatch
  | compatibilityMismatch
  | capabilityViewMismatch
  | artifactTokenMismatch
  | dynamicDependencyFailure
  deriving DecidableEq, Repr

structure CommitIntent where
  commitId : CommitId
  candidateId : CandidateId
  snapshotEpoch : SnapshotEpoch
  compatibilityBasis : CompatibilityBasis
  capabilityViewKey : CapabilityViewKey
  epochId : EpochId
  publishedViewId : PublicationId
  deriving DecidableEq, Repr

structure PublicationFence where
  snapshotEpoch : SnapshotEpoch
  compatibilityBasis : CompatibilityBasis
  capabilityViewKey : CapabilityViewKey
  deriving DecidableEq, Repr

structure PublishedView where
  viewId : PublicationId
  sourceCandidateId : CandidateId
  epochId : EpochId
  deriving DecidableEq, Repr

structure RejectFact where
  rejectId : RejectId
  candidateId : CandidateId
  rejectKind : RejectKind
  deriving DecidableEq, Repr

structure CoordinatorState where
  publishedView : Option PublishedView
  commitHistory : List CommitIntent
  rejectLog : List RejectFact
  acceptedCandidate : Option CandidateId
  deriving Repr

def FenceCompatible (commit : CommitIntent) (fence : PublicationFence) : Prop :=
  commit.snapshotEpoch = fence.snapshotEpoch
    /\ commit.compatibilityBasis = fence.compatibilityBasis
    /\ commit.capabilityViewKey = fence.capabilityViewKey

def PublishedViewFromCommit (commit : CommitIntent) : PublishedView :=
  { viewId := commit.publishedViewId,
    sourceCandidateId := commit.candidateId,
    epochId := commit.epochId }

def ApplyPublishWithFence
    (state : CoordinatorState)
    (commit : CommitIntent)
    (fence : PublicationFence)
    (_compatible : FenceCompatible commit fence) : CoordinatorState :=
  { state with
    acceptedCandidate := none,
    publishedView := some (PublishedViewFromCommit commit),
    commitHistory := commit :: state.commitHistory }

def ApplyReject (state : CoordinatorState) (reject : RejectFact) : CoordinatorState :=
  { state with
    acceptedCandidate := none,
    rejectLog := reject :: state.rejectLog }

def RejectIsNoPublish (before after : CoordinatorState) : Prop :=
  after.publishedView = before.publishedView
    /\ after.commitHistory = before.commitHistory

def AtomicPublication
    (before after : CoordinatorState)
    (commit : CommitIntent)
    (published : PublishedView) : Prop :=
  after.publishedView = some published
    /\ after.commitHistory = commit :: before.commitHistory
    /\ published.sourceCandidateId = commit.candidateId
    /\ published.epochId = commit.epochId

theorem snapshotEpochMismatch_cannotPublish
    (commit : CommitIntent)
    (fence : PublicationFence)
    (mismatch : commit.snapshotEpoch ≠ fence.snapshotEpoch) :
    ¬ FenceCompatible commit fence := by
  intro compatible
  exact mismatch compatible.left

theorem compatibilityBasisMismatch_cannotPublish
    (commit : CommitIntent)
    (fence : PublicationFence)
    (mismatch : commit.compatibilityBasis ≠ fence.compatibilityBasis) :
    ¬ FenceCompatible commit fence := by
  intro compatible
  exact mismatch compatible.right.left

theorem capabilityViewMismatch_cannotPublish
    (commit : CommitIntent)
    (fence : PublicationFence)
    (mismatch : commit.capabilityViewKey ≠ fence.capabilityViewKey) :
    ¬ FenceCompatible commit fence := by
  intro compatible
  exact mismatch compatible.right.right

theorem applyReject_noPublish
    (state : CoordinatorState)
    (reject : RejectFact) :
    RejectIsNoPublish state (ApplyReject state reject) := by
  simp [RejectIsNoPublish, ApplyReject]

theorem applyPublishWithFence_atomic
    (state : CoordinatorState)
    (commit : CommitIntent)
    (fence : PublicationFence)
    (compatible : FenceCompatible commit fence) :
    AtomicPublication
      state
      (ApplyPublishWithFence state commit fence compatible)
      commit
      (PublishedViewFromCommit commit) := by
  simp [AtomicPublication, ApplyPublishWithFence, PublishedViewFromCommit]

structure W034FenceEnvelope where
  snapshotFenceChecked : Bool
  compatibilityFenceChecked : Bool
  capabilityViewFenceChecked : Bool
  rejectNoPublishChecked : Bool
  atomicPublishChecked : Bool
  deriving DecidableEq, Repr

def W034FenceProofEnvelope : W034FenceEnvelope :=
  { snapshotFenceChecked := true,
    compatibilityFenceChecked := true,
    capabilityViewFenceChecked := true,
    rejectNoPublishChecked := true,
    atomicPublishChecked := true }

theorem w034FenceEnvelope_hasCapabilityViewFence :
    W034FenceProofEnvelope.capabilityViewFenceChecked = true := by
  rfl

end OxCalc.CoreEngine.W034.PublicationFences
