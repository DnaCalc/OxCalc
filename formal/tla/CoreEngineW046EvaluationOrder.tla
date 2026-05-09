---- MODULE CoreEngineW046EvaluationOrder ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS MaxTransitions

Nodes == {"X", "Y", "Z"}
FormulaOwners == {"Y", "Z"}
StableInputs == {"X"}
AcyclicOrder == <<"Y", "Z">>
EmptyOrder == <<>>

Phases == {"start", "ordered", "terminal"}
DecisionKinds == {"none", "publish", "verified_clean", "diagnostic_reject", "cycle_reject"}
RejectKinds == {"none", "cycle", "diagnostic"}
ReadSources == {"stable", "prior"}

ASSUME /\ MaxTransitions \in Nat
       /\ MaxTransitions > 0

VARIABLES
  phase,
  order,
  workingValues,
  computedNodes,
  updatedNodes,
  candidateTargetSet,
  rejected,
  published,
  verifiedClean,
  cycleRejected,
  diagnosticFailure,
  rejectKind,
  decisionKind,
  readEvents,
  transitionHistory

vars == <<
  phase,
  order,
  workingValues,
  computedNodes,
  updatedNodes,
  candidateTargetSet,
  rejected,
  published,
  verifiedClean,
  cycleRejected,
  diagnosticFailure,
  rejectKind,
  decisionKind,
  readEvents,
  transitionHistory
>>

ReadEventType == [
  owner : FormulaOwners,
  target : Nodes,
  source : ReadSources
]

ReadEvent(owner, target, source) == [
  owner |-> owner,
  target |-> target,
  source |-> source
]

OrderSet(seq) == {seq[i] : i \in 1..Len(seq)}

AppendTransition(label) == Append(transitionHistory, label)

Init ==
  /\ phase = "start"
  /\ order = EmptyOrder
  /\ workingValues = StableInputs
  /\ computedNodes = EmptyOrder
  /\ updatedNodes = {}
  /\ candidateTargetSet = {}
  /\ rejected = FALSE
  /\ published = FALSE
  /\ verifiedClean = FALSE
  /\ cycleRejected = FALSE
  /\ diagnosticFailure = FALSE
  /\ rejectKind = "none"
  /\ decisionKind = "none"
  /\ readEvents = <<>>
  /\ transitionHistory = <<>>

SelectAcyclicOrder ==
  /\ phase = "start"
  /\ phase' = "ordered"
  /\ order' = AcyclicOrder
  /\ transitionHistory' = AppendTransition("select_acyclic_order")
  /\ decisionKind' = "none"
  /\ UNCHANGED <<workingValues, computedNodes, updatedNodes, candidateTargetSet,
                 rejected, published, verifiedClean, cycleRejected,
                 diagnosticFailure, rejectKind, readEvents>>

CycleReject ==
  /\ phase = "start"
  /\ phase' = "terminal"
  /\ order' = EmptyOrder
  /\ computedNodes' = EmptyOrder
  /\ updatedNodes' = {}
  /\ candidateTargetSet' = {}
  /\ rejected' = TRUE
  /\ published' = FALSE
  /\ verifiedClean' = FALSE
  /\ cycleRejected' = TRUE
  /\ diagnosticFailure' = FALSE
  /\ rejectKind' = "cycle"
  /\ decisionKind' = "cycle_reject"
  /\ readEvents' = <<>>
  /\ transitionHistory' = AppendTransition("cycle_reject")
  /\ UNCHANGED <<workingValues>>

EvaluateYChanged ==
  /\ phase = "ordered"
  /\ order = AcyclicOrder
  /\ computedNodes = EmptyOrder
  /\ "X" \in workingValues
  /\ computedNodes' = <<"Y">>
  /\ workingValues' = workingValues \cup {"Y"}
  /\ updatedNodes' = updatedNodes \cup {"Y"}
  /\ readEvents' = Append(readEvents, ReadEvent("Y", "X", "stable"))
  /\ transitionHistory' = AppendTransition("evaluate_y_changed")
  /\ UNCHANGED <<phase, order, candidateTargetSet, rejected, published,
                 verifiedClean, cycleRejected, diagnosticFailure, rejectKind,
                 decisionKind>>

