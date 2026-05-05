---- MODULE CoreEngineW036Stage2Partition ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS
  Nodes,
  PartitionIds,
  PartitionA,
  PartitionB,
  PartitionANodes,
  PartitionBNodes,
  CrossPartitionDependencyInput,
  Snapshots,
  ActiveSnapshot,
  CapabilityViews,
  ActiveCapabilityView,
  CandidateIds,
  ReaderIds,
  OverlayIds,
  RequiredStage2Evidence,
  AvailableStage2Evidence,
  MaxTransitions

ASSUME /\ Nodes # {}
       /\ PartitionIds # {}
       /\ PartitionA \in PartitionIds
       /\ PartitionB \in PartitionIds
       /\ PartitionA # PartitionB
       /\ PartitionIds = {PartitionA, PartitionB}
       /\ PartitionANodes \subseteq Nodes
       /\ PartitionBNodes \subseteq Nodes
       /\ PartitionANodes # {}
       /\ PartitionBNodes # {}
       /\ PartitionANodes \cap PartitionBNodes = {}
       /\ PartitionANodes \cup PartitionBNodes = Nodes
       /\ CrossPartitionDependencyInput \in BOOLEAN
       /\ Snapshots # {}
       /\ ActiveSnapshot \in Snapshots
       /\ CapabilityViews # {}
       /\ ActiveCapabilityView \in CapabilityViews
       /\ CandidateIds # {}
       /\ ReaderIds # {}
       /\ OverlayIds # {}
       /\ RequiredStage2Evidence # {}
       /\ AvailableStage2Evidence \subseteq RequiredStage2Evidence
       /\ MaxTransitions \in Nat
       /\ MaxTransitions > 0

VARIABLES
  candidateFacts,
  acceptedCandidates,
  scheduledNodes,
  finishedNodes,
  pinnedReaders,
  overlayState,
  availableEvidence,
  stage2Decision,
  stage2PolicyPromoted,
  decisionHistory,
  transitionHistory

vars == <<
  candidateFacts,
  acceptedCandidates,
  scheduledNodes,
  finishedNodes,
  pinnedReaders,
  overlayState,
  availableEvidence,
  stage2Decision,
  stage2PolicyPromoted,
  decisionHistory,
  transitionHistory
>>

NullPublication == "none"
NullCandidate == "none"

AppendTransition(label) == Append(transitionHistory, label)

CandidateRecord(candidateId, snapshot, capabilityView) ==
  [
    candidateId |-> candidateId,
    snapshot |-> snapshot,
    capabilityView |-> capabilityView,
    publicationId |-> NullPublication
  ]

DecisionRecord(kind, reason, candidateId) ==
  [
    kind |-> kind,
    reason |-> reason,
    candidateId |-> candidateId,
    publicationId |-> NullPublication
  ]

OverlayRecord(overlayId, readerId, node, protected, evictionEligible, evicted) ==
  [
    overlayId |-> overlayId,
    readerId |-> readerId,
    node |-> node,
    protected |-> protected,
    evictionEligible |-> evictionEligible,
    evicted |-> evicted
  ]

CandidateFactType ==
  [candidateId : CandidateIds,
   snapshot : Snapshots,
   capabilityView : CapabilityViews,
   publicationId : {NullPublication}]

DecisionType ==
  [kind : {"candidate_accepted", "reject_snapshot", "reject_capability", "stage2_blocked", "bounded_scheduler_ready"},
   reason : {"fence_ok", "stale_snapshot", "capability_view_mismatch", "partition_gap", "incomplete_partition_work", "missing_replay_evidence", "bounded_ready"},
   candidateId : CandidateIds \cup {NullCandidate},
   publicationId : {NullPublication}]

OverlayFactType ==
  [overlayId : OverlayIds,
   readerId : ReaderIds,
   node : Nodes,
   protected : BOOLEAN,
   evictionEligible : BOOLEAN,
   evicted : BOOLEAN]

FenceCompatible(candidate) ==
  /\ candidate.snapshot = ActiveSnapshot
  /\ candidate.capabilityView = ActiveCapabilityView

OwnerOfNode(node) == IF node \in PartitionANodes THEN PartitionA ELSE PartitionB

OwnedNodes(partitionId) == {node \in Nodes : OwnerOfNode(node) = partitionId}

PartitionModelSound ==
  /\ PartitionA \in PartitionIds
  /\ PartitionB \in PartitionIds
  /\ PartitionA # PartitionB
  /\ PartitionIds = {PartitionA, PartitionB}
  /\ PartitionANodes \subseteq Nodes
  /\ PartitionBNodes \subseteq Nodes
  /\ PartitionANodes # {}
  /\ PartitionBNodes # {}
  /\ PartitionANodes \cap PartitionBNodes = {}
  /\ PartitionANodes \cup PartitionBNodes = Nodes
  /\ \A partitionId \in PartitionIds : OwnedNodes(partitionId) # {}

