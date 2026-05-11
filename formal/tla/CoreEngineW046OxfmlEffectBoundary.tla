---- MODULE CoreEngineW046OxfmlEffectBoundary ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS MaxTransitions

Phases == {
  "prepare_formula",
  "lower_descriptors",
  "graph_build",
  "rebind_gate",
  "evaluate_formula",
  "publication",
  "trace_projection"
}

EffectKinds == {
  "ReadValue",
  "ResolveStatic",
  "ResolveDynamic",
  "EmitDependency",
  "EmitDiagnostic",
  "CallFunction",
  "BindLocal",
  "EnterLambda",
  "ProduceCandidate",
  "RejectCandidate",
  "Publish",
  "EmitFormatDelta",
  "EmitDisplayDelta",
  "SurfaceRuntimeFact"
}

BoundaryFacts == {
  "candidate_result",
  "commit_bundle",
  "reject_record",
  "runtime_effect",
  "format_delta",
  "display_delta",
  "let_binding",
  "lambda_carrier",
  "registered_external_packet"
}

VARIABLES
  phase,
  effects,
  boundaryFacts,
  publicationChanged,
  generalOxFuncOpaque,
  exactBlockers,
  transitionHistory

vars == <<
  phase,
  effects,
  boundaryFacts,
  publicationChanged,
  generalOxFuncOpaque,
  exactBlockers,
  transitionHistory
>>

AppendTransition(label) == Append(transitionHistory, label)

Init ==
  /\ phase = "prepare_formula"
  /\ effects = {}
  /\ boundaryFacts = {}
  /\ publicationChanged = FALSE
  /\ generalOxFuncOpaque = TRUE
  /\ exactBlockers = {}
  /\ transitionHistory = <<>>

AllowedEffectsForPhase(p) ==
  CASE p = "prepare_formula" -> {"ResolveStatic", "EmitDiagnostic", "BindLocal", "EnterLambda"}
    [] p = "lower_descriptors" -> {"EmitDependency", "EmitDiagnostic"}
    [] p = "graph_build" -> {"EmitDependency", "EmitDiagnostic"}
    [] p = "rebind_gate" -> {"ResolveDynamic", "EmitDependency", "RejectCandidate"}
    [] p = "evaluate_formula" -> {
         "ReadValue", "CallFunction", "BindLocal", "EnterLambda",
         "ProduceCandidate", "RejectCandidate", "EmitDiagnostic",
         "EmitFormatDelta", "EmitDisplayDelta", "SurfaceRuntimeFact"}
    [] p = "publication" -> {"Publish", "RejectCandidate"}
    [] p = "trace_projection" -> {"SurfaceRuntimeFact"}
    [] OTHER -> {}

FormulaLetLambdaCandidate ==
  /\ phase = "prepare_formula"
  /\ phase' = "evaluate_formula"
  /\ effects' = {"ReadValue", "BindLocal", "EnterLambda", "CallFunction", "ProduceCandidate"}
  /\ boundaryFacts' = {"candidate_result", "let_binding", "lambda_carrier"}
  /\ publicationChanged' = FALSE
  /\ generalOxFuncOpaque' = TRUE
  /\ exactBlockers' = {}
  /\ transitionHistory' = AppendTransition("formula_let_lambda_candidate")

FormulaRejectRuntimeEffect ==
  /\ phase = "prepare_formula"
  /\ phase' = "evaluate_formula"
  /\ effects' = {"SurfaceRuntimeFact", "RejectCandidate"}
  /\ boundaryFacts' = {"reject_record", "runtime_effect"}
  /\ publicationChanged' = FALSE
  /\ generalOxFuncOpaque' = TRUE
  /\ exactBlockers' = {}
  /\ transitionHistory' = AppendTransition("formula_reject_runtime_effect")

