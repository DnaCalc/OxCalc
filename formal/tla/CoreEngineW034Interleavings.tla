---- MODULE CoreEngineW034Interleavings ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS
  Nodes,
  Snapshots,
  CandidateIds,
  PublicationIds,
  CompatibilityBases,
  CapabilityViews,
  ReaderIds,
  OverlayIds,
  RequiredStage2Evidence,
  AvailableStage2Evidence

ASSUME /\ Nodes # {}
       /\ Snapshots # {}
       /\ CandidateIds # {}
       /\ PublicationIds # {}
       /\ CompatibilityBases # {}
       /\ CapabilityViews # {}
       /\ ReaderIds # {}
       /\ OverlayIds # {}
       /\ RequiredStage2Evidence # {}
       /\ AvailableStage2Evidence \subseteq RequiredStage2Evidence
       /\ RequiredStage2Evidence \ AvailableStage2Evidence # {}

NullPublication == "none"
DefaultSnapshot == CHOOSE s \in Snapshots : TRUE
DefaultCompat == CHOOSE c \in CompatibilityBases : TRUE
DefaultCapability == CHOOSE cap \in CapabilityViews : TRUE

VARIABLES
  candidateFacts,
  commitFacts,
  rejectLog,
  staticDeps,
  runtimeDeps,
  dynamicShapeDeps,
  affectedSet,
  pinnedReaders,
  overlayState,
  availableEvidence,
  contentionState,
  stage2Promoted,
  decisionHistory,
  transitionHistory

vars == <<
  candidateFacts,
  commitFacts,
  rejectLog,
  staticDeps,
  runtimeDeps,
  dynamicShapeDeps,
  affectedSet,
  pinnedReaders,
  overlayState,
  availableEvidence,
  contentionState,
  stage2Promoted,
  decisionHistory,
  transitionHistory
>>

AppendTransition(label) == Append(transitionHistory, label)

CandidateRecord(candidateId, node, snapshot, compat, capabilityView) ==
  [
    candidateId |-> candidateId,
    node |-> node,
    snapshot |-> snapshot,
    compat |-> compat,
    capabilityView |-> capabilityView,
    publicationId |-> NullPublication
  ]

CommitRecord(candidate, publicationId) ==
  [
    candidateId |-> candidate.candidateId,
    publicationId |-> publicationId,
    snapshot |-> candidate.snapshot,
    compat |-> candidate.compat,
    capabilityView |-> candidate.capabilityView,
    fenceCompatible |-> TRUE
  ]

RejectRecord(candidate, rejectKind) ==
  [
    candidateId |-> candidate.candidateId,
    rejectKind |-> rejectKind,
    publicationId |-> NullPublication,
    snapshot |-> candidate.snapshot,
    compat |-> candidate.compat,
    capabilityView |-> candidate.capabilityView
  ]

DependencyRecord(source, target, family) ==
  [source |-> source, target |-> target, family |-> family]

PinnedReaderRecord(active, publicationId) ==
  [active |-> active, publicationId |-> publicationId]

OverlayRecord(overlayId, node, readerId, protected, evictionEligible, evicted) ==
  [
    overlayId |-> overlayId,
    node |-> node,
    readerId |-> readerId,
    protected |-> protected,
    evictionEligible |-> evictionEligible,
    evicted |-> evicted
  ]

DecisionRecord(kind, publicationId, candidateId, fenceCompatible) ==
  [
    kind |-> kind,
    publicationId |-> publicationId,
    candidateId |-> candidateId,
    fenceCompatible |-> fenceCompatible
  ]

CandidateFactType ==
  [candidateId : CandidateIds,
   node : Nodes,
   snapshot : Snapshots,
   compat : CompatibilityBases,
   capabilityView : CapabilityViews,
   publicationId : {NullPublication}]

CommitFactType ==
  [candidateId : CandidateIds,
   publicationId : PublicationIds,
   snapshot : Snapshots,
   compat : CompatibilityBases,
   capabilityView : CapabilityViews,
   fenceCompatible : BOOLEAN]

RejectFactType ==
  [candidateId : CandidateIds,
   rejectKind : {"stale_fence"},
   publicationId : {NullPublication},
   snapshot : Snapshots,
   compat : CompatibilityBases,
   capabilityView : CapabilityViews]