NoCrossPartitionDependencies ==
  ~CrossPartitionDependencyInput

AllPartitionsCompleted ==
  \A partitionId \in PartitionIds : OwnedNodes(partitionId) \subseteq finishedNodes

MissingStage2Evidence == RequiredStage2Evidence \ availableEvidence

ReplayEvidenceAvailable == "semantic_replay_equivalence" \in availableEvidence

SchedulerEquivalenceCriteriaVisible ==
  /\ "partition_ownership_model" \in availableEvidence
  /\ "scheduler_equivalence_criteria" \in availableEvidence

BoundedSchedulerPreconditionsSatisfied ==
  /\ PartitionModelSound
  /\ NoCrossPartitionDependencies
  /\ AllPartitionsCompleted
  /\ SchedulerEquivalenceCriteriaVisible
  /\ ReplayEvidenceAvailable

Init ==
  /\ candidateFacts = {}
  /\ acceptedCandidates = {}
  /\ scheduledNodes = {}
  /\ finishedNodes = {}
  /\ pinnedReaders = [readerId \in ReaderIds |-> FALSE]
  /\ overlayState = {}
  /\ availableEvidence = AvailableStage2Evidence
  /\ stage2Decision = "idle"
  /\ stage2PolicyPromoted = FALSE
  /\ decisionHistory = <<>>
  /\ transitionHistory = <<>>

A1ImportCandidate(candidateId, snapshot, capabilityView) ==
  /\ candidateId \in CandidateIds
  /\ snapshot \in Snapshots
  /\ capabilityView \in CapabilityViews
  /\ \A candidate \in candidateFacts : candidate.candidateId # candidateId
  /\ \A candidate \in acceptedCandidates : candidate.candidateId # candidateId
  /\ candidateFacts' =
       candidateFacts \cup {CandidateRecord(candidateId, snapshot, capabilityView)}
  /\ transitionHistory' = AppendTransition("A1:ImportCandidate")
  /\ UNCHANGED <<acceptedCandidates, scheduledNodes, finishedNodes, pinnedReaders, overlayState, availableEvidence, stage2Decision, stage2PolicyPromoted, decisionHistory>>

A2RejectStaleSnapshotCandidate(candidate) ==
  /\ candidate \in candidateFacts
  /\ candidate.snapshot # ActiveSnapshot
  /\ candidateFacts' = candidateFacts \ {candidate}
  /\ decisionHistory' =
       Append(decisionHistory, DecisionRecord("reject_snapshot", "stale_snapshot", candidate.candidateId))
  /\ transitionHistory' = AppendTransition("A2:RejectStaleSnapshotCandidate")
  /\ UNCHANGED <<acceptedCandidates, scheduledNodes, finishedNodes, pinnedReaders, overlayState, availableEvidence, stage2Decision, stage2PolicyPromoted>>

A3RejectCapabilityViewCandidate(candidate) ==
  /\ candidate \in candidateFacts
  /\ candidate.snapshot = ActiveSnapshot
  /\ candidate.capabilityView # ActiveCapabilityView
  /\ candidateFacts' = candidateFacts \ {candidate}
  /\ decisionHistory' =
       Append(decisionHistory, DecisionRecord("reject_capability", "capability_view_mismatch", candidate.candidateId))
  /\ transitionHistory' = AppendTransition("A3:RejectCapabilityViewCandidate")
  /\ UNCHANGED <<acceptedCandidates, scheduledNodes, finishedNodes, pinnedReaders, overlayState, availableEvidence, stage2Decision, stage2PolicyPromoted>>

A4AcceptFenceCompatibleCandidate(candidate) ==
  /\ candidate \in candidateFacts
  /\ FenceCompatible(candidate)
  /\ candidateFacts' = candidateFacts \ {candidate}
  /\ acceptedCandidates' = acceptedCandidates \cup {candidate}
  /\ decisionHistory' =
       Append(decisionHistory, DecisionRecord("candidate_accepted", "fence_ok", candidate.candidateId))
  /\ transitionHistory' = AppendTransition("A4:AcceptFenceCompatibleCandidate")
  /\ UNCHANGED <<scheduledNodes, finishedNodes, pinnedReaders, overlayState, availableEvidence, stage2Decision, stage2PolicyPromoted>>

A5ScheduleOwnedNode(node) ==
  /\ node \in Nodes
  /\ node \notin scheduledNodes
  /\ scheduledNodes' = scheduledNodes \cup {node}
  /\ transitionHistory' = AppendTransition("A5:ScheduleOwnedNode")
  /\ UNCHANGED <<candidateFacts, acceptedCandidates, finishedNodes, pinnedReaders, overlayState, availableEvidence, stage2Decision, stage2PolicyPromoted, decisionHistory>>

