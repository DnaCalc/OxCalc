import Std

namespace OxCalc.CoreEngine.W046.RustRefinementBridge

structure ImplementationTraceFacts where
  graphBuilt : Bool
  reverseConverse : Bool
  invalidationClosureCoversOrder : Bool
  orderRespectsDependencies : Bool
  stableOrPriorReads : Bool
  candidateBuilt : Bool
  publicationBuilt : Bool
  rejectBuilt : Bool
  noPublicationBundle : Bool
  traceEmissionPresent : Bool
  dynamicReject : Bool
  cycleReject : Bool
  rebindReject : Bool
  deriving DecidableEq, Repr

def PublishedRefinesIntegratedKernel (facts : ImplementationTraceFacts) : Prop :=
  facts.graphBuilt = true ∧
  facts.reverseConverse = true ∧
  facts.invalidationClosureCoversOrder = true ∧
  facts.orderRespectsDependencies = true ∧
  facts.stableOrPriorReads = true ∧
  facts.candidateBuilt = true ∧
  facts.publicationBuilt = true ∧
  facts.rejectBuilt = false ∧
  facts.traceEmissionPresent = true

def RejectedRefinesIntegratedKernel (facts : ImplementationTraceFacts) : Prop :=
  facts.graphBuilt = true ∧
  facts.noPublicationBundle = true ∧
  facts.rejectBuilt = true ∧
  facts.traceEmissionPresent = true

def DynamicRejectRefines (facts : ImplementationTraceFacts) : Prop :=
  RejectedRefinesIntegratedKernel facts ∧ facts.dynamicReject = true

def CycleRejectRefines (facts : ImplementationTraceFacts) : Prop :=
  RejectedRefinesIntegratedKernel facts ∧ facts.cycleReject = true

def RebindRejectRefines (facts : ImplementationTraceFacts) : Prop :=
  RejectedRefinesIntegratedKernel facts ∧ facts.rebindReject = true

def ChainPublishFacts : ImplementationTraceFacts :=
  { graphBuilt := true,
    reverseConverse := true,
    invalidationClosureCoversOrder := true,
    orderRespectsDependencies := true,
    stableOrPriorReads := true,
    candidateBuilt := true,
    publicationBuilt := true,
    rejectBuilt := false,
    noPublicationBundle := false,
    traceEmissionPresent := true,
    dynamicReject := false,
    cycleReject := false,
    rebindReject := false }

def LetLambdaPublishFacts : ImplementationTraceFacts := ChainPublishFacts

def DynamicRejectFacts : ImplementationTraceFacts :=
  { graphBuilt := true,
    reverseConverse := true,
    invalidationClosureCoversOrder := true,
    orderRespectsDependencies := true,
    stableOrPriorReads := false,
    candidateBuilt := false,
    publicationBuilt := false,
    rejectBuilt := true,
    noPublicationBundle := true,
    traceEmissionPresent := true,
    dynamicReject := true,
    cycleReject := false,
    rebindReject := false }

def CycleRejectFacts : ImplementationTraceFacts :=
  { graphBuilt := true,
    reverseConverse := true,
    invalidationClosureCoversOrder := true,
    orderRespectsDependencies := false,
    stableOrPriorReads := false,
    candidateBuilt := false,
    publicationBuilt := false,
    rejectBuilt := true,
    noPublicationBundle := true,
    traceEmissionPresent := true,
    dynamicReject := false,
    cycleReject := true,
    rebindReject := false }

def RebindRejectFacts : ImplementationTraceFacts :=
  { graphBuilt := true,
    reverseConverse := true,
    invalidationClosureCoversOrder := true,
    orderRespectsDependencies := false,
    stableOrPriorReads := false,
    candidateBuilt := false,
    publicationBuilt := false,
    rejectBuilt := true,
    noPublicationBundle := true,
    traceEmissionPresent := true,
    dynamicReject := false,
    cycleReject := false,
    rebindReject := true }

theorem chain_publish_refines_kernel :
    PublishedRefinesIntegratedKernel ChainPublishFacts := by
  unfold PublishedRefinesIntegratedKernel ChainPublishFacts
  simp

theorem let_lambda_publish_refines_kernel :
    PublishedRefinesIntegratedKernel LetLambdaPublishFacts := by
  unfold LetLambdaPublishFacts
  exact chain_publish_refines_kernel

theorem dynamic_reject_refines_kernel :
    DynamicRejectRefines DynamicRejectFacts := by
  unfold DynamicRejectRefines RejectedRefinesIntegratedKernel DynamicRejectFacts
  simp

theorem cycle_reject_refines_kernel :
    CycleRejectRefines CycleRejectFacts := by
  unfold CycleRejectRefines RejectedRefinesIntegratedKernel CycleRejectFacts
  simp

theorem rebind_reject_refines_kernel :
    RebindRejectRefines RebindRejectFacts := by
  unfold RebindRejectRefines RejectedRefinesIntegratedKernel RebindRejectFacts
  simp

theorem published_trace_has_no_reject
    {facts : ImplementationTraceFacts}
    (refines : PublishedRefinesIntegratedKernel facts) :
    facts.rejectBuilt = false := by
  exact refines.2.2.2.2.2.2.2.1

theorem rejected_trace_has_no_publication
    {facts : ImplementationTraceFacts}
    (refines : RejectedRefinesIntegratedKernel facts) :
    facts.noPublicationBundle = true := by
  exact refines.2.1

end OxCalc.CoreEngine.W046.RustRefinementBridge
