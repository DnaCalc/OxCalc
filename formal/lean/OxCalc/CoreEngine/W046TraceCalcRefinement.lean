import Std

namespace OxCalc.CoreEngine.W046.TraceCalcRefinement

abbrev ScenarioId := String
abbrev NodeId := String
abbrev ValueText := String
abbrev EventFamily := String
abbrev DiagnosticText := String

inductive ResultState where
  | passed
  | failedAssertion
  | invalidScenario
  | executionError
  | unsupportedFeature
  deriving DecidableEq, Repr

inductive PublicationDecision where
  | noPublish
  | published
  deriving DecidableEq, Repr

structure NodeValue where
  nodeId : NodeId
  value : ValueText
  deriving DecidableEq, Repr

structure RejectFact where
  rejectId : String
  rejectKind : String
  detail : String
  deriving DecidableEq, Repr

structure EffectFact where
  family : String
  ownerNodeId : NodeId
  detail : String
  deriving DecidableEq, Repr

structure InvalidationFact where
  nodeId : NodeId
  reason : String
  deriving DecidableEq, Repr

structure ObservablePacket where
  scenarioId : ScenarioId
  resultState : ResultState
  publishedValues : List NodeValue
  diagnostics : List DiagnosticText
  dependencyEffects : List EffectFact
  invalidationRecords : List InvalidationFact
  rejects : List RejectFact
  publicationDecision : PublicationDecision
  traceFamilies : List EventFamily
  deriving DecidableEq, Repr

def ListSubset [DecidableEq α] (expected observed : List α) : Prop :=
  ∀ item, item ∈ expected -> item ∈ observed

def ExactList (expected observed : List α) : Prop :=
  expected = observed

def ObservableRefinement (oracle engine : ObservablePacket) : Prop :=
  oracle.resultState = ResultState.passed ∧
    engine.scenarioId = oracle.scenarioId ∧
      engine.resultState = oracle.resultState ∧
        ExactList oracle.publishedValues engine.publishedValues ∧
          ExactList oracle.diagnostics engine.diagnostics ∧
            ListSubset oracle.dependencyEffects engine.dependencyEffects ∧
              ListSubset oracle.invalidationRecords engine.invalidationRecords ∧
                ExactList oracle.rejects engine.rejects ∧
                  engine.publicationDecision = oracle.publicationDecision ∧
                    ListSubset oracle.traceFamilies engine.traceFamilies

structure RefinementBinding where
  oracle : ObservablePacket
  engine : ObservablePacket
  covered : Bool
  exactBlockers : List String
  relation : covered = true -> ObservableRefinement oracle engine

def BindingHasExactBlocker (binding : RefinementBinding) : Prop :=
  binding.covered = false ∧ binding.exactBlockers ≠ []

theorem refinement_preserves_result_state
    {oracle engine : ObservablePacket}
    (refines : ObservableRefinement oracle engine) :
    engine.resultState = oracle.resultState := by
  exact refines.right.right.left

theorem refinement_preserves_published_values
    {oracle engine : ObservablePacket}
    (refines : ObservableRefinement oracle engine) :
    ExactList oracle.publishedValues engine.publishedValues := by
  exact refines.right.right.right.left

theorem refinement_preserves_diagnostics
    {oracle engine : ObservablePacket}
    (refines : ObservableRefinement oracle engine) :
    ExactList oracle.diagnostics engine.diagnostics := by
  exact refines.right.right.right.right.left

theorem refinement_preserves_dependency_effects
    {oracle engine : ObservablePacket}
    (refines : ObservableRefinement oracle engine) :
    ListSubset oracle.dependencyEffects engine.dependencyEffects := by
  exact refines.right.right.right.right.right.left

theorem refinement_preserves_invalidation_records
    {oracle engine : ObservablePacket}
    (refines : ObservableRefinement oracle engine) :
    ListSubset oracle.invalidationRecords engine.invalidationRecords := by
  exact refines.right.right.right.right.right.right.left

theorem refinement_preserves_rejects
    {oracle engine : ObservablePacket}
    (refines : ObservableRefinement oracle engine) :
    ExactList oracle.rejects engine.rejects := by
  exact refines.right.right.right.right.right.right.right.left

theorem refinement_preserves_publication_decision
    {oracle engine : ObservablePacket}
    (refines : ObservableRefinement oracle engine) :
    engine.publicationDecision = oracle.publicationDecision := by
  exact refines.right.right.right.right.right.right.right.right.left