A6FinishScheduledNode(node) ==
  /\ node \in scheduledNodes
  /\ node \notin finishedNodes
  /\ finishedNodes' = finishedNodes \cup {node}
  /\ transitionHistory' = AppendTransition("A6:FinishScheduledNode")
  /\ UNCHANGED <<candidateFacts, acceptedCandidates, scheduledNodes, pinnedReaders, overlayState, availableEvidence, stage2Decision, stage2PolicyPromoted, decisionHistory>>

A7AttemptStage2BeforePreconditions ==
  /\ stage2Decision = "idle"
  /\ ~BoundedSchedulerPreconditionsSatisfied
  /\ stage2Decision' =
       IF ~PartitionModelSound \/ ~NoCrossPartitionDependencies
       THEN "blocked_partition_gap"
       ELSE IF ~AllPartitionsCompleted
            THEN "blocked_incomplete_partition_work"
            ELSE "blocked_missing_replay_evidence"
  /\ stage2PolicyPromoted' = FALSE
  /\ decisionHistory' =
       Append(
         decisionHistory,
         IF ~PartitionModelSound \/ ~NoCrossPartitionDependencies
         THEN DecisionRecord("stage2_blocked", "partition_gap", NullCandidate)
         ELSE IF ~AllPartitionsCompleted
              THEN DecisionRecord("stage2_blocked", "incomplete_partition_work", NullCandidate)
              ELSE DecisionRecord("stage2_blocked", "missing_replay_evidence", NullCandidate))
  /\ transitionHistory' = AppendTransition("A7:AttemptStage2BeforePreconditions")
  /\ UNCHANGED <<candidateFacts, acceptedCandidates, scheduledNodes, finishedNodes, pinnedReaders, overlayState, availableEvidence>>

A8RecordBoundedSchedulerReady ==
  /\ stage2Decision = "idle"
  /\ BoundedSchedulerPreconditionsSatisfied
  /\ stage2Decision' = "bounded_scheduler_ready"
  /\ stage2PolicyPromoted' = FALSE
  /\ decisionHistory' =
       Append(decisionHistory, DecisionRecord("bounded_scheduler_ready", "bounded_ready", NullCandidate))
  /\ transitionHistory' = AppendTransition("A8:RecordBoundedSchedulerReady")
  /\ UNCHANGED <<candidateFacts, acceptedCandidates, scheduledNodes, finishedNodes, pinnedReaders, overlayState, availableEvidence>>

A9PinReader(readerId) ==
  /\ readerId \in ReaderIds
  /\ pinnedReaders[readerId] = FALSE
  /\ pinnedReaders' = [pinnedReaders EXCEPT ![readerId] = TRUE]
  /\ transitionHistory' = AppendTransition("A9:PinReader")
  /\ UNCHANGED <<candidateFacts, acceptedCandidates, scheduledNodes, finishedNodes, overlayState, availableEvidence, stage2Decision, stage2PolicyPromoted, decisionHistory>>

A10RetainProtectedOverlay(readerId, overlayId, node) ==
  /\ readerId \in ReaderIds
  /\ overlayId \in OverlayIds
  /\ node \in Nodes
  /\ pinnedReaders[readerId]
  /\ \A entry \in overlayState : entry.overlayId # overlayId
  /\ overlayState' =
       overlayState \cup {OverlayRecord(overlayId, readerId, node, TRUE, FALSE, FALSE)}
  /\ transitionHistory' = AppendTransition("A10:RetainProtectedOverlay")
  /\ UNCHANGED <<candidateFacts, acceptedCandidates, scheduledNodes, finishedNodes, pinnedReaders, availableEvidence, stage2Decision, stage2PolicyPromoted, decisionHistory>>

A11ReleaseReader(readerId) ==
  /\ readerId \in ReaderIds
  /\ pinnedReaders[readerId]
  /\ pinnedReaders' = [pinnedReaders EXCEPT ![readerId] = FALSE]
  /\ overlayState' =
       { IF entry.readerId = readerId
         THEN [entry EXCEPT !.protected = FALSE, !.evictionEligible = TRUE]
         ELSE entry : entry \in overlayState }
  /\ transitionHistory' = AppendTransition("A11:ReleaseReader")
  /\ UNCHANGED <<candidateFacts, acceptedCandidates, scheduledNodes, finishedNodes, availableEvidence, stage2Decision, stage2PolicyPromoted, decisionHistory>>

