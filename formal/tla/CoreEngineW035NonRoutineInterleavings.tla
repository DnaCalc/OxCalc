---- MODULE CoreEngineW035NonRoutineInterleavings ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS
  Nodes,
  ReaderIds,
  OverlayIds,
  PartitionIds,
  RequiredStage2Evidence,
  AvailableStage2Evidence,
  PartitionCoverageSoundInput,
  MaxTransitions

ASSUME /\ Nodes # {}
       /\ ReaderIds # {}
       /\ OverlayIds # {}
       /\ PartitionIds # {}
       /\ RequiredStage2Evidence # {}
       /\ AvailableStage2Evidence \subseteq RequiredStage2Evidence
       /\ PartitionCoverageSoundInput \in BOOLEAN
       /\ MaxTransitions \in Nat
       /\ MaxTransitions > 0

VARIABLES
  pinnedReaders,
  overlayState,
  availableEvidence,
  stage2Decision,
  stage2Promoted,
  decisionHistory,
  transitionHistory

vars == <<
  pinnedReaders,
  overlayState,
  availableEvidence,
  stage2Decision,
  stage2Promoted,
  decisionHistory,
  transitionHistory
>>

AppendTransition(label) == Append(transitionHistory, label)

NullPublication == "none"

OverlayRecord(overlayId, readerId, node, protected, evictionEligible, evicted) ==
  [
    overlayId |-> overlayId,
    readerId |-> readerId,
    node |-> node,
    protected |-> protected,
    evictionEligible |-> evictionEligible,
    evicted |-> evicted
  ]

DecisionRecord(kind, reason) ==
  [
    kind |-> kind,
    reason |-> reason,
    publicationId |-> NullPublication
  ]

OverlayFactType ==
  [overlayId : OverlayIds,
   readerId : ReaderIds,
   node : Nodes,
   protected : BOOLEAN,
   evictionEligible : BOOLEAN,
   evicted : BOOLEAN]

DecisionType ==
  [kind : {"stage2_blocked", "stage2_ready"},
   reason : {"missing_evidence", "partition_gap", "ready"},
   publicationId : {NullPublication}]

PartitionCoverageSound == PartitionCoverageSoundInput

MissingStage2Evidence == RequiredStage2Evidence \ availableEvidence

Stage2PreconditionsSatisfied ==
  /\ MissingStage2Evidence = {}
  /\ PartitionCoverageSound

Init ==
  /\ pinnedReaders = [reader \in ReaderIds |-> FALSE]
  /\ overlayState = {}
  /\ availableEvidence = {}
  /\ stage2Decision = "idle"
  /\ stage2Promoted = FALSE
  /\ decisionHistory = <<>>
  /\ transitionHistory = <<>>

A1PinReader(readerId) ==
  /\ readerId \in ReaderIds
  /\ pinnedReaders[readerId] = FALSE
  /\ pinnedReaders' = [pinnedReaders EXCEPT ![readerId] = TRUE]
  /\ transitionHistory' = AppendTransition("A1:PinReader")
  /\ UNCHANGED <<overlayState, availableEvidence, stage2Decision, stage2Promoted, decisionHistory>>

A2RetainProtectedOverlay(readerId, overlayId, node) ==
  /\ readerId \in ReaderIds
  /\ overlayId \in OverlayIds
  /\ node \in Nodes
  /\ pinnedReaders[readerId]
  /\ \A entry \in overlayState : entry.overlayId # overlayId
  /\ overlayState' =
       overlayState \cup {OverlayRecord(overlayId, readerId, node, TRUE, FALSE, FALSE)}
  /\ transitionHistory' = AppendTransition("A2:RetainProtectedOverlay")
  /\ UNCHANGED <<pinnedReaders, availableEvidence, stage2Decision, stage2Promoted, decisionHistory>>

A3ReleaseReader(readerId) ==
  /\ readerId \in ReaderIds
  /\ pinnedReaders[readerId]
  /\ pinnedReaders' = [pinnedReaders EXCEPT ![readerId] = FALSE]
  /\ overlayState' =
       { IF entry.readerId = readerId
         THEN [entry EXCEPT !.protected = FALSE, !.evictionEligible = TRUE]
         ELSE entry : entry \in overlayState }
  /\ transitionHistory' = AppendTransition("A3:ReleaseReader")
  /\ UNCHANGED <<availableEvidence, stage2Decision, stage2Promoted, decisionHistory>>

