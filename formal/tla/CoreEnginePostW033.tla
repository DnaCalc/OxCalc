---- MODULE CoreEnginePostW033 ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS Nodes, Snapshots, CandidateIds, PublicationIds, CompatibilityBases, CapabilityTokens, CallableIds, TraceIds

ASSUME /\ Nodes # {}
       /\ Snapshots # {}
       /\ CandidateIds # {}
       /\ PublicationIds # {}
       /\ CompatibilityBases # {}
       /\ CapabilityTokens # {}
       /\ CallableIds # {}
       /\ TraceIds # {}

NullPublication == "none"
DefaultSnapshot == CHOOSE s \in Snapshots : TRUE
DefaultCompat == CHOOSE c \in CompatibilityBases : TRUE
DefaultCapability == CHOOSE cap \in CapabilityTokens : TRUE

VARIABLES
  candidateFacts,
  commitFacts,
  rejectLog,
  staticDeps,
  runtimeDeps,
  affectedSet,
  carrierFacts,
  overlayState,
  publishedView,
  decisionHistory,
  transitionHistory,
  stage2Promoted

vars == <<
  candidateFacts,
  commitFacts,
  rejectLog,
  staticDeps,
  runtimeDeps,
  affectedSet,
  carrierFacts,
  overlayState,
  publishedView,
  decisionHistory,
  transitionHistory,
  stage2Promoted
>>

AppendTransition(label) == Append(transitionHistory, label)

CandidateRecord(candidateId, node, snapshot, compat, capabilityToken) ==
  [
    candidateId |-> candidateId,
    node |-> node,
    snapshot |-> snapshot,
    compat |-> compat,
    capabilityToken |-> capabilityToken,
    publicationId |-> NullPublication
  ]

CommitRecord(candidateId, publicationId, snapshot, compat, capabilityToken) ==
  [
    candidateId |-> candidateId,
    publicationId |-> publicationId,
    snapshot |-> snapshot,
    compat |-> compat,
    capabilityToken |-> capabilityToken,
    fenceCompatible |-> TRUE
  ]

RejectRecord(candidateId, rejectKind, traceId) ==
  [
    candidateId |-> candidateId,
    rejectKind |-> rejectKind,
    publicationId |-> NullPublication,
    traceId |-> traceId
  ]

DependencyRecord(source, target) ==
  [source |-> source, target |-> target]

CarrierRecord(callableId, dependencyVisible, runtimeEffectVisible, hasInvocationContract) ==
  [
    callableId |-> callableId,
    dependencyVisible |-> dependencyVisible,
    runtimeEffectVisible |-> runtimeEffectVisible,
    hasInvocationContract |-> hasInvocationContract
  ]

OverlayRecord(node, protected, eligible) ==
  [
    node |-> node,
    protected |-> protected,
    evictionEligible |-> eligible
  ]

PublishedViewRecord(snapshot, publicationId, candidateId) ==
  [
    snapshot |-> snapshot,
    publicationId |-> publicationId,
    candidateId |-> candidateId
  ]

CandidateFactType ==
  [candidateId : CandidateIds,
   node : Nodes,
   snapshot : Snapshots,
   compat : CompatibilityBases,
   capabilityToken : CapabilityTokens,
   publicationId : {NullPublication}]

CommitFactType ==
  [candidateId : CandidateIds,
   publicationId : PublicationIds,
   snapshot : Snapshots,
   compat : CompatibilityBases,
   capabilityToken : CapabilityTokens,
   fenceCompatible : BOOLEAN]

RejectFactType ==
  [candidateId : CandidateIds,
   rejectKind : {"reject", "fence_mismatch", "callable_mismatch", "dynamic_dependency"},
   publicationId : {NullPublication},
   traceId : TraceIds]

DependencyFactType == [source : Nodes, target : Nodes]

CarrierFactType ==
  [callableId : CallableIds,
   dependencyVisible : BOOLEAN,
   runtimeEffectVisible : BOOLEAN,
   hasInvocationContract : BOOLEAN]

OverlayFactType ==
  [node : Nodes, protected : BOOLEAN, evictionEligible : BOOLEAN]