DependencyFactType ==
  [source : Nodes, target : Nodes, family : {"static", "runtime", "dynamic_shape"}]

PinnedReaderType ==
  [ReaderIds -> [active : BOOLEAN, publicationId : PublicationIds \cup {NullPublication}]]

OverlayFactType ==
  [overlayId : OverlayIds,
   node : Nodes,
   readerId : ReaderIds,
   protected : BOOLEAN,
   evictionEligible : BOOLEAN,
   evicted : BOOLEAN]

DecisionType ==
  [kind : {"publish", "reject", "stage2_blocked"},
   publicationId : PublicationIds \cup {NullPublication},
   candidateId : CandidateIds \cup {NullPublication},
   fenceCompatible : BOOLEAN]

FenceCompatible(candidate) ==
  /\ candidate.snapshot = DefaultSnapshot
  /\ candidate.compat = DefaultCompat
  /\ candidate.capabilityView = DefaultCapability

MissingStage2Evidence == RequiredStage2Evidence \ availableEvidence

Init ==
  /\ candidateFacts = {}
  /\ commitFacts = {}
  /\ rejectLog = <<>>
  /\ staticDeps = {}
  /\ runtimeDeps = {}
  /\ dynamicShapeDeps = {}
  /\ affectedSet = {}
  /\ pinnedReaders = [reader \in ReaderIds |-> PinnedReaderRecord(FALSE, NullPublication)]
  /\ overlayState = {}
  /\ availableEvidence = {}
  /\ contentionState = "idle"
  /\ stage2Promoted = FALSE
  /\ decisionHistory = <<>>
  /\ transitionHistory = <<>>

A1ImportCandidate(candidateId, node, snapshot, compat, capabilityView) ==
  /\ candidateId \in CandidateIds
  /\ node \in Nodes
  /\ snapshot \in Snapshots
  /\ compat \in CompatibilityBases
  /\ capabilityView \in CapabilityViews
  /\ CandidateRecord(candidateId, node, snapshot, compat, capabilityView) \notin candidateFacts
  /\ candidateFacts' =
       candidateFacts \cup {CandidateRecord(candidateId, node, snapshot, compat, capabilityView)}
  /\ transitionHistory' = AppendTransition("A1:ImportCandidate")
  /\ UNCHANGED <<commitFacts, rejectLog, staticDeps, runtimeDeps, dynamicShapeDeps, affectedSet, pinnedReaders, overlayState, availableEvidence, contentionState, stage2Promoted, decisionHistory>>

A2PublishCompatibleCandidate(candidate, publicationId) ==
  /\ candidate \in candidateFacts
  /\ publicationId \in PublicationIds
  /\ FenceCompatible(candidate)
  /\ commitFacts' = commitFacts \cup {CommitRecord(candidate, publicationId)}
  /\ candidateFacts' = candidateFacts \ {candidate}
  /\ decisionHistory' =
       Append(decisionHistory, DecisionRecord("publish", publicationId, candidate.candidateId, TRUE))
  /\ transitionHistory' = AppendTransition("A2:PublishCompatibleCandidate")
  /\ UNCHANGED <<rejectLog, staticDeps, runtimeDeps, dynamicShapeDeps, affectedSet, pinnedReaders, overlayState, availableEvidence, contentionState, stage2Promoted>>

A3RejectStaleFenceCandidate(candidate) ==
  /\ candidate \in candidateFacts
  /\ ~FenceCompatible(candidate)
  /\ rejectLog' = Append(rejectLog, RejectRecord(candidate, "stale_fence"))
  /\ candidateFacts' = candidateFacts \ {candidate}
  /\ decisionHistory' =
       Append(decisionHistory, DecisionRecord("reject", NullPublication, candidate.candidateId, FALSE))
  /\ transitionHistory' = AppendTransition("A3:RejectStaleFenceCandidate")
  /\ UNCHANGED <<commitFacts, staticDeps, runtimeDeps, dynamicShapeDeps, affectedSet, pinnedReaders, overlayState, availableEvidence, contentionState, stage2Promoted>>

