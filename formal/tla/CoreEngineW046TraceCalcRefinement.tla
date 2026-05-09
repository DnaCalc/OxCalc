---- MODULE CoreEngineW046TraceCalcRefinement ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS MaxTransitions

Scenarios == {
  "accept_publish",
  "verify_clean",
  "reject_no_publish",
  "dynamic_dependency",
  "invalidation_closure"
}

Results == {"passed"}
PublicationDecisions == {"published", "no_publish"}

VARIABLES
  phase,
  scenario,
  covered,
  exactBlockers,
  oracleResult,
  engineResult,
  oracleValues,
  engineValues,
  oracleDiagnostics,
  engineDiagnostics,
  oracleDependencyEffects,
  engineDependencyEffects,
  oracleInvalidations,
  engineInvalidations,
  oracleRejects,
  engineRejects,
  oraclePublication,
  enginePublication,
  oracleTraceFamilies,
  engineTraceFamilies,
  transitionHistory

vars == <<
  phase,
  scenario,
  covered,
  exactBlockers,
  oracleResult,
  engineResult,
  oracleValues,
  engineValues,
  oracleDiagnostics,
  engineDiagnostics,
  oracleDependencyEffects,
  engineDependencyEffects,
  oracleInvalidations,
  engineInvalidations,
  oracleRejects,
  engineRejects,
  oraclePublication,
  enginePublication,
  oracleTraceFamilies,
  engineTraceFamilies,
  transitionHistory
>>

AppendTransition(label) == Append(transitionHistory, label)

Init ==
  /\ phase = "start"
  /\ scenario = "accept_publish"
  /\ covered = FALSE
  /\ exactBlockers = {}
  /\ oracleResult = "passed"
  /\ engineResult = "passed"
  /\ oracleValues = {}
  /\ engineValues = {}
  /\ oracleDiagnostics = {}
  /\ engineDiagnostics = {}
  /\ oracleDependencyEffects = {}
  /\ engineDependencyEffects = {}
  /\ oracleInvalidations = {}
  /\ engineInvalidations = {}
  /\ oracleRejects = {}
  /\ engineRejects = {}
  /\ oraclePublication = "no_publish"
  /\ enginePublication = "no_publish"
  /\ oracleTraceFamilies = {}
  /\ engineTraceFamilies = {}
  /\ transitionHistory = <<>>

BindAcceptPublish ==
  /\ phase = "start"
  /\ phase' = "terminal"
  /\ scenario' = "accept_publish"
  /\ covered' = TRUE
  /\ exactBlockers' = {}
  /\ oracleResult' = "passed"
  /\ engineResult' = "passed"
  /\ oracleValues' = {"B=2"}
  /\ engineValues' = {"B=2"}
  /\ oracleDiagnostics' = {}
  /\ engineDiagnostics' = {}
  /\ oracleDependencyEffects' = {}
  /\ engineDependencyEffects' = {}
  /\ oracleInvalidations' = {}
  /\ engineInvalidations' = {}
  /\ oracleRejects' = {}
  /\ engineRejects' = {}
  /\ oraclePublication' = "published"
  /\ enginePublication' = "published"
  /\ oracleTraceFamilies' =
       {"candidate.admitted", "candidate.built", "publication.committed"}
  /\ engineTraceFamilies' =
       {"candidate.admitted", "candidate.built", "publication.committed"}
  /\ transitionHistory' = AppendTransition("bind_accept_publish")

BindVerifiedClean ==
  /\ phase = "start"
  /\ phase' = "terminal"
  /\ scenario' = "verify_clean"
  /\ covered' = TRUE
  /\ exactBlockers' = {}
  /\ oracleResult' = "passed"
  /\ engineResult' = "passed"
  /\ oracleValues' = {"A=10"}
  /\ engineValues' = {"A=10"}
  /\ oracleDiagnostics' = {}
  /\ engineDiagnostics' = {}
  /\ oracleDependencyEffects' = {}
  /\ engineDependencyEffects' = {}
  /\ oracleInvalidations' = {}
  /\ engineInvalidations' = {}
  /\ oracleRejects' = {}
  /\ engineRejects' = {}
  /\ oraclePublication' = "no_publish"
  /\ enginePublication' = "no_publish"
  /\ oracleTraceFamilies' = {"candidate.verified_clean"}
  /\ engineTraceFamilies' = {"candidate.verified_clean", "node_verified_clean"}
  /\ transitionHistory' = AppendTransition("bind_verified_clean")

BindRejectNoPublish ==
  /\ phase = "start"
  /\ phase' = "terminal"
  /\ scenario' = "reject_no_publish"
  /\ covered' = TRUE
  /\ exactBlockers' = {}
  /\ oracleResult' = "passed"
  /\ engineResult' = "passed"
  /\ oracleValues' = {"A=2"}
  /\ engineValues' = {"A=2"}
  /\ oracleDiagnostics' = {"capability denied"}
  /\ engineDiagnostics' = {"capability denied"}
  /\ oracleDependencyEffects' = {}
  /\ engineDependencyEffects' = {}
  /\ oracleInvalidations' = {}
  /\ engineInvalidations' = {}
  /\ oracleRejects' = {"capability_mismatch"}
  /\ engineRejects' = {"capability_mismatch"}
  /\ oraclePublication' = "no_publish"
  /\ enginePublication' = "no_publish"
  /\ oracleTraceFamilies' = {"candidate.admitted", "reject.issued"}
  /\ engineTraceFamilies' = {"candidate.admitted", "reject.issued"}
  /\ transitionHistory' = AppendTransition("bind_reject_no_publish")