DecisionType ==
  [kind : {"reject", "publish", "verify_clean"},
   publicationId : PublicationIds \cup {NullPublication},
   candidateId : CandidateIds \cup {NullPublication},
   fenceCompatible : BOOLEAN]

Init ==
  /\ candidateFacts = {}
  /\ commitFacts = {}
  /\ rejectLog = <<>>
  /\ staticDeps = {}
  /\ runtimeDeps = {}
  /\ affectedSet = {}
  /\ carrierFacts = {}
  /\ overlayState = {}
  /\ publishedView = PublishedViewRecord(DefaultSnapshot, NullPublication, NullPublication)
  /\ decisionHistory = <<>>
  /\ transitionHistory = <<>>
  /\ stage2Promoted = FALSE

A1ImportCandidate(candidateId, node, snapshot, compat, capabilityToken) ==
  /\ candidateId \in CandidateIds
  /\ node \in Nodes
  /\ snapshot \in Snapshots
  /\ compat \in CompatibilityBases
  /\ capabilityToken \in CapabilityTokens
  /\ candidateFacts' = candidateFacts \cup {CandidateRecord(candidateId, node, snapshot, compat, capabilityToken)}
  /\ transitionHistory' = AppendTransition("A1:ImportCandidate")
  /\ UNCHANGED <<commitFacts, rejectLog, staticDeps, runtimeDeps, affectedSet, carrierFacts, overlayState, publishedView, decisionHistory, stage2Promoted>>

A2AddStaticDependency(source, target) ==
  /\ source \in Nodes
  /\ target \in Nodes
  /\ staticDeps' = staticDeps \cup {DependencyRecord(source, target)}
  /\ affectedSet' = affectedSet \cup {target}
  /\ transitionHistory' = AppendTransition("A2:AddStaticDependency")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, runtimeDeps, carrierFacts, overlayState, publishedView, decisionHistory, stage2Promoted>>

A3AddRuntimeDependency(source, target) ==
  /\ source \in Nodes
  /\ target \in Nodes
  /\ runtimeDeps' = runtimeDeps \cup {DependencyRecord(source, target)}
  /\ affectedSet' = affectedSet \cup {target}
  /\ transitionHistory' = AppendTransition("A3:AddRuntimeDependency")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, carrierFacts, overlayState, publishedView, decisionHistory, stage2Promoted>>

A4ImportVisibleCarrier(callableId) ==
  /\ callableId \in CallableIds
  /\ carrierFacts' = carrierFacts \cup {CarrierRecord(callableId, TRUE, TRUE, TRUE)}
  /\ transitionHistory' = AppendTransition("A4:ImportVisibleCarrier")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, affectedSet, overlayState, publishedView, decisionHistory, stage2Promoted>>

A5RejectCandidate(candidateId, traceId) ==
  /\ candidateId \in CandidateIds
  /\ traceId \in TraceIds
  /\ rejectLog' = Append(rejectLog, RejectRecord(candidateId, "reject", traceId))
  /\ candidateFacts' = {candidate \in candidateFacts : candidate.candidateId # candidateId}
  /\ decisionHistory' = Append(decisionHistory, [kind |-> "reject", publicationId |-> NullPublication, candidateId |-> candidateId, fenceCompatible |-> FALSE])
  /\ transitionHistory' = AppendTransition("A5:RejectCandidate")
  /\ UNCHANGED <<commitFacts, staticDeps, runtimeDeps, affectedSet, carrierFacts, overlayState, publishedView, stage2Promoted>>

A6PublishWithCompatibleFence(candidateId, publicationId) ==
  /\ candidateId \in CandidateIds
  /\ publicationId \in PublicationIds
  /\ \E candidate \in candidateFacts :
       /\ candidate.candidateId = candidateId
       /\ candidate.snapshot = DefaultSnapshot
       /\ candidate.compat = DefaultCompat
       /\ candidate.capabilityToken = DefaultCapability
  /\ commitFacts' = commitFacts \cup {CommitRecord(candidateId, publicationId, DefaultSnapshot, DefaultCompat, DefaultCapability)}
  /\ candidateFacts' = {candidate \in candidateFacts : candidate.candidateId # candidateId}
  /\ publishedView' = PublishedViewRecord(DefaultSnapshot, publicationId, candidateId)
  /\ decisionHistory' = Append(decisionHistory, [kind |-> "publish", publicationId |-> publicationId, candidateId |-> candidateId, fenceCompatible |-> TRUE])
  /\ transitionHistory' = AppendTransition("A6:PublishWithCompatibleFence")
  /\ UNCHANGED <<rejectLog, staticDeps, runtimeDeps, affectedSet, carrierFacts, overlayState, stage2Promoted>>