A4AddStaticDependency(source, target) ==
  /\ source \in Nodes
  /\ target \in Nodes
  /\ DependencyRecord(source, target, "static") \notin staticDeps
  /\ staticDeps' = staticDeps \cup {DependencyRecord(source, target, "static")}
  /\ affectedSet' = affectedSet \cup {target}
  /\ transitionHistory' = AppendTransition("A4:AddStaticDependency")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, runtimeDeps, dynamicShapeDeps, pinnedReaders, overlayState, availableEvidence, contentionState, stage2Promoted, decisionHistory>>

A5AddRuntimeDependency(source, target) ==
  /\ source \in Nodes
  /\ target \in Nodes
  /\ DependencyRecord(source, target, "runtime") \notin runtimeDeps
  /\ runtimeDeps' = runtimeDeps \cup {DependencyRecord(source, target, "runtime")}
  /\ affectedSet' = affectedSet \cup {target}
  /\ transitionHistory' = AppendTransition("A5:AddRuntimeDependency")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, dynamicShapeDeps, pinnedReaders, overlayState, availableEvidence, contentionState, stage2Promoted, decisionHistory>>

A6AddDynamicShapeDependency(source, target) ==
  /\ source \in Nodes
  /\ target \in Nodes
  /\ DependencyRecord(source, target, "dynamic_shape") \notin dynamicShapeDeps
  /\ dynamicShapeDeps' = dynamicShapeDeps \cup {DependencyRecord(source, target, "dynamic_shape")}
  /\ affectedSet' = affectedSet \cup {target}
  /\ transitionHistory' = AppendTransition("A6:AddDynamicShapeDependency")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, pinnedReaders, overlayState, availableEvidence, contentionState, stage2Promoted, decisionHistory>>

A7PinReader(readerId) ==
  /\ readerId \in ReaderIds
  /\ ~pinnedReaders[readerId].active
  /\ pinnedReaders' =
       [pinnedReaders EXCEPT ![readerId] = PinnedReaderRecord(TRUE, CHOOSE publicationId \in PublicationIds \cup {NullPublication} : TRUE)]
  /\ transitionHistory' = AppendTransition("A7:PinReader")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, dynamicShapeDeps, affectedSet, overlayState, availableEvidence, contentionState, stage2Promoted, decisionHistory>>

A8RetainProtectedOverlay(readerId, overlayId, node) ==
  /\ readerId \in ReaderIds
  /\ overlayId \in OverlayIds
  /\ node \in Nodes
  /\ pinnedReaders[readerId].active
  /\ \A entry \in overlayState : entry.overlayId # overlayId
  /\ overlayState' =
       overlayState \cup {OverlayRecord(overlayId, node, readerId, TRUE, FALSE, FALSE)}
  /\ transitionHistory' = AppendTransition("A8:RetainProtectedOverlay")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, dynamicShapeDeps, affectedSet, pinnedReaders, availableEvidence, contentionState, stage2Promoted, decisionHistory>>

A9ReleaseProtectionAndUnpin(readerId) ==
  /\ readerId \in ReaderIds
  /\ pinnedReaders[readerId].active
  /\ pinnedReaders' = [pinnedReaders EXCEPT ![readerId].active = FALSE]
  /\ overlayState' =
       { IF entry.readerId = readerId
         THEN [entry EXCEPT !.protected = FALSE, !.evictionEligible = TRUE]
         ELSE entry : entry \in overlayState }
  /\ transitionHistory' = AppendTransition("A9:ReleaseProtectionAndUnpin")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, dynamicShapeDeps, affectedSet, availableEvidence, contentionState, stage2Promoted, decisionHistory>>

A10EvictEligibleOverlay(overlayId) ==
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
  /\ transitionHistory' = AppendTransition("A10:EvictEligibleOverlay")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, dynamicShapeDeps, affectedSet, pinnedReaders, availableEvidence, contentionState, stage2Promoted, decisionHistory>>

A11RecordAvailableStage2Evidence(evidenceKind) ==
  /\ evidenceKind \in AvailableStage2Evidence
  /\ evidenceKind \notin availableEvidence
  /\ availableEvidence' = availableEvidence \cup {evidenceKind}
  /\ transitionHistory' = AppendTransition("A11:RecordAvailableStage2Evidence")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, dynamicShapeDeps, affectedSet, pinnedReaders, overlayState, contentionState, stage2Promoted, decisionHistory>>

