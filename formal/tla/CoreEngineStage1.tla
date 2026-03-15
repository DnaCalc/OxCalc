---- MODULE CoreEngineStage1 ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS Nodes, Snapshots, CandidateIds, ReaderIds, CompatibilityBases, OverlayKeys, PublishBundleIds, RejectKinds

ASSUME /\ Nodes # {}
       /\ Snapshots # {}
       /\ CandidateIds # {}
       /\ ReaderIds # {}
       /\ CompatibilityBases # {}
       /\ OverlayKeys # {}
       /\ PublishBundleIds # {}
       /\ RejectKinds # {}

NodeStates == {
  "clean",
  "dirty_pending",
  "needed",
  "evaluating",
  "verified_clean",
  "publish_ready",
  "rejected_pending_repair",
  "cycle_blocked"
}

NullPublication == "none"
DefaultSnapshot == CHOOSE s \in Snapshots : TRUE
DefaultCompat == CHOOSE c \in CompatibilityBases : TRUE

VARIABLES
  structSnapshot,
  runtimeView,
  coordState,
  inFlight,
  acceptedCandidate,
  publishedView,
  pinnedReaders,
  overlayState,
  rejectLog,
  nodeCalcState,
  demandSet,
  evictionEligibility,
  compatBasis,
  publishHistory,
  decisionHistory,
  transitionHistory

vars == <<
  structSnapshot,
  runtimeView,
  coordState,
  inFlight,
  acceptedCandidate,
  publishedView,
  pinnedReaders,
  overlayState,
  rejectLog,
  nodeCalcState,
  demandSet,
  evictionEligibility,
  compatBasis,
  publishHistory,
  decisionHistory,
  transitionHistory
>>

NodeStateType == [Nodes -> NodeStates]
PinnedReaderType == [ReaderIds -> [active : BOOLEAN, snapshot : Snapshots, publicationId : STRING]]
CompatBasisType == [snapshot : Snapshots, basis : CompatibilityBases]

AppendTransition(label) == Append(transitionHistory, label)

AcceptedCandidateRecord(candidateId, node, compat) ==
  [
    candidateId |-> candidateId,
    node |-> node,
    compat |-> compat,
    publicationId |-> NullPublication
  ]

RejectRecord(candidateId, rejectKind) ==
  [candidateId |-> candidateId, rejectKind |-> rejectKind, publicationId |-> NullPublication]

OverlayRecord(node, key, protected, eligible, detail) ==
  [node |-> node, key |-> key, protected |-> protected, evictionEligible |-> eligible, detail |-> detail]

PublishedViewRecord(snapshot, publicationId) ==
  [snapshot |-> snapshot, publicationId |-> publicationId]

Init ==
  /\ structSnapshot = DefaultSnapshot
  /\ runtimeView = [snapshot |-> DefaultSnapshot, epoch |-> 0, staleNodes |-> {}]
  /\ coordState = [phase |-> "idle", publicationCounter |-> 0]
  /\ inFlight = {}
  /\ acceptedCandidate = {}
  /\ publishedView = PublishedViewRecord(DefaultSnapshot, NullPublication)
  /\ pinnedReaders = [r \in ReaderIds |-> [active |-> FALSE, snapshot |-> DefaultSnapshot, publicationId |-> NullPublication]]
  /\ overlayState = {}
  /\ rejectLog = <<>>
  /\ nodeCalcState = [n \in Nodes |-> "clean"]
  /\ demandSet = {}
  /\ evictionEligibility = {}
  /\ compatBasis = [snapshot |-> DefaultSnapshot, basis |-> DefaultCompat]
  /\ publishHistory = <<>>
  /\ decisionHistory = <<>>
  /\ transitionHistory = <<>>

A1MarkDirty(node) ==
  /\ node \in Nodes
  /\ nodeCalcState[node] \in {"clean", "verified_clean"}
  /\ nodeCalcState' = [nodeCalcState EXCEPT ![node] = "dirty_pending"]
  /\ overlayState' = overlayState \cup {OverlayRecord(node, CHOOSE k \in OverlayKeys : TRUE, TRUE, FALSE, "dirty_pending")}
  /\ transitionHistory' = AppendTransition("A1:MarkDirty")
  /\ UNCHANGED <<structSnapshot, runtimeView, coordState, inFlight, acceptedCandidate, publishedView, pinnedReaders, rejectLog, demandSet, evictionEligibility, compatBasis, publishHistory, decisionHistory>>

A2MarkNeeded(node) ==
  /\ node \in Nodes
  /\ nodeCalcState[node] = "dirty_pending"
  /\ nodeCalcState' = [nodeCalcState EXCEPT ![node] = "needed"]
  /\ demandSet' = demandSet \cup {node}
  /\ transitionHistory' = AppendTransition("A2:MarkNeeded")
  /\ UNCHANGED <<structSnapshot, runtimeView, coordState, inFlight, acceptedCandidate, publishedView, pinnedReaders, overlayState, rejectLog, evictionEligibility, compatBasis, publishHistory, decisionHistory>>

