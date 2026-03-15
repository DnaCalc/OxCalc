import Std

namespace OxCalc.CoreEngine.Stage1

abbrev TreeNodeId := Nat
abbrev StructuralSnapshotId := Nat
abbrev EpochId := Nat
abbrev CandidateWorkId := String
abbrev PublishedViewId := String
abbrev CommitBundleId := String
abbrev PinnedReaderId := String
abbrev CompatibilityBasis := String

inductive NodeCalcState where
  | clean
  | dirtyPending
  | needed
  | evaluating
  | verifiedClean
  | publishReady
  | rejectedPendingRepair
  | cycleBlocked
  deriving DecidableEq, Repr

inductive OverlayKind where
  | invalidationExecutionState
  | dynamicDependency
  | capabilityFenceAttachment
  | observerPriorityMetadata
  deriving DecidableEq, Repr

structure StructuralSnapshot where
  snapshotId : StructuralSnapshotId
  rootNodeId : TreeNodeId
  nodes : List TreeNodeId
  deriving Repr

structure OverlayKey where
  ownerNodeId : TreeNodeId
  overlayKind : OverlayKind
  structuralSnapshotId : StructuralSnapshotId
  compatibilityBasis : CompatibilityBasis
  payloadIdentity : Option String
  deriving DecidableEq, Repr

structure OverlayEntry where
  key : OverlayKey
  isProtected : Bool
  evictionEligible : Bool
  detail : String
  deriving DecidableEq, Repr

structure RuntimeViewState where
  snapshotId : StructuralSnapshotId
  epochId : EpochId
  nodeCalcState : TreeNodeId -> NodeCalcState
  overlays : List OverlayEntry
  demandSet : List TreeNodeId

structure AcceptedCandidateResultRef where
  candidateResultId : CandidateWorkId
  structuralSnapshotId : StructuralSnapshotId
  compatibilityBasis : CompatibilityBasis
  targetSet : List TreeNodeId
  deriving DecidableEq, Repr

structure RejectDetail where
  candidateResultId : CandidateWorkId
  rejectKind : String
  detail : String
  deriving DecidableEq, Repr

structure CommitBundleRef where
  commitBundleId : CommitBundleId
  candidateResultId : CandidateWorkId
  structuralSnapshotId : StructuralSnapshotId
  publishedViewId : PublishedViewId
  deriving DecidableEq, Repr

structure PinnedView where
  readerId : PinnedReaderId
  structuralSnapshotId : StructuralSnapshotId
  publishedViewId : Option PublishedViewId
  deriving DecidableEq, Repr

structure CoordinatorState where
  publishedViewId : Option PublishedViewId
  inFlightCandidate : Option CandidateWorkId
  acceptedCandidate : Option AcceptedCandidateResultRef
  pinnedReaders : List PinnedView
  rejectLog : List RejectDetail
  publicationCounter : Nat
  deriving Repr

structure CoreState where
  structuralSnapshot : StructuralSnapshot
  runtimeView : RuntimeViewState
  coordinator : CoordinatorState
  commitHistory : List CommitBundleRef

inductive Transition where
  | t1StructuralSuccessor
  | t2RuntimeAttachmentInitialization
  | t3InvalidationMarking
  | t4CandidateWorkAdmission
  | t5AcceptedCandidateResultIntake
  | t6Reject
  | t7AcceptAndPublish
  | t8PinReader
  | t9UnpinAndEvictEligibility
  deriving DecidableEq, Repr

def CandidateIsNotPublication (state : CoreState) : Prop :=
  match state.coordinator.acceptedCandidate, state.coordinator.publishedViewId with
  | some accepted, some published => accepted.candidateResultId != published
  | _, _ => True

def RejectIsNoPublish (before after : CoreState) : Prop :=
  before.coordinator.publishedViewId = after.coordinator.publishedViewId ->
    before.commitHistory = after.commitHistory ->
    True

def PinnedReadersReferenceCurrentSnapshot (state : CoreState) : Prop :=
  state.coordinator.pinnedReaders.all (fun pinned => pinned.structuralSnapshotId == state.structuralSnapshot.snapshotId) = true

def ProtectedOverlayNotEvictionEligible (state : CoreState) : Prop :=
  state.runtimeView.overlays.all (fun overlay => !(overlay.isProtected && overlay.evictionEligible)) = true

end OxCalc.CoreEngine.Stage1
