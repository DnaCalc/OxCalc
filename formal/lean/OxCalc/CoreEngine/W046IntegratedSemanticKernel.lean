import Std

namespace OxCalc.CoreEngine.W046.IntegratedSemanticKernel

inductive TerminalDecision where
  | none
  | verifiedClean
  | rejected
  | published
  deriving DecidableEq, Repr

structure KernelState where
  prepared : Bool
  descriptorsLowered : Bool
  graphBuilt : Bool
  reverseConverse : Bool
  diagnosticsPreserved : Bool
  invalidationClosed : Bool
  noUnderInvalidation : Bool
  rebindRequired : Bool
  dirtyNeeded : Bool
  orderSelected : Bool
  stablePriorReads : Bool
  evaluating : Bool
  candidateProduced : Bool
  oxfmlNoDirectPublish : Bool
  traceCalcObservableRefinement : Bool
  terminalDecision : TerminalDecision
  traceEmitted : Bool
  deriving DecidableEq, Repr

def Published (state : KernelState) : Prop :=
  state.terminalDecision = TerminalDecision.published

def Rejected (state : KernelState) : Prop :=
  state.terminalDecision = TerminalDecision.rejected

def VerifiedClean (state : KernelState) : Prop :=
  state.terminalDecision = TerminalDecision.verifiedClean

def Terminal (state : KernelState) : Prop :=
  Published state ∨ Rejected state ∨ VerifiedClean state

def IntegratedKernelInvariant (state : KernelState) : Prop :=
  (Published state ->
    state.prepared = true ∧
    state.descriptorsLowered = true ∧
    state.graphBuilt = true ∧
    state.reverseConverse = true ∧
    state.diagnosticsPreserved = true ∧
    state.invalidationClosed = true ∧
    state.noUnderInvalidation = true ∧
    state.rebindRequired = false ∧
    state.dirtyNeeded = true ∧
    state.orderSelected = true ∧
    state.stablePriorReads = true ∧
    state.candidateProduced = true ∧
    state.oxfmlNoDirectPublish = true ∧
    state.traceCalcObservableRefinement = true) ∧
  (Rejected state -> state.terminalDecision ≠ TerminalDecision.published) ∧
  (VerifiedClean state -> state.candidateProduced = false ∧ state.terminalDecision ≠ TerminalDecision.published) ∧
  (state.rebindRequired = true -> state.terminalDecision ≠ TerminalDecision.published) ∧
  (state.traceEmitted = true -> Terminal state)

inductive KernelStep where
  | prepareFormula
  | lowerDescriptors
  | buildGraph
  | closeInvalidation
  | markDirtyNeeded
  | selectOrder
  | beginEvaluate
  | rejectForRebind
  | formulaReject
  | formulaVerifiedClean
  | produceCandidate
  | publishCandidate
  | emitTrace
  deriving DecidableEq, Repr

def StepPre (step : KernelStep) (before : KernelState) : Prop :=
  match step with
  | .prepareFormula => before.terminalDecision = TerminalDecision.none
  | .lowerDescriptors => before.prepared = true
  | .buildGraph => before.descriptorsLowered = true
  | .closeInvalidation => before.graphBuilt = true ∧ before.reverseConverse = true
  | .markDirtyNeeded => before.invalidationClosed = true
  | .selectOrder => before.dirtyNeeded = true ∧ before.rebindRequired = false
  | .beginEvaluate => before.orderSelected = true
  | .rejectForRebind => before.invalidationClosed = true ∧ before.rebindRequired = true
  | .formulaReject => before.evaluating = true
  | .formulaVerifiedClean => before.evaluating = true ∧ before.stablePriorReads = true
  | .produceCandidate => before.evaluating = true ∧ before.stablePriorReads = true ∧ before.oxfmlNoDirectPublish = true
  | .publishCandidate => before.candidateProduced = true ∧ before.terminalDecision = TerminalDecision.none
  | .emitTrace => Terminal before

def StepPost (step : KernelStep) (before after : KernelState) : Prop :=
  match step with
  | .prepareFormula => after = { before with prepared := true, oxfmlNoDirectPublish := true }
  | .lowerDescriptors => after = { before with descriptorsLowered := true }
  | .buildGraph => after = { before with graphBuilt := true, reverseConverse := true, diagnosticsPreserved := true }
  | .closeInvalidation => after = { before with invalidationClosed := true, noUnderInvalidation := true }
  | .markDirtyNeeded => after = { before with dirtyNeeded := true }
  | .selectOrder => after = { before with orderSelected := true }
  | .beginEvaluate => after = { before with evaluating := true, stablePriorReads := true }
  | .rejectForRebind => after = { before with terminalDecision := TerminalDecision.rejected }
  | .formulaReject => after = { before with terminalDecision := TerminalDecision.rejected }
  | .formulaVerifiedClean => after = { before with terminalDecision := TerminalDecision.verifiedClean, candidateProduced := false }
  | .produceCandidate => after = { before with candidateProduced := true }
  | .publishCandidate => after = { before with terminalDecision := TerminalDecision.published, traceCalcObservableRefinement := true }
  | .emitTrace => after = { before with traceEmitted := true }

def LegalStep (step : KernelStep) (before after : KernelState) : Prop :=
  StepPre step before ∧ StepPost step before after