FormulaFormatDisplayCandidate ==
  /\ phase = "prepare_formula"
  /\ phase' = "evaluate_formula"
  /\ effects' = {"EmitFormatDelta", "EmitDisplayDelta", "ProduceCandidate"}
  /\ boundaryFacts' = {"candidate_result", "format_delta", "display_delta"}
  /\ publicationChanged' = FALSE
  /\ generalOxFuncOpaque' = TRUE
  /\ exactBlockers' = {}
  /\ transitionHistory' = AppendTransition("formula_format_display_candidate")

DynamicRebindReject ==
  /\ phase = "prepare_formula"
  /\ phase' = "rebind_gate"
  /\ effects' = {"ResolveDynamic", "EmitDependency", "RejectCandidate"}
  /\ boundaryFacts' = {"runtime_effect", "reject_record"}
  /\ publicationChanged' = FALSE
  /\ generalOxFuncOpaque' = TRUE
  /\ exactBlockers' = {}
  /\ transitionHistory' = AppendTransition("dynamic_rebind_reject")

CoordinatorPublication ==
  /\ phase = "prepare_formula"
  /\ phase' = "publication"
  /\ effects' = {"Publish"}
  /\ boundaryFacts' = {"commit_bundle"}
  /\ publicationChanged' = TRUE
  /\ generalOxFuncOpaque' = TRUE
  /\ exactBlockers' = {}
  /\ transitionHistory' = AppendTransition("coordinator_publication")

RegisteredExternalWatchOnly ==
  /\ phase = "prepare_formula"
  /\ phase' = "evaluate_formula"
  /\ effects' = {"CallFunction", "SurfaceRuntimeFact"}
  /\ boundaryFacts' = {"registered_external_packet", "runtime_effect"}
  /\ publicationChanged' = FALSE
  /\ generalOxFuncOpaque' = TRUE
  /\ exactBlockers' = {"registered_external_publication_consequence_breadth_not_frozen"}
  /\ transitionHistory' = AppendTransition("registered_external_watch_only")

TerminalStutter ==
  /\ Len(transitionHistory) > 0
  /\ UNCHANGED vars

Next ==
  \/ FormulaLetLambdaCandidate
  \/ FormulaRejectRuntimeEffect
  \/ FormulaFormatDisplayCandidate
  \/ DynamicRebindReject
  \/ CoordinatorPublication
  \/ RegisteredExternalWatchOnly
  \/ TerminalStutter

Spec == Init /\ [][Next]_vars

TypeInvariant ==
  /\ phase \in Phases
  /\ effects \subseteq EffectKinds
  /\ boundaryFacts \subseteq BoundaryFacts
  /\ publicationChanged \in BOOLEAN
  /\ generalOxFuncOpaque \in BOOLEAN
  /\ exactBlockers \subseteq STRING

PhaseAuthority ==
  effects \subseteq AllowedEffectsForPhase(phase)

NoDirectFormulaPublication ==
  phase = "evaluate_formula" =>
    /\ "Publish" \notin effects
    /\ publicationChanged = FALSE

RejectPreservesPublication ==
  "RejectCandidate" \in effects => publicationChanged = FALSE

CandidateIsNotPublication ==
  "ProduceCandidate" \in effects => publicationChanged = FALSE

PublicationOnlyByCoordinator ==
  publicationChanged => phase = "publication" /\ effects = {"Publish"}

LetLambdaCarrierKeepsOxFuncOpaque ==
  ({"let_binding", "lambda_carrier"} \cap boundaryFacts) # {} => generalOxFuncOpaque = TRUE

FormatDisplayBoundaryDistinct ==
  "format_delta" \in boundaryFacts /\ "display_delta" \in boundaryFacts =>
    "format_delta" # "display_delta"

RuntimeEffectIsNotPublication ==
  "runtime_effect" \in boundaryFacts /\ "Publish" \notin effects => publicationChanged = FALSE

WatchRowsHaveExactBlockers ==
  "registered_external_packet" \in boundaryFacts => exactBlockers # {}

ExplorationConstraint ==
  Len(transitionHistory) <= MaxTransitions

====
