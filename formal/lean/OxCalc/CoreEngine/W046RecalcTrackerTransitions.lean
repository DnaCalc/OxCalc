import Std

namespace OxCalc.CoreEngine.W046.RecalcTrackerTransitions

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

inductive RecalcAction where
  | markDirty
  | markNeeded
  | recordCycleBlockedFromClosure
  | beginEvaluate
  | verifyClean
  | produceCandidateResult
  | produceDependencyShapeUpdate
  | rejectOrFallback
  | reenterRejectedPendingRepair
  | trackerPublishAndClear
  | releaseAndEvictEligible
  | admitCandidateWork
  | recordAcceptedCandidateResult
  | rejectCandidateWork
  | acceptAndPublish
  deriving DecidableEq, Repr

structure EngineRecalcState where
  nodeState : NodeCalcState
  demandPresent : Bool
  executionOverlayProtected : Bool
  executionOverlayEvictionEligible : Bool
  dynamicDependencyOverlayPresent : Bool
  capabilityFenceAttached : Bool
  candidatePayloadAttached : Bool
  inFlightCandidate : Bool
  acceptedCandidate : Bool
  publishedVersion : Nat
  publicationCount : Nat
  rejectCount : Nat
  deriving DecidableEq, Repr

def PreservesPublication (before after : EngineRecalcState) : Prop :=
  after.publishedVersion = before.publishedVersion ∧
    after.publicationCount = before.publicationCount

def ActionPrecondition : RecalcAction -> EngineRecalcState -> Prop
  | RecalcAction.markDirty, _ => True
  | RecalcAction.markNeeded, before =>
      before.nodeState = NodeCalcState.dirtyPending
  | RecalcAction.recordCycleBlockedFromClosure, _ => True
  | RecalcAction.beginEvaluate, before =>
      before.nodeState = NodeCalcState.needed
  | RecalcAction.verifyClean, before =>
      before.nodeState = NodeCalcState.evaluating
  | RecalcAction.produceCandidateResult, before =>
      before.nodeState = NodeCalcState.evaluating
  | RecalcAction.produceDependencyShapeUpdate, before =>
      before.nodeState = NodeCalcState.evaluating
  | RecalcAction.rejectOrFallback, before =>
      before.nodeState = NodeCalcState.evaluating ∨
        before.nodeState = NodeCalcState.publishReady
  | RecalcAction.reenterRejectedPendingRepair, before =>
      before.nodeState = NodeCalcState.rejectedPendingRepair
  | RecalcAction.trackerPublishAndClear, before =>
      before.nodeState = NodeCalcState.publishReady
  | RecalcAction.releaseAndEvictEligible, before =>
      before.nodeState = NodeCalcState.clean ∨
        before.nodeState = NodeCalcState.verifiedClean
  | RecalcAction.admitCandidateWork, _ => True
  | RecalcAction.recordAcceptedCandidateResult, before =>
      before.inFlightCandidate = true
  | RecalcAction.rejectCandidateWork, before =>
      before.inFlightCandidate = true ∨ before.acceptedCandidate = true
  | RecalcAction.acceptAndPublish, before =>
      before.acceptedCandidate = true

def ActionPostcondition : RecalcAction -> EngineRecalcState -> EngineRecalcState -> Prop
  | RecalcAction.markDirty, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.dirtyPending,
          executionOverlayProtected := true,
          executionOverlayEvictionEligible := false }
  | RecalcAction.markNeeded, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.needed,
          demandPresent := true,
          executionOverlayProtected := true,
          executionOverlayEvictionEligible := false }
  | RecalcAction.recordCycleBlockedFromClosure, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.cycleBlocked,
          demandPresent := true,
          executionOverlayProtected := true,
          executionOverlayEvictionEligible := false }
  | RecalcAction.beginEvaluate, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.evaluating,
          executionOverlayProtected := true,
          executionOverlayEvictionEligible := false,
          capabilityFenceAttached := true }
  | RecalcAction.verifyClean, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.verifiedClean,
          demandPresent := false,
          executionOverlayProtected := true,
          executionOverlayEvictionEligible := false }
  | RecalcAction.produceCandidateResult, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.publishReady,
          executionOverlayProtected := true,
          executionOverlayEvictionEligible := false,
          candidatePayloadAttached := true }
  | RecalcAction.produceDependencyShapeUpdate, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.publishReady,
          executionOverlayProtected := true,
          executionOverlayEvictionEligible := false,
          dynamicDependencyOverlayPresent := true,
          candidatePayloadAttached := true }
  | RecalcAction.rejectOrFallback, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.rejectedPendingRepair,
          demandPresent := true,
          executionOverlayProtected := true,
          executionOverlayEvictionEligible := false,
          dynamicDependencyOverlayPresent := false }
  | RecalcAction.reenterRejectedPendingRepair, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.needed,
          executionOverlayProtected := true,
          executionOverlayEvictionEligible := false }
  | RecalcAction.trackerPublishAndClear, before, after =>
      after =
        { before with
          nodeState := NodeCalcState.clean,
          demandPresent := false,
          executionOverlayProtected := false,
          executionOverlayEvictionEligible := true }
  | RecalcAction.releaseAndEvictEligible, before, after =>
      after =
        { before with
          demandPresent := false,
          executionOverlayProtected := false,
          executionOverlayEvictionEligible := true }
  | RecalcAction.admitCandidateWork, before, after =>
      after = { before with inFlightCandidate := true }
  | RecalcAction.recordAcceptedCandidateResult, before, after =>
      after = { before with acceptedCandidate := true }
  | RecalcAction.rejectCandidateWork, before, after =>
      after =
        { before with
          inFlightCandidate := false,
          acceptedCandidate := false,
          rejectCount := before.rejectCount + 1 }
  | RecalcAction.acceptAndPublish, before, after =>
      after =
        { before with
          inFlightCandidate := false,
          acceptedCandidate := false,
          publishedVersion := before.publishedVersion + 1,
          publicationCount := before.publicationCount + 1 }

