---- MODULE CoreEngineW046IntegratedKernel ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS MaxTransitions

Phases == {
  "start",
  "prepared",
  "descriptors_lowered",
  "graph_built",
  "invalidation_closed",
  "dirty_needed",
  "order_selected",
  "evaluating",
  "candidate_ready",
  "verified_clean",
  "rejected",
  "published",
  "traced"
}

TerminalPhases == {"verified_clean", "rejected", "published", "traced"}

VARIABLES
  phase,
  prepared,
  descriptorsLowered,
  graphBuilt,
  reverseConverse,
  diagnosticsPreserved,
  invalidationClosed,
  noUnderInvalidation,
  rebindRequired,
  dirtyNeeded,
  orderSelected,
  stablePriorReads,
  evaluating,
  candidateProduced,
  oxfmlNoDirectPublish,
  traceCalcObservableRefinement,
  published,
  rejected,
  verifiedClean,
  traceEmitted,
  exactBlockers,
  transitionHistory

vars == <<
  phase,
  prepared,
  descriptorsLowered,
  graphBuilt,
  reverseConverse,
  diagnosticsPreserved,
  invalidationClosed,
  noUnderInvalidation,
  rebindRequired,
  dirtyNeeded,
  orderSelected,
  stablePriorReads,
  evaluating,
  candidateProduced,
  oxfmlNoDirectPublish,
  traceCalcObservableRefinement,
  published,
  rejected,
  verifiedClean,
  traceEmitted,
  exactBlockers,
  transitionHistory
>>

AppendTransition(label) == Append(transitionHistory, label)

Init ==
  /\ phase = "start"
  /\ prepared = FALSE
  /\ descriptorsLowered = FALSE
  /\ graphBuilt = FALSE
  /\ reverseConverse = FALSE
  /\ diagnosticsPreserved = FALSE
  /\ invalidationClosed = FALSE
  /\ noUnderInvalidation = FALSE
  /\ rebindRequired = FALSE
  /\ dirtyNeeded = FALSE
  /\ orderSelected = FALSE
  /\ stablePriorReads = FALSE
  /\ evaluating = FALSE
  /\ candidateProduced = FALSE
  /\ oxfmlNoDirectPublish = FALSE
  /\ traceCalcObservableRefinement = FALSE
  /\ published = FALSE
  /\ rejected = FALSE
  /\ verifiedClean = FALSE
  /\ traceEmitted = FALSE
  /\ exactBlockers = {}
  /\ transitionHistory = <<>>

PrepareFormula ==
  /\ phase = "start"
  /\ phase' = "prepared"
  /\ prepared' = TRUE
  /\ oxfmlNoDirectPublish' = TRUE
  /\ transitionHistory' = AppendTransition("T01.PrepareFormula")
  /\ UNCHANGED <<descriptorsLowered, graphBuilt, reverseConverse, diagnosticsPreserved,
      invalidationClosed, noUnderInvalidation, rebindRequired, dirtyNeeded, orderSelected,
      stablePriorReads, evaluating, candidateProduced, traceCalcObservableRefinement,
      published, rejected, verifiedClean, traceEmitted, exactBlockers>>

LowerDescriptors ==
  /\ phase = "prepared"
  /\ phase' = "descriptors_lowered"
  /\ descriptorsLowered' = TRUE
  /\ transitionHistory' = AppendTransition("T02.LowerDescriptors")
  /\ UNCHANGED <<prepared, graphBuilt, reverseConverse, diagnosticsPreserved,
      invalidationClosed, noUnderInvalidation, rebindRequired, dirtyNeeded, orderSelected,
      stablePriorReads, evaluating, candidateProduced, oxfmlNoDirectPublish,
      traceCalcObservableRefinement, published, rejected, verifiedClean, traceEmitted,
      exactBlockers>>

BuildGraph ==
  /\ phase = "descriptors_lowered"
  /\ phase' = "graph_built"
  /\ graphBuilt' = TRUE
  /\ reverseConverse' = TRUE
  /\ diagnosticsPreserved' = TRUE
  /\ transitionHistory' = AppendTransition("T03-T05.Graph")
  /\ UNCHANGED <<prepared, descriptorsLowered, invalidationClosed, noUnderInvalidation,
      rebindRequired, dirtyNeeded, orderSelected, stablePriorReads, evaluating,
      candidateProduced, oxfmlNoDirectPublish, traceCalcObservableRefinement,
      published, rejected, verifiedClean, traceEmitted, exactBlockers>>

