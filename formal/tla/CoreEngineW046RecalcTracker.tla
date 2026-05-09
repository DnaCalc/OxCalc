---- MODULE CoreEngineW046RecalcTracker ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS MaxTransitions

Node == "A"

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

DecisionKinds == {
  "verify_clean",
  "cycle_blocked",
  "produce_candidate",
  "dependency_shape_update",
  "admit_candidate",
  "record_accepted_candidate",
  "reject",
  "publish"
}

PublishedVersions == 0..MaxTransitions

ASSUME /\ MaxTransitions \in Nat
       /\ MaxTransitions > 0

VARIABLES
  nodeState,
  demandSet,
  executionProtected,
  executionEvictionEligible,
  dynamicDependencyOverlay,
  capabilityFence,
  candidatePayload,
  inFlightCandidate,
  acceptedCandidate,
  publishedVersion,
  publicationCount,
  rejectCount,
  decisionHistory,
  transitionHistory

vars == <<
  nodeState,
  demandSet,
  executionProtected,
  executionEvictionEligible,
  dynamicDependencyOverlay,
  capabilityFence,
  candidatePayload,
  inFlightCandidate,
  acceptedCandidate,
  publishedVersion,
  publicationCount,
  rejectCount,
  decisionHistory,
  transitionHistory
>>

DecisionType ==
  [
    kind : DecisionKinds,
    beforePublishedVersion : PublishedVersions,
    afterPublishedVersion : PublishedVersions,
    acceptedBefore : BOOLEAN,
    publicationAdvanced : BOOLEAN
  ]

DecisionRecord(kind, beforeVersion, afterVersion, acceptedBeforeFlag, advanced) ==
  [
    kind |-> kind,
    beforePublishedVersion |-> beforeVersion,
    afterPublishedVersion |-> afterVersion,
    acceptedBefore |-> acceptedBeforeFlag,
    publicationAdvanced |-> advanced
  ]

AppendDecision(kind, beforeVersion, afterVersion, acceptedBeforeFlag, advanced) ==
  Append(decisionHistory, DecisionRecord(kind, beforeVersion, afterVersion, acceptedBeforeFlag, advanced))

AppendTransition(label) == Append(transitionHistory, label)

Init ==
  /\ nodeState = "clean"
  /\ demandSet = {}
  /\ executionProtected = FALSE
  /\ executionEvictionEligible = FALSE
  /\ dynamicDependencyOverlay = FALSE
  /\ capabilityFence = FALSE
  /\ candidatePayload = FALSE
  /\ inFlightCandidate = FALSE
  /\ acceptedCandidate = FALSE
  /\ publishedVersion = 0
  /\ publicationCount = 0
  /\ rejectCount = 0
  /\ decisionHistory = <<>>
  /\ transitionHistory = <<>>

MarkDirty ==
  /\ nodeState' = "dirty_pending"
  /\ executionProtected' = TRUE
  /\ executionEvictionEligible' = FALSE
  /\ transitionHistory' = AppendTransition("mark_dirty")
  /\ UNCHANGED <<demandSet, dynamicDependencyOverlay, capabilityFence, candidatePayload,
                 inFlightCandidate, acceptedCandidate, publishedVersion, publicationCount,
                 rejectCount, decisionHistory>>

MarkNeeded ==
  /\ nodeState = "dirty_pending"
  /\ nodeState' = "needed"
  /\ demandSet' = demandSet \cup {Node}
  /\ executionProtected' = TRUE
  /\ executionEvictionEligible' = FALSE
  /\ transitionHistory' = AppendTransition("mark_needed")
  /\ UNCHANGED <<dynamicDependencyOverlay, capabilityFence, candidatePayload,
                 inFlightCandidate, acceptedCandidate, publishedVersion, publicationCount,
                 rejectCount, decisionHistory>>