BindDynamicDependency ==
  /\ phase = "start"
  /\ phase' = "terminal"
  /\ scenario' = "dynamic_dependency"
  /\ covered' = TRUE
  /\ exactBlockers' = {}
  /\ oracleResult' = "passed"
  /\ engineResult' = "passed"
  /\ oracleValues' = {"Out=20"}
  /\ engineValues' = {"Out=20"}
  /\ oracleDiagnostics' = {}
  /\ engineDiagnostics' = {}
  /\ oracleDependencyEffects' = {"DynamicDependency:Out"}
  /\ engineDependencyEffects' = {"DynamicDependency:Out", "Instrumentation:Out"}
  /\ oracleInvalidations' = {"Out:DependencyShapeChanged"}
  /\ engineInvalidations' = {"Out:DependencyShapeChanged"}
  /\ oracleRejects' = {}
  /\ engineRejects' = {}
  /\ oraclePublication' = "published"
  /\ enginePublication' = "published"
  /\ oracleTraceFamilies' = {"candidate.built", "publication.committed"}
  /\ engineTraceFamilies' =
       {"candidate.built", "publication.committed", "oxcalc.local.event.extra"}
  /\ transitionHistory' = AppendTransition("bind_dynamic_dependency")

BindInvalidationClosure ==
  /\ phase = "start"
  /\ phase' = "terminal"
  /\ scenario' = "invalidation_closure"
  /\ covered' = TRUE
  /\ exactBlockers' = {}
  /\ oracleResult' = "passed"
  /\ engineResult' = "passed"
  /\ oracleValues' = {"Y=60", "Z=63"}
  /\ engineValues' = {"Y=60", "Z=63"}
  /\ oracleDiagnostics' = {}
  /\ engineDiagnostics' = {}
  /\ oracleDependencyEffects' = {}
  /\ engineDependencyEffects' = {}
  /\ oracleInvalidations' = {"Y:StructuralRecalcOnly", "Z:StructuralRecalcOnly"}
  /\ engineInvalidations' =
       {"Y:StructuralRecalcOnly", "Z:StructuralRecalcOnly", "extra:conservative"}
  /\ oracleRejects' = {}
  /\ engineRejects' = {}
  /\ oraclePublication' = "published"
  /\ enginePublication' = "published"
  /\ oracleTraceFamilies' = {"topo_group_scheduled", "publication.committed"}
  /\ engineTraceFamilies' =
       {"topo_group_scheduled", "publication.committed", "evaluate_node"}
  /\ transitionHistory' = AppendTransition("bind_invalidation_closure")

TerminalStutter ==
  /\ phase = "terminal"
  /\ UNCHANGED vars

Next ==
  \/ BindAcceptPublish
  \/ BindVerifiedClean
  \/ BindRejectNoPublish
  \/ BindDynamicDependency
  \/ BindInvalidationClosure
  \/ TerminalStutter

Spec == Init /\ [][Next]_vars

TypeInvariant ==
  /\ phase \in {"start", "terminal"}
  /\ scenario \in Scenarios
  /\ covered \in BOOLEAN
  /\ exactBlockers \subseteq STRING
  /\ oracleResult \in Results
  /\ engineResult \in Results
  /\ oracleValues \subseteq STRING
  /\ engineValues \subseteq STRING
  /\ oracleDiagnostics \subseteq STRING
  /\ engineDiagnostics \subseteq STRING
  /\ oracleDependencyEffects \subseteq STRING
  /\ engineDependencyEffects \subseteq STRING
  /\ oracleInvalidations \subseteq STRING
  /\ engineInvalidations \subseteq STRING
  /\ oracleRejects \subseteq STRING
  /\ engineRejects \subseteq STRING
  /\ oraclePublication \in PublicationDecisions
  /\ enginePublication \in PublicationDecisions
  /\ oracleTraceFamilies \subseteq STRING
  /\ engineTraceFamilies \subseteq STRING

CoveredRowsHaveNoExactBlockers ==
  covered => exactBlockers = {}

ResultStateMatches ==
  covered => engineResult = oracleResult

PublishedValuesMatch ==
  covered => engineValues = oracleValues

DiagnosticsMatch ==
  covered => engineDiagnostics = oracleDiagnostics

RejectsMatch ==
  covered => engineRejects = oracleRejects

DependencyEffectsPreserved ==
  covered => oracleDependencyEffects \subseteq engineDependencyEffects

InvalidationRecordsPreserved ==
  covered => oracleInvalidations \subseteq engineInvalidations

PublicationDecisionMatches ==
  covered => enginePublication = oraclePublication

RequiredTraceFamiliesPreserved ==
  covered => oracleTraceFamilies \subseteq engineTraceFamilies

RejectIsNoPublish ==
  covered /\ oracleRejects # {} => enginePublication = "no_publish"

NoSemanticMismatchForCoveredRows ==
  covered =>
    /\ ResultStateMatches
    /\ PublishedValuesMatch
    /\ DiagnosticsMatch
    /\ RejectsMatch
    /\ DependencyEffectsPreserved
    /\ InvalidationRecordsPreserved
    /\ PublicationDecisionMatches
    /\ RequiredTraceFamiliesPreserved

ExplorationConstraint ==
  Len(transitionHistory) <= MaxTransitions

====