CloseInvalidationNoRebind ==
  /\ phase = "graph_built"
  /\ phase' = "invalidation_closed"
  /\ invalidationClosed' = TRUE
  /\ noUnderInvalidation' = TRUE
  /\ rebindRequired' = FALSE
  /\ transitionHistory' = AppendTransition("T06-T07.CloseInvalidation")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, dirtyNeeded, orderSelected, stablePriorReads, evaluating,
      candidateProduced, oxfmlNoDirectPublish, traceCalcObservableRefinement,
      published, rejected, verifiedClean, traceEmitted, exactBlockers>>

CloseInvalidationWithRebind ==
  /\ phase = "graph_built"
  /\ phase' = "invalidation_closed"
  /\ invalidationClosed' = TRUE
  /\ noUnderInvalidation' = TRUE
  /\ rebindRequired' = TRUE
  /\ transitionHistory' = AppendTransition("T06-T07.CloseInvalidationRebind")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, dirtyNeeded, orderSelected, stablePriorReads, evaluating,
      candidateProduced, oxfmlNoDirectPublish, traceCalcObservableRefinement,
      published, rejected, verifiedClean, traceEmitted, exactBlockers>>

MarkDirtyNeeded ==
  /\ phase = "invalidation_closed"
  /\ phase' = "dirty_needed"
  /\ dirtyNeeded' = TRUE
  /\ transitionHistory' = AppendTransition("T08.MarkDirtyNeeded")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, invalidationClosed, noUnderInvalidation, rebindRequired,
      orderSelected, stablePriorReads, evaluating, candidateProduced,
      oxfmlNoDirectPublish, traceCalcObservableRefinement, published, rejected,
      verifiedClean, traceEmitted, exactBlockers>>

RejectForRebind ==
  /\ phase = "dirty_needed"
  /\ rebindRequired = TRUE
  /\ phase' = "rejected"
  /\ rejected' = TRUE
  /\ transitionHistory' = AppendTransition("T10.RebindGateReject")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, invalidationClosed, noUnderInvalidation, rebindRequired,
      dirtyNeeded, orderSelected, stablePriorReads, evaluating, candidateProduced,
      oxfmlNoDirectPublish, traceCalcObservableRefinement, published, verifiedClean,
      traceEmitted, exactBlockers>>

SelectOrder ==
  /\ phase = "dirty_needed"
  /\ rebindRequired = FALSE
  /\ phase' = "order_selected"
  /\ orderSelected' = TRUE
  /\ transitionHistory' = AppendTransition("T09.SelectEvaluationOrder")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, invalidationClosed, noUnderInvalidation, rebindRequired,
      dirtyNeeded, stablePriorReads, evaluating, candidateProduced,
      oxfmlNoDirectPublish, traceCalcObservableRefinement, published, rejected,
      verifiedClean, traceEmitted, exactBlockers>>

BeginEvaluate ==
  /\ phase = "order_selected"
  /\ phase' = "evaluating"
  /\ evaluating' = TRUE
  /\ stablePriorReads' = TRUE
  /\ transitionHistory' = AppendTransition("T11-T13.BeginEvaluateRead")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, invalidationClosed, noUnderInvalidation, rebindRequired,
      dirtyNeeded, orderSelected, candidateProduced, oxfmlNoDirectPublish,
      traceCalcObservableRefinement, published, rejected, verifiedClean, traceEmitted,
      exactBlockers>>

FormulaReject ==
  /\ phase = "evaluating"
  /\ phase' = "rejected"
  /\ rejected' = TRUE
  /\ transitionHistory' = AppendTransition("T16.RejectCandidate")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, invalidationClosed, noUnderInvalidation, rebindRequired,
      dirtyNeeded, orderSelected, stablePriorReads, evaluating, candidateProduced,
      oxfmlNoDirectPublish, traceCalcObservableRefinement, published, verifiedClean,
      traceEmitted, exactBlockers>>

VerifiedClean ==
  /\ phase = "evaluating"
  /\ phase' = "verified_clean"
  /\ verifiedClean' = TRUE
  /\ candidateProduced' = FALSE
  /\ transitionHistory' = AppendTransition("T14.VerifyClean")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, invalidationClosed, noUnderInvalidation, rebindRequired,
      dirtyNeeded, orderSelected, stablePriorReads, evaluating, oxfmlNoDirectPublish,
      traceCalcObservableRefinement, published, rejected, traceEmitted, exactBlockers>>

ProduceCandidate ==
  /\ phase = "evaluating"
  /\ phase' = "candidate_ready"
  /\ candidateProduced' = TRUE
  /\ transitionHistory' = AppendTransition("T15.ProduceCandidate")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, invalidationClosed, noUnderInvalidation, rebindRequired,
      dirtyNeeded, orderSelected, stablePriorReads, evaluating, oxfmlNoDirectPublish,
      traceCalcObservableRefinement, published, rejected, verifiedClean, traceEmitted,
      exactBlockers>>