def LegalTransition
    (action : RecalcAction)
    (before after : EngineRecalcState) : Prop :=
  ActionPrecondition action before ∧ ActionPostcondition action before after

structure RecalcTransitionSemanticModel where
  transition : RecalcAction
  before : EngineRecalcState
  after : EngineRecalcState
  legal : LegalTransition transition before after

theorem markNeeded_requires_dirty
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.markNeeded before after) :
    before.nodeState = NodeCalcState.dirtyPending := by
  exact legal.left

theorem beginEvaluate_requires_needed
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.beginEvaluate before after) :
    before.nodeState = NodeCalcState.needed := by
  exact legal.left

theorem recordCycleBlockedFromClosure_is_no_publish
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.recordCycleBlockedFromClosure before after) :
    after.nodeState = NodeCalcState.cycleBlocked ∧
      after.demandPresent = true ∧
        PreservesPublication before after := by
  rcases legal with ⟨_, post⟩
  rw [post]
  unfold PreservesPublication
  simp

theorem verifyClean_clears_demand
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.verifyClean before after) :
    after.demandPresent = false := by
  rcases legal with ⟨_, post⟩
  rw [post]

theorem verifyClean_is_no_publish
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.verifyClean before after) :
    PreservesPublication before after := by
  rcases legal with ⟨_, post⟩
  rw [post]
  unfold PreservesPublication
  simp

theorem produceCandidate_is_not_publication
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.produceCandidateResult before after) :
    after.candidatePayloadAttached = true ∧ PreservesPublication before after := by
  rcases legal with ⟨_, post⟩
  rw [post]
  unfold PreservesPublication
  simp

theorem produceDependencyShapeUpdate_is_not_publication
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.produceDependencyShapeUpdate before after) :
    after.dynamicDependencyOverlayPresent = true ∧
      after.candidatePayloadAttached = true ∧
        PreservesPublication before after := by
  rcases legal with ⟨_, post⟩
  rw [post]
  unfold PreservesPublication
  simp

theorem rejectOrFallback_requires_evaluating_or_publish_ready
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.rejectOrFallback before after) :
    before.nodeState = NodeCalcState.evaluating ∨
      before.nodeState = NodeCalcState.publishReady := by
  exact legal.left

theorem rejectOrFallback_is_no_publish
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.rejectOrFallback before after) :
    after.nodeState = NodeCalcState.rejectedPendingRepair ∧
      after.demandPresent = true ∧
        after.dynamicDependencyOverlayPresent = false ∧
          PreservesPublication before after := by
  rcases legal with ⟨_, post⟩
  rw [post]
  unfold PreservesPublication
  simp

theorem trackerPublishAndClear_requires_publish_ready
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.trackerPublishAndClear before after) :
    before.nodeState = NodeCalcState.publishReady := by
  exact legal.left

theorem trackerPublishAndClear_clears_demand
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.trackerPublishAndClear before after) :
    after.nodeState = NodeCalcState.clean ∧ after.demandPresent = false := by
  rcases legal with ⟨_, post⟩
  rw [post]
  simp

theorem rejectCandidateWork_requires_candidate
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.rejectCandidateWork before after) :
    before.inFlightCandidate = true ∨ before.acceptedCandidate = true := by
  exact legal.left

theorem rejectCandidateWork_is_no_publish
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.rejectCandidateWork before after) :
    after.inFlightCandidate = false ∧
      after.acceptedCandidate = false ∧
        PreservesPublication before after := by
  rcases legal with ⟨_, post⟩
  rw [post]
  unfold PreservesPublication
  simp

theorem recordAcceptedCandidate_is_not_publication
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.recordAcceptedCandidateResult before after) :
    after.acceptedCandidate = true ∧ PreservesPublication before after := by
  rcases legal with ⟨_, post⟩
  rw [post]
  unfold PreservesPublication
  simp

theorem acceptAndPublish_requires_accepted_candidate
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.acceptAndPublish before after) :
    before.acceptedCandidate = true := by
  exact legal.left

theorem acceptAndPublish_advances_publication
    {before after : EngineRecalcState}
    (legal : LegalTransition RecalcAction.acceptAndPublish before after) :
    after.publishedVersion = before.publishedVersion + 1 ∧
      after.publicationCount = before.publicationCount + 1 ∧
        after.inFlightCandidate = false ∧
          after.acceptedCandidate = false := by
  rcases legal with ⟨_, post⟩
  rw [post]
  simp

def InitialSample : EngineRecalcState :=
  { nodeState := NodeCalcState.clean,
    demandPresent := false,
    executionOverlayProtected := false,
    executionOverlayEvictionEligible := false,
    dynamicDependencyOverlayPresent := false,
    capabilityFenceAttached := false,
    candidatePayloadAttached := false,
    inFlightCandidate := false,
    acceptedCandidate := false,
    publishedVersion := 0,
    publicationCount := 0,
    rejectCount := 0 }

def DirtySample : EngineRecalcState :=
  { InitialSample with
    nodeState := NodeCalcState.dirtyPending,
    executionOverlayProtected := true }

theorem sample_mark_needed_legal :
    LegalTransition RecalcAction.markNeeded DirtySample
      { DirtySample with
        nodeState := NodeCalcState.needed,
        demandPresent := true,
        executionOverlayProtected := true,
        executionOverlayEvictionEligible := false } := by
  unfold LegalTransition ActionPrecondition ActionPostcondition DirtySample
  simp

end OxCalc.CoreEngine.W046.RecalcTrackerTransitions