A3BeginEvaluate(node, candidateId, compat) ==
  /\ node \in Nodes
  /\ candidateId \in CandidateIds
  /\ compat \in CompatibilityBases
  /\ nodeCalcState[node] = "needed"
  /\ nodeCalcState' = [nodeCalcState EXCEPT ![node] = "evaluating"]
  /\ inFlight' = inFlight \cup {AcceptedCandidateRecord(candidateId, node, compat)}
  /\ compatBasis' = [snapshot |-> structSnapshot, basis |-> compat]
  /\ transitionHistory' = AppendTransition("A3:BeginEvaluate")
  /\ UNCHANGED <<structSnapshot, runtimeView, coordState, acceptedCandidate, publishedView, pinnedReaders, overlayState, rejectLog, demandSet, evictionEligibility, publishHistory, decisionHistory>>

A3bVerifyClean(node) ==
  /\ node \in Nodes
  /\ nodeCalcState[node] = "evaluating"
  /\ nodeCalcState' = [nodeCalcState EXCEPT ![node] = "verified_clean"]
  /\ demandSet' = demandSet \ {node}
  /\ decisionHistory' = Append(decisionHistory, [kind |-> "verify_clean", publicationId |-> NullPublication])
  /\ transitionHistory' = AppendTransition("A3b:VerifyClean")
  /\ UNCHANGED <<structSnapshot, runtimeView, coordState, inFlight, acceptedCandidate, publishedView, pinnedReaders, overlayState, rejectLog, evictionEligibility, compatBasis, publishHistory>>

A4AdmitCandidateWork(candidateId, node, compat) ==
  /\ node \in Nodes
  /\ candidateId \in CandidateIds
  /\ compat \in CompatibilityBases
  /\ inFlight' = inFlight \cup {AcceptedCandidateRecord(candidateId, node, compat)}
  /\ coordState' = [coordState EXCEPT !.phase = "candidate_admitted"]
  /\ compatBasis' = [snapshot |-> structSnapshot, basis |-> compat]
  /\ transitionHistory' = AppendTransition("A4:AdmitCandidateWork")
  /\ UNCHANGED <<structSnapshot, runtimeView, acceptedCandidate, publishedView, pinnedReaders, overlayState, rejectLog, nodeCalcState, demandSet, evictionEligibility, publishHistory, decisionHistory>>

A5RecordAcceptedCandidateResult(candidateId, node, compat) ==
  /\ node \in Nodes
  /\ candidateId \in CandidateIds
  /\ compat \in CompatibilityBases
  /\ \E work \in inFlight : work.candidateId = candidateId
  /\ acceptedCandidate' = acceptedCandidate \cup {AcceptedCandidateRecord(candidateId, node, compat)}
  /\ inFlight' = {work \in inFlight : work.candidateId # candidateId}
  /\ nodeCalcState' = [nodeCalcState EXCEPT ![node] = "publish_ready"]
  /\ coordState' = [coordState EXCEPT !.phase = "candidate_recorded"]
  /\ overlayState' = overlayState \cup {OverlayRecord(node, CHOOSE k \in OverlayKeys : TRUE, TRUE, FALSE, "candidate_shape_update")}
  /\ transitionHistory' = AppendTransition("A5:RecordAcceptedCandidateResult")
  /\ UNCHANGED <<structSnapshot, runtimeView, publishedView, pinnedReaders, rejectLog, demandSet, evictionEligibility, compatBasis, publishHistory, decisionHistory>>

A6RejectCandidateWork(candidateId, rejectKind) ==
  /\ candidateId \in CandidateIds
  /\ rejectKind \in RejectKinds
  /\ rejectLog' = Append(rejectLog, RejectRecord(candidateId, rejectKind))
  /\ acceptedCandidate' = {candidate \in acceptedCandidate : candidate.candidateId # candidateId}
  /\ inFlight' = {work \in inFlight : work.candidateId # candidateId}
  /\ coordState' = [coordState EXCEPT !.phase = "candidate_rejected"]
  /\ decisionHistory' = Append(decisionHistory, [kind |-> "reject", publicationId |-> NullPublication])
  /\ transitionHistory' = AppendTransition("A6:RejectCandidateWork")
  /\ UNCHANGED <<structSnapshot, runtimeView, publishedView, pinnedReaders, overlayState, nodeCalcState, demandSet, evictionEligibility, compatBasis, publishHistory>>

A7AcceptAndPublish(candidateId, publicationId) ==
  /\ candidateId \in CandidateIds
  /\ publicationId \in PublishBundleIds
  /\ \E candidate \in acceptedCandidate : candidate.candidateId = candidateId
  /\ publishedView' = PublishedViewRecord(structSnapshot, publicationId)
  /\ publishHistory' = Append(publishHistory, [candidateId |-> candidateId, publicationId |-> publicationId])
  /\ acceptedCandidate' = {candidate \in acceptedCandidate : candidate.candidateId # candidateId}
  /\ coordState' = [coordState EXCEPT !.phase = "publication_committed", !.publicationCounter = @ + 1]
  /\ decisionHistory' = Append(decisionHistory, [kind |-> "publish", publicationId |-> publicationId])
  /\ transitionHistory' = AppendTransition("A7:AcceptAndPublish")
  /\ UNCHANGED <<structSnapshot, runtimeView, inFlight, pinnedReaders, overlayState, rejectLog, nodeCalcState, demandSet, evictionEligibility, compatBasis>>