A4EvictEligibleOverlay(overlayId) ==
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
  /\ transitionHistory' = AppendTransition("A4:EvictEligibleOverlay")
  /\ UNCHANGED <<pinnedReaders, availableEvidence, stage2Decision, stage2Promoted, decisionHistory>>

A5RecordAvailableStage2Evidence(evidenceKind) ==
  /\ evidenceKind \in AvailableStage2Evidence
  /\ evidenceKind \notin availableEvidence
  /\ availableEvidence' = availableEvidence \cup {evidenceKind}
  /\ transitionHistory' = AppendTransition("A5:RecordAvailableStage2Evidence")
  /\ UNCHANGED <<pinnedReaders, overlayState, stage2Decision, stage2Promoted, decisionHistory>>

A6AttemptStage2BeforePreconditions ==
  /\ stage2Decision = "idle"
  /\ ~Stage2PreconditionsSatisfied
  /\ stage2Decision' =
       IF MissingStage2Evidence # {}
       THEN "blocked_missing_preconditions"
       ELSE "blocked_partition_gap"
  /\ stage2Promoted' = FALSE
  /\ decisionHistory' =
       Append(
         decisionHistory,
         IF MissingStage2Evidence # {}
         THEN DecisionRecord("stage2_blocked", "missing_evidence")
         ELSE DecisionRecord("stage2_blocked", "partition_gap"))
  /\ transitionHistory' = AppendTransition("A6:AttemptStage2BeforePreconditions")
  /\ UNCHANGED <<pinnedReaders, overlayState, availableEvidence>>

A7RecordStage2PromotionReady ==
  /\ stage2Decision = "idle"
  /\ Stage2PreconditionsSatisfied
  /\ stage2Decision' = "promotion_ready"
  /\ stage2Promoted' = TRUE
  /\ decisionHistory' = Append(decisionHistory, DecisionRecord("stage2_ready", "ready"))
  /\ transitionHistory' = AppendTransition("A7:RecordStage2PromotionReady")
  /\ UNCHANGED <<pinnedReaders, overlayState, availableEvidence>>

Next ==
  \/ \E readerId \in ReaderIds : A1PinReader(readerId)
  \/ \E readerId \in ReaderIds, overlayId \in OverlayIds, node \in Nodes :
       A2RetainProtectedOverlay(readerId, overlayId, node)
  \/ \E readerId \in ReaderIds : A3ReleaseReader(readerId)
  \/ \E overlayId \in OverlayIds : A4EvictEligibleOverlay(overlayId)
  \/ \E evidenceKind \in AvailableStage2Evidence : A5RecordAvailableStage2Evidence(evidenceKind)
  \/ A6AttemptStage2BeforePreconditions
  \/ A7RecordStage2PromotionReady

TypeInvariant ==
  /\ pinnedReaders \in [ReaderIds -> BOOLEAN]
  /\ overlayState \subseteq OverlayFactType
  /\ availableEvidence \subseteq RequiredStage2Evidence
  /\ stage2Decision \in {"idle", "blocked_missing_preconditions", "blocked_partition_gap", "promotion_ready"}
  /\ stage2Promoted \in BOOLEAN
  /\ decisionHistory \in Seq(DecisionType)

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

BlockedStage2DecisionIsNoPromotion ==
  stage2Decision \in {"blocked_missing_preconditions", "blocked_partition_gap"} =>
    stage2Promoted = FALSE

Stage2PromotionRequiresPreconditions ==
  stage2Promoted => Stage2PreconditionsSatisfied

Stage2BlockedWhenEvidenceMissing ==
  MissingStage2Evidence # {} => stage2Promoted = FALSE

PromotionReadyRequiresSoundPartitions ==
  stage2Decision = "promotion_ready" => PartitionCoverageSound

ExplorationConstraint == Len(transitionHistory) <= MaxTransitions

Spec == Init /\ [][Next]_vars

=============================================================================