A7RetainProtectedOverlay(node) ==
  /\ node \in Nodes
  /\ overlayState' = overlayState \cup {OverlayRecord(node, TRUE, FALSE)}
  /\ transitionHistory' = AppendTransition("A7:RetainProtectedOverlay")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, affectedSet, carrierFacts, publishedView, decisionHistory, stage2Promoted>>

A8MarkUnprotectedEligible(node) ==
  /\ node \in Nodes
  /\ overlayState' = overlayState \cup {OverlayRecord(node, FALSE, TRUE)}
  /\ transitionHistory' = AppendTransition("A8:MarkUnprotectedEligible")
  /\ UNCHANGED <<candidateFacts, commitFacts, rejectLog, staticDeps, runtimeDeps, affectedSet, carrierFacts, publishedView, decisionHistory, stage2Promoted>>

Next ==
  \/ \E candidateId \in CandidateIds, node \in Nodes, snapshot \in Snapshots, compat \in CompatibilityBases, capabilityToken \in CapabilityTokens :
       A1ImportCandidate(candidateId, node, snapshot, compat, capabilityToken)
  \/ \E source \in Nodes, target \in Nodes : A2AddStaticDependency(source, target)
  \/ \E source \in Nodes, target \in Nodes : A3AddRuntimeDependency(source, target)
  \/ \E callableId \in CallableIds : A4ImportVisibleCarrier(callableId)
  \/ \E candidateId \in CandidateIds, traceId \in TraceIds : A5RejectCandidate(candidateId, traceId)
  \/ \E candidateId \in CandidateIds, publicationId \in PublicationIds : A6PublishWithCompatibleFence(candidateId, publicationId)
  \/ \E node \in Nodes : A7RetainProtectedOverlay(node)
  \/ \E node \in Nodes : A8MarkUnprotectedEligible(node)

TypeInvariant ==
  /\ candidateFacts \subseteq CandidateFactType
  /\ commitFacts \subseteq CommitFactType
  /\ rejectLog \in Seq(RejectFactType)
  /\ staticDeps \subseteq DependencyFactType
  /\ runtimeDeps \subseteq DependencyFactType
  /\ affectedSet \subseteq Nodes
  /\ carrierFacts \subseteq CarrierFactType
  /\ overlayState \subseteq OverlayFactType
  /\ publishedView \in [snapshot : Snapshots, publicationId : PublicationIds \cup {NullPublication}, candidateId : CandidateIds \cup {NullPublication}]
  /\ decisionHistory \in Seq(DecisionType)
  /\ stage2Promoted \in BOOLEAN

RejectIsNoPublish ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind = "reject" => decisionHistory[i].publicationId = NullPublication

RuntimeDependenciesAffected ==
  \A dep \in runtimeDeps : dep.target \in affectedSet

StaticDependenciesAffected ==
  \A dep \in staticDeps : dep.target \in affectedSet

CarrierVisibilityHasInvocationContract ==
  \A carrier \in carrierFacts :
    (carrier.dependencyVisible \/ carrier.runtimeEffectVisible) => carrier.hasInvocationContract

ProtectedOverlaySafety ==
  \A entry \in overlayState : ~(entry.protected /\ entry.evictionEligible)

PublishRequiresCompatibleFence ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind = "publish" => decisionHistory[i].fenceCompatible

CandidateIsNotPublication ==
  \A candidate \in candidateFacts : candidate.publicationId = NullPublication

NoStage2ContentionPromotion ==
  stage2Promoted = FALSE

SmokeConstraint == Len(transitionHistory) <= 4

Spec == Init /\ [][Next]_vars

=============================================================================