def InitialKernelState : KernelState :=
  { prepared := false,
    descriptorsLowered := false,
    graphBuilt := false,
    reverseConverse := false,
    diagnosticsPreserved := false,
    invalidationClosed := false,
    noUnderInvalidation := false,
    rebindRequired := false,
    dirtyNeeded := false,
    orderSelected := false,
    stablePriorReads := false,
    evaluating := false,
    candidateProduced := false,
    oxfmlNoDirectPublish := false,
    traceCalcObservableRefinement := false,
    terminalDecision := TerminalDecision.none,
    traceEmitted := false }

def SamplePublishedState : KernelState :=
  { prepared := true,
    descriptorsLowered := true,
    graphBuilt := true,
    reverseConverse := true,
    diagnosticsPreserved := true,
    invalidationClosed := true,
    noUnderInvalidation := true,
    rebindRequired := false,
    dirtyNeeded := true,
    orderSelected := true,
    stablePriorReads := true,
    evaluating := true,
    candidateProduced := true,
    oxfmlNoDirectPublish := true,
    traceCalcObservableRefinement := true,
    terminalDecision := TerminalDecision.published,
    traceEmitted := true }

theorem sample_published_integrated_kernel :
    IntegratedKernelInvariant SamplePublishedState := by
  unfold IntegratedKernelInvariant SamplePublishedState
  simp [Published, Rejected, VerifiedClean, Terminal]

theorem published_requires_graph_and_invalidation
    {state : KernelState}
    (inv : IntegratedKernelInvariant state)
    (published : Published state) :
    state.graphBuilt = true ∧ state.reverseConverse = true ∧
      state.invalidationClosed = true ∧ state.noUnderInvalidation = true := by
  rcases inv with ⟨publishLaw, _rejectLaw, _verifiedLaw, _rebindLaw, _traceLaw⟩
  rcases publishLaw published with ⟨_prepared, _lowered, graphBuilt, reverseConverse,
    _diagnostics, invalidationClosed, noUnderInvalidation, _rebindClear,
    _dirtyNeeded, _orderSelected, _stableReads, _candidate, _oxfml, _refinement⟩
  exact ⟨graphBuilt, reverseConverse, invalidationClosed, noUnderInvalidation⟩

theorem published_requires_order_reads_candidate_and_refinement
    {state : KernelState}
    (inv : IntegratedKernelInvariant state)
    (published : Published state) :
    state.orderSelected = true ∧ state.stablePriorReads = true ∧
      state.candidateProduced = true ∧ state.traceCalcObservableRefinement = true := by
  rcases inv with ⟨publishLaw, _rejectLaw, _verifiedLaw, _rebindLaw, _traceLaw⟩
  rcases publishLaw published with ⟨_prepared, _lowered, _graphBuilt, _reverseConverse,
    _diagnostics, _invalidationClosed, _noUnderInvalidation, _rebindClear,
    _dirtyNeeded, orderSelected, stablePriorReads, candidateProduced, _oxfml, refinement⟩
  exact ⟨orderSelected, stablePriorReads, candidateProduced, refinement⟩

theorem published_requires_oxfml_no_direct_publish
    {state : KernelState}
    (inv : IntegratedKernelInvariant state)
    (published : Published state) :
    state.oxfmlNoDirectPublish = true := by
  rcases inv with ⟨publishLaw, _rejectLaw, _verifiedLaw, _rebindLaw, _traceLaw⟩
  rcases publishLaw published with ⟨_prepared, _lowered, _graphBuilt, _reverseConverse,
    _diagnostics, _invalidationClosed, _noUnderInvalidation, _rebindClear,
    _dirtyNeeded, _orderSelected, _stablePriorReads, _candidateProduced, oxfml, _refinement⟩
  exact oxfml

theorem rebind_required_blocks_publication
    {state : KernelState}
    (inv : IntegratedKernelInvariant state)
    (requiresRebind : state.rebindRequired = true) :
    state.terminalDecision ≠ TerminalDecision.published := by
  exact inv.right.right.right.left requiresRebind

theorem trace_emitted_after_terminal
    {state : KernelState}
    (inv : IntegratedKernelInvariant state)
    (trace : state.traceEmitted = true) :
    Terminal state := by
  exact inv.right.right.right.right trace

def SampleRebindRejectState : KernelState :=
  { InitialKernelState with
    prepared := true,
    descriptorsLowered := true,
    graphBuilt := true,
    reverseConverse := true,
    diagnosticsPreserved := true,
    invalidationClosed := true,
    noUnderInvalidation := true,
    rebindRequired := true,
    dirtyNeeded := true,
    terminalDecision := TerminalDecision.rejected,
    traceEmitted := true }

theorem sample_rebind_reject_integrated_kernel :
    IntegratedKernelInvariant SampleRebindRejectState := by
  unfold IntegratedKernelInvariant SampleRebindRejectState InitialKernelState
  simp [Published, Rejected, VerifiedClean, Terminal]

def SampleVerifiedCleanState : KernelState :=
  { InitialKernelState with
    prepared := true,
    descriptorsLowered := true,
    graphBuilt := true,
    reverseConverse := true,
    diagnosticsPreserved := true,
    invalidationClosed := true,
    noUnderInvalidation := true,
    dirtyNeeded := true,
    orderSelected := true,
    stablePriorReads := true,
    evaluating := true,
    candidateProduced := false,
    oxfmlNoDirectPublish := true,
    terminalDecision := TerminalDecision.verifiedClean,
    traceEmitted := true }

theorem sample_verified_clean_integrated_kernel :
    IntegratedKernelInvariant SampleVerifiedCleanState := by
  unfold IntegratedKernelInvariant SampleVerifiedCleanState InitialKernelState
  simp [Published, Rejected, VerifiedClean, Terminal]

end OxCalc.CoreEngine.W046.IntegratedSemanticKernel