A12EvictEligibleOverlay(overlayId) ==
  /\ overlayId \in OverlayIds
  /\ \E entry \in overlayState :
       /\ entry.overlayId = overlayId
       /\ entry.evictionEligible
       /\ ~entry.protected
       /\ ~entry.evicted
  /\ overlayState' =
       { IF entry.overlayId = overlayId
         THEN [entry EXCEPT !.evicted = TRUE]
         ELSE entry : entry \in overlayState }
  /\ transitionHistory' = AppendTransition("A12:EvictEligibleOverlay")
  /\ UNCHANGED <<candidateFacts, acceptedCandidates, scheduledNodes, finishedNodes, pinnedReaders, availableEvidence, stage2Decision, stage2PolicyPromoted, decisionHistory>>

Next ==
  \/ \E candidateId \in CandidateIds, snapshot \in Snapshots, capabilityView \in CapabilityViews :
       A1ImportCandidate(candidateId, snapshot, capabilityView)
  \/ \E candidate \in candidateFacts : A2RejectStaleSnapshotCandidate(candidate)
  \/ \E candidate \in candidateFacts : A3RejectCapabilityViewCandidate(candidate)
  \/ \E candidate \in candidateFacts : A4AcceptFenceCompatibleCandidate(candidate)
  \/ \E node \in Nodes : A5ScheduleOwnedNode(node)
  \/ \E node \in Nodes : A6FinishScheduledNode(node)
  \/ A7AttemptStage2BeforePreconditions
  \/ A8RecordBoundedSchedulerReady
  \/ \E readerId \in ReaderIds : A9PinReader(readerId)
  \/ \E readerId \in ReaderIds, overlayId \in OverlayIds, node \in Nodes :
       A10RetainProtectedOverlay(readerId, overlayId, node)
  \/ \E readerId \in ReaderIds : A11ReleaseReader(readerId)
  \/ \E overlayId \in OverlayIds : A12EvictEligibleOverlay(overlayId)

TypeInvariant ==
  /\ candidateFacts \subseteq CandidateFactType
  /\ acceptedCandidates \subseteq CandidateFactType
  /\ scheduledNodes \subseteq Nodes
  /\ finishedNodes \subseteq Nodes
  /\ pinnedReaders \in [ReaderIds -> BOOLEAN]
  /\ overlayState \subseteq OverlayFactType
  /\ availableEvidence \subseteq RequiredStage2Evidence
  /\ stage2Decision \in {"idle", "blocked_partition_gap", "blocked_incomplete_partition_work", "blocked_missing_replay_evidence", "bounded_scheduler_ready"}
  /\ stage2PolicyPromoted \in BOOLEAN
  /\ decisionHistory \in Seq(DecisionType)

ConcretePartitionOwnershipSound ==
  /\ PartitionModelSound
  /\ \A node \in scheduledNodes : OwnerOfNode(node) \in PartitionIds
  /\ \A node \in finishedNodes : OwnerOfNode(node) \in PartitionIds

FinishedNodesWereScheduled ==
  finishedNodes \subseteq scheduledNodes

BoundedReadyRequiresPreconditions ==
  stage2Decision = "bounded_scheduler_ready" => BoundedSchedulerPreconditionsSatisfied

BlockedStage2DecisionIsNoPromotion ==
  stage2Decision \in {"blocked_partition_gap", "blocked_incomplete_partition_work", "blocked_missing_replay_evidence"} =>
    stage2PolicyPromoted = FALSE

Stage2PolicyPromotionRequiresReplayEvidence ==
  stage2PolicyPromoted => ReplayEvidenceAvailable

NoStage2PolicyPromotion ==
  stage2PolicyPromoted = FALSE

RejectIsNoPublish ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind \in {"reject_snapshot", "reject_capability"} =>
      decisionHistory[i].publicationId = NullPublication

AcceptedCandidateRequiresFences ==
  \A candidate \in acceptedCandidates : FenceCompatible(candidate)

NoStaleSnapshotAccepted ==
  \A candidate \in acceptedCandidates : candidate.snapshot = ActiveSnapshot

NoCapabilityViewFenceAccepted ==
  \A candidate \in acceptedCandidates : candidate.capabilityView = ActiveCapabilityView

ProtectedOverlayPinnedAndRetained ==
  \A entry \in overlayState :
    entry.protected =>
      /\ pinnedReaders[entry.readerId]
      /\ ~entry.evicted
      /\ ~entry.evictionEligible

ReleasedReaderDoesNotReleaseOtherReaderProtectedOverlay ==
  \A entry \in overlayState :
    entry.protected => pinnedReaders[entry.readerId]

EvictedOverlayWasUnprotected ==
  \A entry \in overlayState : entry.evicted => ~entry.protected

ExplorationConstraint == Len(transitionHistory) <= MaxTransitions

Spec == Init /\ [][Next]_vars

=============================================================================