theorem refinement_preserves_required_trace_families
    {oracle engine : ObservablePacket}
    (refines : ObservableRefinement oracle engine) :
    ListSubset oracle.traceFamilies engine.traceFamilies := by
  exact refines.right.right.right.right.right.right.right.right.right

theorem refined_reject_no_publish
    {oracle engine : ObservablePacket}
    (refines : ObservableRefinement oracle engine)
    (oracleNoPublish : oracle.publicationDecision = PublicationDecision.noPublish)
    (_oracleRejects : oracle.rejects ≠ []) :
    engine.publicationDecision = PublicationDecision.noPublish := by
  exact Eq.trans (refinement_preserves_publication_decision refines) oracleNoPublish

def SampleAcceptOracle : ObservablePacket :=
  { scenarioId := "tc_accept_publish_001",
    resultState := ResultState.passed,
    publishedValues := [{ nodeId := "B", value := "2" }],
    diagnostics := [],
    dependencyEffects := [],
    invalidationRecords := [],
    rejects := [],
    publicationDecision := PublicationDecision.published,
    traceFamilies := ["candidate.admitted", "candidate.built", "publication.committed"] }

def SampleAcceptEngine : ObservablePacket := SampleAcceptOracle

theorem sample_accept_refines :
    ObservableRefinement SampleAcceptOracle SampleAcceptEngine := by
  change ObservableRefinement SampleAcceptOracle SampleAcceptOracle
  unfold ObservableRefinement ExactList ListSubset SampleAcceptOracle
  simp

def SampleRejectOracle : ObservablePacket :=
  { scenarioId := "tc_reject_no_publish_001",
    resultState := ResultState.passed,
    publishedValues := [{ nodeId := "A", value := "2" }],
    diagnostics := ["capability denied"],
    dependencyEffects := [],
    invalidationRecords := [],
    rejects :=
      [{ rejectId := "rej1",
         rejectKind := "capability_mismatch",
         detail := "capability denied" }],
    publicationDecision := PublicationDecision.noPublish,
    traceFamilies := ["candidate.admitted", "reject.issued"] }

def SampleRejectEngine : ObservablePacket := SampleRejectOracle

theorem sample_reject_refines :
    ObservableRefinement SampleRejectOracle SampleRejectEngine := by
  change ObservableRefinement SampleRejectOracle SampleRejectOracle
  unfold ObservableRefinement ExactList ListSubset SampleRejectOracle
  simp

theorem sample_reject_is_no_publish :
    SampleRejectEngine.publicationDecision = PublicationDecision.noPublish := by
  exact refined_reject_no_publish sample_reject_refines rfl (by decide)

def SampleDynamicOracle : ObservablePacket :=
  { scenarioId := "tc_w035_dynamic_dependency_switch_publish_001",
    resultState := ResultState.passed,
    publishedValues := [{ nodeId := "Out", value := "20" }],
    diagnostics := [],
    dependencyEffects :=
      [{ family := "DynamicDependency",
         ownerNodeId := "Out",
         detail := "dynamic dependency switched" }],
    invalidationRecords :=
      [{ nodeId := "Out", reason := "DependencyShapeChanged" }],
    rejects := [],
    publicationDecision := PublicationDecision.published,
    traceFamilies := ["candidate.built", "publication.committed"] }

def SampleDynamicEngine : ObservablePacket :=
  { SampleDynamicOracle with
    dependencyEffects :=
      SampleDynamicOracle.dependencyEffects ++
        [{ family := "Instrumentation",
           ownerNodeId := "Out",
           detail := "extra non-semantic timing event" }] }

theorem sample_dynamic_refines :
    ObservableRefinement SampleDynamicOracle SampleDynamicEngine := by
  unfold ObservableRefinement SampleDynamicOracle SampleDynamicEngine ExactList ListSubset
  simp [SampleDynamicOracle]

def SampleBlockedBinding : RefinementBinding :=
  { oracle := SampleDynamicOracle,
    engine := { SampleDynamicOracle with dependencyEffects := [] },
    covered := false,
    exactBlockers := ["treecalc_local_dynamic_dependency_projection_gap"],
    relation := by
      intro covered
      cases covered }

theorem sample_blocked_binding_has_exact_blocker :
    BindingHasExactBlocker SampleBlockedBinding := by
  unfold BindingHasExactBlocker SampleBlockedBinding
  simp

end OxCalc.CoreEngine.W046.TraceCalcRefinement