A8PinReader(readerId) ==
  /\ readerId \in ReaderIds
  /\ pinnedReaders' = [pinnedReaders EXCEPT ![readerId] = [active |-> TRUE, snapshot |-> structSnapshot, publicationId |-> publishedView.publicationId]]
  /\ transitionHistory' = AppendTransition("A8:PinReader")
  /\ UNCHANGED <<structSnapshot, runtimeView, coordState, inFlight, acceptedCandidate, publishedView, overlayState, rejectLog, nodeCalcState, demandSet, evictionEligibility, compatBasis, publishHistory, decisionHistory>>

A9UnpinAndReleaseProtection(readerId) ==
  /\ readerId \in ReaderIds
  /\ pinnedReaders[readerId].active
  /\ pinnedReaders' = [pinnedReaders EXCEPT ![readerId].active = FALSE]
  /\ transitionHistory' = AppendTransition("A9:UnpinAndReleaseProtection")
  /\ UNCHANGED <<structSnapshot, runtimeView, coordState, inFlight, acceptedCandidate, publishedView, overlayState, rejectLog, nodeCalcState, demandSet, evictionEligibility, compatBasis, publishHistory, decisionHistory>>

A10MarkEvictionEligible(node) ==
  /\ node \in Nodes
  /\ nodeCalcState[node] \in {"clean", "verified_clean"}
  /\ evictionEligibility' = evictionEligibility \cup {node}
  /\ overlayState' = { [entry EXCEPT !.evictionEligible = TRUE, !.protected = FALSE] : entry \in overlayState }
  /\ transitionHistory' = AppendTransition("A10:MarkEvictionEligible")
  /\ UNCHANGED <<structSnapshot, runtimeView, coordState, inFlight, acceptedCandidate, publishedView, pinnedReaders, rejectLog, nodeCalcState, demandSet, compatBasis, publishHistory, decisionHistory>>

Next ==
  \/ \E node \in Nodes : A1MarkDirty(node)
  \/ \E node \in Nodes : A2MarkNeeded(node)
  \/ \E node \in Nodes, candidateId \in CandidateIds, compat \in CompatibilityBases : A3BeginEvaluate(node, candidateId, compat)
  \/ \E node \in Nodes : A3bVerifyClean(node)
  \/ \E node \in Nodes, candidateId \in CandidateIds, compat \in CompatibilityBases : A4AdmitCandidateWork(candidateId, node, compat)
  \/ \E node \in Nodes, candidateId \in CandidateIds, compat \in CompatibilityBases : A5RecordAcceptedCandidateResult(candidateId, node, compat)
  \/ \E candidateId \in CandidateIds, rejectKind \in RejectKinds : A6RejectCandidateWork(candidateId, rejectKind)
  \/ \E candidateId \in CandidateIds, publicationId \in PublishBundleIds : A7AcceptAndPublish(candidateId, publicationId)
  \/ \E readerId \in ReaderIds : A8PinReader(readerId)
  \/ \E readerId \in ReaderIds : A9UnpinAndReleaseProtection(readerId)
  \/ \E node \in Nodes : A10MarkEvictionEligible(node)

TypeInvariant ==
  /\ structSnapshot \in Snapshots
  /\ runtimeView.snapshot \in Snapshots
  /\ coordState.phase \in {"idle", "candidate_admitted", "candidate_recorded", "candidate_rejected", "publication_committed"}
  /\ publishedView.snapshot \in Snapshots
  /\ publishedView.publicationId \in STRING
  /\ pinnedReaders \in PinnedReaderType
  /\ nodeCalcState \in NodeStateType
  /\ demandSet \subseteq Nodes
  /\ evictionEligibility \subseteq Nodes
  /\ compatBasis \in CompatBasisType

NoTornPublication == publishedView.publicationId = NullPublication \/ publishedView.publicationId \in PublishBundleIds

RejectIsNoPublish ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind = "reject" => decisionHistory[i].publicationId = NullPublication

CandidateIsNotPublication ==
  \A candidate \in acceptedCandidate : candidate.publicationId = NullPublication

PinnedReaderStability ==
  \A readerId \in ReaderIds :
    pinnedReaders[readerId].active =>
      /\ pinnedReaders[readerId].snapshot \in Snapshots
      /\ pinnedReaders[readerId].publicationId \in STRING

OverlayEvictionSafety ==
  \A entry \in overlayState : ~(entry.protected /\ entry.evictionEligible)

Spec == Init /\ [][Next]_vars

=============================================================================
