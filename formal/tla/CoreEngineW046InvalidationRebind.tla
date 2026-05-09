---- MODULE CoreEngineW046InvalidationRebind ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS MaxTransitions

Nodes == {"A", "B", "C"}
Reasons ==
  {"StructuralRebindRequired", "StructuralRecalcOnly", "UpstreamPublication",
   "DependencyAdded", "DependencyRemoved", "DependencyReclassified"}
RebindReasons ==
  {"StructuralRebindRequired", "DependencyAdded", "DependencyRemoved", "DependencyReclassified"}

ForwardEdges == {
  [owner |-> "B", target |-> "A"],
  [owner |-> "C", target |-> "B"]
}

SeedInput == {
  [node |-> "A", reason |-> "UpstreamPublication"],
  [node |-> "B", reason |-> "DependencyAdded"],
  [node |-> "B", reason |-> "DependencyReclassified"]
}

EvaluationTargets == {"A", "B", "C"}

ASSUME /\ MaxTransitions \in Nat
       /\ MaxTransitions > 0

VARIABLES
  closureRecords,
  phase,
  rejected,
  published,
  transitionHistory

vars == <<closureRecords, phase, rejected, published, transitionHistory>>

SeedNodes == {seed.node : seed \in SeedInput}

SeedReasons(node) == {seed.reason : seed \in {entry \in SeedInput : entry.node = node}}

Reach0 == SeedNodes

Reach1 == Reach0 \cup {edge.owner : edge \in {entry \in ForwardEdges : entry.target \in Reach0}}

Reach2 == Reach1 \cup {edge.owner : edge \in {entry \in ForwardEdges : entry.target \in Reach1}}

ReverseReachableClosure == Reach2

RecordReasons(node) ==
  IF SeedReasons(node) # {}
  THEN SeedReasons(node)
  ELSE {"UpstreamPublication"}

RequiresRebind(node) == (RecordReasons(node) \cap RebindReasons) # {}

CalcState(node) ==
  IF RequiresRebind(node) THEN "DirtyPending"
  ELSE IF "UpstreamPublication" \in RecordReasons(node) THEN "Needed"
  ELSE "DirtyPending"

ClosureRecord(node) ==
  [
    node |-> node,
    reasons |-> RecordReasons(node),
    requiresRebind |-> RequiresRebind(node),
    calcState |-> CalcState(node)
  ]

RecordNodes == {record.node : record \in closureRecords}

RebindBlocked ==
  \E record \in closureRecords :
    /\ record.node \in EvaluationTargets
    /\ record.requiresRebind

Init ==
  /\ closureRecords = {}
  /\ phase = "empty"
  /\ rejected = FALSE
  /\ published = FALSE
  /\ transitionHistory = <<>>

CloseInvalidation ==
  /\ phase = "empty"
  /\ closureRecords' = {ClosureRecord(node) : node \in ReverseReachableClosure}
  /\ phase' = "closed"
  /\ rejected' = FALSE
  /\ published' = FALSE
  /\ transitionHistory' = Append(transitionHistory, "CloseInvalidation")

RunRebindGate ==
  /\ phase = "closed"
  /\ IF RebindBlocked
     THEN /\ rejected' = TRUE
          /\ published' = FALSE
          /\ phase' = "rejected"
          /\ transitionHistory' = Append(transitionHistory, "RebindGateReject")
     ELSE /\ rejected' = FALSE
          /\ published' = TRUE
          /\ phase' = "published"
          /\ transitionHistory' = Append(transitionHistory, "RebindGatePass")
  /\ UNCHANGED closureRecords

StutterFinal ==
  /\ phase \in {"rejected", "published"}
  /\ UNCHANGED vars

Next == CloseInvalidation \/ RunRebindGate \/ StutterFinal

Spec == Init /\ [][Next]_vars

TypeInvariant ==
  /\ closureRecords \subseteq
       [node : Nodes,
        reasons : SUBSET Reasons,
        requiresRebind : BOOLEAN,
        calcState : {"DirtyPending", "Needed", "CycleBlocked"}]
  /\ phase \in {"empty", "closed", "rejected", "published"}
  /\ rejected \in BOOLEAN
  /\ published \in BOOLEAN

NoUnderInvalidation ==
  phase = "empty" \/ ReverseReachableClosure \subseteq RecordNodes

RebindReasonsSetFlag ==
  \A record \in closureRecords :
    ((record.reasons \cap RebindReasons) # {}) => record.requiresRebind

NonRebindReasonsDoNotSetFlag ==
  \A record \in closureRecords :
    ((record.reasons \cap RebindReasons) = {}) => ~record.requiresRebind

DynamicTransitionSeedsPresent ==
  phase = "empty" \/
    \E record \in closureRecords :
      /\ record.node = "B"
      /\ "DependencyAdded" \in record.reasons
      /\ "DependencyReclassified" \in record.reasons
      /\ record.requiresRebind

DependentPropagationRecordsUpstream ==
  phase = "empty" \/
    \E record \in closureRecords :
      /\ record.node = "C"
      /\ "UpstreamPublication" \in record.reasons

RebindNoPublish ==
  \A record \in closureRecords :
    record.requiresRebind => published = FALSE

RejectedDecisionIsNoPublish ==
  rejected => published = FALSE

ExplorationConstraint ==
  Len(transitionHistory) <= MaxTransitions

====
