import Std

namespace OxCalc.CoreEngine.W046.InvalidationRebind

abbrev NodeId := Nat

inductive NodeCalcState where
  | dirtyPending
  | needed
  | cycleBlocked
  deriving DecidableEq, Repr

inductive InvalidationReasonKind where
  | structuralRebindRequired
  | structuralRecalcOnly
  | upstreamPublication
  | dependencyAdded
  | dependencyRemoved
  | dependencyReclassified
  deriving DecidableEq, Repr

structure Edge where
  owner : NodeId
  target : NodeId
  deriving DecidableEq, Repr

structure InvalidationSeed where
  node : NodeId
  reason : InvalidationReasonKind
  deriving DecidableEq, Repr

structure NodeInvalidationRecord where
  node : NodeId
  calcState : NodeCalcState
  requiresRebind : Bool
  reasons : List InvalidationReasonKind
  deriving DecidableEq, Repr

structure InvalidationClosure where
  records : List NodeInvalidationRecord
  deriving DecidableEq, Repr

structure RebindGateDecision where
  rejected : Bool
  published : Bool
  deriving DecidableEq, Repr

def ReasonRequiresRebind : InvalidationReasonKind -> Bool
  | InvalidationReasonKind.structuralRebindRequired => true
  | InvalidationReasonKind.dependencyAdded => true
  | InvalidationReasonKind.dependencyRemoved => true
  | InvalidationReasonKind.dependencyReclassified => true
  | InvalidationReasonKind.structuralRecalcOnly => false
  | InvalidationReasonKind.upstreamPublication => false

def SeedCalcState (reason : InvalidationReasonKind) (inCycle : Bool) : NodeCalcState :=
  if inCycle then
    NodeCalcState.cycleBlocked
  else
    match reason with
    | InvalidationReasonKind.upstreamPublication => NodeCalcState.needed
    | InvalidationReasonKind.structuralRebindRequired
    | InvalidationReasonKind.structuralRecalcOnly
    | InvalidationReasonKind.dependencyAdded
    | InvalidationReasonKind.dependencyRemoved
    | InvalidationReasonKind.dependencyReclassified => NodeCalcState.dirtyPending

def SeedRecord (seed : InvalidationSeed) (inCycle : Bool) : NodeInvalidationRecord :=
  { node := seed.node,
    calcState := SeedCalcState seed.reason inCycle,
    requiresRebind := ReasonRequiresRebind seed.reason,
    reasons := [seed.reason] }

def UpstreamRecord (node : NodeId) (inCycle : Bool) : NodeInvalidationRecord :=
  { node,
    calcState := if inCycle then NodeCalcState.cycleBlocked else NodeCalcState.dirtyPending,
    requiresRebind := false,
    reasons := [InvalidationReasonKind.upstreamPublication] }

def RecordFor (closure : InvalidationClosure) (node : NodeId) : Prop :=
  exists record, record ∈ closure.records ∧ record.node = node

def RecordRequiresRebind (closure : InvalidationClosure) (node : NodeId) : Prop :=
  exists record,
    record ∈ closure.records ∧ record.node = node ∧ record.requiresRebind = true

inductive ReverseReachable (edges : List Edge) : NodeId -> NodeId -> Prop where
  | seed (node : NodeId) : ReverseReachable edges node node
  | step
      {seed target dependent : NodeId}
      (reachable : ReverseReachable edges seed target)
      (edge : Edge)
      (edgeMember : edge ∈ edges)
      (edgeTarget : edge.target = target)
      (edgeOwner : edge.owner = dependent) :
      ReverseReachable edges seed dependent

def NoUnderInvalidation
    (edges : List Edge)
    (seeds : List InvalidationSeed)
    (closure : InvalidationClosure) : Prop :=
  forall seed,
    seed ∈ seeds ->
      forall node,
        ReverseReachable edges seed.node node ->
          RecordFor closure node

def RebindGateRejectsBeforePublish
    (closure : InvalidationClosure)
    (decision : RebindGateDecision) : Prop :=
  forall record,
    record ∈ closure.records ->
      record.requiresRebind = true ->
        decision.rejected = true ∧ decision.published = false

structure InvalidationRebindSemanticModel
    (edges : List Edge)
    (seeds : List InvalidationSeed)
    (closure : InvalidationClosure)
    (decision : RebindGateDecision) where
  noUnderInvalidation : NoUnderInvalidation edges seeds closure
  rebindGateRejectsBeforePublish : RebindGateRejectsBeforePublish closure decision