RecordCycleBlockedFromClosure ==
  /\ nodeState' = "cycle_blocked"
  /\ demandSet' = demandSet \cup {Node}
  /\ executionProtected' = TRUE
  /\ executionEvictionEligible' = FALSE
  /\ decisionHistory' =
       AppendDecision("cycle_blocked", publishedVersion, publishedVersion, acceptedCandidate, FALSE)
  /\ transitionHistory' = AppendTransition("record_cycle_blocked_from_closure")
  /\ UNCHANGED <<dynamicDependencyOverlay, capabilityFence, candidatePayload,
                 inFlightCandidate, acceptedCandidate, publishedVersion, publicationCount,
                 rejectCount>>

BeginEvaluate ==
  /\ nodeState = "needed"
  /\ nodeState' = "evaluating"
  /\ capabilityFence' = TRUE
  /\ executionProtected' = TRUE
  /\ executionEvictionEligible' = FALSE
  /\ transitionHistory' = AppendTransition("begin_evaluate")
  /\ UNCHANGED <<demandSet, dynamicDependencyOverlay, candidatePayload,
                 inFlightCandidate, acceptedCandidate, publishedVersion, publicationCount,
                 rejectCount, decisionHistory>>

VerifyClean ==
  /\ nodeState = "evaluating"
  /\ nodeState' = "verified_clean"
  /\ demandSet' = demandSet \ {Node}
  /\ executionProtected' = TRUE
  /\ executionEvictionEligible' = FALSE
  /\ decisionHistory' =
       AppendDecision("verify_clean", publishedVersion, publishedVersion, acceptedCandidate, FALSE)
  /\ transitionHistory' = AppendTransition("verify_clean")
  /\ UNCHANGED <<dynamicDependencyOverlay, capabilityFence, candidatePayload,
                 inFlightCandidate, acceptedCandidate, publishedVersion, publicationCount,
                 rejectCount>>

ProduceCandidateResult ==
  /\ nodeState = "evaluating"
  /\ nodeState' = "publish_ready"
  /\ candidatePayload' = TRUE
  /\ executionProtected' = TRUE
  /\ executionEvictionEligible' = FALSE
  /\ decisionHistory' =
       AppendDecision("produce_candidate", publishedVersion, publishedVersion, acceptedCandidate, FALSE)
  /\ transitionHistory' = AppendTransition("produce_candidate_result")
  /\ UNCHANGED <<demandSet, dynamicDependencyOverlay, capabilityFence,
                 inFlightCandidate, acceptedCandidate, publishedVersion, publicationCount,
                 rejectCount>>

ProduceDependencyShapeUpdate ==
  /\ nodeState = "evaluating"
  /\ nodeState' = "publish_ready"
  /\ dynamicDependencyOverlay' = TRUE
  /\ candidatePayload' = TRUE
  /\ executionProtected' = TRUE
  /\ executionEvictionEligible' = FALSE
  /\ decisionHistory' =
       AppendDecision("dependency_shape_update", publishedVersion, publishedVersion, acceptedCandidate, FALSE)
  /\ transitionHistory' = AppendTransition("produce_dependency_shape_update")
  /\ UNCHANGED <<demandSet, capabilityFence, inFlightCandidate, acceptedCandidate,
                 publishedVersion, publicationCount, rejectCount>>

RejectOrFallback ==
  /\ nodeState \in {"evaluating", "publish_ready"}
  /\ nodeState' = "rejected_pending_repair"
  /\ demandSet' = demandSet \cup {Node}
  /\ dynamicDependencyOverlay' = FALSE
  /\ executionProtected' = TRUE
  /\ executionEvictionEligible' = FALSE
  /\ decisionHistory' =
       AppendDecision("reject", publishedVersion, publishedVersion, acceptedCandidate, FALSE)
  /\ transitionHistory' = AppendTransition("reject_or_fallback")
  /\ UNCHANGED <<capabilityFence, candidatePayload, inFlightCandidate, acceptedCandidate,
                 publishedVersion, publicationCount, rejectCount>>