A12AttemptStage2ContentionWithoutGate ==
  /\ MissingStage2Evidence # {}
  /\ contentionState = "idle"
  /\ contentionState' = "blocked_missing_preconditions"
  /\ stage2Promoted' = FALSE
  /\ decisionHistory' =
       Append(decisionHistory, DecisionRecord("stage2_blocked", NullPublication, NullPublication, FALSE))
  /\ transitionHistory' = AppendTransition("A12:AttemptStage2ContentionWithoutGate")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, dynamicShapeDeps, affectedSet, pinnedReaders, overlayState, availableEvidence>>

Next ==
  \/ \E candidateId \in CandidateIds, node \in Nodes, snapshot \in Snapshots, compat \in CompatibilityBases, capabilityView \in CapabilityViews :
       A1ImportCandidate(candidateId, node, snapshot, compat, capabilityView)
  \/ \E candidate \in candidateFacts, publicationId \in PublicationIds :
       A2PublishCompatibleCandidate(candidate, publicationId)
  \/ \E candidate \in candidateFacts :
       A3RejectStaleFenceCandidate(candidate)
  \/ \E source \in Nodes, target \in Nodes : A4AddStaticDependency(source, target)
  \/ \E source \in Nodes, target \in Nodes : A5AddRuntimeDependency(source, target)
  \/ \E source \in Nodes, target \in Nodes : A6AddDynamicShapeDependency(source, target)
  \/ \E readerId \in ReaderIds : A7PinReader(readerId)
  \/ \E readerId \in ReaderIds, overlayId \in OverlayIds, node \in Nodes :
       A8RetainProtectedOverlay(readerId, overlayId, node)
  \/ \E readerId \in ReaderIds : A9ReleaseProtectionAndUnpin(readerId)
  \/ \E overlayId \in OverlayIds : A10EvictEligibleOverlay(overlayId)
  \/ \E evidenceKind \in AvailableStage2Evidence :
       A11RecordAvailableStage2Evidence(evidenceKind)
  \/ A12AttemptStage2ContentionWithoutGate

TypeInvariant ==
  /\ candidateFacts \subseteq CandidateFactType
  /\ commitFacts \subseteq CommitFactType
  /\ rejectLog \in Seq(RejectFactType)
  /\ staticDeps \subseteq DependencyFactType
  /\ runtimeDeps \subseteq DependencyFactType
  /\ dynamicShapeDeps \subseteq DependencyFactType
  /\ affectedSet \subseteq Nodes
  /\ pinnedReaders \in PinnedReaderType
  /\ overlayState \subseteq OverlayFactType
  /\ availableEvidence \subseteq RequiredStage2Evidence
  /\ contentionState \in {"idle", "blocked_missing_preconditions"}
  /\ stage2Promoted \in BOOLEAN
  /\ decisionHistory \in Seq(DecisionType)

RejectIsNoPublish ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind = "reject" => decisionHistory[i].publicationId = NullPublication

PublishRequiresCompatibleFence ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind = "publish" => decisionHistory[i].fenceCompatible

NoStaleFencePublication ==
  \A commit \in commitFacts : commit.fenceCompatible

StaticDependenciesAffected ==
  \A dep \in staticDeps : dep.target \in affectedSet

RuntimeDependenciesAffected ==
  \A dep \in runtimeDeps : dep.target \in affectedSet

DynamicShapeDependenciesAffected ==
  \A dep \in dynamicShapeDeps : dep.target \in affectedSet

ProtectedOverlayPinnedAndRetained ==
  \A entry \in overlayState :
    entry.protected =>
      /\ pinnedReaders[entry.readerId].active
      /\ ~entry.evicted
      /\ ~entry.evictionEligible

EvictedOverlayWasUnprotected ==
  \A entry \in overlayState : entry.evicted => ~entry.protected

NoStage2ContentionPromotion ==
  stage2Promoted = FALSE

Stage2PreconditionsStillMissing ==
  MissingStage2Evidence # {}

SmokeConstraint == Len(transitionHistory) <= 4

Spec == Init /\ [][Next]_vars

=============================================================================