EvaluateYClean ==
  /\ phase = "ordered"
  /\ order = AcyclicOrder
  /\ computedNodes = EmptyOrder
  /\ "X" \in workingValues
  /\ computedNodes' = <<"Y">>
  /\ workingValues' = workingValues \cup {"Y"}
  /\ updatedNodes' = updatedNodes
  /\ readEvents' = Append(readEvents, ReadEvent("Y", "X", "stable"))
  /\ transitionHistory' = AppendTransition("evaluate_y_clean")
  /\ UNCHANGED <<phase, order, candidateTargetSet, rejected, published,
                 verifiedClean, cycleRejected, diagnosticFailure, rejectKind,
                 decisionKind>>

EvaluateZChanged ==
  /\ phase = "ordered"
  /\ order = AcyclicOrder
  /\ computedNodes = <<"Y">>
  /\ "X" \in workingValues
  /\ "Y" \in workingValues
  /\ computedNodes' = Append(computedNodes, "Z")
  /\ workingValues' = workingValues \cup {"Z"}
  /\ updatedNodes' = updatedNodes \cup {"Z"}
  /\ readEvents' =
       Append(
         Append(readEvents, ReadEvent("Z", "X", "stable")),
         ReadEvent("Z", "Y", "prior"))
  /\ transitionHistory' = AppendTransition("evaluate_z_changed")
  /\ UNCHANGED <<phase, order, candidateTargetSet, rejected, published,
                 verifiedClean, cycleRejected, diagnosticFailure, rejectKind,
                 decisionKind>>

EvaluateZClean ==
  /\ phase = "ordered"
  /\ order = AcyclicOrder
  /\ computedNodes = <<"Y">>
  /\ "X" \in workingValues
  /\ "Y" \in workingValues
  /\ computedNodes' = Append(computedNodes, "Z")
  /\ workingValues' = workingValues \cup {"Z"}
  /\ updatedNodes' = updatedNodes
  /\ readEvents' =
       Append(
         Append(readEvents, ReadEvent("Z", "X", "stable")),
         ReadEvent("Z", "Y", "prior"))
  /\ transitionHistory' = AppendTransition("evaluate_z_clean")
  /\ UNCHANGED <<phase, order, candidateTargetSet, rejected, published,
                 verifiedClean, cycleRejected, diagnosticFailure, rejectKind,
                 decisionKind>>

DiagnosticFailure ==
  /\ phase = "ordered"
  /\ computedNodes # order
  /\ phase' = "terminal"
  /\ rejected' = TRUE
  /\ published' = FALSE
  /\ verifiedClean' = FALSE
  /\ cycleRejected' = FALSE
  /\ diagnosticFailure' = TRUE
  /\ rejectKind' = "diagnostic"
  /\ decisionKind' = "diagnostic_reject"
  /\ candidateTargetSet' = OrderSet(order)
  /\ transitionHistory' = AppendTransition("diagnostic_failure")
  /\ UNCHANGED <<order, workingValues, computedNodes, updatedNodes, readEvents>>

PublishCandidate ==
  /\ phase = "ordered"
  /\ computedNodes = order
  /\ updatedNodes # {}
  /\ phase' = "terminal"
  /\ candidateTargetSet' = OrderSet(order)
  /\ rejected' = FALSE
  /\ published' = TRUE
  /\ verifiedClean' = FALSE
  /\ cycleRejected' = FALSE
  /\ diagnosticFailure' = FALSE
  /\ rejectKind' = "none"
  /\ decisionKind' = "publish"
  /\ transitionHistory' = AppendTransition("publish_candidate")
  /\ UNCHANGED <<order, workingValues, computedNodes, updatedNodes, readEvents>>