ReenterRejectedPendingRepair ==
  /\ nodeState = "rejected_pending_repair"
  /\ nodeState' = "needed"
  /\ executionProtected' = TRUE
  /\ executionEvictionEligible' = FALSE
  /\ transitionHistory' = AppendTransition("reenter_rejected_pending_repair")
  /\ UNCHANGED <<demandSet, dynamicDependencyOverlay, capabilityFence, candidatePayload,
                 inFlightCandidate, acceptedCandidate, publishedVersion, publicationCount,
                 rejectCount, decisionHistory>>

TrackerPublishAndClear ==
  /\ nodeState = "publish_ready"
  /\ nodeState' = "clean"
  /\ demandSet' = demandSet \ {Node}
  /\ executionProtected' = FALSE
  /\ executionEvictionEligible' = TRUE
  /\ transitionHistory' = AppendTransition("publish_and_clear_tracker")
  /\ UNCHANGED <<dynamicDependencyOverlay, capabilityFence, candidatePayload,
                 inFlightCandidate, acceptedCandidate, publishedVersion, publicationCount,
                 rejectCount, decisionHistory>>

ReleaseAndEvictEligible ==
  /\ nodeState \in {"clean", "verified_clean"}
  /\ demandSet' = demandSet \ {Node}
  /\ executionProtected' = FALSE
  /\ executionEvictionEligible' = TRUE
  /\ transitionHistory' = AppendTransition("release_and_evict_eligible")
  /\ UNCHANGED <<nodeState, dynamicDependencyOverlay, capabilityFence, candidatePayload,
                 inFlightCandidate, acceptedCandidate, publishedVersion, publicationCount,
                 rejectCount, decisionHistory>>

AdmitCandidateWork ==
  /\ inFlightCandidate' = TRUE
  /\ decisionHistory' =
       AppendDecision("admit_candidate", publishedVersion, publishedVersion, acceptedCandidate, FALSE)
  /\ transitionHistory' = AppendTransition("admit_candidate_work")
  /\ UNCHANGED <<nodeState, demandSet, executionProtected, executionEvictionEligible,
                 dynamicDependencyOverlay, capabilityFence, candidatePayload, acceptedCandidate,
                 publishedVersion, publicationCount, rejectCount>>

RecordAcceptedCandidateResult ==
  /\ inFlightCandidate
  /\ acceptedCandidate' = TRUE
  /\ decisionHistory' =
       AppendDecision("record_accepted_candidate", publishedVersion, publishedVersion, acceptedCandidate, FALSE)
  /\ transitionHistory' = AppendTransition("record_accepted_candidate_result")
  /\ UNCHANGED <<nodeState, demandSet, executionProtected, executionEvictionEligible,
                 dynamicDependencyOverlay, capabilityFence, candidatePayload, inFlightCandidate,
                 publishedVersion, publicationCount, rejectCount>>

RejectCandidateWork ==
  /\ inFlightCandidate \/ acceptedCandidate
  /\ inFlightCandidate' = FALSE
  /\ acceptedCandidate' = FALSE
  /\ rejectCount' = rejectCount + 1
  /\ decisionHistory' =
       AppendDecision("reject", publishedVersion, publishedVersion, acceptedCandidate, FALSE)
  /\ transitionHistory' = AppendTransition("reject_candidate_work")
  /\ UNCHANGED <<nodeState, demandSet, executionProtected, executionEvictionEligible,
                 dynamicDependencyOverlay, capabilityFence, candidatePayload, publishedVersion,
                 publicationCount>>

AcceptAndPublish ==
  /\ acceptedCandidate
  /\ publishedVersion < MaxTransitions
  /\ publishedVersion' = publishedVersion + 1
  /\ publicationCount' = publicationCount + 1
  /\ inFlightCandidate' = FALSE
  /\ acceptedCandidate' = FALSE
  /\ decisionHistory' =
       AppendDecision("publish", publishedVersion, publishedVersion + 1, TRUE, TRUE)
  /\ transitionHistory' = AppendTransition("accept_and_publish")
  /\ UNCHANGED <<nodeState, demandSet, executionProtected, executionEvictionEligible,
                 dynamicDependencyOverlay, capabilityFence, candidatePayload, rejectCount>>

