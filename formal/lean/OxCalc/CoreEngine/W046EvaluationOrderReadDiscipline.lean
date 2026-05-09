import Std

namespace OxCalc.CoreEngine.W046.EvaluationOrderReadDiscipline

abbrev NodeId := Nat

structure DependencyEdge where
  owner : NodeId
  target : NodeId
  deriving DecidableEq, Repr

def BeforeInOrder (order : List NodeId) (target owner : NodeId) : Prop :=
  ∃ left middle right,
    order = left ++ [target] ++ middle ++ [owner] ++ right

def FormulaDependencyBeforeDependent
    (order formulaOwners : List NodeId)
    (dependencyEdges : List DependencyEdge) : Prop :=
  ∀ edge,
    edge ∈ dependencyEdges ->
      edge.owner ∈ formulaOwners ->
        edge.target ∈ formulaOwners ->
          BeforeInOrder order edge.target edge.owner

def StableOrPriorRead
    (order stableInputs formulaOwners : List NodeId)
    (dependencyEdges : List DependencyEdge) : Prop :=
  ∀ edge,
    edge ∈ dependencyEdges ->
      edge.owner ∈ formulaOwners ->
        edge.target ∈ stableInputs ∨ BeforeInOrder order edge.target edge.owner

structure EvaluationOrderSemanticModel where
  dependencyEdges : List DependencyEdge
  formulaOwners : List NodeId
  stableInputs : List NodeId
  order : List NodeId
  dependencyBeforeDependent :
    FormulaDependencyBeforeDependent order formulaOwners dependencyEdges
  stableOrPriorReads :
    StableOrPriorRead order stableInputs formulaOwners dependencyEdges

structure EvaluationDecision where
  order : List NodeId
  computedNodes : List NodeId
  updatedNodes : List NodeId
  candidateTargetSet : List NodeId
  diagnosticFailure : Bool
  rejected : Bool
  published : Bool
  verifiedClean : Bool
  cycleRejected : Bool
  deriving DecidableEq, Repr

def SubsetList (xs ys : List NodeId) : Prop :=
  ∀ n, n ∈ xs -> n ∈ ys

def DiagnosticShortCircuit (decision : EvaluationDecision) : Prop :=
  decision.diagnosticFailure = true ->
    decision.rejected = true ∧ decision.published = false

def CycleRejectNoEvaluation (decision : EvaluationDecision) : Prop :=
  decision.cycleRejected = true ->
    decision.rejected = true ∧
      decision.published = false ∧
        decision.computedNodes = []

def VerifiedCleanNoPublication (decision : EvaluationDecision) : Prop :=
  decision.verifiedClean = true ->
    decision.updatedNodes = [] ∧
      decision.published = false

def NoTornCandidateBundle (decision : EvaluationDecision) : Prop :=
  decision.published = true ->
    decision.candidateTargetSet = decision.order ∧
      SubsetList decision.updatedNodes decision.candidateTargetSet

structure EvaluationDecisionSemanticModel where
  decision : EvaluationDecision
  diagnosticShortCircuit : DiagnosticShortCircuit decision
  cycleRejectNoEvaluation : CycleRejectNoEvaluation decision
  verifiedCleanNoPublication : VerifiedCleanNoPublication decision
  noTornCandidateBundle : NoTornCandidateBundle decision

theorem model_dependency_before_dependent
    (model : EvaluationOrderSemanticModel) :
    FormulaDependencyBeforeDependent
      model.order
      model.formulaOwners
      model.dependencyEdges := by
  exact model.dependencyBeforeDependent

theorem model_stable_or_prior_reads
    (model : EvaluationOrderSemanticModel) :
    StableOrPriorRead
      model.order
      model.stableInputs
      model.formulaOwners
      model.dependencyEdges := by
  exact model.stableOrPriorReads

theorem model_diagnostic_short_circuit
    (model : EvaluationDecisionSemanticModel)
    (failed : model.decision.diagnosticFailure = true) :
    model.decision.rejected = true ∧ model.decision.published = false := by
  exact model.diagnosticShortCircuit failed

theorem model_cycle_reject_no_evaluation
    (model : EvaluationDecisionSemanticModel)
    (cycle : model.decision.cycleRejected = true) :
    model.decision.rejected = true ∧
      model.decision.published = false ∧
        model.decision.computedNodes = [] := by
  exact model.cycleRejectNoEvaluation cycle

theorem model_verified_clean_no_publication
    (model : EvaluationDecisionSemanticModel)
    (clean : model.decision.verifiedClean = true) :
    model.decision.updatedNodes = [] ∧ model.decision.published = false := by
  exact model.verifiedCleanNoPublication clean