theorem seedRecord_rebind_flag_matches_reason
    (seed : InvalidationSeed)
    (inCycle : Bool) :
    (SeedRecord seed inCycle).requiresRebind = ReasonRequiresRebind seed.reason := by
  rfl

theorem dependencyAdded_requiresRebind :
    ReasonRequiresRebind InvalidationReasonKind.dependencyAdded = true := by
  rfl

theorem dependencyRemoved_requiresRebind :
    ReasonRequiresRebind InvalidationReasonKind.dependencyRemoved = true := by
  rfl

theorem dependencyReclassified_requiresRebind :
    ReasonRequiresRebind InvalidationReasonKind.dependencyReclassified = true := by
  rfl

theorem upstreamPublication_doesNotRequireRebind :
    ReasonRequiresRebind InvalidationReasonKind.upstreamPublication = false := by
  rfl

theorem structuralRecalcOnly_doesNotRequireRebind :
    ReasonRequiresRebind InvalidationReasonKind.structuralRecalcOnly = false := by
  rfl

def TargetTransitionReasons
    (previousTarget nextTarget : Option NodeId) : List InvalidationReasonKind :=
  match previousTarget, nextTarget with
  | none, some _ => [InvalidationReasonKind.dependencyAdded]
  | some _, none => [InvalidationReasonKind.dependencyRemoved]
  | _, _ => []

def DependencyTransitionReasons
    (previousTarget nextTarget : Option NodeId)
    (reclassified : Bool) : List InvalidationReasonKind :=
  TargetTransitionReasons previousTarget nextTarget ++
    if reclassified then [InvalidationReasonKind.dependencyReclassified] else []

theorem dynamicTargetAdded_emitsDependencyAdded
    (target : NodeId) :
    InvalidationReasonKind.dependencyAdded ∈
      DependencyTransitionReasons none (some target) false := by
  unfold DependencyTransitionReasons TargetTransitionReasons
  simp

theorem dynamicTargetRemoved_emitsDependencyRemoved
    (target : NodeId) :
    InvalidationReasonKind.dependencyRemoved ∈
      DependencyTransitionReasons (some target) none false := by
  unfold DependencyTransitionReasons TargetTransitionReasons
  simp

theorem dependencyReclassification_emitsDependencyReclassified
    (previousTarget nextTarget : Option NodeId) :
    InvalidationReasonKind.dependencyReclassified ∈
      DependencyTransitionReasons previousTarget nextTarget true := by
  unfold DependencyTransitionReasons
  simp

theorem semanticModel_noUnderInvalidation
    {edges : List Edge}
    {seeds : List InvalidationSeed}
    {closure : InvalidationClosure}
    {decision : RebindGateDecision}
    (model : InvalidationRebindSemanticModel edges seeds closure decision) :
    NoUnderInvalidation edges seeds closure := by
  exact model.noUnderInvalidation

theorem semanticModel_rebindGateRejectsBeforePublish
    {edges : List Edge}
    {seeds : List InvalidationSeed}
    {closure : InvalidationClosure}
    {decision : RebindGateDecision}
    (model : InvalidationRebindSemanticModel edges seeds closure decision) :
    RebindGateRejectsBeforePublish closure decision := by
  exact model.rebindGateRejectsBeforePublish

def SampleEdgeBA : Edge := { owner := 2, target := 1 }

def SampleSeedA : InvalidationSeed :=
  { node := 1, reason := InvalidationReasonKind.upstreamPublication }

def SampleRebindSeedB : InvalidationSeed :=
  { node := 2, reason := InvalidationReasonKind.dependencyAdded }

def SampleClosure : InvalidationClosure :=
  { records := [
      SeedRecord SampleSeedA false,
      SeedRecord SampleRebindSeedB false,
      UpstreamRecord 3 false
    ] }

def SampleRejectedDecision : RebindGateDecision :=
  { rejected := true, published := false }

theorem sampleReachable_A_to_B :
    ReverseReachable [SampleEdgeBA] 1 2 := by
  apply ReverseReachable.step (target := 1) (edge := SampleEdgeBA)
  · exact ReverseReachable.seed 1
  · unfold SampleEdgeBA
    simp
  · rfl
  · rfl

theorem sampleClosure_contains_rebind_seed :
    RecordRequiresRebind SampleClosure 2 := by
  unfold RecordRequiresRebind SampleClosure SampleRebindSeedB SeedRecord ReasonRequiresRebind
  simp

theorem sampleRejectedDecision_noPublish :
    SampleRejectedDecision.published = false := by
  rfl

end OxCalc.CoreEngine.W046.InvalidationRebind