Next ==
  \/ MarkDirty
  \/ MarkNeeded
  \/ RecordCycleBlockedFromClosure
  \/ BeginEvaluate
  \/ VerifyClean
  \/ ProduceCandidateResult
  \/ ProduceDependencyShapeUpdate
  \/ RejectOrFallback
  \/ ReenterRejectedPendingRepair
  \/ TrackerPublishAndClear
  \/ ReleaseAndEvictEligible
  \/ AdmitCandidateWork
  \/ RecordAcceptedCandidateResult
  \/ RejectCandidateWork
  \/ AcceptAndPublish

Spec == Init /\ [][Next]_vars

TypeInvariant ==
  /\ nodeState \in NodeStates
  /\ demandSet \subseteq {Node}
  /\ executionProtected \in BOOLEAN
  /\ executionEvictionEligible \in BOOLEAN
  /\ dynamicDependencyOverlay \in BOOLEAN
  /\ capabilityFence \in BOOLEAN
  /\ candidatePayload \in BOOLEAN
  /\ inFlightCandidate \in BOOLEAN
  /\ acceptedCandidate \in BOOLEAN
  /\ publishedVersion \in PublishedVersions
  /\ publicationCount \in PublishedVersions
  /\ rejectCount \in PublishedVersions
  /\ decisionHistory \in Seq(DecisionType)

DemandClearedForVerifiedClean ==
  nodeState = "verified_clean" => Node \notin demandSet

RejectedPendingRepairRetainsDemand ==
  nodeState = "rejected_pending_repair" => Node \in demandSet

CycleBlockedRetainsDemand ==
  nodeState = "cycle_blocked" => Node \in demandSet

PublishReadyHasCandidateSignal ==
  nodeState = "publish_ready" => candidatePayload \/ dynamicDependencyOverlay

RejectIsNoPublish ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind = "reject" =>
      /\ decisionHistory[i].beforePublishedVersion = decisionHistory[i].afterPublishedVersion
      /\ decisionHistory[i].publicationAdvanced = FALSE

VerifiedCleanIsNoPublish ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind = "verify_clean" =>
      /\ decisionHistory[i].beforePublishedVersion = decisionHistory[i].afterPublishedVersion
      /\ decisionHistory[i].publicationAdvanced = FALSE

CycleBlockedIsNoPublish ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind = "cycle_blocked" =>
      /\ decisionHistory[i].beforePublishedVersion = decisionHistory[i].afterPublishedVersion
      /\ decisionHistory[i].publicationAdvanced = FALSE

CandidateStepsAreNoPublish ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind \in
      {"produce_candidate", "dependency_shape_update", "admit_candidate", "record_accepted_candidate"} =>
        /\ decisionHistory[i].beforePublishedVersion = decisionHistory[i].afterPublishedVersion
        /\ decisionHistory[i].publicationAdvanced = FALSE

PublishRequiresAcceptedCandidate ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].kind = "publish" =>
      /\ decisionHistory[i].acceptedBefore = TRUE
      /\ decisionHistory[i].publicationAdvanced = TRUE
      /\ decisionHistory[i].afterPublishedVersion = decisionHistory[i].beforePublishedVersion + 1

PublicationOnlyFromAcceptedCandidate ==
  \A i \in 1..Len(decisionHistory) :
    decisionHistory[i].publicationAdvanced = TRUE =>
      /\ decisionHistory[i].kind = "publish"
      /\ decisionHistory[i].acceptedBefore = TRUE

ExplorationConstraint ==
  Len(transitionHistory) <= MaxTransitions

====