PublishCandidate ==
  /\ phase = "candidate_ready"
  /\ candidateProduced = TRUE
  /\ rebindRequired = FALSE
  /\ phase' = "published"
  /\ published' = TRUE
  /\ traceCalcObservableRefinement' = TRUE
  /\ transitionHistory' = AppendTransition("T17.PublishCandidate")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, invalidationClosed, noUnderInvalidation, rebindRequired,
      dirtyNeeded, orderSelected, stablePriorReads, evaluating, candidateProduced,
      oxfmlNoDirectPublish, rejected, verifiedClean, traceEmitted, exactBlockers>>

EmitTrace ==
  /\ phase \in {"published", "rejected", "verified_clean"}
  /\ phase' = "traced"
  /\ traceEmitted' = TRUE
  /\ transitionHistory' = AppendTransition("T18.EmitTraceAndEvidence")
  /\ UNCHANGED <<prepared, descriptorsLowered, graphBuilt, reverseConverse,
      diagnosticsPreserved, invalidationClosed, noUnderInvalidation, rebindRequired,
      dirtyNeeded, orderSelected, stablePriorReads, evaluating, candidateProduced,
      oxfmlNoDirectPublish, traceCalcObservableRefinement, published, rejected,
      verifiedClean, exactBlockers>>

TerminalStutter ==
  /\ phase = "traced"
  /\ UNCHANGED vars

Next ==
  \/ PrepareFormula
  \/ LowerDescriptors
  \/ BuildGraph
  \/ CloseInvalidationNoRebind
  \/ CloseInvalidationWithRebind
  \/ MarkDirtyNeeded
  \/ RejectForRebind
  \/ SelectOrder
  \/ BeginEvaluate
  \/ FormulaReject
  \/ VerifiedClean
  \/ ProduceCandidate
  \/ PublishCandidate
  \/ EmitTrace
  \/ TerminalStutter

Spec == Init /\ [][Next]_vars

TypeInvariant ==
  /\ phase \in Phases
  /\ prepared \in BOOLEAN
  /\ descriptorsLowered \in BOOLEAN
  /\ graphBuilt \in BOOLEAN
  /\ reverseConverse \in BOOLEAN
  /\ diagnosticsPreserved \in BOOLEAN
  /\ invalidationClosed \in BOOLEAN
  /\ noUnderInvalidation \in BOOLEAN
  /\ rebindRequired \in BOOLEAN
  /\ dirtyNeeded \in BOOLEAN
  /\ orderSelected \in BOOLEAN
  /\ stablePriorReads \in BOOLEAN
  /\ evaluating \in BOOLEAN
  /\ candidateProduced \in BOOLEAN
  /\ oxfmlNoDirectPublish \in BOOLEAN
  /\ traceCalcObservableRefinement \in BOOLEAN
  /\ published \in BOOLEAN
  /\ rejected \in BOOLEAN
  /\ verifiedClean \in BOOLEAN
  /\ traceEmitted \in BOOLEAN
  /\ exactBlockers \subseteq STRING

GraphFactsBeforeInvalidation ==
  invalidationClosed => graphBuilt /\ reverseConverse /\ diagnosticsPreserved

DirtyNeededAfterInvalidation ==
  dirtyNeeded => invalidationClosed /\ noUnderInvalidation

OrderAfterDirtyNoRebind ==
  orderSelected => dirtyNeeded /\ ~rebindRequired

EvaluationReadsAfterOrder ==
  evaluating => orderSelected /\ stablePriorReads /\ oxfmlNoDirectPublish

CandidateAfterEvaluation ==
  candidateProduced => evaluating /\ stablePriorReads /\ oxfmlNoDirectPublish

PublishedRequiresIntegratedSpine ==
  published =>
    /\ prepared
    /\ descriptorsLowered
    /\ graphBuilt
    /\ reverseConverse
    /\ diagnosticsPreserved
    /\ invalidationClosed
    /\ noUnderInvalidation
    /\ ~rebindRequired
    /\ dirtyNeeded
    /\ orderSelected
    /\ stablePriorReads
    /\ candidateProduced
    /\ oxfmlNoDirectPublish
    /\ traceCalcObservableRefinement

RejectNoPublish ==
  rejected => ~published

VerifiedCleanNoPublish ==
  verifiedClean => ~published /\ ~candidateProduced

RebindNoPublish ==
  rebindRequired => ~published

TraceAfterTerminal ==
  traceEmitted => published \/ rejected \/ verifiedClean

NoPrematureTrace ==
  phase = "traced" => traceEmitted

ExplorationConstraint ==
  Len(transitionHistory) <= MaxTransitions

====