FinalizeVerifiedClean ==
  /\ phase = "ordered"
  /\ computedNodes = order
  /\ updatedNodes = {}
  /\ phase' = "terminal"
  /\ candidateTargetSet' = {}
  /\ rejected' = FALSE
  /\ published' = FALSE
  /\ verifiedClean' = TRUE
  /\ cycleRejected' = FALSE
  /\ diagnosticFailure' = FALSE
  /\ rejectKind' = "none"
  /\ decisionKind' = "verified_clean"
  /\ transitionHistory' = AppendTransition("finalize_verified_clean")
  /\ UNCHANGED <<order, workingValues, computedNodes, updatedNodes, readEvents>>

TerminalStutter ==
  /\ phase = "terminal"
  /\ UNCHANGED vars

Next ==
  \/ SelectAcyclicOrder
  \/ CycleReject
  \/ EvaluateYChanged
  \/ EvaluateYClean
  \/ EvaluateZChanged
  \/ EvaluateZClean
  \/ DiagnosticFailure
  \/ PublishCandidate
  \/ FinalizeVerifiedClean
  \/ TerminalStutter

Spec == Init /\ [][Next]_vars

TypeInvariant ==
  /\ phase \in Phases
  /\ order \in Seq(FormulaOwners)
  /\ workingValues \subseteq Nodes
  /\ computedNodes \in {EmptyOrder, <<"Y">>, AcyclicOrder}
  /\ updatedNodes \subseteq FormulaOwners
  /\ candidateTargetSet \subseteq FormulaOwners
  /\ rejected \in BOOLEAN
  /\ published \in BOOLEAN
  /\ verifiedClean \in BOOLEAN
  /\ cycleRejected \in BOOLEAN
  /\ diagnosticFailure \in BOOLEAN
  /\ rejectKind \in RejectKinds
  /\ decisionKind \in DecisionKinds
  /\ readEvents \in Seq(ReadEventType)

SelectedOrderIsTopological ==
  /\ (phase # "start" /\ ~cycleRejected) => order = AcyclicOrder
  /\ order = AcyclicOrder => <<"Y", "Z">> = order

EvaluationOrderPrefix ==
  computedNodes \in {EmptyOrder, <<"Y">>, AcyclicOrder}

WorkingValuesContainStableInputs ==
  StableInputs \subseteq workingValues

WorkingValuesCoverComputedNodes ==
  OrderSet(computedNodes) \subseteq workingValues

StableOrPriorReadDiscipline ==
  \A i \in 1..Len(readEvents) :
    \/ /\ readEvents[i].source = "stable"
       /\ readEvents[i].target \in StableInputs
    \/ /\ readEvents[i].source = "prior"
       /\ readEvents[i].owner = "Z"
       /\ readEvents[i].target = "Y"

NoFutureReadEvents ==
  \A i \in 1..Len(readEvents) :
    readEvents[i].source \in {"stable", "prior"}

DiagnosticShortCircuitNoPublish ==
  diagnosticFailure => /\ rejected /\ ~published /\ decisionKind = "diagnostic_reject"

CycleRejectNoEvaluation ==
  cycleRejected => /\ rejected /\ ~published /\ computedNodes = EmptyOrder /\ decisionKind = "cycle_reject"

VerifiedCleanNoPublication ==
  verifiedClean => /\ updatedNodes = {} /\ ~published /\ decisionKind = "verified_clean"

NoTornCandidateBundle ==
  published =>
    /\ decisionKind = "publish"
    /\ computedNodes = order
    /\ candidateTargetSet = OrderSet(order)
    /\ updatedNodes \subseteq candidateTargetSet

RejectNoPublication ==
  rejected => ~published

TerminalDecisionIsExclusive ==
  phase = "terminal" =>
    \/ /\ published
       /\ ~verifiedClean
       /\ ~rejected
    \/ /\ verifiedClean
       /\ ~published
       /\ ~rejected
    \/ /\ rejected
       /\ ~published
       /\ ~verifiedClean

ExplorationConstraint ==
  Len(transitionHistory) <= MaxTransitions

====