theorem model_no_torn_candidate_bundle
    (model : EvaluationDecisionSemanticModel)
    (published : model.decision.published = true) :
    model.decision.candidateTargetSet = model.decision.order ∧
      SubsetList model.decision.updatedNodes model.decision.candidateTargetSet := by
  exact model.noTornCandidateBundle published

def SampleEdges : List DependencyEdge :=
  [
    { owner := 3, target := 2 },
    { owner := 4, target := 2 },
    { owner := 4, target := 3 }
  ]

def SampleFormulaOwners : List NodeId := [3, 4]

def SampleStableInputs : List NodeId := [2]

def SampleOrder : List NodeId := [3, 4]

theorem sample_dependency_before_dependent :
    FormulaDependencyBeforeDependent
      SampleOrder
      SampleFormulaOwners
      SampleEdges := by
  intro edge edgeMem ownerMem targetMem
  unfold SampleEdges at edgeMem
  simp at edgeMem
  rcases edgeMem with edgeEq | edgeEq | edgeEq
  · subst edge
    unfold SampleFormulaOwners at targetMem
    simp at targetMem
  · subst edge
    unfold SampleFormulaOwners at targetMem
    simp at targetMem
  · subst edge
    unfold BeforeInOrder SampleOrder
    exact ⟨[], [], [], by simp⟩

theorem sample_stable_or_prior_reads :
    StableOrPriorRead
      SampleOrder
      SampleStableInputs
      SampleFormulaOwners
      SampleEdges := by
  intro edge edgeMem ownerMem
  unfold SampleEdges at edgeMem
  simp at edgeMem
  rcases edgeMem with edgeEq | edgeEq | edgeEq
  · subst edge
    left
    unfold SampleStableInputs
    simp
  · subst edge
    left
    unfold SampleStableInputs
    simp
  · subst edge
    right
    unfold BeforeInOrder SampleOrder
    exact ⟨[], [], [], by simp⟩

def SampleOrderModel : EvaluationOrderSemanticModel :=
  { dependencyEdges := SampleEdges,
    formulaOwners := SampleFormulaOwners,
    stableInputs := SampleStableInputs,
    order := SampleOrder,
    dependencyBeforeDependent := sample_dependency_before_dependent,
    stableOrPriorReads := sample_stable_or_prior_reads }

def SamplePublishedDecision : EvaluationDecision :=
  { order := SampleOrder,
    computedNodes := SampleOrder,
    updatedNodes := SampleOrder,
    candidateTargetSet := SampleOrder,
    diagnosticFailure := false,
    rejected := false,
    published := true,
    verifiedClean := false,
    cycleRejected := false }

theorem sample_published_no_torn_candidate_bundle :
    NoTornCandidateBundle SamplePublishedDecision := by
  intro published
  unfold SamplePublishedDecision at published
  simp at published
  unfold SamplePublishedDecision SubsetList
  simp

def SampleDiagnosticFailureDecision : EvaluationDecision :=
  { order := SampleOrder,
    computedNodes := [3],
    updatedNodes := [3],
    candidateTargetSet := SampleOrder,
    diagnosticFailure := true,
    rejected := true,
    published := false,
    verifiedClean := false,
    cycleRejected := false }

theorem sample_diagnostic_failure_short_circuits :
    DiagnosticShortCircuit SampleDiagnosticFailureDecision := by
  intro failure
  unfold SampleDiagnosticFailureDecision at failure
  simp at failure
  unfold SampleDiagnosticFailureDecision
  simp

def SampleVerifiedCleanDecision : EvaluationDecision :=
  { order := SampleOrder,
    computedNodes := SampleOrder,
    updatedNodes := [],
    candidateTargetSet := [],
    diagnosticFailure := false,
    rejected := false,
    published := false,
    verifiedClean := true,
    cycleRejected := false }

theorem sample_verified_clean_no_publication :
    VerifiedCleanNoPublication SampleVerifiedCleanDecision := by
  intro clean
  unfold SampleVerifiedCleanDecision at clean
  simp at clean
  unfold SampleVerifiedCleanDecision
  simp

def SampleCycleRejectDecision : EvaluationDecision :=
  { order := [],
    computedNodes := [],
    updatedNodes := [],
    candidateTargetSet := [],
    diagnosticFailure := false,
    rejected := true,
    published := false,
    verifiedClean := false,
    cycleRejected := true }

theorem sample_cycle_reject_no_evaluation :
    CycleRejectNoEvaluation SampleCycleRejectDecision := by
  intro cycle
  unfold SampleCycleRejectDecision at cycle
  simp at cycle
  unfold SampleCycleRejectDecision
  simp

end OxCalc.CoreEngine.W046.EvaluationOrderReadDiscipline
